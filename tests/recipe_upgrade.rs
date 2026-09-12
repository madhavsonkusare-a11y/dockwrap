//! A recipe version upgrade, with real application content in the way.
//!
//! Task 12 asks for more than "the new image starts". The risk in an upgrade is
//! the database: a new version opens the old file, runs its migrations, and
//! either carries the person's data forward or does not. Nothing about a
//! successful health probe distinguishes those two outcomes — a Memos that
//! silently created an empty database answers on its port exactly like one
//! that migrated an existing one.
//!
//! So this writes a memo through the running application's own API on 0.29.1,
//! upgrades to the shipped 0.30.0 over the preserved data directory, and then
//! signs in with the same credentials and reads that memo back. The account
//! surviving proves the user table migrated; the memo surviving proves the
//! content did.
//!
//! Opt-in: `#[ignore]` plus `LOCAL_STORE_RUN_DOCKER_TEST=1`. Everything is
//! label-scoped to this run's Compose project and removed on the way out, even
//! when an assertion fails part-way.
use local_store::{
    recipes,
    runtime::{self, CommandSpec, ProcessOutput, ProcessRunner, SystemProcessRunner},
};
use std::time::Duration;

/// The version installed first. One minor release behind what ships, which is
/// the upgrade a person actually performs.
const PREVIOUS: &str = "0.29.1";
const USERNAME: &str = "upgrade-proof-user";
/// Long enough that Memos accepts it; this account exists only inside this
/// test's private data directory and is deleted with it.
const PASSWORD: &str = "upgrade-proof-password-123";
const MARKER: &str = "content written before the upgrade";

fn run(program: &str, args: &[&str], timeout: Duration) -> ProcessOutput {
    SystemProcessRunner
        .run(&CommandSpec::new(
            program,
            args.iter().map(|value| (*value).to_owned()).collect(),
            None,
            timeout,
        ))
        .unwrap_or_else(|error| panic!("{program} must be runnable: {error}"))
}

fn docker(args: &[&str]) -> String {
    let output = run("docker", args, Duration::from_secs(180));
    assert!(
        output.success,
        "docker {} failed",
        args.first().copied().unwrap_or("?")
    );
    output.stdout.trim().to_owned()
}

/// A JSON request against the running app. Output is not echoed on failure:
/// the bodies carry an access token and this account's password.
fn api(args: &[&str]) -> String {
    let output = run("curl", args, Duration::from_secs(60));
    assert!(
        output.success,
        "curl failed (output withheld: request bodies carry credentials)"
    );
    output.stdout.trim().to_owned()
}

fn json_field(body: &str, field: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(body).ok()?;
    value
        .get(field)
        .and_then(|value| value.as_str())
        .map(str::to_owned)
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
            run("docker", &args, short)
                .stdout
                .split_whitespace()
                .map(str::to_owned)
                .collect()
        };
        for id in ids("container", &["ls", "-aq"]) {
            run("docker", &["rm", "-f", &id], short);
        }
        for id in ids("network", &["ls", "-q"]) {
            run("docker", &["network", "rm", &id], short);
        }
        for id in ids("volume", &["ls", "-q"]) {
            run("docker", &["volume", "rm", "-f", &id], short);
        }
    }
}

/// Re-pin the reviewed recipe to an earlier version, and give it a run-specific
/// id so this never touches a real install of Memos.
///
/// Only a test does this. The product installs a reviewed recipe exactly as
/// reviewed; re-pinning a version is not something it offers.
fn recipe_at(version: &str, id: &str, host_port: u16) -> recipes::Recipe {
    let shipped = recipes::recipe("memos").expect("memos is a reviewed recipe");
    let mut recipe = shipped.with_host_port(host_port).expect("port remap");
    recipe.compose = recipe
        .compose
        .replace(
            &format!("neosmemo/memos:{}", shipped.version),
            &format!("neosmemo/memos:{version}"),
        )
        .replace(
            "container_name: local-store-memos",
            &format!("container_name: local-store-{id}"),
        )
        .replace("  memos:\n", &format!("  {id}:\n"));
    recipe.image = format!("neosmemo/memos:{version}");
    recipe.version = version.to_owned();
    recipe.id = id.to_owned();
    recipe
}

fn wait_until_healthy(address: &str) {
    runtime::wait_for_health(address, Duration::from_secs(120))
        .unwrap_or_else(|error| panic!("{address} never became healthy: {error}"));
}

