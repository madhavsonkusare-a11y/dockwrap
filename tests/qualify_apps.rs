//! Every offered app, through one harness.
//!
//! This replaces the per-app lifecycle tests, which had each grown their own
//! copy of the same scaffolding — a Docker helper, a cleanup guard, a private
//! config root, a hand-written evidence file. What is left here is the part
//! that is genuinely per-app: which questions it is installed with, and how to
//! tell whether a person could actually use it.
use local_store::qualification::{qualify, Batch, Evidence, Resume, ScriptProbe};
use std::collections::BTreeMap;
use std::path::PathBuf;

fn scratch() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".cache")
}

fn script(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("scripts")
        .join(name)
}

/// The apps this harness knows how to use, and how.
///
/// A probe script takes the phase and the address; anything after that is the
/// app's own business.
///
/// `flatnotes` is deliberately absent: its probe still relies on a note the
/// old test wrote into the managed folder from the host, so it is not yet
/// self-contained. Its dedicated test stays until the probe creates that note
/// through the app itself.
fn plan() -> Vec<(&'static str, BTreeMap<String, String>, ScriptProbe)> {
    vec![
        (
            "privatebin",
            BTreeMap::new(),
            ScriptProbe::new(
                script("privatebin-browser-probe.mjs"),
                "an encrypted paste is created in a real browser and decrypted from its own link",
            )
            .with_args(vec![scratch()
                .join("privatebin-probe-state.json")
                .to_string_lossy()
                .into_owned()]),
        ),
        (
            "nodered",
            BTreeMap::new(),
            ScriptProbe::new(
                script("nodered-probe.mjs"),
                "the editor holds its websocket and a deployed flow returns a value it computes",
            )
            .with_args(vec![scratch()
                .join("nodered-probe-state.json")
                .to_string_lossy()
                .into_owned()]),
        ),
    ]
}

/// Run the whole shortlist, resuming rather than repeating.
///
/// Opt in like the other Docker tests. Set `LOCAL_STORE_QUALIFY_RETRY=1` to
/// try the failures again; by default a recorded result is a result.
#[test]
#[ignore = "real Docker; enable LOCAL_STORE_RUN_DOCKER_TEST=1"]
fn every_offered_app_is_usable_and_keeps_its_data() {
    assert_eq!(
        std::env::var("LOCAL_STORE_RUN_DOCKER_TEST").as_deref(),
        Ok("1")
    );
    let results = scratch().join("qualification");
    let batch = Batch::open(&results).expect("a results directory");
    let plan = plan();
    let apps: Vec<String> = plan.iter().map(|(app, ..)| (*app).to_owned()).collect();
    let resume = if std::env::var("LOCAL_STORE_QUALIFY_RETRY").as_deref() == Ok("1") {
        Resume::RetryFailures
    } else {
        Resume::SkipRecorded
    };

    let todo: Vec<String> = batch
        .remaining(&apps, resume)
        .into_iter()
        .cloned()
        .collect();
    println!(
        "{} app(s) to qualify, {} already recorded",
        todo.len(),
        apps.len() - todo.len()
    );

    for (app, answers, probe) in plan {
        if !todo.iter().any(|name| name == app) {
            println!("{app}: already recorded, skipping");
            continue;
        }
        let evidence = qualify(app, &answers, &probe, &scratch())
            .unwrap_or_else(|error| panic!("{app} could not be run at all: {}", error.message));
        batch.record(&evidence).expect("a result is recorded");
        // The batch directory is scratch; a reviewed template names a proof
        // that has to be in the tree, so the same result is written there too.
        std::fs::write(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("docs/evidence")
                .join(format!("{app}-qualification.json")),
            evidence.to_json()
                + "
",
        )
        .expect("evidence is written where a review can read it");
        println!(
            "{app}: {}",
            if evidence.passed {
                "passed".to_owned()
            } else {
                format!("FAILED at {:?}", evidence.failure().map(|step| &step.step))
            }
        );
    }

    // Everything recorded has to have passed, and every app has to have a
    // record — a batch that skipped an app is not a batch that passed it.
    let recorded: Vec<Evidence> = batch.results();
    for app in &apps {
        let found = recorded
            .iter()
            .find(|evidence| &evidence.app == app)
            .unwrap_or_else(|| panic!("{app} has no recorded result"));
        assert!(
            found.passed,
            "{app} failed at {:?}: {:?}",
            found.failure().map(|step| &step.step),
            found.failure().and_then(|step| step.detail.as_deref())
        );
        // A run that recorded nothing proves nothing.
        assert!(found.steps.len() > 5, "{app} recorded too few steps");
    }
}
