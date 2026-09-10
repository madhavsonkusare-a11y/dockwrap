//! Run ranked candidates through the qualification harness, in bulk.
//!
//! Everything up to now has said what *could* be installed. This says what
//! actually works, which is the only number the v1 goal depends on and the one
//! nobody has. It installs each candidate for real, uses it, restarts it,
//! reinstalls it over its own data, and removes it — then records what
//! happened.
//!
//! **This is evidence gathering, not offering.** A candidate qualified here is
//! not thereby installable by anybody: `offerings` still resolves only
//! reviewed recipes and approved templates, and nothing in this example
//! changes that. It exists so a promotion decision has something to read.
//!
//! Resumable, because a hundred apps is hours and something will interrupt it.
//! Run again and it picks up where it stopped.
//!
//!     LOCAL_STORE_RUN_DOCKER_TEST=1 cargo run --release --example qualify_batch -- --limit 10
use local_store::qualification::{qualify_template, AnswersOnly, Batch, Resume, Subject};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// One candidate worth running, read from the ranking.
struct Candidate {
    id: String,
    source: String,
    revision: String,
    path: String,
}

fn ranked(limit: usize) -> Vec<Candidate> {
    let ranking: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root().join("catalog/candidate-ranking.json"))
            .expect("run scripts/rank-candidates.py first"),
    )
    .expect("the ranking is JSON");
    let queue: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root().join("catalog/candidate-queue.json"))
            .expect("run scripts/build-candidate-queue.py first"),
    )
    .expect("the queue is JSON");

    // Provenance lives in the queue; reach and licence live in the ranking.
    let mut provenance: BTreeMap<(String, String), (String, String)> = BTreeMap::new();
    let mut required: BTreeMap<(String, String), u64> = BTreeMap::new();
    for entry in queue["candidates"].as_array().unwrap_or(&Vec::new()) {
        let key = (
            entry["id"].as_str().unwrap_or_default().to_owned(),
            entry["source"].as_str().unwrap_or_default().to_owned(),
        );
        provenance.insert(
            key.clone(),
            (
                entry["provenance"]["revision"]
                    .as_str()
                    .unwrap_or_default()
                    .to_owned(),
                entry["provenance"]["path"]
                    .as_str()
                    .unwrap_or_default()
                    .to_owned(),
            ),
        );
        required.insert(key, entry["required_inputs"].as_u64().unwrap_or(0));
    }

    let mut chosen = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for entry in ranking["candidates"].as_array().unwrap_or(&Vec::new()) {
        if chosen.len() >= limit {
            break;
        }
        if entry["open_source"].as_bool() != Some(true)
            || !entry["importable"].as_bool().unwrap_or(false)
        {
            continue;
        }
        let key = (
            entry["id"].as_str().unwrap_or_default().to_owned(),
            entry["source"].as_str().unwrap_or_default().to_owned(),
        );
        // An app that needs answers cannot be run unattended; inventing an
        // administrator password would prove nothing about the real thing.
        if required.get(&key).copied().unwrap_or(0) != 0 {
            continue;
        }
        // One definition per app: two sources packaging the same thing is one
        // question, and the higher-ranked one is already first.
        if !seen.insert(key.0.clone()) {
            continue;
        }
        let Some((revision, path)) = provenance.get(&key) else {
            continue;
        };
        if revision.is_empty() || path.is_empty() {
            continue;
        }
        chosen.push(Candidate {
            id: key.0,
            source: key.1,
            revision: revision.clone(),
            path: path.clone(),
        });
    }
    chosen
}

/// The pinned definition and its companion config, extracted beside the
/// archives by `scripts/extract-definitions.py`.
fn definition(candidate: &Candidate) -> Option<(String, Option<String>)> {
    let base = root()
        .join(".cache/definitions")
        .join(&candidate.revision)
        .join(Path::new(&candidate.path).parent()?);
    // The extractor normalises CapRover's YAML into JSON, which is what its
    // importer reads.
    let mut file = PathBuf::from(Path::new(&candidate.path).file_name()?);
    if matches!(
        file.extension().and_then(|e| e.to_str()),
        Some("yml" | "yaml")
    ) {
        file.set_extension("json");
    }
    let definition = std::fs::read_to_string(base.join(file)).ok()?;
    let config = std::fs::read_to_string(base.join("config.json")).ok();
    Some((definition, config))
}

fn main() {
    if std::env::var("LOCAL_STORE_RUN_DOCKER_TEST").as_deref() != Ok("1") {
        eprintln!("This starts real containers. Set LOCAL_STORE_RUN_DOCKER_TEST=1 to run it.");
        std::process::exit(2);
    }
    let args: Vec<String> = std::env::args().collect();
    let limit = args
        .iter()
        .position(|arg| arg == "--limit")
        .and_then(|at| args.get(at + 1))
        .and_then(|value| value.parse().ok())
        .unwrap_or(10usize);
    let resume = if args.iter().any(|arg| arg == "--retry-failures") {
        Resume::RetryFailures
    } else {
        Resume::SkipRecorded
    };

    let results = root().join(".cache/qualification");
    let batch = Batch::open(&results).expect("a results directory");
    let candidates = ranked(limit);
    let ids: Vec<String> = candidates.iter().map(|c| c.id.clone()).collect();
    let todo = batch.remaining(&ids, resume);
    println!(
        "{} candidate(s) selected, {} already recorded, {} to run",
        ids.len(),
        ids.len() - todo.len(),
        todo.len()
    );

    let scratch = root().join(".cache");
    let (mut passed, mut failed, mut skipped) = (0, 0, 0);
    for candidate in &candidates {
        if !todo.iter().any(|id| **id == candidate.id) {
            continue;
        }
        let Some((text, config)) = definition(candidate) else {
            println!("{}: no pinned definition on disk, skipping", candidate.id);
            skipped += 1;
            continue;
        };
        let outcome = match candidate.source.as_str() {
            "runtipi" => {
                local_store::importers::runtipi::import(&candidate.id, &text, config.as_deref())
            }
            "caprover" => local_store::importers::caprover::import(&candidate.id, &text),
            other => {
                println!("{}: no importer for {other}", candidate.id);
                skipped += 1;
                continue;
            }
        };
        let Some(template) = outcome.ok().and_then(|outcome| outcome.template) else {
            println!("{}: no longer importable, skipping", candidate.id);
            skipped += 1;
            continue;
        };
        let about = Subject {
            app: candidate.id.clone(),
            kind: "candidate mapping",
            // Nothing here is promoted. Saying so in the evidence keeps a
            // passing run from reading like an approval.
            promotion: "not reviewed".to_owned(),
            source_revision: candidate.revision.clone(),
        };
        println!("{}: running…", candidate.id);
        match qualify_template(&about, template, &BTreeMap::new(), &AnswersOnly, &scratch) {
            Ok(evidence) => {
                let ok = evidence.passed;
                if let Err(error) = batch.record(&evidence) {
                    println!("  could not record: {}", error.message);
                }
                if ok {
                    passed += 1;
                    println!("  passed");
                } else {
                    failed += 1;
                    println!(
                        "  FAILED at {:?}: {:?}",
                        evidence.failure().map(|step| &step.step),
                        evidence.failure().and_then(|step| step.detail.as_deref())
                    );
                }
            }
            Err(error) => {
                failed += 1;
                println!("  could not run: {}", error.message);
            }
        }
    }
    println!("passed {passed}, failed {failed}, skipped {skipped}");
}
