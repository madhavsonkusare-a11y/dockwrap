//! Node-RED qualification: the transaction, managed storage, and the two
//! things only a real first use can show — that the editor comes up and holds
//! its live connection, and that the runtime executes a flow somebody deployed
//! and still has it after a restart and a reinstall.
use local_store::{
    runtime::{self, CommandSpec, ProcessRunner, SystemProcessRunner},
    templates::ReviewedTemplate,
};
use std::{collections::BTreeMap, time::Duration};

fn docker(args: &[&str]) -> String {
    let out = SystemProcessRunner
        .run(&CommandSpec::new(
            "docker",
            args.iter().map(|s| s.to_string()).collect(),
            None,
            Duration::from_secs(300),
        ))
        .unwrap();
    assert!(out.success, "Docker command failed: {}", out.stderr);
    out.stdout.trim().to_owned()
}

fn probe(mode: &str, url: &str, state: &std::path::Path) {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out = SystemProcessRunner
        .run(&CommandSpec::new(
            "node",
            vec![
                root.join("scripts/nodered-probe.mjs")
                    .to_string_lossy()
                    .into_owned(),
                mode.into(),
                url.into(),
                state.to_string_lossy().into_owned(),
            ],
            Some(root),
            Duration::from_secs(120),
        ))
        .unwrap();
    assert!(out.success, "Node-RED probe failed: {}", out.stderr);
}

struct Cleanup(String);

impl Drop for Cleanup {
    fn drop(&mut self) {
        let filter = format!("label=com.docker.compose.project={}", self.0);
        for (kind, list) in [("container", "-aq"), ("network", "-q"), ("volume", "-q")] {
            let cmd = CommandSpec::new(
                "docker",
                vec![
                    kind.into(),
                    "ls".into(),
                    list.into(),
                    "--filter".into(),
                    filter.clone(),
                ],
                None,
                Duration::from_secs(30),
            );
            if let Ok(out) = SystemProcessRunner.run(&cmd) {
                if !out.success {
                    continue;
                }
                for id in out.stdout.split_whitespace() {
                    let mut args = vec![kind.into(), "rm".into()];
                    if kind == "container" {
                        args.push("-f".into());
                    }
                    args.push(id.into());
                    let _ = SystemProcessRunner.run(&CommandSpec::new(
                        "docker",
                        args,
                        None,
                        Duration::from_secs(30),
                    ));
                }
            }
        }
    }
}

/// Everything under `/data` that a person would lose, read as the container's
/// own user so the check is about the app's access rather than root's.
fn in_container(container: &str, args: &[&str]) -> String {
    let mut full: Vec<&str> = vec!["exec", "--user", "1000:1000", container];
    full.extend_from_slice(args);
    docker(&full)
}

