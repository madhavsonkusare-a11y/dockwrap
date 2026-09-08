//! Opt-in real Docker test. Requires the Memos image; never run in offline CI.
use local_store::{
    runtime::{CommandSpec, HealthProbe, HttpHealthProbe, ProcessRunner, SystemProcessRunner},
    storage,
};
use std::{
    path::Path,
    process::{Child, Command, Output},
    time::{Duration, Instant},
};

struct OwnedChild(Child);
impl Drop for OwnedChild {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
fn docker(args: &[&str]) -> String {
    let output = SystemProcessRunner
        .run(&CommandSpec::new(
            "docker",
            args.iter().map(|value| (*value).to_owned()).collect(),
            None,
            Duration::from_secs(180),
        ))
        .unwrap();
    assert!(output.success, "Docker failed: {}", output.stderr);
    output.stdout.trim().into()
}
fn cli(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_local-store"))
        .args(args)
        .env("APPDATA", root)
        .env("XDG_CONFIG_HOME", root)
        .output()
        .unwrap()
}
fn success(output: Output) {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
#[ignore = "starts real Docker containers; run explicitly with --ignored --nocapture"]
fn interrupted_install_preserves_data_and_can_be_recovered() {
    assert_eq!(
        std::env::var("LOCAL_STORE_RUN_DOCKER_TEST").as_deref(),
        Ok("1")
    );
    // Never touch an existing recipe deployment. Require a cached image so
    // provisioning cannot be held up by an unbounded registry download here.
    docker(&[
        "image",
        "inspect",
        "neosmemo/memos:0.30.0",
        "--format",
        "{{.Id}}",
    ]);
    assert!(docker(&[
        "container",
        "ls",
        "-aq",
        "--filter",
        "name=^/local-store-memos$"
    ])
    .is_empty());
    for resource in ["container", "network", "volume"] {
        assert!(docker(&[
            resource,
            "ls",
            "-q",
            "--filter",
            "label=com.docker.compose.project=local-store-memos"
        ])
        .is_empty());
    }
    assert!(!docker(&["network", "ls", "--format", "{{.Name}}"])
        .lines()
        .any(|name| name.starts_with("local-store-memos")));
    drop(std::net::TcpListener::bind("127.0.0.1:5230").expect("recipe port must be unused"));
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".cache")
        .join(format!(
            "install-interruption-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
    std::fs::create_dir_all(&root).unwrap();
    println!("Private recovery root: {}", root.display());
    // Hold the registry commit while the actual CLI brings up the container.
    // This creates a deterministic crash window, without production test hooks.
    let registry_guard = storage::lock_registry_at(&root).unwrap();
    let mut install = OwnedChild(
        Command::new(env!("CARGO_BIN_EXE_local-store"))
            .args(["install", "memos"])
            .env("APPDATA", &root)
            .env("XDG_CONFIG_HOME", &root)
            .spawn()
            .unwrap(),
    );
    let deadline = Instant::now() + Duration::from_secs(60);
    while !HttpHealthProbe.ready("http://127.0.0.1:5230") {
        assert!(
            install.0.try_wait().unwrap().is_none(),
            "install exited before interruption"
        );
        assert!(Instant::now() < deadline, "container never became healthy");
        std::thread::sleep(Duration::from_millis(100));
    }
    let project = root.join("local-store/apps/memos");
    assert_eq!(
        project
            .canonicalize()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap(),
        root.canonicalize().unwrap()
    );
    let marker = project.join("data/interruption-marker");
    std::fs::write(&marker, b"preserve-on-recovery").unwrap();
    install.0.kill().unwrap();
    install.0.wait().unwrap();
    drop(registry_guard);
    let inspection = cli(&root, &["recovery", "--json", "--docker"]);
    assert!(inspection.status.success(), "recovery inspection failed");
    let candidates: serde_json::Value = serde_json::from_slice(&inspection.stdout).unwrap();
    assert_eq!(candidates[0]["recipe_id"], "memos");
    assert_eq!(candidates[0]["docker_ownership_verified"], true);
    assert_eq!(candidates[0]["ownership_status"], "verified");
    assert!(storage::load_or_migrate_registry_at(&root)
        .unwrap()
        .apps
        .is_empty());
    assert!(
        HttpHealthProbe.ready("http://127.0.0.1:5230"),
        "crash should leave daemon-owned container running"
    );
    let retry = cli(&root, &["install", "memos"]);
    assert!(!retry.status.success());
    assert!(String::from_utf8_lossy(&retry.stderr).contains("already in use"));
    assert!(String::from_utf8_lossy(&retry.stderr).contains("Setup files remain at"));
    // Recovery is now the product's own action rather than a Compose command
    // this test types for it. It takes the app's operation lock, re-derives the
    // candidate and re-verifies Docker ownership before removing anything.
    let recovered = cli(&root, &["recover", "memos"]);
    assert!(
        recovered.status.success(),
        "recover failed: {}",
        String::from_utf8_lossy(&recovered.stderr)
    );
    let said = String::from_utf8_lossy(&recovered.stdout).into_owned();
    assert!(said.contains("Removed 1 container"), "{said}");
    assert!(said.contains("setup files and data were kept"), "{said}");
    // Keeping data is the default, and it has to be true of the real file.
    assert_eq!(std::fs::read(&marker).unwrap(), b"preserve-on-recovery");
    assert!(
        docker(&[
            "container",
            "ls",
            "-aq",
            "--filter",
            "label=com.docker.compose.project=local-store-memos"
        ])
        .is_empty(),
        "recover left containers behind"
    );
    success(cli(&root, &["install", "memos"]));
    // With the app now installed, the same command must refuse: this is no
    // longer wreckage to clear.
    let refused = cli(&root, &["recover", "memos"]);
    assert!(!refused.status.success());
    assert!(String::from_utf8_lossy(&refused.stderr).contains("no retained setup files"));
    assert_eq!(std::fs::read(&marker).unwrap(), b"preserve-on-recovery");
    success(cli(&root, &["uninstall", "memos", "--delete-data"]));
    assert!(!project.exists());
    assert!(storage::load_or_migrate_registry_at(&root)
        .unwrap()
        .apps
        .is_empty());
    for resource in ["container", "network", "volume"] {
        assert!(docker(&[
            resource,
            "ls",
            "-q",
            "--filter",
            "label=com.docker.compose.project=local-store-memos"
        ])
        .is_empty());
    }
    std::fs::write(root.join("result.txt"), "PASS: killed healthy pre-commit CLI; orphan detected; port refusal; the product's own recover action removed the orphan and preserved data; reinstall and deletion passed\n").unwrap();
    println!("PASS: interruption recovery; evidence {}", root.display());
}
