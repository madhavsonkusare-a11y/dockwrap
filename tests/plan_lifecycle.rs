//! Opt-in real plan installation; no user registry, no automatic CI execution.
use local_store::{
    plan, recipes,
    runtime::{self, CommandSpec, ProcessRunner, SystemProcessRunner},
    setup::{PlanTemplate, SecretSpec},
};
use std::{collections::BTreeMap, time::Duration};

fn docker(args: Vec<String>) -> String {
    let output = SystemProcessRunner
        .run(&CommandSpec::new(
            "docker",
            args,
            None,
            Duration::from_secs(180),
        ))
        .unwrap();
    assert!(
        output.success,
        "Docker command failed (output withheld to protect fixture credentials)"
    );
    output.stdout.trim().to_owned()
}
fn args(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).into()).collect()
}

#[test]
#[ignore = "real Docker lifecycle; enable LOCAL_STORE_RUN_DOCKER_TEST=1"]
fn n8n_plan_preserves_its_encryption_key_and_volume_across_reinstall() {
    assert_eq!(
        std::env::var("LOCAL_STORE_RUN_DOCKER_TEST").as_deref(),
        Ok("1")
    );
    let recipe = recipes::recipe("n8n").unwrap();
    let image_id = docker(vec![
        "image".into(),
        "inspect".into(),
        recipe.image.clone(),
        "--format".into(),
        "{{.Id}}".into(),
    ]);
    let id = format!(
        "plan-proof-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );
    let project_name = format!("local-store-{id}");
    for resource in ["container", "network", "volume"] {
        assert!(docker(vec![
            resource.into(),
            "ls".into(),
            "-q".into(),
            "--filter".into(),
            format!("label=com.docker.compose.project={project_name}")
        ])
        .is_empty());
    }
    let mut plan = plan::plan_for_recipe(&recipe).unwrap();
    plan.id = id.clone();
    let container = format!("{project_name}-n8n");
    assert!(docker(vec![
        "container".into(),
        "ls".into(),
        "-aq".into(),
        "--filter".into(),
        format!("name=^/{container}$")
    ])
    .is_empty());
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    plan.services[0].published.as_mut().unwrap().host = listener.local_addr().unwrap().port();
    drop(listener);
    plan.services[0]
        .environment
        .push(("N8N_ENCRYPTION_KEY".into(), "${ENCRYPTION_KEY}".into()));
    let template = PlanTemplate {
        first_start: None,
        seeds: Vec::new(),
        plan,
        fields: Vec::new(),
        secrets: vec![SecretSpec {
            key: "ENCRYPTION_KEY".into(),
            length: 32,
            format: local_store::setup::SecretFormat::Alphanumeric,
        }],
    };
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".cache")
        .join(&id);
    std::fs::create_dir_all(&root).unwrap();
    println!("Private plan proof root: {}", root.display());
    // One opt-in test in this binary; production storage uses private config.
    std::env::set_var("APPDATA", &root);
    std::env::set_var("XDG_CONFIG_HOME", &root);
    let pending = runtime::begin_template_install(
        &template,
        "n8n plan proof",
        &BTreeMap::new(),
        runtime::lock_operation(&id).unwrap(),
        &|_| {},
    )
    .unwrap();
    assert!(runtime::lock_operation(&id).is_err());
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
    let secret_file = project.join(runtime::SECRETS_FILE);
    let original = std::fs::read(&secret_file).unwrap();
    let secrets: BTreeMap<String, String> = serde_json::from_slice(&original).unwrap();
    let key = &secrets["ENCRYPTION_KEY"];
    // This is a real n8n setting, not an unused fixture environment variable.
    let active_key = docker(args(&[
        "exec",
        &container,
        "printenv",
        "N8N_ENCRYPTION_KEY",
    ]));
    assert!(
        active_key == *key,
        "container encryption key differs from persisted setup key"
    );
    docker(args(&[
        "exec",
        &container,
        "sh",
        "-c",
        "printf plan-proof > /home/node/.n8n/plan-proof-marker",
    ]));
    runtime::stop_with(&SystemProcessRunner, &app).unwrap();
    assert_eq!(
        runtime::status_with(&SystemProcessRunner, &app).unwrap(),
        runtime::AppStatus::Stopped
    );
    runtime::start_with(&SystemProcessRunner, &app).unwrap();
    runtime::wait_for_health(&app.launch_url, Duration::from_secs(60)).unwrap();
    runtime::uninstall_and_remove(&app, false).unwrap();
    assert!(
        std::fs::read(&secret_file).unwrap() == original,
        "keep-data uninstall changed credentials"
    );
    let pending = runtime::begin_template_install(
        &template,
        "n8n plan proof",
        &BTreeMap::new(),
        runtime::lock_operation(&id).unwrap(),
        &|_| {},
    )
    .unwrap();
    let registry = local_store::storage::registry_v2_path_for_root(&root);
    let registry_bytes = std::fs::read(&registry).unwrap();
    std::fs::write(&registry, b"invalid-test-registry").unwrap();
    assert!(pending.commit(&|_| {}).is_err());
    assert!(
        std::fs::read(&secret_file).unwrap() == original,
        "rollback changed credentials"
    );
    assert!(docker(vec![
        "container".into(),
        "ls".into(),
        "-aq".into(),
        "--filter".into(),
        format!("name=^/{container}$")
    ])
    .is_empty());
    std::fs::write(&registry, registry_bytes).unwrap();
    let again = runtime::install_template(&template, "n8n plan proof", &BTreeMap::new()).unwrap();
    assert!(
        std::fs::read(&secret_file).unwrap() == original,
        "reinstall regenerated credentials"
    );
    assert!(
        docker(args(&[
            "exec",
            &container,
            "printenv",
            "N8N_ENCRYPTION_KEY"
        ])) == *key,
        "reinstalled container key changed"
    );
    assert_eq!(
        docker(args(&[
            "exec",
            &container,
            "cat",
            "/home/node/.n8n/plan-proof-marker"
        ])),
        "plan-proof"
    );
    runtime::uninstall_and_remove(&again, true).unwrap();
    assert!(local_store::storage::load_registry_v2_at(&root)
        .unwrap()
        .apps
        .is_empty());
    assert!(!project.exists());
    for resource in ["container", "network", "volume"] {
        assert!(docker(vec![
            resource.into(),
            "ls".into(),
            "-q".into(),
            "--filter".into(),
            format!("label=com.docker.compose.project={project_name}")
        ])
        .is_empty());
    }
    let report = serde_json::json!({"passed":true,"image":recipe.image,"image_id":image_id,
        "scope":"plan transaction and registry; no GUI or encrypted workflow migration",
        "checks":["install and health","encryption key reaches container","stop/status/start/health",
        "keep-data uninstall","pending lock held through commit","failed registry commit rolls back and preserves credentials","reinstall reuses encryption key","volume marker survives","explicit deletion removes owned resources and registry entry"]});
    std::fs::write(
        root.join("report.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    println!(
        "PASS: plan lifecycle and secret reuse; report {}",
        root.join("report.json").display()
    );
}
