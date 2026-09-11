//! Opt-in real multi-service plan installation: a web app beside a database.
//!
//! Phase 2 of the roadmap is met only when "a web app plus database installs
//! without hand-editing Compose and survives restart/removal tests". Every
//! other check on plans looks at rendered text or hands Compose a file to
//! parse; this is the one place where two real containers start, reach each
//! other over the project network, and survive a reinstall over kept data.
//!
//! The reinstall is the part that matters. Postgres writes its password into
//! the volume when it initialises, so a regenerated secret does not fail
//! loudly — it leaves the app holding a password its own data no longer
//! accepts. That is why the password is proven over TCP, before and after,
//! against a row written while the first password was in force.
//!
//! Nothing here runs automatically: `#[ignore]` plus `LOCAL_STORE_RUN_DOCKER_TEST=1`.
//! Every resource is label-scoped to this run's Compose project and removed on
//! the way out, including when an assertion fails part-way; containers that
//! belong to anything else on this computer are counted before and after and
//! must survive untouched.
use local_store::{
    plan::{DeploymentPlan, PlanMount, PlanOverrides, PlanService, PublishedPort},
    runtime::{self, CommandSpec, ProcessOutput, ProcessRunner, SystemProcessRunner},
    setup::{FieldKind, PlanTemplate, SecretSpec, SetupField},
};
use std::{collections::BTreeMap, time::Duration};

/// Both pinned, both amd64/arm64, and neither needs a privileged container.
const WEB_IMAGE: &str = "adminer:4.8.1-standalone";
const DB_IMAGE: &str = "postgres:16.10-alpine";
const DB_USER: &str = "app";
/// The answer a person would type into the setup field.
const DB_NAME: &str = "notebook";
const MARKER: &str = "multi-service-proof";

fn run(args: &[&str], timeout: Duration) -> ProcessOutput {
    SystemProcessRunner
        .run(&CommandSpec::new(
            "docker",
            args.iter().map(|value| (*value).to_owned()).collect(),
            None,
            timeout,
        ))
        .expect("the Docker CLI must be runnable")
}

/// Output is withheld on failure on purpose: several of these commands carry
/// the fixture's database password, and a failing assertion must not print it.
fn docker(args: &[&str]) -> String {
    let output = run(args, Duration::from_secs(180));
    assert!(
        output.success,
        "docker {} failed (output withheld to protect fixture credentials)",
        args.first().copied().unwrap_or("?")
    );
    output.stdout.trim().to_owned()
}

fn label_filter(project: &str) -> String {
    format!("label=com.docker.compose.project={project}")
}

/// Ids of the resources this run's Compose project owns. Containers are listed
/// with `-a`, so a stopped one still counts as present.
fn owned(project: &str, resource: &str) -> Vec<String> {
    let filter = label_filter(project);
    let listing = match resource {
        "container" => docker(&[resource, "ls", "-aq", "--filter", &filter]),
        _ => docker(&[resource, "ls", "-q", "--filter", &filter]),
    };
    listing
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect()
}

fn all_container_ids() -> Vec<String> {
    docker(&["container", "ls", "-aq"])
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect()
}

/// Removes exactly what this run's Compose project owns, including when an
/// assertion fails part-way through. Label-scoped, so everything else on this
/// computer is left alone. Nothing here asserts: a panic while unwinding would
/// abort the process and hide the failure that actually matters.
struct ProjectCleanup {
    project: String,
}

impl Drop for ProjectCleanup {
    fn drop(&mut self) {
        let filter = label_filter(&self.project);
        let short = Duration::from_secs(60);
        let ids = |resource: &str, list: &[&str]| -> Vec<String> {
            let mut args = vec![resource];
            args.extend_from_slice(list);
            args.extend_from_slice(&["--filter", &filter]);
            run(&args, short)
                .stdout
                .split_whitespace()
                .map(str::to_owned)
                .collect()
        };
        for id in ids("container", &["ls", "-aq"]) {
            run(&["rm", "-f", &id], short);
        }
        for id in ids("network", &["ls", "-q"]) {
            run(&["network", "rm", &id], short);
        }
        for id in ids("volume", &["ls", "-q"]) {
            run(&["volume", "rm", "-f", &id], short);
        }
    }
}

