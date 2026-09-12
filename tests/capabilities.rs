//! Every IPC command must be declared in three places that cannot disagree.
//!
//! A command registered in `generate_handler!` but missing from `build.rs` gets
//! no generated permission, and one missing from the launcher capability is
//! never granted. Either way the command exists, compiles, and is **rejected at
//! runtime** — with no build error and no test failure, because the frontend
//! suites drive a mocked `window.__TAURI__` that has no access control at all.
//!
//! This has already happened once: `cancel_app_setup` and `app_readiness` were
//! added to `generate_handler!` alone and would have shipped inert.
use std::collections::BTreeSet;

fn read(path: &str) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|error| panic!("read {path}: {error}"))
}

/// The section of `text` between `open` and the next `close`.
fn between<'a>(text: &'a str, open: &str, close: &str) -> &'a str {
    let start = text
        .find(open)
        .unwrap_or_else(|| panic!("{open:?} not found"))
        + open.len();
    let rest = &text[start..];
    let end = rest
        .find(close)
        .unwrap_or_else(|| panic!("{close:?} not found after {open:?}"));
    &rest[..end]
}

fn quoted(text: &str) -> BTreeSet<String> {
    text.split('"')
        .skip(1)
        .step_by(2)
        .map(str::to_owned)
        .collect()
}

/// Commands passed to `tauri::generate_handler!` in `src/main.rs`.
///
/// Some are defined in `commands.rs` and listed with that prefix; others are
/// defined in `main.rs` itself and listed bare. Both forms count.
fn registered_commands() -> BTreeSet<String> {
    between(&read("src/main.rs"), "generate_handler![", "]")
        .split(',')
        .map(|entry| {
            let entry = entry.trim();
            entry.rsplit("::").next().unwrap_or(entry).trim()
        })
        .filter(|name| !name.is_empty() && name.chars().all(|c| c.is_ascii_lowercase() || c == '_'))
        .map(str::to_owned)
        .collect()
}

/// Commands declared to `tauri_build`, which generates their permissions.
fn declared_commands() -> BTreeSet<String> {
    quoted(between(&read("build.rs"), ".commands(&[", "])"))
}

/// Commands the launcher capability actually grants.
fn granted_commands() -> BTreeSet<String> {
    let config: serde_json::Value =
        serde_json::from_str(&read("tauri.conf.json")).expect("tauri.conf.json must be valid JSON");
    let capabilities = config["app"]["security"]["capabilities"]
        .as_array()
        .expect("capabilities must be an array");
    let launcher = capabilities
        .iter()
        .find(|capability| capability["identifier"] == "launcher")
        .expect("a launcher capability must exist");
    assert_eq!(
        launcher["windows"].as_array().map(Vec::len),
        Some(1),
        "the launcher capability must apply to exactly one window"
    );
    assert_eq!(
        launcher["windows"][0], "launcher",
        "app windows must never be granted launcher commands"
    );
    launcher["permissions"]
        .as_array()
        .expect("permissions must be an array")
        .iter()
        .filter_map(|permission| permission.as_str())
        .filter_map(|permission| permission.strip_prefix("allow-"))
        .map(|command| command.replace('-', "_"))
        .collect()
}

#[test]
fn every_registered_command_is_declared_and_granted() {
    let registered = registered_commands();
    assert!(
        registered.len() > 10,
        "the handler list was not parsed: {registered:?}"
    );

    let declared = declared_commands();
    let missing: Vec<_> = registered.difference(&declared).collect();
    assert!(
        missing.is_empty(),
        "registered but absent from build.rs, so no permission is generated: {missing:?}"
    );

    let granted = granted_commands();
    let ungranted: Vec<_> = registered.difference(&granted).collect();
    assert!(
        ungranted.is_empty(),
        "registered but not granted to the launcher, so rejected at runtime: {ungranted:?}"
    );
}

#[test]
fn nothing_is_declared_or_granted_that_no_longer_exists() {
    let registered = registered_commands();

    let stale_declarations: Vec<_> = declared_commands()
        .difference(&registered)
        .cloned()
        .collect();
    assert!(
        stale_declarations.is_empty(),
        "declared in build.rs but no longer registered: {stale_declarations:?}"
    );

    let stale_grants: Vec<_> = granted_commands()
        .difference(&registered)
        .cloned()
        .collect();
    assert!(
        stale_grants.is_empty(),
        "granted by the capability but no longer registered: {stale_grants:?}"
    );
}

#[test]
fn the_launcher_capability_keeps_its_security_boundary() {
    let config: serde_json::Value =
        serde_json::from_str(&read("tauri.conf.json")).expect("tauri.conf.json must be valid JSON");
    let security = &config["app"]["security"];
    let csp = security["csp"].as_str().expect("a CSP must be configured");
    for directive in [
        "default-src 'self'",
        "object-src 'none'",
        "frame-src 'none'",
        "base-uri 'self'",
    ] {
        assert!(csp.contains(directive), "CSP lost {directive:?}: {csp}");
    }
    assert!(
        !csp.contains("unsafe-inline") && !csp.contains("unsafe-eval"),
        "the CSP must not allow inline or evaluated script: {csp}"
    );
}
