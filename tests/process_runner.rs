//! Bounded process execution against real child processes.
//!
//! These tests never require Docker and never touch user application data.
//! They drive `/bin/sh` on Unix and `cmd.exe`/`powershell` on Windows, which
//! are present on every platform the project builds for.
use local_store::runtime::{
    CancelToken, CommandSpec, ProcessRunner, SystemProcessRunner, MAX_CAPTURED_BYTES,
};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

/// A script interpreted by PowerShell on Windows and `sh` elsewhere. Used when
/// a test needs precise control over the bytes a child writes.
fn script(unix: &str, windows: &str, timeout: Duration) -> CommandSpec {
    let _ = (unix, windows);
    #[cfg(windows)]
    {
        CommandSpec::new(
            "powershell",
            vec![
                "-NoProfile".into(),
                "-NonInteractive".into(),
                "-Command".into(),
                windows.into(),
            ],
            None,
            timeout,
        )
    }
    #[cfg(unix)]
    {
        CommandSpec::new("/bin/sh", vec!["-c".into(), unix.into()], None, timeout)
    }
}

/// A script interpreted by the platform command shell. Used where the test
/// needs a shell that backgrounds a grandchild holding the inherited pipe.
fn shell(unix: &str, windows: &str, timeout: Duration) -> CommandSpec {
    let _ = (unix, windows);
    #[cfg(windows)]
    {
        CommandSpec::new("cmd.exe", vec!["/C".into(), windows.into()], None, timeout)
    }
    #[cfg(unix)]
    {
        CommandSpec::new("/bin/sh", vec!["-c".into(), unix.into()], None, timeout)
    }
}

fn scratch(name: &str) -> PathBuf {
    let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_file(&path);
    path
}

#[test]
fn a_successful_command_reports_its_output() {
    let output = SystemProcessRunner
        .run(&script(
            "echo bounded-ok",
            "Write-Output 'bounded-ok'",
            Duration::from_secs(60),
        ))
        .expect("the command should run");
    assert!(output.success);
    assert!(output.stdout.contains("bounded-ok"), "{:?}", output);
    assert!(!output.truncated);
}

#[test]
fn a_nonzero_exit_keeps_the_failure_output() {
    let output = SystemProcessRunner
        .run(&script(
            "echo bounded-boom 1>&2; exit 3",
            "[Console]::Error.Write('bounded-boom'); exit 3",
            Duration::from_secs(60),
        ))
        .expect("a failing command still reports an exit status");
    assert!(!output.success);
    assert!(output.stderr.contains("bounded-boom"), "{:?}", output);
}

#[test]
fn a_missing_executable_fails_immediately() {
    let started = Instant::now();
    let error = SystemProcessRunner
        .run(&CommandSpec::new(
            "local-store-no-such-program",
            Vec::new(),
            None,
            Duration::from_secs(60),
        ))
        .expect_err("a missing program cannot run");
    assert!(error.to_string().contains("failed to run"), "{error}");
    assert_eq!(
        error.code,
        local_store::runtime::ProcessErrorCode::ProcessUnavailable
    );
    assert!(started.elapsed() < Duration::from_secs(30));
}

#[test]
fn a_command_that_outlives_its_deadline_is_terminated_with_its_children() {
    let marker = scratch("deadline-marker.txt");
    let path = marker.to_string_lossy().replace('\'', "''");
    let started = Instant::now();
    let error = SystemProcessRunner
        .run(&script(
            &format!("sleep 8; echo late > '{path}'"),
            &format!("Start-Sleep -Seconds 8; Set-Content -LiteralPath '{path}' -Value 'late'"),
            Duration::from_secs(1),
        ))
        .expect_err("the command must not be allowed to finish");
    assert!(error.to_string().contains("timed out"), "{error}");
    assert_eq!(error.code, local_store::runtime::ProcessErrorCode::TimedOut);
    assert!(
        started.elapsed() < Duration::from_secs(30),
        "the runner waited {:?}",
        started.elapsed()
    );
    // The whole tree, not just the shell, has to be gone: the marker would
    // only appear if the sleeping descendant had survived termination.
    std::thread::sleep(Duration::from_secs(12));
    assert!(!marker.exists(), "a terminated child kept running");
}

