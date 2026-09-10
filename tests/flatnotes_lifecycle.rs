//! flatnotes qualification: the first app whose install asks a person
//! questions.
//!
//! Everything before this proved a transaction. This proves the *answers*: the
//! username and password typed into the setup form are the ones that sign in,
//! a wrong password does not, and the two credentials the app generates for
//! itself are reused on a reinstall rather than minted fresh.
use local_store::{
    runtime::{self, CommandSpec, ProcessRunner, SystemProcessRunner},
    templates::ReviewedTemplate,
};
use std::{collections::BTreeMap, time::Duration};

const USERNAME: &str = "local-store-probe";
const PASSWORD: &str = "a-password-somebody-chose-9271";
const NOTE: &str = "local-store-probe";
const NOTE_BODY: &str = "This note was written into the managed folder.";

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

fn probe(mode: &str, url: &str) {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out = SystemProcessRunner
        .run(&CommandSpec::new(
            "node",
            vec![
                root.join("scripts/flatnotes-probe.mjs")
                    .to_string_lossy()
                    .into_owned(),
                mode.into(),
                url.into(),
                USERNAME.into(),
                PASSWORD.into(),
            ],
            Some(root),
            Duration::from_secs(150),
        ))
        .unwrap();
    assert!(out.success, "flatnotes probe failed: {}", out.stderr);
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

fn answers() -> BTreeMap<String, String> {
    [
        ("FLATNOTES_AUTH_TYPE", "password"),
        ("FLATNOTES_USERNAME", USERNAME),
        ("FLATNOTES_PASSWORD", PASSWORD),
    ]
    .into_iter()
    .map(|(key, value)| (key.to_owned(), value.to_owned()))
    .collect()
}

#[test]
#[ignore = "real Docker; enable LOCAL_STORE_RUN_DOCKER_TEST=1"]
fn flatnotes_carries_the_answers_a_person_typed_into_the_running_app() {
    assert_eq!(
        std::env::var("LOCAL_STORE_RUN_DOCKER_TEST").as_deref(),
        Ok("1")
    );
    // Resolved the way the launcher and the CLI resolve it, so this fails if
    // the review is withheld, unwired, or maps to something the install path
    // would not accept.
    let offering =
        local_store::offerings::offering("flatnotes").expect("flatnotes is offered to install");
    let reviewed: ReviewedTemplate =
        local_store::templates::reviewed_template("flatnotes").expect("flatnotes is reviewed");
    let mut template = offering.plan_template(None).expect("flatnotes should map");

    // The answers have to be the ones the review actually asks for; a key no
    // field declared is refused before anything is written.
    let answers = answers();
    template
        .accept_answers(&answers)
        .expect("these are the answers this app asks for");
    assert_eq!(template.secrets.len(), 2, "two generated credentials");

    let image = template.plan.services[0].image.clone();
    docker(&["pull", &image]);
    let prior = docker(&["ps", "-aq"]);
    let id = format!(
        "flatnotes-proof-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );
    let project = format!("local-store-{id}");
    let container = format!("{project}-flatnotes");
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
        "flatnotes qualification",
        &answers,
        runtime::lock_operation(&id).unwrap(),
        &|_| {},
    )
    .unwrap()
    .commit(&|_| {})
    .unwrap();
    runtime::wait_for_health(&app.launch_url, Duration::from_secs(120)).unwrap();

    let bindings = docker(&[
        "inspect",
        &container,
        "--format",
        "{{json .HostConfig.PortBindings}}",
    ]);
    assert!(bindings.contains("127.0.0.1"), "{bindings}");
    assert!(!bindings.contains("0.0.0.0"), "{bindings}");

    // The typed answer reached the container as the value that was typed.
    assert_eq!(
        docker(&["exec", &container, "printenv", "FLATNOTES_USERNAME"]),
        USERNAME
    );
    // Both generated credentials are long, distinct, and neither is an answer
    // anybody supplied.
    let signing = docker(&["exec", &container, "printenv", "FLATNOTES_SECRET_KEY"]);
    let totp = docker(&["exec", &container, "printenv", "FLATNOTES_TOTP_KEY"]);
    assert!(signing.len() >= 32 && totp.len() >= 32, "short credentials");
    assert_ne!(signing, totp, "both credentials are the same value");
    assert!(signing != PASSWORD && totp != PASSWORD);

    // flatnotes keeps notes as plain markdown in the folder we manage. Writing
    // one here and finding it in the app is what proves that claim.
    let project_dir = root.join("local-store/apps").join(&id);
    let notes = project_dir.join("data");
    std::fs::write(notes.join(format!("{NOTE}.md")), NOTE_BODY).unwrap();

    probe("first-use", &app.launch_url);

    // The credential file is written beside the data it unlocks.
    let secrets = std::fs::read_to_string(project_dir.join("local-store-secrets.json")).unwrap();
    assert!(secrets.contains("FLATNOTES_SECRET_KEY"));
    assert!(
        !secrets.contains(PASSWORD),
        "a typed answer was stored in the generated-credential file"
    );

    // Restart: the same password still signs in and the note is still there.
    runtime::stop_with(&SystemProcessRunner, &app).unwrap();
    runtime::start_with(&SystemProcessRunner, &app).unwrap();
    runtime::wait_for_health(&app.launch_url, Duration::from_secs(120)).unwrap();
    probe("verify", &app.launch_url);

    // A keep-data uninstall and reinstall with the same answers: the generated
    // credentials must be the ones already on disk, or every signed session
    // and every two-factor enrolment silently stops working.
    runtime::uninstall_and_remove(&app, false).unwrap();
    let again = runtime::install_template(&template, "flatnotes qualification", &answers).unwrap();
    runtime::wait_for_health(&again.launch_url, Duration::from_secs(120)).unwrap();
    assert_eq!(
        docker(&["exec", &container, "printenv", "FLATNOTES_SECRET_KEY"]),
        signing,
        "the reinstall minted a new signing credential over preserved notes"
    );
    assert_eq!(
        docker(&["exec", &container, "printenv", "FLATNOTES_TOTP_KEY"]),
        totp
    );
    probe("verify", &again.launch_url);

    let image_id = docker(&["inspect", &container, "--format", "{{.Image}}"]);
    runtime::uninstall_and_remove(&again, true).unwrap();
    let filter = format!("label=com.docker.compose.project={project}");
    for resource in ["container", "network", "volume"] {
        assert!(docker(&[resource, "ls", "-q", "--filter", &filter]).is_empty());
    }
    assert!(!project_dir.exists(), "the notes folder survived a delete");
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
        "scope": "Runtipi reviewed mapping with typed setup answers, transaction, managed storage \
                  and browser first use, on one Windows host and one architecture",
        "browser_first_use": "passed",
        "promotion": reviewed.promotion.state,
        "source_revision": reviewed.origin.revision,
        "image": image,
        "image_id": image_id,
        "checks": [
            "loopback endpoint, and no binding on any other interface",
            "HTTP readiness",
            "the typed username reaches the container as the value typed",
            "two credentials are generated, distinct, and are not any typed answer",
            "a wrong password is refused by the running app",
            "the typed password signs in through a real browser",
            "notes are plain markdown in the managed folder, and the app reads them",
            "generated credentials are stored beside the data, without any typed answer",
            "sign-in and the note survive a restart",
            "a keep-data reinstall reuses both generated credentials rather than minting new ones",
            "sign-in and the note survive that reinstall",
            "delete-data removes the notes folder",
            "unrelated containers retained"
        ]
    });
    std::fs::write(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("docs/evidence/flatnotes-setup-answers-windows-2026-09-10.json"),
        serde_json::to_string_pretty(&evidence).unwrap(),
    )
    .unwrap();
}
