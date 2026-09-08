//! Smoke tests for the CLI in the configuration that is actually shipped.
//!
//! `tests/cli_queries.rs` exercises the debug binary. On Windows that is a
//! *different program* from the released one: release sets
//! `windows_subsystem = "windows"`, so the shipped executable is a GUI-subsystem
//! image while the debug one is a console image. Behaviour that depends on that
//! flag — whether a shell waits for the process, and where its output goes —
//! cannot be observed by the debug tests at all.
//!
//! Run with `cargo test --release --test cli_packaged` to cover the shipped
//! image. The tests also pass in debug, which is what makes them useful: they
//! assert the CLI contract holds in *both* profiles.
//!
//! Nothing here needs Docker, a display server, or user registry data.
use serde_json::Value;
use std::process::{Command, Output};

const EXE: &str = env!("CARGO_BIN_EXE_local-store");

/// Spawn the binary directly, the way a script or another program would.
/// `Command` passes explicit stdio handles, so this path does not depend on
/// the executable's subsystem.
fn run(args: &[&str]) -> Output {
    Command::new(EXE)
        .args(args)
        .env("PATH", "") // Missing Docker is intentional and deterministic.
        .output()
        .expect("the packaged CLI must start")
}

#[test]
fn the_binary_under_test_is_built_the_way_it_ships() {
    #[cfg(windows)]
    {
        // Subsystem lives at offset 68 of the optional header, which follows
        // the 4-byte PE signature and the 20-byte COFF header.
        const IMAGE_SUBSYSTEM_WINDOWS_GUI: u16 = 2;
        const IMAGE_SUBSYSTEM_WINDOWS_CUI: u16 = 3;
        let image = std::fs::read(EXE).expect("the CLI binary must be readable");
        let pe_offset = u32::from_le_bytes(image[0x3C..0x40].try_into().unwrap()) as usize;
        assert_eq!(
            &image[pe_offset..pe_offset + 4],
            b"PE\0\0",
            "not a PE image"
        );
        let at = pe_offset + 4 + 20 + 68;
        let subsystem = u16::from_le_bytes(image[at..at + 2].try_into().unwrap());
        // This test binary and the CLI share a profile, so `debug_assertions`
        // says which configuration is under test.
        let expected = if cfg!(debug_assertions) {
            IMAGE_SUBSYSTEM_WINDOWS_CUI
        } else {
            IMAGE_SUBSYSTEM_WINDOWS_GUI
        };
        assert_eq!(
            subsystem, expected,
            "release must ship a GUI-subsystem image and debug a console one; \
             if this changed, the shell behaviour documented in docs/cli.md changed with it"
        );
    }
    #[cfg(not(windows))]
    {
        assert!(std::fs::metadata(EXE).is_ok(), "the CLI binary must exist");
    }
}

#[test]
fn the_packaged_binary_reports_its_version() {
    let output = run(&["--version"]);
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).expect("version output must be UTF-8");
    assert!(text.contains(env!("CARGO_PKG_VERSION")), "{text:?}");
}

#[test]
fn packaged_doctor_writes_only_json_and_its_exit_code_tracks_readiness() {
    let output = run(&["doctor", "--json"]);
    assert!(output.stderr.is_empty(), "stdout-only contract");
    let report: Value =
        serde_json::from_slice(&output.stdout).expect("stdout must contain only JSON");

    // The contract is that the exit code reports what the JSON says, not that
    // Docker is absent. Asserting the latter made this pass only on a machine
    // without Docker: the Windows CI runner has it, so the packaged binary
    // correctly reported ready and exited zero.
    let ready = report["ready"].as_bool().expect("ready must be a boolean");
    assert_eq!(
        output.status.code(),
        Some(if ready { 0 } else { 1 }),
        "exit code must follow readiness (ready={ready})"
    );
    assert_eq!(report["checks"].as_array().unwrap().len(), 2);
    // Task 13's deadline means an unreachable Docker fails rather than hanging;
    // if this ever blocks, the packaged runner has lost its bound.
    assert!(report["checks"]
        .as_array()
        .unwrap()
        .iter()
        .all(|check| !check["detail"].as_str().unwrap().is_empty()));
}

