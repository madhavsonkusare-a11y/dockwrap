//! Docker Compose runtime behind testable process and health-check abstractions.
use crate::{
    model::{InstalledApp, RuntimeSpec},
    recipes::Recipe,
    storage,
};
use serde::Serialize;
use std::{
    fs,
    io::{Read, Write},
    net::{TcpStream, ToSocketAddrs},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone, PartialEq)]
pub struct CommandSpec {
    pub program: String,
    pub args: Vec<String>,
    pub cwd: Option<PathBuf>,
}
impl CommandSpec {
    fn docker(args: Vec<String>, cwd: Option<PathBuf>) -> Self {
        Self {
            program: "docker".into(),
            args,
            cwd,
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct ProcessOutput {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}
pub trait ProcessRunner: Send + Sync {
    fn run(&self, command: &CommandSpec) -> Result<ProcessOutput, String>;
}
pub struct SystemProcessRunner;
impl ProcessRunner for SystemProcessRunner {
    fn run(&self, spec: &CommandSpec) -> Result<ProcessOutput, String> {
        let mut command = Command::new(&spec.program);
        command
            .args(&spec.args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if let Some(cwd) = &spec.cwd {
            command.current_dir(cwd);
        }
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000);
        }
        let output = command
            .output()
            .map_err(|error| format!("failed to run {}: {error}", spec.program))?;
        Ok(ProcessOutput {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })
    }
}
pub trait HealthProbe: Send + Sync {
    fn ready(&self, url: &str) -> bool;
}
pub struct HttpHealthProbe;
impl HealthProbe for HttpHealthProbe {
    fn ready(&self, url: &str) -> bool {
        let Ok(parsed) = tauri::Url::parse(url) else {
            return false;
        };
        if parsed.scheme() != "http" {
            return false;
        }
        let Some(host) = parsed.host_str() else {
            return false;
        };
        let port = parsed.port().unwrap_or(80);
        let Ok(mut addresses) = (host, port).to_socket_addrs() else {
            return false;
        };
        let Some(address) = addresses.next() else {
            return false;
        };
        let Ok(mut stream) = TcpStream::connect_timeout(&address, Duration::from_secs(2)) else {
            return false;
        };
        let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
        let target = if parsed.path().is_empty() {
            "/"
        } else {
            parsed.path()
        };
        if write!(
            stream,
            "GET {target} HTTP/1.0\r\nHost: {host}\r\nConnection: close\r\n\r\n"
        )
        .is_err()
        {
            return false;
        }
        let mut status = [0_u8; 64];
        let Ok(read) = stream.read(&mut status) else {
            return false;
        };
        let line = String::from_utf8_lossy(&status[..read]);
        line.starts_with("HTTP/1.0 2")
            || line.starts_with("HTTP/1.1 2")
            || line.starts_with("HTTP/1.0 3")
            || line.starts_with("HTTP/1.1 3")
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DoctorCheck {
    pub id: &'static str,
    pub label: &'static str,
    pub ok: bool,
    pub detail: String,
}
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DoctorReport {
    pub ready: bool,
    pub checks: Vec<DoctorCheck>,
}
pub fn doctor_with(runner: &dyn ProcessRunner) -> DoctorReport {
    let specs = [
        (
            "docker",
            "Docker engine",
            CommandSpec::docker(
                vec![
                    "version".into(),
                    "--format".into(),
                    "{{.Server.Version}}".into(),
                ],
                None,
            ),
        ),
        (
            "compose",
            "Docker Compose",
            CommandSpec::docker(
                vec!["compose".into(), "version".into(), "--short".into()],
                None,
            ),
        ),
    ];
    let checks = specs
        .into_iter()
        .map(|(id, label, spec)| match runner.run(&spec) {
            Ok(output) if output.success => DoctorCheck {
                id,
                label,
                ok: true,
                detail: output.stdout.trim().to_owned(),
            },
            Ok(output) => DoctorCheck {
                id,
                label,
                ok: false,
                detail: concise_error(&output),
            },
            Err(error) => DoctorCheck {
                id,
                label,
                ok: false,
                detail: error,
            },
        })
        .collect::<Vec<_>>();
    DoctorReport {
        ready: checks.iter().all(|check| check.ok),
        checks,
    }
}
pub fn doctor() -> DoctorReport {
    doctor_with(&SystemProcessRunner)
}

fn compose_command(app: &InstalledApp, args: &[&str]) -> Result<CommandSpec, String> {
    match &app.runtime {
        RuntimeSpec::Compose {
            project_name,
            project_dir,
            compose_file,
        } => {
            let mut all = vec![
                "compose".into(),
                "-f".into(),
                compose_file.to_string_lossy().into_owned(),
                "-p".into(),
                project_name.clone(),
            ];
            all.extend(args.iter().map(|arg| (*arg).to_owned()));
            Ok(CommandSpec::docker(all, Some(project_dir.clone())))
        }
        RuntimeSpec::External => {
            Err("This is a connected app; Local Store does not manage its server.".into())
        }
    }
}
fn checked_run(
    runner: &dyn ProcessRunner,
    spec: &CommandSpec,
    action: &str,
) -> Result<ProcessOutput, String> {
    let output = runner.run(spec)?;
    if output.success {
        Ok(output)
    } else {
        Err(format!("{action} failed: {}", concise_error(&output)))
    }
}
fn concise_error(output: &ProcessOutput) -> String {
    let value = if output.stderr.trim().is_empty() {
        output.stdout.trim()
    } else {
        output.stderr.trim()
    };
    if value.is_empty() {
        "the command returned an error".into()
    } else {
        value.chars().take(1000).collect()
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AppStatus {
    Connected,
    Running,
    Stopped,
    Error,
}
pub fn status_with(runner: &dyn ProcessRunner, app: &InstalledApp) -> Result<AppStatus, String> {
    if !app.is_managed() {
        return Ok(AppStatus::Connected);
    }
    let output = runner.run(&compose_command(
        app,
        &["ps", "--status", "running", "--quiet"],
    )?)?;
    if !output.success {
        Ok(AppStatus::Error)
    } else if output.stdout.trim().is_empty() {
        Ok(AppStatus::Stopped)
    } else {
        Ok(AppStatus::Running)
    }
}
pub fn start_with(runner: &dyn ProcessRunner, app: &InstalledApp) -> Result<(), String> {
    checked_run(runner, &compose_command(app, &["up", "-d"])?, "Start").map(|_| ())
}
pub fn stop_with(runner: &dyn ProcessRunner, app: &InstalledApp) -> Result<(), String> {
    checked_run(runner, &compose_command(app, &["stop"])?, "Stop").map(|_| ())
}
pub fn logs_with(runner: &dyn ProcessRunner, app: &InstalledApp) -> Result<String, String> {
    let output = checked_run(
        runner,
        &compose_command(app, &["logs", "--tail", "200", "--no-color"])?,
        "Logs",
    )?;
    let mut logs = output.stdout;
    logs.push_str(&output.stderr);
    if logs.len() > 32_768 {
        let minimum = logs.len() - 32_768;
        let start = logs
            .char_indices()
            .find_map(|(index, _)| (index >= minimum).then_some(index))
            .unwrap_or(0);
        logs = logs[start..].to_owned();
    }
    Ok(logs)
}
pub fn start(app: &InstalledApp) -> Result<(), String> {
    start_with(&SystemProcessRunner, app)
}
pub fn stop(app: &InstalledApp) -> Result<(), String> {
    stop_with(&SystemProcessRunner, app)
}
pub fn status(app: &InstalledApp) -> Result<AppStatus, String> {
    status_with(&SystemProcessRunner, app)
}
pub fn logs(app: &InstalledApp) -> Result<String, String> {
    logs_with(&SystemProcessRunner, app)
}

pub fn wait_for_health_with(
    probe: &dyn HealthProbe,
    target: &str,
    timeout: Duration,
) -> Result<(), String> {
    let deadline = Instant::now() + timeout;
    loop {
        if probe.ready(target) {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(format!("Health check timed out for {target}"));
        }
        std::thread::sleep(Duration::from_millis(400));
    }
}
pub fn wait_for_health(target: &str, timeout: Duration) -> Result<(), String> {
    wait_for_health_with(&HttpHealthProbe, target, timeout)
}

pub fn install_recipe_with(
    runner: &dyn ProcessRunner,
    probe: &dyn HealthProbe,
    recipe: &Recipe,
    root: &Path,
    now: u64,
) -> Result<InstalledApp, String> {
    recipe.validate()?;
    let report = doctor_with(runner);
    if !report.ready {
        return Err("Docker and Docker Compose must be running before installation.".into());
    }
    let project_dir = root.join(&recipe.id);
    let created_project = !project_dir.exists();
    if !created_project {
        let allowed = recipe
            .data_directories
            .iter()
            .map(String::as_str)
            .chain(std::iter::once("compose.yaml"))
            .collect::<Vec<_>>();
        for entry in fs::read_dir(&project_dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let name = entry.file_name();
            if !allowed.iter().any(|allowed| name == *allowed) {
                return Err(
                    "The preserved app directory contains unexpected files; review it before reinstalling."
                        .into(),
                );
            }
        }
    }
    fs::create_dir_all(&project_dir).map_err(|e| e.to_string())?;
    let compose_file = project_dir.join("compose.yaml");
    let previous_compose = fs::read(&compose_file).ok();
    let install = (|| {
        fs::write(&compose_file, recipe.compose.as_bytes()).map_err(|e| e.to_string())?;
        for directory in &recipe.data_directories {
            fs::create_dir_all(project_dir.join(directory)).map_err(|e| e.to_string())?;
        }
        let app = InstalledApp {
            id: recipe.id.clone(),
            catalog_id: Some(recipe.catalog_name.clone()),
            display_name: recipe.display_name.clone(),
            launch_url: recipe.launch_url.clone(),
            icon_path: None,
            runtime: RuntimeSpec::Compose {
                project_name: format!("local-store-{}", recipe.id),
                project_dir: project_dir.clone(),
                compose_file: compose_file.clone(),
            },
            created_at_unix: now,
            updated_at_unix: now,
        };
        checked_run(
            runner,
            &compose_command(&app, &["config", "--quiet"])?,
            "Recipe validation",
        )?;
        start_with(runner, &app)?;
        wait_for_health_with(probe, &recipe.health_url, Duration::from_secs(60))?;
        Ok(app)
    })();
    if install.is_err() {
        let fallback = InstalledApp {
            id: recipe.id.clone(),
            catalog_id: None,
            display_name: recipe.display_name.clone(),
            launch_url: recipe.launch_url.clone(),
            icon_path: None,
            runtime: RuntimeSpec::Compose {
                project_name: format!("local-store-{}", recipe.id),
                project_dir: project_dir.clone(),
                compose_file: compose_file.clone(),
            },
            created_at_unix: now,
            updated_at_unix: now,
        };
        let _ = runner.run(&compose_command(&fallback, &["down"]).expect("compose runtime"));
        if created_project {
            let _ = fs::remove_dir_all(&project_dir);
        } else if let Some(previous) = previous_compose {
            let _ = fs::write(&compose_file, previous);
        } else {
            let _ = fs::remove_file(&compose_file);
        }
    }
    install
}
pub fn install_recipe(recipe: &Recipe) -> Result<InstalledApp, String> {
    install_recipe_with(
        &SystemProcessRunner,
        &HttpHealthProbe,
        recipe,
        &storage::managed_apps_root(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs(),
    )
}
pub fn uninstall_with(
    runner: &dyn ProcessRunner,
    app: &InstalledApp,
    delete_data: bool,
) -> Result<(), String> {
    let args = if delete_data {
        &["down", "--volumes"][..]
    } else {
        &["down"][..]
    };
    checked_run(runner, &compose_command(app, args)?, "Uninstall")?;
    if delete_data {
        if let RuntimeSpec::Compose { project_dir, .. } = &app.runtime {
            fs::remove_dir_all(project_dir).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
pub fn uninstall(app: &InstalledApp, delete_data: bool) -> Result<(), String> {
    if delete_data {
        let RuntimeSpec::Compose { project_dir, .. } = &app.runtime else {
            return Err("Connected apps do not have Local Store managed data.".into());
        };
        let root = storage::managed_apps_root();
        let canonical_root = root.canonicalize().map_err(|e| e.to_string())?;
        let canonical_project = project_dir.canonicalize().map_err(|e| e.to_string())?;
        if project_dir != &root.join(&app.id)
            || canonical_project.parent() != Some(canonical_root.as_path())
        {
            return Err(
                "Data deletion is available only for directories created by Local Store.".into(),
            );
        }
    }
    uninstall_with(&SystemProcessRunner, app, delete_data)
}

/// Compatibility path for one release while old CLI registrations are imported.
pub fn boot_and_wait(app: &crate::model::AppDef) -> Result<(), String> {
    if let Some(compose) = &app.compose {
        checked_run(
            &SystemProcessRunner,
            &CommandSpec::docker(
                vec![
                    "compose".into(),
                    "-f".into(),
                    compose.clone(),
                    "up".into(),
                    "-d".into(),
                ],
                None,
            ),
            "Start",
        )?;
    }
    wait_for_health(
        app.health.as_deref().unwrap_or(&app.url),
        Duration::from_secs(60),
    )
}
pub fn launch_browser(url: &str) {
    match crate::windowing::validated_external_url(url) {
        Ok(url) => {
            if let Err(error) = open::that_detached(url) {
                eprintln!("Could not open browser: {error}");
            }
        }
        Err(error) => eprintln!("Could not open browser: {error}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::VecDeque, sync::Mutex};
    struct FakeRunner {
        outputs: Mutex<VecDeque<Result<ProcessOutput, String>>>,
        calls: Mutex<Vec<CommandSpec>>,
    }
    impl FakeRunner {
        fn passing(count: usize) -> Self {
            Self {
                outputs: Mutex::new(
                    (0..count)
                        .map(|_| {
                            Ok(ProcessOutput {
                                success: true,
                                stdout: "1".into(),
                                stderr: String::new(),
                            })
                        })
                        .collect(),
                ),
                calls: Mutex::new(Vec::new()),
            }
        }
    }
    impl ProcessRunner for FakeRunner {
        fn run(&self, spec: &CommandSpec) -> Result<ProcessOutput, String> {
            self.calls.lock().unwrap().push(spec.clone());
            self.outputs
                .lock()
                .unwrap()
                .pop_front()
                .unwrap_or_else(|| Err("unexpected command".into()))
        }
    }
    struct Ready(bool);
    impl HealthProbe for Ready {
        fn ready(&self, _: &str) -> bool {
            self.0
        }
    }
    fn managed(root: &Path) -> InstalledApp {
        InstalledApp {
            id: "memos".into(),
            catalog_id: Some("Memos".into()),
            display_name: "Memos".into(),
            launch_url: "http://localhost:5230".into(),
            icon_path: None,
            runtime: RuntimeSpec::Compose {
                project_name: "local-store-memos".into(),
                project_dir: root.into(),
                compose_file: root.join("compose.yaml"),
            },
            created_at_unix: 1,
            updated_at_unix: 1,
        }
    }
    #[test]
    fn doctor_reports_both_prerequisites() {
        let runner = FakeRunner::passing(2);
        let report = doctor_with(&runner);
        assert!(report.ready);
        assert_eq!(report.checks.len(), 2);
    }
    #[test]
    fn lifecycle_commands_are_bounded_and_project_scoped() {
        let root = PathBuf::from("apps/memos");
        let runner = FakeRunner::passing(4);
        let app = managed(&root);
        start_with(&runner, &app).unwrap();
        stop_with(&runner, &app).unwrap();
        logs_with(&runner, &app).unwrap();
        assert_eq!(status_with(&runner, &app).unwrap(), AppStatus::Running);
        let calls = runner.calls.lock().unwrap();
        assert!(calls
            .iter()
            .all(|call| call.cwd.as_deref() == Some(root.as_path())));
        assert!(calls[2]
            .args
            .ends_with(&["logs", "--tail", "200", "--no-color"].map(str::to_owned)));
    }
    #[test]
    fn failed_install_rolls_back_created_directory() {
        let root =
            std::env::temp_dir().join(format!("local-store-install-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let runner = FakeRunner {
            outputs: Mutex::new(VecDeque::from([
                Ok(ProcessOutput {
                    success: true,
                    stdout: "1".into(),
                    stderr: String::new(),
                }),
                Ok(ProcessOutput {
                    success: true,
                    stdout: "1".into(),
                    stderr: String::new(),
                }),
                Ok(ProcessOutput {
                    success: false,
                    stdout: String::new(),
                    stderr: "invalid".into(),
                }),
                Ok(ProcessOutput {
                    success: true,
                    stdout: String::new(),
                    stderr: String::new(),
                }),
            ])),
            calls: Mutex::new(Vec::new()),
        };
        assert!(install_recipe_with(
            &runner,
            &Ready(true),
            &crate::recipes::recipe("memos").unwrap(),
            &root,
            5
        )
        .is_err());
        assert!(!root.join("memos").exists());
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn successful_install_supports_every_reviewed_recipe() {
        let root =
            std::env::temp_dir().join(format!("local-store-install-ok-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        for recipe in crate::recipes::verified_recipes() {
            let runner = FakeRunner::passing(4);
            let app = install_recipe_with(&runner, &Ready(true), &recipe, &root, 5).unwrap();
            for directory in &recipe.data_directories {
                assert!(root.join(&recipe.id).join(directory).is_dir());
            }
            assert_eq!(app.created_at_unix, 5);
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reinstall_reuses_preserved_data_without_deleting_it_on_failure() {
        let root =
            std::env::temp_dir().join(format!("local-store-reinstall-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let project = root.join("memos");
        fs::create_dir_all(project.join("data")).unwrap();
        fs::write(project.join("data/keep.db"), b"keep").unwrap();
        fs::write(project.join("compose.yaml"), b"previous").unwrap();
        let runner = FakeRunner {
            outputs: Mutex::new(VecDeque::from([
                Ok(ProcessOutput {
                    success: true,
                    stdout: "1".into(),
                    stderr: String::new(),
                }),
                Ok(ProcessOutput {
                    success: true,
                    stdout: "1".into(),
                    stderr: String::new(),
                }),
                Ok(ProcessOutput {
                    success: false,
                    stdout: String::new(),
                    stderr: "invalid".into(),
                }),
                Ok(ProcessOutput {
                    success: true,
                    stdout: String::new(),
                    stderr: String::new(),
                }),
            ])),
            calls: Mutex::new(Vec::new()),
        };
        assert!(install_recipe_with(
            &runner,
            &Ready(true),
            &crate::recipes::recipe("memos").unwrap(),
            &root,
            5
        )
        .is_err());
        assert_eq!(fs::read(project.join("data/keep.db")).unwrap(), b"keep");
        assert_eq!(fs::read(project.join("compose.yaml")).unwrap(), b"previous");
        fs::remove_dir_all(root).unwrap();
    }
}
