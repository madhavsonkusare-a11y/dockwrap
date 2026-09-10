//! Finishing an interrupted install instead of clearing it.
//!
//! Adoption is the mutation that writes a registry entry, so the thing it must
//! never do is claim an app is installed when it is not. Every refusal below
//! is a way it could: an app that never answers, containers somebody else
//! owns, an app already properly installed, an app another operation is
//! holding, or a Docker command that failed. It also has to register the
//! address the retained files actually publish, not the one the app prefers
//! today.
use local_store::runtime::{CommandSpec, HealthProbe, ProcessError, ProcessOutput, ProcessRunner};
use local_store::{error::ErrorCode, recovery};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;

struct Docker {
    replies: Mutex<Vec<String>>,
    calls: Mutex<Vec<Vec<String>>>,
    fail_on: Option<String>,
}

impl Docker {
    fn new(replies: Vec<String>) -> Self {
        Self {
            replies: Mutex::new(replies),
            calls: Mutex::new(Vec::new()),
            fail_on: None,
        }
    }
    fn failing(replies: Vec<String>, on: &str) -> Self {
        Self {
            replies: Mutex::new(replies),
            calls: Mutex::new(Vec::new()),
            fail_on: Some(on.to_owned()),
        }
    }
    fn asked(&self) -> Vec<Vec<String>> {
        self.calls.lock().unwrap().clone()
    }
}

impl ProcessRunner for Docker {
    fn run_cancellable(
        &self,
        spec: &CommandSpec,
        _: &local_store::runtime::CancelToken,
    ) -> Result<ProcessOutput, ProcessError> {
        self.run(spec)
    }
    fn run(&self, spec: &CommandSpec) -> Result<ProcessOutput, ProcessError> {
        assert_eq!(spec.program, "docker");
        self.calls.lock().unwrap().push(spec.args.clone());
        let failed = self
            .fail_on
            .as_ref()
            .is_some_and(|needle| spec.args.iter().any(|arg| arg == needle));
        let mut replies = self.replies.lock().unwrap();
        let stdout = if replies.is_empty() {
            String::new()
        } else {
            replies.remove(0)
        };
        Ok(ProcessOutput {
            success: !failed,
            stdout,
            stderr: String::new(),
            truncated: false,
        })
    }
}

/// Answers whichever way the test needs, and records what it was asked about.
struct Probe {
    answers: bool,
    asked: Mutex<Vec<String>>,
}

impl Probe {
    fn answering() -> Self {
        Self {
            answers: true,
            asked: Mutex::new(Vec::new()),
        }
    }
    fn silent() -> Self {
        Self {
            answers: false,
            asked: Mutex::new(Vec::new()),
        }
    }
    fn urls(&self) -> Vec<String> {
        self.asked.lock().unwrap().clone()
    }
}

impl HealthProbe for Probe {
    fn ready(&self, url: &str) -> bool {
        self.asked.lock().unwrap().push(url.to_owned());
        self.answers
    }
}

/// The operation lock and the config-root environment are both process-wide,
/// so these tests take turns.
static SERIAL: Mutex<()> = Mutex::new(());

