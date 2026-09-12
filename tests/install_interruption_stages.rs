//! The interruption stages the first recovery test did not reach.
//!
//! `install_interruption.rs` kills a healthy install just before its registry
//! commit — the latest possible moment, and the easiest to reason about. The
//! two earlier stages behave differently and were never covered:
//!
//! * during the image pull, when there is no container yet and possibly no
//!   project directory either;
//! * during container startup, when a container exists that nothing in the
//!   registry accounts for.
//!
//! The second is also where adoption earns its place: the files and the
//! container are minutes of work, and clearing them was previously the only
//! answer the product had.
use local_store::runtime::{HealthProbe, HttpHealthProbe};
use std::path::Path;
use std::process::{Command, Output};
use std::time::{Duration, Instant};

/// An app that asks no setup questions, so the CLI can drive the whole flow,
/// and small enough that removing and re-pulling its image is cheap.
const APP: &str = "privatebin";
const IMAGE: &str = "privatebin/nginx-fpm-alpine:2.0.6";
const PROJECT: &str = "local-store-privatebin";

struct OwnedChild(std::process::Child);
impl Drop for OwnedChild {
    fn drop(&mut self) {
        let _ = self.0.kill();
    }
}

fn docker(args: &[&str]) -> String {
    let out = Command::new("docker").args(args).output().unwrap();
    assert!(
        out.status.success(),
        "docker {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_owned()
}

fn cli(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_local-store"))
        .args(args)
        .env("APPDATA", root)
        .env("XDG_CONFIG_HOME", root)
        .output()
        .unwrap()
}

fn owned_containers() -> String {
    docker(&[
        "container",
        "ls",
        "-aq",
        "--filter",
        &format!("label=com.docker.compose.project={PROJECT}"),
    ])
}

/// Remove everything this test could have created, and nothing else.
fn clean_project() {
    for (kind, list) in [("container", "-aq"), ("network", "-q"), ("volume", "-q")] {
        let ids = docker(&[
            kind,
            "ls",
            list,
            "--filter",
            &format!("label=com.docker.compose.project={PROJECT}"),
        ]);
        for id in ids.split_whitespace() {
            let mut args = vec![kind, "rm"];
            if kind == "container" {
                args.push("-f");
            }
            args.push(id);
            let _ = Command::new("docker").args(&args).output();
        }
    }
}

struct Cleanup;
impl Drop for Cleanup {
    fn drop(&mut self) {
        clean_project();
    }
}

fn private_root(name: &str) -> std::path::PathBuf {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".cache")
        .join(format!(
            "interruption-{name}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
    std::fs::create_dir_all(&root).unwrap();
    root
}

fn require_clean_start() {
    assert!(
        owned_containers().is_empty(),
        "this machine already has containers for {APP}; refusing to touch them"
    );
}

#[test]
#[ignore = "real Docker; enable LOCAL_STORE_RUN_DOCKER_TEST=1"]
fn an_install_interrupted_while_pulling_leaves_nothing_and_can_be_retried() {
    assert_eq!(
        std::env::var("LOCAL_STORE_RUN_DOCKER_TEST").as_deref(),
        Ok("1")
    );
    require_clean_start();
    let _cleanup = Cleanup;
    let root = private_root("pull");

    // The pull has to actually happen for this stage to exist. The image is
    // restored by the retry at the end of this test, so the machine ends up
    // with what it started with.
    let _ = Command::new("docker")
        .args(["image", "rm", "-f", IMAGE])
        .output();
    assert!(
        Command::new("docker")
            .args(["image", "inspect", IMAGE])
            .output()
            .unwrap()
            .status
            .success()
            .eq(&false),
        "the image is still cached, so this test would not interrupt a pull"
    );

    let mut install = OwnedChild(
        Command::new(env!("CARGO_BIN_EXE_local-store"))
            .args(["install", APP])
            .env("APPDATA", &root)
            .env("XDG_CONFIG_HOME", &root)
            .spawn()
            .unwrap(),
    );

    // Kill while the pull is still in flight: no container has been created
    // yet, which is what makes this a different stage from the others.
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        assert!(
            install.0.try_wait().unwrap().is_none(),
            "the install finished before the pull could be interrupted"
        );
        assert!(
            owned_containers().is_empty(),
            "a container existed during what should still be the pull"
        );
        if Instant::now() > deadline - Duration::from_secs(17) {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    install.0.kill().unwrap();
    install.0.wait().unwrap();

    // Nothing running, nothing registered.
    assert!(owned_containers().is_empty());
    let listed = cli(&root, &["list"]);
    assert!(listed.status.success());
    assert!(
        !String::from_utf8_lossy(&listed.stdout).contains(APP),
        "an interrupted pull registered an app"
    );

    // Recovery must not invent a candidate out of a directory with no Compose
    // file in it, and must not fail either. Both outcomes are legitimate here
    // depending on how far the install got, so the assertion is about what
    // recovery says, not about which of the two it found.
    let inspection = cli(&root, &["recovery", "--json", "--docker"]);
    assert!(
        inspection.status.success(),
        "recovery inspection failed after an interrupted pull: {}",
        String::from_utf8_lossy(&inspection.stderr)
    );
    let candidates: serde_json::Value = serde_json::from_slice(&inspection.stdout).unwrap();
    let candidates = candidates.as_array().expect("recovery returns a list");
    for candidate in candidates {
        assert_eq!(candidate["recipe_id"], APP);
        // No container was ever created, so nothing can be owned.
        assert_eq!(candidate["ownership_status"], "no_containers");
    }

    // Killing the CLI does not kill the `docker compose` it started: Docker
    // keeps pulling on its own, and a retry launched into the middle of that
    // collides with it. That is a real property of the system rather than an
    // artefact of this test, and it is why a person who kills an install and
    // immediately retries can see it fail once. Wait for Docker's own work to
    // settle, so what follows tests the retry rather than the collision.
    let settle = Instant::now() + Duration::from_secs(180);
    while Instant::now() < settle {
        let cached = Command::new("docker")
            .args(["image", "inspect", IMAGE])
            .output()
            .unwrap()
            .status
            .success();
        if cached {
            break;
        }
        std::thread::sleep(Duration::from_millis(500));
    }

    // The point of the stage: a retry has to work.
    let retry = cli(&root, &["install", APP]);
    assert!(
        retry.status.success(),
        "reinstall after an interrupted pull failed: {}",
        String::from_utf8_lossy(&retry.stderr)
    );
    assert!(!owned_containers().is_empty());
    let removed = cli(&root, &["uninstall", APP, "--delete-data"]);
    assert!(removed.status.success());
    assert!(owned_containers().is_empty());
}

#[test]
#[ignore = "real Docker; enable LOCAL_STORE_RUN_DOCKER_TEST=1"]
fn an_install_interrupted_during_startup_can_be_adopted_instead_of_cleared() {
    assert_eq!(
        std::env::var("LOCAL_STORE_RUN_DOCKER_TEST").as_deref(),
        Ok("1")
    );
    require_clean_start();
    let _cleanup = Cleanup;
    let root = private_root("startup");
    // Cached, so this test interrupts startup rather than a download.
    docker(&["pull", IMAGE]);

    let mut install = OwnedChild(
        Command::new(env!("CARGO_BIN_EXE_local-store"))
            .args(["install", APP])
            .env("APPDATA", &root)
            .env("XDG_CONFIG_HOME", &root)
            .spawn()
            .unwrap(),
    );

    // Kill as soon as a container exists. Whether it had become healthy by
    // then does not change what has to happen next: there is a container and
    // no registry entry, which is the state adoption exists for.
    let deadline = Instant::now() + Duration::from_secs(90);
    while owned_containers().is_empty() {
        assert!(
            install.0.try_wait().unwrap().is_none(),
            "the install finished before a container could be interrupted"
        );
        assert!(Instant::now() < deadline, "no container was ever created");
        std::thread::sleep(Duration::from_millis(50));
    }
    install.0.kill().unwrap();
    install.0.wait().unwrap();

    let orphan = owned_containers();
    assert!(!orphan.is_empty(), "the container was removed by the kill");
    let listed = cli(&root, &["list"]);
    assert!(!String::from_utf8_lossy(&listed.stdout).contains(APP));

    // Recovery sees it, and sees that we own it.
    let inspection = cli(&root, &["recovery", "--json", "--docker"]);
    assert!(inspection.status.success());
    let candidates: serde_json::Value = serde_json::from_slice(&inspection.stdout).unwrap();
    assert_eq!(candidates[0]["recipe_id"], APP);
    assert_eq!(candidates[0]["ownership_status"], "verified");

    // Adoption finishes what the install started, rather than throwing away a
    // container and a download.
    let adopted = cli(&root, &["adopt", APP]);
    assert!(
        adopted.status.success(),
        "adoption failed: {}",
        String::from_utf8_lossy(&adopted.stderr)
    );
    let said = String::from_utf8_lossy(&adopted.stdout).into_owned();
    assert!(said.contains("http://localhost:"), "{said}");

    // It is in My Apps now, and it answers at the address that was recorded.
    let listed = cli(&root, &["list"]);
    assert!(String::from_utf8_lossy(&listed.stdout).contains(APP));
    let address = said
        .split_whitespace()
        .find(|word| word.starts_with("http://localhost:"))
        .expect("adoption reported an address")
        .trim_end_matches(|c: char| !c.is_ascii_digit())
        .to_owned();
    assert!(
        HttpHealthProbe.ready(&address),
        "the adopted app does not answer at {address}"
    );

    // Adopting again is refused, because it is a real app now.
    let again = cli(&root, &["adopt", APP]);
    assert!(
        !again.status.success(),
        "an installed app was adopted twice"
    );

    // And it uninstalls like any other app, leaving nothing behind.
    let removed = cli(&root, &["uninstall", APP, "--delete-data"]);
    assert!(
        removed.status.success(),
        "uninstall of an adopted app failed: {}",
        String::from_utf8_lossy(&removed.stderr)
    );
    assert!(owned_containers().is_empty());
    assert!(!root.join("local-store/apps").join(APP).exists());
}