#[test]
#[ignore = "real Docker; enable LOCAL_STORE_RUN_DOCKER_TEST=1"]
fn nodered_runtipi_transaction_runs_and_keeps_a_deployed_flow() {
    assert_eq!(
        std::env::var("LOCAL_STORE_RUN_DOCKER_TEST").as_deref(),
        Ok("1")
    );
    // Resolved the way the launcher and the CLI resolve it, so this fails if
    // the review is withheld, unwired, or maps to something the install path
    // would not accept.
    let offering =
        local_store::offerings::offering("nodered").expect("nodered is offered to install");
    let reviewed: ReviewedTemplate =
        local_store::templates::reviewed_template("nodered").expect("nodered is reviewed");
    let mut template = offering.plan_template(None).expect("nodered should map");
    let image = template.plan.services[0].image.clone();
    // The review pins a tag the upstream definition does not name, so what
    // actually runs has to be the reviewed one.
    assert!(
        image.ends_with(":5.0.7"),
        "the reviewed image pin did not reach the plan: {image}"
    );
    docker(&["pull", &image]);
    let prior = docker(&["ps", "-aq"]);
    let id = format!(
        "nodered-proof-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );
    let project = format!("local-store-{id}");
    let container = format!("{project}-nodered");
    let _cleanup = Cleanup(project.clone());
    template.plan.id = id.clone();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    template
        .plan
        .set_published_host(listener.local_addr().unwrap().port());
    drop(listener);
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".cache")
        .join(&id);
    std::fs::create_dir_all(&root).unwrap();
    std::env::set_var("APPDATA", &root);
    std::env::set_var("XDG_CONFIG_HOME", &root);

    let app = runtime::begin_template_install(
        &template,
        "Node-RED qualification",
        &BTreeMap::new(),
        runtime::lock_operation(&id).unwrap(),
        &|_| {},
    )
    .unwrap()
    .commit(&|_| {})
    .unwrap();
    runtime::wait_for_health(&app.launch_url, Duration::from_secs(120)).unwrap();

    // Loopback only: an unauthenticated editor must not be on any other
    // interface, which is the single most important thing about this app.
    let bindings = docker(&[
        "inspect",
        &container,
        "--format",
        "{{json .HostConfig.PortBindings}}",
    ]);
    assert!(bindings.contains("127.0.0.1"), "{bindings}");
    assert!(!bindings.contains("0.0.0.0"), "{bindings}");

    // The documented container user, checked rather than assumed.
    assert_eq!(in_container(&container, &["id", "-u"]), "1000");

    let state = root.join("nodered-probe.json");
    probe("deploy", &app.launch_url, &state);

    // Node-RED stores what a person would lose in the managed folder, and the
    // credential has to be encrypted there rather than sitting in plain text.
    let creds = in_container(&container, &["cat", "/data/flows_cred.json"]);
    assert!(
        !creds.contains("probe-only-not-a-real-secret"),
        "the credential was written in plain text"
    );
    assert!(
        creds.contains("\"$\""),
        "the credentials file is not encrypted: {creds}"
    );
    // The key that decrypts it lives beside the data it unlocks.
    let runtime_config = in_container(&container, &["cat", "/data/.config.runtime.json"]);
    assert!(
        runtime_config.contains("_credentialSecret"),
        "{runtime_config}"
    );
    let flows = in_container(&container, &["cat", "/data/flows.json"]);
    assert!(
        flows.contains("local store probe"),
        "the flow was not saved"
    );

    // Restart: the flow has to still run, and the editor has to still come up.
    runtime::stop_with(&SystemProcessRunner, &app).unwrap();
    runtime::start_with(&SystemProcessRunner, &app).unwrap();
    runtime::wait_for_health(&app.launch_url, Duration::from_secs(120)).unwrap();
    probe("verify", &app.launch_url, &state);
    let creds_after = in_container(&container, &["cat", "/data/flows_cred.json"]);
    assert_eq!(
        creds, creds_after,
        "the encrypted credentials were rewritten"
    );

    // A keep-data uninstall and reinstall: the same flow, still running, and
    // credentials the reinstalled app can still decrypt.
    runtime::uninstall_and_remove(&app, false).unwrap();
    let again =
        runtime::install_template(&template, "Node-RED qualification", &BTreeMap::new()).unwrap();
    runtime::wait_for_health(&again.launch_url, Duration::from_secs(120)).unwrap();
    probe("verify", &again.launch_url, &state);
    assert_eq!(
        in_container(&container, &["cat", "/data/flows_cred.json"]),
        creds,
        "the reinstall could not keep the credentials it was given"
    );

    let image_id = docker(&["inspect", &container, "--format", "{{.Image}}"]);
    runtime::uninstall_and_remove(&again, true).unwrap();
    let filter = format!("label=com.docker.compose.project={project}");
    for resource in ["container", "network", "volume"] {
        assert!(docker(&[resource, "ls", "-q", "--filter", &filter]).is_empty());
    }
    assert!(!root.join("local-store/apps").join(&id).exists());
    assert!(local_store::storage::load_registry_v2_at(&root)
        .unwrap()
        .apps
        .is_empty());
    let after = docker(&["ps", "-aq"]);
    assert!(prior
        .lines()
        .all(|id| after.lines().any(|other| id == other)));

    let evidence = serde_json::json!({
        "passed": true,
        "scope": "Runtipi reviewed mapping with a reviewed image pin, transaction, managed storage \
                  and browser first use, on one Windows host and one architecture",
        "browser_first_use": "passed",
        "promotion": reviewed.promotion.state,
        "source_revision": reviewed.origin.revision,
        "definition_image": "nodered/node-red:5.0.6",
        "image": image,
        "image_id": image_id,
        "checks": [
            "loopback endpoint, and no binding on any other interface",
            "HTTP readiness",
            "container runs as UID 1000, not root",
            "editor renders its workspace and palette in a real browser",
            "editor opens and holds its /comms websocket",
            "editor shows the deployed flow the admin API sent",
            "a deployed function node executes and returns its computed value",
            "credentials are encrypted at rest, not written in plain text",
            "the credential key is stored beside the data it unlocks",
            "the flow still runs after a restart",
            "the flow still runs after a keep-data reinstall",
            "encrypted credentials survive both unchanged",
            "delete-data cleanup",
            "unrelated containers retained"
        ]
    });
    std::fs::write(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("docs/evidence/nodered-first-use-windows-2026-09-10.json"),
        serde_json::to_string_pretty(&evidence).unwrap(),
    )
    .unwrap();
}
