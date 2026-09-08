use local_store::runtime::{CommandSpec, ProcessError, ProcessOutput, ProcessRunner};
use local_store::{recovery, storage};
use std::path::PathBuf;

struct DockerLabels(std::sync::Mutex<Vec<String>>);
impl ProcessRunner for DockerLabels {
    fn run_cancellable(
        &self,
        spec: &CommandSpec,
        _: &local_store::runtime::CancelToken,
    ) -> Result<ProcessOutput, ProcessError> {
        self.run(spec)
    }
    fn run(&self, spec: &CommandSpec) -> Result<ProcessOutput, ProcessError> {
        assert_eq!(spec.program, "docker");
        assert!(spec.args[1] == "ls" || spec.args[1] == "inspect");
        assert!(spec.timeout <= std::time::Duration::from_secs(30));
        Ok(ProcessOutput {
            success: true,
            stdout: self.0.lock().unwrap().remove(0),
            stderr: String::new(),
            truncated: false,
        })
    }
}

#[test]
fn ownership_requires_all_compose_labels_and_exact_paths() {
    let root = root("labels");
    let project = root.join("local-store/apps/memos");
    std::fs::create_dir_all(&project).unwrap();
    std::fs::write(project.join("compose.yaml"), b"fixture").unwrap();
    let mut candidate = recovery::inspect_at(&root).unwrap().remove(0);
    let labels = serde_json::json!({
        "com.docker.compose.project":"local-store-memos",
        "com.docker.compose.service":"memos",
        "com.docker.compose.oneoff":"False",
        "com.docker.compose.project.config_files":project.join("compose.yaml"),
        "com.docker.compose.project.working_dir":project,
    });
    let verify = |candidate: &mut recovery::RecoveryCandidate, labels: serde_json::Value| {
        recovery::verify_with(
            candidate,
            &DockerLabels(std::sync::Mutex::new(vec![
                "a".repeat(64),
                labels.to_string(),
            ])),
        )
        .unwrap();
    };
    verify(&mut candidate, labels.clone());
    assert!(candidate.docker_ownership_verified);
    for key in [
        "com.docker.compose.project",
        "com.docker.compose.service",
        "com.docker.compose.oneoff",
        "com.docker.compose.project.config_files",
        "com.docker.compose.project.working_dir",
    ] {
        let mut wrong = labels.clone();
        wrong[key] = "wrong".into();
        verify(&mut candidate, wrong);
        assert!(!candidate.docker_ownership_verified, "accepted wrong {key}");
        assert_eq!(
            candidate.ownership_status,
            recovery::OwnershipStatus::Mismatch
        );
    }
    recovery::verify_with(
        &mut candidate,
        &DockerLabels(std::sync::Mutex::new(vec![String::new()])),
    )
    .unwrap();
    assert_eq!(
        candidate.ownership_status,
        recovery::OwnershipStatus::NoContainers
    );
    assert!(recovery::verify_with(
        &mut candidate,
        &DockerLabels(std::sync::Mutex::new(vec!["--malformed-id".into()]))
    )
    .is_err());
    assert!(!candidate.docker_ownership_verified);
}

fn root(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "recovery-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

#[test]
fn missing_config_is_not_created_even_by_cli() {
    let root = root("missing");
    assert!(recovery::inspect_at(&root).unwrap().is_empty());
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_local-store"))
        .args(["recovery", "--json"])
        .env("APPDATA", &root)
        .env("XDG_CONFIG_HOME", &root)
        .env("PATH", "")
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output);
    assert_eq!(String::from_utf8(output.stdout).unwrap().trim(), "[]");
    assert!(!root.exists());
}

#[test]
fn retained_files_are_reported_without_claiming_docker_ownership() {
    let root = root("retained");
    let project = root.join("local-store/apps/memos");
    std::fs::create_dir_all(&project).unwrap();
    let compose = project.join("compose.yaml");
    std::fs::write(&compose, b"retained content must not execute").unwrap();
    let candidates = recovery::inspect_at(&root).unwrap();
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].recipe_id, "memos");
    assert!(!candidates[0].docker_ownership_verified);
    assert_eq!(
        std::fs::read(&compose).unwrap(),
        b"retained content must not execute"
    );
    assert!(!storage::registry_v2_path_for_root(&root).exists());
    let app = local_store::model::InstalledApp {
        id: "memos".into(),
        catalog_id: None,
        display_name: "Memos".into(),
        launch_url: "http://localhost:5230".into(),
        icon_path: None,
        runtime: local_store::model::RuntimeSpec::External,
        created_at_unix: 1,
        updated_at_unix: 1,
    };
    storage::save_registry_v2_at(&root, &storage::RegistryV2::new(vec![app])).unwrap();
    assert!(recovery::inspect_at(&root).unwrap().is_empty());
    std::fs::write(storage::registry_v2_path_for_root(&root), b"broken").unwrap();
    assert!(
        recovery::inspect_at(&root).is_err(),
        "corrupt registry cannot imply absence"
    );
}
