//! Isolated Runtipi transaction/storage qualification. Browser crypto is a separate gate.
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

struct Cleanup(String);

fn browser_probe(mode: &str, url: &str, state: &std::path::Path) {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out = SystemProcessRunner
        .run(&CommandSpec::new(
            "node",
            vec![
                root.join("scripts/privatebin-browser-probe.mjs")
                    .to_string_lossy()
                    .into_owned(),
                mode.into(),
                url.into(),
                state.to_string_lossy().into_owned(),
            ],
            Some(root),
            Duration::from_secs(90),
        ))
        .unwrap();
    assert!(out.success, "browser probe failed: {}", out.stderr);
}
impl Drop for Cleanup {
    fn drop(&mut self) {
        // The run-specific Compose label is the only cleanup scope.
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

#[test]
#[ignore = "real Docker; enable LOCAL_STORE_RUN_DOCKER_TEST=1"]
fn privatebin_runtipi_transaction_preserves_managed_data() {
    assert_eq!(
        std::env::var("LOCAL_STORE_RUN_DOCKER_TEST").as_deref(),
        Ok("1")
    );
    // Resolved the way the launcher and the CLI resolve it, not around them:
    // this fails if the review is withheld, unwired, or maps to something the
    // install path would not accept. The proof has to be about the artefact
    // somebody would actually install.
    let offering =
        local_store::offerings::offering("privatebin").expect("privatebin is offered to install");
    let reviewed: ReviewedTemplate =
        local_store::templates::reviewed_template("privatebin").expect("privatebin is reviewed");
    assert!(
        offering.recipe(None).unwrap().is_none(),
        "an imported app must not claim a reviewed Compose file of its own"
    );
    let mut template = offering.plan_template(None).unwrap();
    let image = template.plan.services[0].image.clone();
    docker(&["pull", &image]);
    let prior = docker(&["ps", "-aq"]);
    let id = format!(
        "privatebin-proof-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );
    let project = format!("local-store-{id}");
    let container = format!("{project}-privatebin");
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
        "PrivateBin qualification",
        &BTreeMap::new(),
        runtime::lock_operation(&id).unwrap(),
        &|_| {},
    )
    .unwrap()
    .commit(&|_| {})
    .unwrap();
    runtime::wait_for_health(&app.launch_url, Duration::from_secs(90)).unwrap();
    // This test renames the plan so it cannot collide with a real install, and
    // an id nothing offers gets no icon rather than somebody else's. That is
    // the guard working. The production case — where the plan carries the
    // offering's own id — is covered without Docker by
    // `an_offering_maps_to_a_plan_that_carries_its_own_id` and
    // `every_offering_resolves_the_catalog_entry_its_icon_comes_from`.
    assert_eq!(app.catalog_id, None);
    let browser_state = root.join("browser-paste.json");
    browser_probe("create", &app.launch_url, &browser_state);
    let bindings = docker(&[
        "inspect",
        &container,
        "--format",
        "{{json .HostConfig.PortBindings}}",
    ]);
    assert!(bindings.contains("127.0.0.1"));
    // Run as the documented PHP application UID/GID, not root. This is a storage
    // probe, explicitly not a substitute for encrypted browser paste creation.
    docker(&["exec", "--user", "65534:82", &container, "php", "-r", "if(file_put_contents('/srv/data/local-store-proof.txt','privatebin-proof')===false) exit(1);"]);
    runtime::stop_with(&SystemProcessRunner, &app).unwrap();
    runtime::start_with(&SystemProcessRunner, &app).unwrap();
    runtime::wait_for_health(&app.launch_url, Duration::from_secs(90)).unwrap();
    browser_probe("read", &app.launch_url, &browser_state);
    assert_eq!(
        docker(&[
            "exec",
            "--user",
            "65534:82",
            &container,
            "cat",
            "/srv/data/local-store-proof.txt"
        ]),
        "privatebin-proof"
    );
    runtime::uninstall_and_remove(&app, false).unwrap();
    let again =
        runtime::install_template(&template, "PrivateBin qualification", &BTreeMap::new()).unwrap();
    runtime::wait_for_health(&again.launch_url, Duration::from_secs(90)).unwrap();
    browser_probe("read", &again.launch_url, &browser_state);
    assert_eq!(
        docker(&[
            "exec",
            "--user",
            "65534:82",
            &container,
            "cat",
            "/srv/data/local-store-proof.txt"
        ]),
        "privatebin-proof"
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
    let evidence = serde_json::json!({"passed":true,"scope":"Runtipi reviewed mapping, transaction, managed storage and browser first use, on one Windows host and one architecture","browser_first_use":"passed","promotion": reviewed.promotion.state,"source_revision":reviewed.origin.revision,"image":image,"image_id":image_id,"checks":["loopback endpoint","HTTP readiness","loopback secure context exposes window.crypto.subtle","encrypted paste created in the browser and decrypted from its own link","PHP UID 65534/GID 82 can write managed data","restart persistence","that paste reopens and decrypts after restart","keep-data reinstall persistence","that paste reopens and decrypts after a keep-data reinstall","delete-data cleanup","unrelated containers retained"]});
    std::fs::write(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("docs/evidence/privatebin-storage-windows-2026-09-09.json"),
        serde_json::to_string_pretty(&evidence).unwrap(),
    )
    .unwrap();
}
