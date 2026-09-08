//! One imported CapRover app, installed for real.
//!
//! `docs/caprover-import-report.md` measures which pinned definitions can be
//! expressed as a plan. Expressible is a claim about shape; this is the only
//! place a CapRover definition is turned into containers and asked to work.
//!
//! CodiMD is the case worth proving. It has two services, a generated
//! credential that MariaDB bakes into its volume on first start and CodiMD
//! must present on every start afterwards, a `srv-captain--` reference that
//! only resolves if the adapter rewrote it to a Compose service name, and
//! `notExposeAsWebApp: 'true'` written as a quoted string — the shape that
//! would publish a database to the host if it were read as a boolean.
//!
//! What runs here is the reviewed template itself — `src/templates/codimd.json`
//! carries the pinned upstream definition and the review decisions taken about
//! it, and this test installs whatever that produces. Nothing is transcribed:
//! if the review changes, this proves the changed thing or fails.
//!
//! Opt-in: `#[ignore]` plus `LOCAL_STORE_RUN_DOCKER_TEST=1`. Everything it
//! creates is label-scoped to its own Compose project and removed on the way
//! out, including when an assertion fails part-way.
use local_store::{
    runtime::{self, CommandSpec, ProcessOutput, ProcessRunner, SystemProcessRunner},
    templates,
};
use std::{collections::BTreeMap, time::Duration};

const WEB_IMAGE: &str = "linuxserver/codimd:1.6.0-ls44";
const DB_IMAGE: &str = "linuxserver/mariadb:110.4.14mariabionic-ls77";
const TIMEZONE: &str = "Etc/UTC";
const MARKER: &str = "caprover-import-proof";

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

/// Output is withheld on failure deliberately: several of these commands carry
/// the generated database password.
fn docker(args: &[&str]) -> String {
    let output = run(args, Duration::from_secs(180));
    assert!(
        output.success,
        "docker {} failed (output withheld to protect the generated credential)",
        args.first().copied().unwrap_or("?")
    );
    output.stdout.trim().to_owned()
}

fn label_filter(project: &str) -> String {
    format!("label=com.docker.compose.project={project}")
}

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

/// Removes exactly what this run owns, including when an assertion fails.
/// Nothing here asserts: a panic while unwinding would abort the process and
/// hide the failure that matters.
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

/// MariaDB answering on its own loopback means the database finished
/// initialising; the published health probe only says CodiMD is listening.
fn wait_for_database(container: &str, password: &str) {
    for _ in 0..90 {
        let probe = run(
            &[
                "exec",
                "-e",
                &format!("MYSQL_PWD={password}"),
                container,
                "mysql",
                "-h",
                "127.0.0.1",
                "-ucodimd",
                "codimd",
                "-e",
                "select 1",
            ],
            Duration::from_secs(30),
        );
        if probe.success {
            return;
        }
        std::thread::sleep(Duration::from_secs(2));
    }
    panic!("the database never accepted the generated credential within three minutes");
}

/// One statement over TCP as the application makes it, so the generated
/// password is authenticated rather than bypassed by a local socket.
fn mysql(container: &str, password: &str, statement: &str) -> String {
    docker(&[
        "exec",
        "-e",
        &format!("MYSQL_PWD={password}"),
        container,
        "mysql",
        "-h",
        "127.0.0.1",
        "-ucodimd",
        "codimd",
        "-N",
        "-B",
        "-e",
        statement,
    ])
}