fn serially() -> std::sync::MutexGuard<'static, ()> {
    SERIAL
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn root(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "adopt-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

/// What an interrupted install leaves: a project directory with a Compose file
/// that publishes a port, and no registry entry naming it.
///
/// The published port is deliberately *not* the one the recipe prefers. An
/// interrupted install may have stepped past a busy port, and adopting it
/// under the preferred address would register a link that goes nowhere.
fn retained(name: &str, host_port: u16) -> (PathBuf, PathBuf) {
    let root = root(name);
    let project = root.join("local-store/apps/memos");
    std::fs::create_dir_all(project.join("data")).unwrap();
    std::fs::write(
        project.join("compose.yaml"),
        format!("services:\n  memos:\n    ports:\n      - \"127.0.0.1:{host_port}:5230\"\n")
            .as_bytes(),
    )
    .unwrap();
    std::fs::write(project.join("data/notes.db"), b"a person's notes").unwrap();
    std::env::set_var("APPDATA", &root);
    std::env::set_var("XDG_CONFIG_HOME", &root);
    (root, project)
}

fn owning_labels(project: &PathBuf) -> String {
    serde_json::json!({
        "com.docker.compose.project": "local-store-memos",
        "com.docker.compose.service": "memos",
        "com.docker.compose.oneoff": "False",
        "com.docker.compose.project.config_files": project.join("compose.yaml"),
        "com.docker.compose.project.working_dir": project,
    })
    .to_string()
}

fn registry_ids(root: &Path) -> Vec<String> {
    local_store::storage::load_registry_v2_at(root)
        .map(|registry| registry.apps.into_iter().map(|app| app.id).collect())
        .unwrap_or_default()
}

#[test]
fn adoption_registers_the_app_at_the_address_its_files_publish() {
    let _serial = serially();
    // 5231, not the 5230 this recipe prefers.
    let (root, project) = retained("registers", 5231);
    let id = "a".repeat(64);
    let docker = Docker::new(vec![
        id.clone(),              // verify: containers under the project label
        owning_labels(&project), // verify: their labels
        String::new(),           // compose up -d
        id,                      // count what the project owns
    ]);
    let probe = Probe::answering();
    let done = recovery::adopt_with(&docker, &probe, "memos", Duration::from_secs(60))
        .expect("adoption should succeed");

    assert_eq!(done.launch_url, "http://localhost:5231");
    assert_eq!(done.containers, 1);
    assert_eq!(probe.urls(), vec!["http://localhost:5231".to_owned()]);

    // The app is in the registry, at that address, and its data is untouched.
    let registry = local_store::storage::load_registry_v2_at(&root).unwrap();
    let app = registry.apps.iter().find(|app| app.id == "memos").unwrap();
    assert_eq!(app.launch_url, "http://localhost:5231");
    assert_eq!(app.display_name, "Memos");
    assert!(app.is_managed());
    assert!(project.join("data/notes.db").is_file());

    // It was started from the retained file, under its own project name.
    let up = docker
        .asked()
        .into_iter()
        .find(|args| args.contains(&"up".to_owned()))
        .expect("compose up was never run");
    assert!(up.contains(&"-p".to_owned()) && up.contains(&"local-store-memos".to_owned()));
    assert!(up.contains(&"-d".to_owned()));
}

#[test]
fn an_app_that_never_answers_is_not_registered() {
    let _serial = serially();
    let (root, project) = retained("silent", 5230);
    let id = "b".repeat(64);
    let docker = Docker::new(vec![id, owning_labels(&project), String::new()]);
    let probe = Probe::silent();
    let error =
        recovery::adopt_with(&docker, &probe, "memos", Duration::from_millis(300)).unwrap_err();
    assert_eq!(error.code, ErrorCode::TimedOut);

    // Nothing registered, and the files are still there to recover instead.
    assert!(registry_ids(&root).is_empty());
    assert!(project.join("compose.yaml").is_file());
    assert!(project.join("data/notes.db").is_file());
}

#[test]
fn containers_that_do_not_match_the_retained_files_are_not_adopted() {
    let _serial = serially();
    let (root, project) = retained("mismatch", 5230);
    let elsewhere = serde_json::json!({
        "com.docker.compose.project": "local-store-memos",
        "com.docker.compose.service": "memos",
        "com.docker.compose.oneoff": "False",
        "com.docker.compose.project.config_files": project.join("other.yaml"),
        "com.docker.compose.project.working_dir": project,
    })
    .to_string();
    let docker = Docker::new(vec!["c".repeat(64), elsewhere]);
    let probe = Probe::answering();
    let error =
        recovery::adopt_with(&docker, &probe, "memos", Duration::from_secs(60)).unwrap_err();
    assert_eq!(error.code, ErrorCode::UnsafePath);

    assert!(registry_ids(&root).is_empty());
    assert!(
        !docker
            .asked()
            .iter()
            .any(|args| args.contains(&"up".to_owned())),
        "Docker was asked to start containers it does not own"
    );
    assert!(probe.urls().is_empty(), "an unowned app was probed");
}

#[test]
fn an_app_that_is_properly_installed_has_nothing_to_adopt() {
    let _serial = serially();
    let (root, project) = retained("installed", 5230);
    let registry = local_store::storage::RegistryV2::new(vec![local_store::model::InstalledApp {
        id: "memos".into(),
        catalog_id: None,
        display_name: "Memos".into(),
        launch_url: "http://localhost:5230".into(),
        icon_path: None,
        runtime: local_store::model::RuntimeSpec::Compose {
            project_name: "local-store-memos".into(),
            project_dir: project.clone(),
            compose_file: project.join("compose.yaml"),
        },
        created_at_unix: 1,
        updated_at_unix: 1,
    }]);
    local_store::storage::save_registry_v2_at(&root, &registry).unwrap();

    let docker = Docker::new(vec![]);
    let probe = Probe::answering();
    let error =
        recovery::adopt_with(&docker, &probe, "memos", Duration::from_secs(60)).unwrap_err();
    assert_eq!(error.code, ErrorCode::NotFound);
    assert!(docker.asked().is_empty(), "Docker was touched");
}

#[test]
fn a_failed_start_registers_nothing() {
    let _serial = serially();
    let (root, project) = retained("up-fails", 5230);
    let docker = Docker::failing(
        vec!["d".repeat(64), owning_labels(&project), String::new()],
        "up",
    );
    let probe = Probe::answering();
    let error =
        recovery::adopt_with(&docker, &probe, "memos", Duration::from_secs(60)).unwrap_err();
    assert_eq!(error.code, ErrorCode::ProcessFailed);
    assert!(registry_ids(&root).is_empty());
    assert!(
        probe.urls().is_empty(),
        "a container that never started was probed"
    );
}

#[test]
fn a_busy_app_is_refused_before_anything_is_read() {
    let _serial = serially();
    let (_root, _project) = retained("busy", 5230);
    let held = local_store::runtime::lock_operation("memos").expect("first lock");
    let docker = Docker::new(vec![]);
    let probe = Probe::answering();
    let error =
        recovery::adopt_with(&docker, &probe, "memos", Duration::from_secs(60)).unwrap_err();
    assert_eq!(error.code, ErrorCode::OperationBusy);
    assert!(docker.asked().is_empty(), "Docker was touched while busy");
    drop(held);
}

/// Retained files with no published port cannot be adopted: there would be no
/// address to record, and an entry in My Apps that opens nothing is worse than
/// no entry at all.
#[test]
fn retained_files_that_publish_no_address_are_refused() {
    let _serial = serially();
    let root = root("no-port");
    let project = root.join("local-store/apps/memos");
    std::fs::create_dir_all(&project).unwrap();
    std::fs::write(project.join("compose.yaml"), b"services:\n  memos: {}\n").unwrap();
    std::env::set_var("APPDATA", &root);
    std::env::set_var("XDG_CONFIG_HOME", &root);

    let docker = Docker::new(vec![String::new()]);
    let probe = Probe::answering();
    let error =
        recovery::adopt_with(&docker, &probe, "memos", Duration::from_secs(60)).unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidInput);
    assert!(registry_ids(&root).is_empty());
    assert!(
        !docker
            .asked()
            .iter()
            .any(|args| args.contains(&"up".to_owned())),
        "an app with no address was started anyway"
    );
}
