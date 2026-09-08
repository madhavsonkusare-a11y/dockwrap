//! Real process contention; no Docker or user configuration required.
use std::{
    path::Path,
    process::{Child, Command},
    time::{Duration, Instant},
};

struct Holder(Child);
impl Drop for Holder {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn helper(root: &Path, id: &str, hold: bool) -> Command {
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args(["--exact", "operation_helper", "--ignored", "--nocapture"])
        .env("APPDATA", root)
        .env("XDG_CONFIG_HOME", root)
        .env("LOCAL_STORE_OPERATION_TEST_ID", id)
        .env(
            "LOCAL_STORE_OPERATION_TEST_HOLD",
            if hold { "yes" } else { "no" },
        );
    command
}

#[test]
fn competing_processes_are_refused_and_a_killed_holder_releases_the_lock() {
    let root = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "operation-contention-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&root).unwrap();
    let mut holder = Holder(helper(&root, "memos", true).spawn().unwrap());
    let deadline = Instant::now() + Duration::from_secs(20);
    while !root.join("ready").exists() {
        assert!(
            holder.0.try_wait().unwrap().is_none(),
            "holder exited early"
        );
        assert!(Instant::now() < deadline, "holder did not become ready");
        std::thread::sleep(Duration::from_millis(20));
    }
    let started = Instant::now();
    let blocked = helper(&root, "memos", false).output().unwrap();
    assert!(
        !blocked.status.success(),
        "second process acquired the same lock"
    );
    assert!(
        started.elapsed() < Duration::from_secs(5),
        "busy must fail promptly"
    );
    assert!(
        helper(&root, "n8n", false).status().unwrap().success(),
        "different apps must remain independent"
    );
    // The real install entry point must refuse before trying to invoke Docker.
    let cli = Command::new(env!("CARGO_BIN_EXE_local-store"))
        .args(["install", "memos"])
        .env("APPDATA", &root)
        .env("XDG_CONFIG_HOME", &root)
        .env("PATH", "")
        .output()
        .unwrap();
    assert!(!cli.status.success());
    assert!(String::from_utf8_lossy(&cli.stderr).contains("Another process"));
    assert!(!root.join("local-store/apps/memos").exists());
    let project_dir = root.join("local-store/apps/memos");
    let app = local_store::model::InstalledApp {
        id: "memos".into(),
        catalog_id: None,
        display_name: "Memos".into(),
        launch_url: "http://localhost:5230".into(),
        icon_path: None,
        runtime: local_store::model::RuntimeSpec::Compose {
            project_name: "local-store-memos".into(),
            compose_file: project_dir.join("compose.yaml"),
            project_dir,
        },
        created_at_unix: 1,
        updated_at_unix: 1,
    };
    local_store::storage::save_registry_v2_at(
        &root,
        &local_store::storage::RegistryV2::new(vec![app]),
    )
    .unwrap();
    for action in ["start", "stop", "uninstall"] {
        let output = Command::new(env!("CARGO_BIN_EXE_local-store"))
            .args([action, "memos"])
            .env("APPDATA", &root)
            .env("XDG_CONFIG_HOME", &root)
            .env("PATH", "")
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("Another process"),
            "{action}: {:?}",
            output
        );
        assert_eq!(
            local_store::storage::load_registry_v2_at(&root)
                .unwrap()
                .apps
                .len(),
            1
        );
    }
    holder.0.kill().unwrap();
    holder.0.wait().unwrap();
    assert!(
        helper(&root, "memos", false).status().unwrap().success(),
        "crashed holder wedged the lock"
    );
    assert!(
        root.join("local-store/operation-locks/memos.lock").exists(),
        "lock sidecar must never be unlinked"
    );
}

#[test]
#[ignore]
fn operation_helper() {
    let Ok(id) = std::env::var("LOCAL_STORE_OPERATION_TEST_ID") else {
        return;
    };
    let _held = local_store::runtime::lock_operation(&id).expect("operation lock");
    if std::env::var("LOCAL_STORE_OPERATION_TEST_HOLD").unwrap() == "yes" {
        std::fs::write(
            std::path::PathBuf::from(std::env::var("APPDATA").unwrap()).join("ready"),
            b"ready",
        )
        .unwrap();
        std::thread::sleep(Duration::from_secs(60));
    }
}