#[test]
fn a_cancelled_command_stops_without_waiting_for_its_deadline() {
    let cancel = CancelToken::new();
    let trigger = cancel.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(500));
        trigger.cancel();
    });
    let started = Instant::now();
    let error = SystemProcessRunner
        .run_cancellable(
            &script(
                "sleep 120",
                "Start-Sleep -Seconds 120",
                Duration::from_secs(600),
            ),
            &cancel,
        )
        .expect_err("cancellation must abandon the command");
    assert!(error.to_string().contains("cancelled"), "{error}");
    assert_eq!(
        error.code,
        local_store::runtime::ProcessErrorCode::Cancelled
    );
    assert!(
        started.elapsed() < Duration::from_secs(60),
        "cancellation took {:?}",
        started.elapsed()
    );
}

#[test]
fn oversized_output_on_both_streams_at_once_stays_bounded() {
    let bytes = MAX_CAPTURED_BYTES + 50_000;
    // Both streams must be written *concurrently*. A reader that drained one
    // stream to the end before starting the other would deadlock here as soon
    // as the neglected pipe filled, which is exactly the regression guarded.
    let chunks = bytes.div_ceil(8192);
    let output = SystemProcessRunner
        .run(&script(
            &format!(
                "yes oooooooooo | head -c {bytes} & \
                 yes eeeeeeeeee | head -c {bytes} >&2; wait"
            ),
            &format!(
                "$out = 'o' * 8192; $err = 'e' * 8192; \
                 for ($i = 0; $i -lt {chunks}; $i++) \
                 {{ [Console]::Out.Write($out); [Console]::Error.Write($err) }}"
            ),
            Duration::from_secs(120),
        ))
        .expect("a noisy command still completes");
    assert!(output.success, "{:?}", output.stderr.len());
    assert!(
        output.truncated,
        "the capture bound should have been reached"
    );
    assert!(
        output.stdout.len() <= MAX_CAPTURED_BYTES,
        "{}",
        output.stdout.len()
    );
    assert!(
        output.stderr.len() <= MAX_CAPTURED_BYTES,
        "{}",
        output.stderr.len()
    );
    assert!(output.stdout.starts_with('o'), "stdout was not captured");
    assert!(output.stderr.starts_with('e'), "stderr was not captured");
}

#[test]
fn truncating_multibyte_output_never_produces_a_replacement_character() {
    // Both scripts emit an odd number of bytes before the capture bound so the
    // cut lands inside a two-byte character rather than neatly between two.
    // Unix: 21-byte lines, and 262144 = 21 * 12483 + 1.
    // Windows: one leading ASCII byte ahead of the two-byte repeats.
    let repeats = MAX_CAPTURED_BYTES;
    let output = SystemProcessRunner
        .run(&script(
            "yes éééééééééé | head -c 600000",
            &format!(
                "[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false); \
                 [Console]::Out.Write('x' + [string]([char]0xE9) * {repeats})"
            ),
            Duration::from_secs(120),
        ))
        .expect("the command should run");
    assert!(output.success);
    assert!(
        output.truncated,
        "the capture bound should have been reached"
    );
    assert!(
        output.stdout.contains('\u{00e9}'),
        "the multibyte character was not delivered"
    );
    assert!(
        !output.stdout.contains('\u{fffd}'),
        "truncation corrupted a character"
    );
}

#[test]
fn a_grandchild_holding_the_pipe_does_not_stall_the_result() {
    let started = Instant::now();
    let output = SystemProcessRunner
        .run(&shell(
            "sleep 20 & echo parent-done",
            "start /B ping -n 20 127.0.0.1 & echo parent-done",
            Duration::from_secs(120),
        ))
        .expect("the parent exits even though the pipe stays open");
    assert!(output.success);
    assert!(output.stdout.contains("parent-done"), "{:?}", output.stdout);
    // The parent exits at once; only the bounded reader grace may be spent.
    assert!(
        started.elapsed() < Duration::from_secs(15),
        "the runner waited {:?} for an inherited pipe",
        started.elapsed()
    );
}