/// A web application beside a database, with one typed setup field and one
/// generated secret — the shape a person is meant to install without seeing
/// Compose at all.
fn template(id: &str, host_port: u16) -> PlanTemplate {
    PlanTemplate {
        first_start: None,
        seeds: Vec::new(),
        plan: DeploymentPlan {
            id: id.to_owned(),
            services: vec![
                PlanService {
                    name: "web".into(),
                    image: WEB_IMAGE.into(),
                    digest: None,
                    environment: vec![("ADMINER_DEFAULT_SERVER".into(), "db".into())],
                    companion: None,
                    published: Some(PublishedPort {
                        host: host_port,
                        container: 8080,
                    }),
                    mounts: Vec::new(),
                    depends_on: vec!["db".into()],
                    overrides: PlanOverrides::default(),
                },
                PlanService {
                    name: "db".into(),
                    image: DB_IMAGE.into(),
                    digest: None,
                    environment: vec![
                        ("POSTGRES_USER".into(), DB_USER.into()),
                        ("POSTGRES_DB".into(), "${DB_NAME}".into()),
                        ("POSTGRES_PASSWORD".into(), "${DB_PASSWORD}".into()),
                    ],
                    companion: None,
                    published: None,
                    mounts: vec![PlanMount::Volume {
                        name: "db-data".into(),
                        target: "/var/lib/postgresql/data".into(),
                        read_only: false,
                    }],
                    depends_on: Vec::new(),
                    overrides: PlanOverrides::default(),
                },
            ],
            named_volumes: vec!["db-data".into()],
        },
        fields: vec![SetupField {
            key: "DB_NAME".into(),
            label: "Database name".into(),
            kind: FieldKind::Text {
                min_len: 1,
                max_len: 40,
            },
            required: true,
            default: None,
            sensitive: false,
        }],
        secrets: vec![SecretSpec {
            key: "DB_PASSWORD".into(),
            length: 32,
            format: local_store::setup::SecretFormat::Alphanumeric,
        }],
    }
}

/// Wait for Postgres to finish initialising. The published health probe only
/// tells us the web service answers, which it does whether or not the database
/// behind it is up yet.
fn wait_for_database(container: &str) {
    for _ in 0..60 {
        let probe = run(
            &[
                "exec",
                container,
                "pg_isready",
                "-U",
                DB_USER,
                "-h",
                "127.0.0.1",
            ],
            Duration::from_secs(30),
        );
        if probe.success {
            return;
        }
        std::thread::sleep(Duration::from_secs(2));
    }
    panic!("the database never became ready within two minutes");
}

/// Run one statement over TCP as the application would, so the generated
/// password is actually authenticated rather than bypassed by the local socket
/// trust rule inside the image.
fn psql(container: &str, password: &str, statement: &str) -> String {
    docker(&[
        "exec",
        "-e",
        &format!("PGPASSWORD={password}"),
        container,
        "psql",
        "-h",
        "127.0.0.1",
        "-U",
        DB_USER,
        "-d",
        DB_NAME,
        "-tAc",
        statement,
    ])
}

fn networks_of(container: &str) -> Vec<String> {
    docker(&[
        "inspect",
        container,
        "--format",
        "{{range $name, $_ := .NetworkSettings.Networks}}{{$name}} {{end}}",
    ])
    .split_whitespace()
    .map(str::to_owned)
    .collect()
}