/// Sign in and return an access token. Memos hands the session back in a
/// gRPC-gateway metadata header rather than a cookie, so the token from the
/// body is what a client actually uses.
fn sign_in(address: &str) -> String {
    let body =
        format!(r#"{{"passwordCredentials":{{"username":"{USERNAME}","password":"{PASSWORD}"}}}}"#);
    let url = format!("{address}/api/v1/auth/signin");
    let response = api(&[
        "-s",
        "-X",
        "POST",
        "-H",
        "Content-Type: application/json",
        "-d",
        &body,
        &url,
    ]);
    json_field(&response, "accessToken")
        .unwrap_or_else(|| panic!("sign-in returned no access token for {address}"))
}

fn memo_contents(address: &str, token: &str) -> Vec<String> {
    let response = api(&[
        "-s",
        "-H",
        &format!("Authorization: Bearer {token}"),
        &format!("{address}/api/v1/memos"),
    ]);
    let value: serde_json::Value = serde_json::from_str(&response).expect("memo list must be JSON");
    value["memos"]
        .as_array()
        .map(|memos| {
            memos
                .iter()
                .filter_map(|memo| memo["content"].as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

#[test]
#[ignore = "real Docker upgrade; enable LOCAL_STORE_RUN_DOCKER_TEST=1"]
fn an_upgrade_carries_the_account_and_the_notes_that_existed_before_it() {
    assert_eq!(
        std::env::var("LOCAL_STORE_RUN_DOCKER_TEST").as_deref(),
        Ok("1")
    );
    let shipped_version = recipes::recipe("memos").unwrap().version;
    assert_ne!(
        shipped_version, PREVIOUS,
        "the recipe now ships {PREVIOUS}; pick an earlier version to upgrade from"
    );
    for image in [
        format!("neosmemo/memos:{PREVIOUS}"),
        format!("neosmemo/memos:{shipped_version}"),
    ] {
        let pull = run("docker", &["pull", &image], Duration::from_secs(600));
        assert!(pull.success, "could not pull {image}");
    }

    let id = format!(
        "upgrade-proof-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );
    let project_name = format!("local-store-{id}");
    for resource in ["container", "network", "volume"] {
        assert!(
            owned(&project_name, resource).is_empty(),
            "a {resource} already carries this run's project label"
        );
    }

    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let host_port = listener.local_addr().unwrap().port();
    drop(listener);
    let address = format!("http://localhost:{host_port}");

    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".cache")
        .join(&id);
    std::fs::create_dir_all(&root).unwrap();
    println!("Private upgrade proof root: {}", root.display());
    // One opt-in test in this binary; production storage uses private config.
    std::env::set_var("APPDATA", &root);
    std::env::set_var("XDG_CONFIG_HOME", &root);

    let _cleanup = ProjectCleanup {
        project: project_name.clone(),
    };

    // ---- the version somebody already has -------------------------------
    let previous = recipe_at(PREVIOUS, &id, host_port);
    let installed = runtime::install_recipe(&previous).expect("previous version should install");
    wait_until_healthy(&installed.launch_url);

    // A real account, made through the running application rather than by
    // writing to its database behind its back.
    let created = api(&[
        "-s",
        "-X",
        "POST",
        "-H",
        "Content-Type: application/json",
        "-d",
        &format!(r#"{{"username":"{USERNAME}","password":"{PASSWORD}","role":"HOST"}}"#),
        &format!("{address}/api/v1/users"),
    ]);
    assert_eq!(
        json_field(&created, "username").as_deref(),
        Some(USERNAME),
        "the account was not created"
    );

    let token = sign_in(&address);
    api(&[
        "-s",
        "-X",
        "POST",
        "-H",
        &format!("Authorization: Bearer {token}"),
        "-H",
        "Content-Type: application/json",
        "-d",
        &format!(r#"{{"content":"{MARKER}","visibility":"PRIVATE"}}"#),
        &format!("{address}/api/v1/memos"),
    ]);
    assert_eq!(
        memo_contents(&address, &token),
        vec![MARKER.to_owned()],
        "the note was not stored before the upgrade"
    );

    let project = root.join("local-store/apps").join(&id);
    let database = project.join("data/memos_prod.db");
    assert!(database.is_file(), "no database at {}", database.display());
    let before = std::fs::metadata(&database).unwrap().len();

    // ---- the upgrade -----------------------------------------------------
    // Keeping data is what makes this an upgrade rather than a fresh install.
    runtime::uninstall_and_remove(&installed, false).unwrap();
    assert!(owned(&project_name, "container").is_empty());
    assert!(
        database.is_file(),
        "the keep-data uninstall took the database"
    );

    let upgraded = recipe_at(&shipped_version, &id, host_port);
    assert_ne!(
        upgraded.image, previous.image,
        "both installs used one image"
    );
    let running = runtime::install_recipe(&upgraded).expect("upgrade should install");
    wait_until_healthy(&running.launch_url);
    assert_eq!(
        docker(&[
            "inspect",
            &format!("local-store-{id}"),
            "--format",
            "{{.Config.Image}}"
        ]),
        upgraded.image,
        "the container is not running the upgraded image"
    );

    // ---- what the person kept -------------------------------------------
    // Signing in proves the account row migrated: a fresh database would have
    // no such user, and Memos would refuse these credentials.
    let token = sign_in(&address);
    assert_eq!(
        memo_contents(&address, &token),
        vec![MARKER.to_owned()],
        "the note written before the upgrade is gone"
    );
    assert!(
        std::fs::metadata(&database).unwrap().len() >= before,
        "the database shrank across the upgrade"
    );

    // ---- and it still cleans up -----------------------------------------
    runtime::uninstall_and_remove(&running, true).unwrap();
    assert!(!project.exists());
    for resource in ["container", "network", "volume"] {
        assert!(
            owned(&project_name, resource).is_empty(),
            "a {resource} outlived explicit deletion"
        );
    }

    let report = serde_json::json!({
        "passed": true,
        "ran_at": "2026-09-08",
        "recipe": "memos",
        "from": previous.image,
        "to": upgraded.image,
        "scope": "one reviewed recipe across one minor version, with content written by the application itself",
        "checks": [
            "previous version installs and becomes healthy",
            "an account and a note are created through the running application's API",
            "keep-data uninstall preserves the SQLite database",
            "the upgraded image installs over the preserved data directory",
            "the running container is the upgraded image, not the previous one",
            "the account created before the upgrade can still sign in",
            "the note written before the upgrade reads back unchanged",
            "the database did not shrink across the migration",
            "explicit deletion removes every owned container, network and volume"
        ]
    });
    std::fs::write(
        root.join("report.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    println!(
        "PASS: {} upgraded to {} with its content intact; report {}",
        previous.image,
        upgraded.image,
        root.join("report.json").display()
    );
}
