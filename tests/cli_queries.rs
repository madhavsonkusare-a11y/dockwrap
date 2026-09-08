//! Exercise the real CLI without Docker, a display server, or user registry data.
use serde_json::Value;
use std::process::{Command, Output};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_local-store"))
        .args(args)
        .env("PATH", "") // Missing Docker is intentional and deterministic.
        .output()
        .expect("CLI must start")
}

fn json(args: &[&str]) -> Value {
    let output = run(args);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    serde_json::from_slice(&output.stdout).expect("stdout must contain only JSON")
}

#[test]
fn catalog_pages_are_distinct_and_supply_a_next_offset() {
    let first = json(&["catalog", "--json", "--limit", "2"]);
    let second = json(&["catalog", "--json", "--limit=2", "--offset=2"]);
    assert_eq!(first["entries"].as_array().unwrap().len(), 2);
    assert_eq!(first["next_offset"], 2);
    assert_eq!(second["offset"], 2);
    assert_eq!(second["next_offset"], 4);
    assert_eq!(first["total"], second["total"]);
    for a in first["entries"].as_array().unwrap() {
        assert!(!second["entries"]
            .as_array()
            .unwrap()
            .iter()
            .any(|b| a["id"] == b["id"]));
    }
    let past_end = json(&["catalog", "--json", "--offset", &first["total"].to_string()]);
    assert!(past_end["entries"].as_array().unwrap().is_empty());
    assert!(past_end["next_offset"].is_null());
}

#[test]
fn catalog_preview_filter_preserves_the_recipe_allowlist() {
    let result = json(&["catalog", "--capability", "preview_install", "--json"]);
    assert_eq!(result["total"], 3);
    assert!(result["next_offset"].is_null());
    let mut ids: Vec<_> = result["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["recipe_id"].as_str().unwrap())
        .collect();
    ids.sort();
    assert_eq!(ids, ["memos", "n8n", "uptime-kuma"]);
    let none = json(&[
        "catalog",
        "--query",
        "no-project-with-this-name-zzzz",
        "--json",
    ]);
    assert_eq!(none["total"], 0);
    assert!(none["next_offset"].is_null());
}

#[test]
fn doctor_reports_missing_docker_as_json_with_failure_exit() {
    let output = run(&["doctor", "--json"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stderr.is_empty());
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["ready"], false);
    let checks = report["checks"].as_array().unwrap();
    assert_eq!(checks.len(), 2);
    assert!(checks.iter().all(|check| check["ok"] == false));
    for check in checks {
        assert_eq!(check["error"]["code"], "process_unavailable");
        assert_eq!(check["error"]["message"], check["detail"]);
    }
    assert!(checks
        .iter()
        .all(|check| !check["detail"].as_str().unwrap().is_empty()));
}

#[test]
fn help_succeeds_and_bad_options_are_usage_errors() {
    for args in [
        vec!["--help"],
        vec!["doctor", "--help"],
        vec!["catalog", "--help"],
    ] {
        let output = run(&args);
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        assert!(String::from_utf8_lossy(&output.stdout).contains("Usage:"));
    }
    for args in [
        vec!["doctor", "--invalid"],
        vec!["doctor", "--json", "--json"],
        vec!["catalog", "--limit", "0"],
        vec!["catalog", "--json", "--offset", "-1"],
        vec!["catalog", "--json", "--unknown"],
    ] {
        let output = run(&args);
        assert_eq!(output.status.code(), Some(2), "{args:?}");
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
}

#[test]
fn malformed_actions_fail_before_accessing_the_registry() {
    for args in [
        vec!["uninstall", "memos", "--delete-dtaa"],
        vec!["remove", "memos", "--delete-data"],
        vec!["start", "memos", "extra"],
        vec!["add", "notes", "--url", "file:///tmp/private"],
        vec!["open", "memos", "--browser", "--unknown"],
    ] {
        let output = run(&args);
        assert_eq!(output.status.code(), Some(2), "{args:?}");
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
}

#[test]
fn human_catalog_output_has_stable_ids_and_paging_guidance() {
    let output = run(&["catalog", "--limit", "1"]);
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("ID"));
    assert!(text.contains("Next page:"));
    assert!(text.contains("--offset 1 --limit 1"));
    assert!(!text.contains("reviewed installs"));
}
