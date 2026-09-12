//! The registry survives more than one Local Store process at a time.
//!
//! Saving the registry is atomic, so the file is never torn — but atomicity
//! alone does not prevent a *lost update*: two processes each read the same
//! registry, and whichever saves second silently discards the other's change.
//! These tests drive real subprocesses, because an in-process test cannot
//! observe a cross-process lock at all.
//!
//! No Docker, no network, and every process is pointed at a private config root.
use local_store::storage;
use std::{
    path::{Path, PathBuf},
    process::{Child, Command},
    time::{Duration, Instant},
};

const CLI: &str = env!("CARGO_BIN_EXE_local-store");
/// Passed to the helper below, which cannot use the `#[cfg(test)]`-only
/// `LOCAL_STORE_CONFIG_DIR` because an integration test links the library
/// without that flag.
const HELPER_ROOT: &str = "LOCAL_STORE_LOCK_TEST_ROOT";

fn scratch(name: &str) -> PathBuf {
    let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).expect("scratch config root");
    path
}

/// The shipped binary reads the real platform configuration variables, so a
/// private root is given through those rather than a test-only override.
fn add_app(root: &Path, name: &str, port: u16) -> Child {
    Command::new(CLI)
        .args(["add", name, "--url", &format!("http://127.0.0.1:{port}")])
        .env("APPDATA", root)
        .env("XDG_CONFIG_HOME", root)
        .env("HOME", root)
        .env("PATH", "")
        .spawn()
        .expect("the CLI must start")
}

#[test]
fn concurrent_processes_do_not_lose_each_other_s_registry_writes() {
    let root = scratch("registry-contention");
    // Started together so their read-modify-write windows overlap. Without a
    // cross-process lock this is exactly where entries disappear.
    let mut children: Vec<Child> = (0..8)
        .map(|index| add_app(&root, &format!("app-{index}"), 9000 + index))
        .collect();

    for (index, child) in children.iter_mut().enumerate() {
        let status = child.wait().expect("the CLI must exit");
        assert!(status.success(), "add app-{index} failed: {status}");
    }

    let registry = storage::load_registry_v2_at(&root).expect("registry must be readable");
    let mut ids: Vec<_> = registry.apps.iter().map(|app| app.id.as_str()).collect();
    ids.sort_unstable();
    let expected: Vec<String> = (0..8).map(|index| format!("app-{index}")).collect();
    assert_eq!(
        ids,
        expected.iter().map(String::as_str).collect::<Vec<_>>(),
        "a concurrent write was lost"
    );
}

#[test]
fn a_second_process_waits_rather_than_writing_over_the_first() {
    let root = scratch("registry-exclusion");
    // Seed the registry so the directory, and therefore the lock, exists.
    assert!(add_app(&root, "seed", 8999).wait().unwrap().success());

    let held = storage::lock_registry_at(&root).expect("first holder takes the lock");
    let mut blocked = add_app(&root, "waiter", 8998);

    // It must not sail past the held lock and write.
    std::thread::sleep(Duration::from_millis(600));
    assert!(
        blocked.try_wait().unwrap().is_none(),
        "a second process wrote while the registry was locked"
    );

    drop(held);
    assert!(
        blocked.wait().unwrap().success(),
        "the waiting process should proceed once the lock is released"
    );
    let registry = storage::load_registry_v2_at(&root).unwrap();
    assert_eq!(registry.apps.len(), 2, "both writes must survive");
}

#[test]
fn a_lock_held_by_a_process_that_dies_is_released() {
    let root = scratch("registry-crash");
    let mut holder = Command::new(std::env::current_exe().expect("test binary path"))
        .args([
            "--exact",
            "helper_holds_the_registry_lock_until_killed",
            "--ignored",
            "--nocapture",
        ])
        .env(HELPER_ROOT, &root)
        .spawn()
        .expect("helper must start");

    // Wait until the helper actually owns the lock.
    let deadline = Instant::now() + Duration::from_secs(30);
    while storage::lock_registry_at(&root).is_ok() {
        assert!(Instant::now() < deadline, "helper never took the lock");
        std::thread::sleep(Duration::from_millis(50));
    }

    // Killed outright: no unlock, no destructors, no clean exit.
    holder.kill().expect("helper must be killable");
    holder.wait().expect("helper must be reaped");

    // The operating system releases file locks when the holder dies, which is
    // what stops one crashed process from wedging the registry permanently.
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        if storage::lock_registry_at(&root).is_ok() {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "the lock outlived the process that held it"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// Not a test: a helper process for the case above. Ignored so ordinary runs
/// skip it, and only ever invoked by name with [`HELPER_ROOT`] set.
#[test]
#[ignore]
fn helper_holds_the_registry_lock_until_killed() {
    let Ok(root) = std::env::var(HELPER_ROOT) else {
        return;
    };
    let _lock = storage::lock_registry_at(Path::new(&root)).expect("helper takes the lock");
    println!("locked");
    std::thread::sleep(Duration::from_secs(120));
}