#[test]
#[ignore = "real Docker lifecycle; enable LOCAL_STORE_RUN_DOCKER_TEST=1"]
fn an_imported_caprover_app_installs_and_keeps_its_generated_credential() {
    assert_eq!(
        std::env::var("LOCAL_STORE_RUN_DOCKER_TEST").as_deref(),
        Ok("1")
    );
    for image in [WEB_IMAGE, DB_IMAGE] {
        let pull = run(&["pull", image], Duration::from_secs(900));
        assert!(pull.success, "could not pull {image}");
    }

    let id = format!(
        "cap-proof-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );
    let project_name = format!("local-store-{id}");
    // Service names come from the review, not from this run's id, so both
    // containers carry the service name after the project.
    let web_container = format!("{project_name}-codimd");
    let db_container = format!("{project_name}-mariadb");

    // The reviewed template is what produces the plan; nothing here writes one.
    let reviewed = templates::reviewed_template("codimd").expect("codimd is reviewed");
    let mut template = reviewed
        .plan_template()
        .unwrap_or_else(|reason| panic!("{reason}"));
    // Installed under a run-specific id so this never collides with a real
    // install of the same app, or with another run of this test.
    template.plan.id = id.clone();
    assert_eq!(template.plan.services.len(), 2);
    assert_eq!(template.secrets.len(), 1, "one generated credential");
    assert_eq!(template.fields.len(), 1, "one typed setup field");
    let secret_key = template.secrets[0].key.clone();
    let field_key = template.fields[0].key.clone();

    // The database must not have been given a way onto the host.
    let db = template
        .plan
        .services
        .iter()
        .find(|service| service.name == "mariadb")
        .expect("the database service");
    assert!(db.published.is_none());
    // The platform's own service address must already be a Compose name.
    let web = template
        .plan
        .services
        .iter()
        .find(|service| service.name == "codimd")
        .expect("the web service");
    assert_eq!(
        web.environment
            .iter()
            .find(|(key, _)| key == "DB_HOST")
            .map(|(_, value)| value.as_str()),
        Some("mariadb")
    );

    for resource in ["container", "network", "volume"] {
        assert!(
            owned(&project_name, resource).is_empty(),
            "a {resource} already carries this run's project label"
        );
    }
    let unrelated_before = all_container_ids();

    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    template
        .plan
        .set_published_host(listener.local_addr().unwrap().port());
    drop(listener);

    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".cache")
        .join(&id);
    std::fs::create_dir_all(&root).unwrap();
    println!("Private CapRover proof root: {}", root.display());
    // One opt-in test in this binary; production storage uses private config.
    std::env::set_var("APPDATA", &root);
    std::env::set_var("XDG_CONFIG_HOME", &root);

    let _cleanup = ProjectCleanup {
        project: project_name.clone(),
    };

    let answers: BTreeMap<String, String> = [(field_key.clone(), TIMEZONE.to_owned())]
        .into_iter()
        .collect();
    let app = runtime::begin_template_install(
        &template,
        "CodiMD import proof",
        &answers,
        runtime::lock_operation(&id).unwrap(),
        &|_| {},
    )
    .unwrap()
    .commit(&|_| {})
    .unwrap();

    let project = root.join("local-store/apps").join(&id);
    let compose = std::fs::read_to_string(project.join("compose.yaml")).unwrap();
    assert!(
        !compose.contains("$$"),
        "a platform expression reached Compose"
    );
    assert!(!compose.contains("${"), "a placeholder reached Compose");

    let secret_file = project.join(runtime::SECRETS_FILE);
    let original = std::fs::read(&secret_file).unwrap();
    let secrets: BTreeMap<String, String> = serde_json::from_slice(&original).unwrap();
    let password = secrets[&secret_key].clone();
    assert_eq!(password.chars().count(), 16);

    assert_eq!(owned(&project_name, "container").len(), 2);
    // The typed answer reached the container that needed it.
    assert_eq!(
        docker(&["exec", &web_container, "printenv", "TZ"]),
        TIMEZONE
    );
    // One credential, presented by both services.
    assert_eq!(
        docker(&["exec", &db_container, "printenv", "MYSQL_PASSWORD"]),
        password
    );
    assert_eq!(
        docker(&["exec", &web_container, "printenv", "DB_PASS"]),
        password
    );
    // The quoted "true" must have kept the database off the host.
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

    wait_for_database(&db_container, &password);
    runtime::wait_for_health(&app.launch_url, Duration::from_secs(120)).unwrap();

    // Real data, written while the first credential was in force.
    mysql(&db_container, &password, "create table proof (note text)");
    mysql(
        &db_container,
        &password,
        &format!("insert into proof values ('{MARKER}')"),
    );
    assert_eq!(
        mysql(&db_container, &password, "select note from proof"),
        MARKER
    );

    runtime::stop_with(&SystemProcessRunner, &app).unwrap();
    assert_eq!(
        runtime::status_with(&SystemProcessRunner, &app).unwrap(),
        runtime::AppStatus::Stopped
    );
    runtime::start_with(&SystemProcessRunner, &app).unwrap();
    wait_for_database(&db_container, &password);
    runtime::wait_for_health(&app.launch_url, Duration::from_secs(120)).unwrap();

    // Keep-data uninstall, then reinstall over the preserved volume. MariaDB
    // still holds the first password; a regenerated one would leave CodiMD
    // unable to open its own database.
    runtime::uninstall_and_remove(&app, false).unwrap();
    assert!(owned(&project_name, "container").is_empty());
    assert!(
        !owned(&project_name, "volume").is_empty(),
        "a keep-data uninstall deleted the database volume"
    );

    let again = runtime::install_template(&template, "CodiMD import proof", &answers).unwrap();
    assert!(
        std::fs::read(&secret_file).unwrap() == original,
        "the reinstall regenerated the database credential"
    );
    wait_for_database(&db_container, &password);
    runtime::wait_for_health(&again.launch_url, Duration::from_secs(120)).unwrap();
    assert_eq!(
        mysql(&db_container, &password, "select note from proof"),
        MARKER,
        "the reinstalled app cannot read the data it kept"
    );

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
        "reviewed_template": "src/templates/codimd.json",
        "source": reviewed.origin.repository,
        "definition": reviewed.origin.path,
        "revision": reviewed.origin.revision,
        "images": [WEB_IMAGE, DB_IMAGE],
        "scope": "one imported definition through the shared plan transaction; not a reviewed recipe",
        "checks": [
            "the reviewed template resolves to a valid plan",
            "srv-captain-- reference resolves to a Compose service name",
            "quoted notExposeAsWebApp keeps the database off the host",
            "typed setup answer reaches the container",
            "one generated credential is presented by both services",
            "credential authenticates against MariaDB over TCP",
            "stop/status/start/health",
            "row survives a restart",
            "keep-data uninstall preserves the volume and the credential",
            "reinstall reuses the credential and reads the preserved row",
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
        "PASS: imported CapRover app lifecycle; report {}",
        root.join("report.json").display()
    );
}