#[test]
fn packaged_catalog_search_works_entirely_offline() {
    let output = run(&["catalog", "--json", "--limit", "2"]);
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let page: Value = serde_json::from_slice(&output.stdout).expect("stdout must be JSON");
    assert_eq!(page["entries"].as_array().unwrap().len(), 2);
    assert_eq!(page["next_offset"], 2);
}

#[test]
fn packaged_usage_errors_exit_two_and_leave_stdout_empty() {
    for args in [
        vec!["catalog", "--limit", "0"],
        vec!["doctor", "--json", "--json"],
        vec!["start", "memos", "extra"],
    ] {
        let output = run(&args);
        assert_eq!(output.status.code(), Some(2), "{args:?}");
        assert!(output.stdout.is_empty(), "{args:?}");
        assert!(!output.stderr.is_empty(), "{args:?}");
    }
}

/// The Windows command shell is the interactive path that works for a
/// GUI-subsystem image: `cmd.exe` passes its own handles to the child and waits
/// for it, so redirection, pipes and `%ERRORLEVEL%` all behave.
///
/// Windows PowerShell is deliberately not covered here: it starts a
/// GUI-subsystem process detached, so it captures no output and sets no exit
/// code. That is a real limitation of the shipped image, recorded in
/// `docs/cli.md`, and no test can make it pass.
#[cfg(windows)]
#[test]
fn the_windows_command_shell_receives_stdout_and_the_exit_code() {
    use std::os::windows::process::CommandExt;

    fn through_cmd(tail: &str) -> Output {
        // `raw_arg` bypasses Rust's argument quoting, which cmd.exe parses by
        // its own rules. The doubled outer quotes are what cmd.exe strips when
        // the command itself is a quoted path.
        Command::new("cmd.exe")
            .raw_arg(format!("/C \"\"{EXE}\" {tail}\""))
            .output()
            .expect("cmd.exe must run the CLI")
    }

    let version = through_cmd("--version");
    assert!(version.status.success());
    assert!(
        String::from_utf8_lossy(&version.stdout).contains(env!("CARGO_PKG_VERSION")),
        "cmd.exe captured no version output: {version:?}"
    );

    let page = through_cmd("catalog --json --limit 1");
    assert!(page.status.success());
    let parsed: Value =
        serde_json::from_slice(&page.stdout).expect("cmd.exe must receive the JSON page");
    assert_eq!(parsed["entries"].as_array().unwrap().len(), 1);

    let invalid = through_cmd("catalog --limit 0");
    assert_eq!(
        invalid.status.code(),
        Some(2),
        "cmd.exe must propagate the usage exit code"
    );
}

#[test]
fn malformed_native_activations_exit_as_usage_errors() {
    for args in [
        vec!["localstore://open/memos", "extra"],
        vec!["localstore://open/a/../memos"],
        vec!["localstore://open/%00"],
        vec!["open", " "],
    ] {
        let output = run(&args);
        assert_eq!(output.status.code(), Some(2), "{args:?}");
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
}

#[test]
fn native_open_missing_app_fails_without_creating_registry_or_starting_gui() {
    let root = std::env::temp_dir().join(format!(
        "local-store-activation-empty-{}",
        std::process::id()
    ));
    assert!(!root.exists(), "test requires an unused config root");
    for args in [
        vec!["open", "missing-app"],
        vec!["localstore://open/missing-app"],
    ] {
        let output = Command::new(EXE)
            .args(&args)
            .env("APPDATA", &root)
            .env("XDG_CONFIG_HOME", &root)
            .env("LOCAL_STORE_CONFIG_DIR", &root)
            .env("PATH", "")
            .output()
            .expect("native activation must return");
        assert_eq!(output.status.code(), Some(1), "{args:?}");
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains("No app with ID or name"));
    }
    assert!(
        !root.exists(),
        "a missing app must not create config or protocol handlers"
    );
}