#[test]
#[ignore = "real Docker lifecycle; enable LOCAL_STORE_RUN_DOCKER_TEST=1"]
fn a_web_app_and_its_database_install_together_and_survive_a_reinstall() {
    assert_eq!(
        std::env::var("LOCAL_STORE_RUN_DOCKER_TEST").as_deref(),
        Ok("1")
    );
    // Pulled up front so the install's own 60-second health window measures
    // startup rather than a download.
    for image in [WEB_IMAGE, DB_IMAGE] {
        let pull = run(&["pull", image], Duration::from_secs(600));
        assert!(pull.success, "could not pull {image}");
    }
    let image_ids: Vec<String> = [WEB_IMAGE, DB_IMAGE]
        .iter()
        .map(|image| docker(&["image", "inspect", image, "--format", "{{.Id}}"]))
        .collect();

    let id = format!(
        "multi-proof-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );
    let project_name = format!("local-store-{id}");
    let web_container = format!("{project_name}-web");
    let db_container = format!("{project_name}-db");

    // Nothing may exist under this project or these names before we start; an
    // inherited container would make every assertion below meaningless.
    for resource in ["container", "network", "volume"] {
        assert!(
            owned(&project_name, resource).is_empty(),
            "a {resource} already carries this run's project label"
        );
    }
    for container in [&web_container, &db_container] {
        assert!(
            docker(&[
                "container",
                "ls",
                "-aq",
                "--filter",
                &format!("name=^/{container}$")
            ])
            .is_empty(),
            "a container is already named {container}"
        );
    }
    let unrelated_before = all_container_ids();

    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let host_port = listener.local_addr().unwrap().port();
    drop(listener);
    let mut template = template(&id, host_port);
    template.plan.services[0]
        .overrides
        .healthy_dependencies
        .insert("db".into());
    template.plan.services[1].overrides.healthcheck = Some(local_store::plan::PlanHealthcheck::from_compose(&serde_json::json!({
        "test":"sleep 5; pg_isready -U app -d notebook",
        "interval":"30s", "start_interval":"1s", "start_period":"10s", "timeout":"15s", "retries":5
    })).unwrap());

    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".cache")
        .join(&id);
    std::fs::create_dir_all(&root).unwrap();
    println!("Private multi-service proof root: {}", root.display());
    // One opt-in test in this binary; production storage uses private config.
    std::env::set_var("APPDATA", &root);
    std::env::set_var("XDG_CONFIG_HOME", &root);

    // Registered before the first container exists, so a failure anywhere
    // below still takes this project's resources with it.
    let _cleanup = ProjectCleanup {
        project: project_name.clone(),
    };

    let answers: BTreeMap<String, String> = [("DB_NAME".to_owned(), DB_NAME.to_owned())]
        .into_iter()
        .collect();
    let pending = runtime::begin_template_install(
        &template,
        "Notebook plan proof",
        &answers,
        runtime::lock_operation(&id).unwrap(),
        &|_| {},
    )
    .unwrap();
    assert!(
        runtime::lock_operation(&id).is_err(),
        "the operation lock must stay held until commit"
    );
    let app = pending.commit(&|_| {}).unwrap();
    assert_eq!(
        local_store::storage::load_registry_v2_at(&root)
            .unwrap()
            .apps
            .len(),
        1
    );

    let managed = root.join("local-store/apps");
    let project = managed.join(&id);
    assert_eq!(
        project.canonicalize().unwrap().parent(),
        Some(managed.canonicalize().unwrap().as_path())
    );
    let compose = std::fs::read_to_string(project.join("compose.yaml")).unwrap();
    assert!(
        !compose.contains("${"),
        "an unresolved placeholder reached Compose"
    );

    let secret_file = project.join(runtime::SECRETS_FILE);
    let original = std::fs::read(&secret_file).unwrap();
    let secrets: BTreeMap<String, String> = serde_json::from_slice(&original).unwrap();
    let password = secrets["DB_PASSWORD"].clone();
    assert_eq!(password.chars().count(), 32);
    assert_eq!(
        secrets.len(),
        1,
        "only the declared secret belongs in this file"
    );

    // Both containers exist, and the typed answer reached the one that needed it.
    assert_eq!(owned(&project_name, "container").len(), 2);
    // Docker's timestamps prove startup followed the deliberately slow probe,
    // rather than only asserting that both containers are healthy eventually.
    let db_state: serde_json::Value = serde_json::from_str(&docker(&[
        "inspect",
        &db_container,
        "--format",
        "{{json .State}}",
    ]))
    .unwrap();
    let first_healthy = db_state["Health"]["Log"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["ExitCode"] == 0)
        .expect("database probe passed")["End"]
        .as_str()
        .unwrap()
        .to_owned();
    let web_started = docker(&[
        "inspect",
        &web_container,
        "--format",
        "{{.State.StartedAt}}",
    ]);
    let sortable = |stamp: &str| {
        let utc = stamp.strip_suffix('Z').expect("Docker timestamp is UTC");
        let (seconds, nanos) = utc.split_once('.').unwrap_or((utc, ""));
        format!("{seconds}.{nanos:0<9}")
    };
    assert!(
        sortable(&first_healthy) <= sortable(&web_started),
        "web started before the database passed its delayed health check"
    );
    assert_eq!(
        docker(&["exec", &db_container, "printenv", "POSTGRES_DB"]),
        DB_NAME,
        "the setup answer never reached the database"
    );

    // The database must stay off the host. A published 5432 here is the single
    // worst thing a multi-service plan could do, so it is checked directly.
    let bindings = docker(&[
        "inspect",
        &db_container,
        "--format",
        "{{json .NetworkSettings.Ports}}",
    ]);
    assert!(
        !bindings.contains("HostPort"),
        "the database is published to the host: {bindings}"
    );
    assert!(
        docker(&["port", &db_container]).is_empty(),
        "the database has a host port mapping"
    );

    // ...but the two services still share a network, which is how the web app
    // reaches it at all.
    let web_networks = networks_of(&web_container);
    let db_networks = networks_of(&db_container);
    assert!(
        web_networks.iter().any(|name| db_networks.contains(name)),
        "the web service and the database share no network: {web_networks:?} vs {db_networks:?}"
    );

    wait_for_database(&db_container);
    runtime::wait_for_health(&app.launch_url, Duration::from_secs(60)).unwrap();

    // Real data, written while the first password was in force.
    psql(&db_container, &password, "create table proof (note text)");
    psql(
        &db_container,
        &password,
        &format!("insert into proof values ('{MARKER}')"),
    );
    assert_eq!(
        psql(&db_container, &password, "select note from proof"),
        MARKER
    );

    // Restart.
    runtime::stop_with(&SystemProcessRunner, &app).unwrap();
    assert_eq!(
        runtime::status_with(&SystemProcessRunner, &app).unwrap(),
        runtime::AppStatus::Stopped
    );
    runtime::start_with(&SystemProcessRunner, &app).unwrap();
    runtime::wait_for_health(&app.launch_url, Duration::from_secs(60)).unwrap();
    wait_for_database(&db_container);
    assert_eq!(
        psql(&db_container, &password, "select note from proof"),
        MARKER,
        "the row did not survive a restart"
    );

    // Uninstall while keeping data: containers go, the volume and the
    // credential stay.
    runtime::uninstall_and_remove(&app, false).unwrap();
    assert!(owned(&project_name, "container").is_empty());
    assert!(
        !owned(&project_name, "volume").is_empty(),
        "a keep-data uninstall deleted the database volume"
    );
    assert!(
        std::fs::read(&secret_file).unwrap() == original,
        "a keep-data uninstall changed the credential"
    );

    // Reinstall over the preserved volume. A fresh password here would leave
    // the app unable to open the data it is being reinstalled onto.
    let again = runtime::install_template(&template, "Notebook plan proof", &answers).unwrap();
    assert!(
        std::fs::read(&secret_file).unwrap() == original,
        "the reinstall regenerated the database password"
    );
    wait_for_database(&db_container);
    runtime::wait_for_health(&again.launch_url, Duration::from_secs(60)).unwrap();
    assert_eq!(
        psql(&db_container, &password, "select note from proof"),
        MARKER,
        "the reinstalled app cannot read the data it kept"
    );
    assert_eq!(
        local_store::storage::load_registry_v2_at(&root)
            .unwrap()
            .apps
            .len(),
        1,
        "a reinstall must not leave a second registry entry"
    );

    // Explicit deletion removes everything this project owns, and nothing else.
    runtime::uninstall_and_remove(&again, true).unwrap();
    assert!(local_store::storage::load_registry_v2_at(&root)
        .unwrap()
        .apps
        .is_empty());
    assert!(!project.exists());
    for resource in ["container", "network", "volume"] {
        assert!(
            owned(&project_name, resource).is_empty(),
            "a {resource} outlived explicit deletion"
        );
    }
    let after = all_container_ids();
    let lost: Vec<&String> = unrelated_before
        .iter()
        .filter(|id| !after.contains(id))
        .collect();
    assert!(
        lost.is_empty(),
        "containers not owned by this test disappeared: {lost:?}"
    );

    let report = serde_json::json!({
        "passed": true,
        "images": [WEB_IMAGE, DB_IMAGE],
        "image_ids": image_ids,
        "startup_order": {"database_first_healthy": first_healthy, "web_started": web_started},
        "scope": "two-service plan transaction, Compose network and registry; no GUI",
        "checks": [
            "install of a web app beside a database from one plan",
            "web starts only after delayed database health probe succeeds",
            "typed setup answer reaches the database container",
            "database is not published to the host",
            "the two services share the project network",
            "generated password authenticates over TCP",
            "pending lock held through commit",
            "stop/status/start/health",
            "row survives a restart",
            "keep-data uninstall preserves the volume and the credential",
            "reinstall reuses the password and reads the preserved row",
            "explicit deletion removes every owned container, network and volume",
            "containers owned by anything else survived"
        ]
    });
    std::fs::write(
        root.join("report.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    println!(
        "PASS: multi-service plan lifecycle; report {}",
        root.join("report.json").display()
    );
}
