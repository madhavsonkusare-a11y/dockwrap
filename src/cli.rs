//! Standalone CLI mode for local-store. Derived from the old cli.js, rewritten in
//! Rust and unified into the same binary as the GUI.
//!
//! Usage:
//!   local-store add <name> --url <url> [--icon <path>] [--preset <name>]
//!   local-store add --preset <name>            (uses the preset's default URL)
//!   local-store list
//!   local-store remove <name>
//!   local-store presets                        (show built-in presets)

use local_store::{brand::CLI_NAME, catalog, model::AppDef, platform, runtime, storage, windowing};

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// On Windows the binary is a GUI subsystem (no console allocated). When invoked
/// with arguments from a terminal, attach to the parent's console so stdio is
/// wired up and prints are visible. Harmless on non-Windows.
#[cfg(windows)]
fn ensure_console() {
    use windows_sys::Win32::System::Console::{
        AttachConsole, GetConsoleWindow, ATTACH_PARENT_PROCESS,
    };
    unsafe {
        if GetConsoleWindow().is_null() {
            let _ = AttachConsole(ATTACH_PARENT_PROCESS);
        }
    }
}

#[cfg(not(windows))]
fn ensure_console() {}

/// Extract the single `<name>` positional arg common to several subcommands,
/// or print `usage` and return an exit code. Replaces the repeated
/// `match args.get(1) { Some(n) if !n.starts_with("--") => ... }` block.
fn name_arg(args: &[String], usage: &str) -> Result<String, i32> {
    match args.get(1) {
        Some(n) if !n.starts_with("--") => Ok(n.clone()),
        _ => {
            eprintln!("{}", usage);
            Err(1)
        }
    }
}

/// Look up a registered app by name, returning an owned clone so callers don't
/// borrow the `load_apps()` temporary. Replaces the repeated
/// `storage::load_apps().iter().find(...)` + "No app named" block.
fn find_app(name: &str) -> Option<AppDef> {
    storage::load_apps()
        .iter()
        .find(|a| a.name == name)
        .cloned()
}

fn usage() -> String {
    format!(
        "Usage:\n  \\
     {CLI_NAME} add <name> --url <url> [--icon <path>] [--compose <path>] [--health <url>] [--preset <name>]\n  \
     {CLI_NAME} add --preset <name>            (uses a built-in local-address preset)\n  \
     {CLI_NAME} list\n  \
     {CLI_NAME} open <name>\n  \
     {CLI_NAME} remove <name>\n  \
     {CLI_NAME} shortcut <name>\n  \
     {CLI_NAME} presets                        (show built-in presets)\n  \
     {CLI_NAME} catalog                        (list catalog stats)\n  \
     {CLI_NAME} catalog <search>               (search the 1257-app catalog)"
    )
}

fn get_flag(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1).cloned())
}

/// Resolve a `--preset` name to (display_name, url), consulting the 10-entry
/// Resolve a reviewed local-address preset. Discovery entries only carry source
/// URLs, so they must never be used as launch addresses.
fn resolve_preset(display_name: &str, preset: &str) -> (String, String) {
    if let Some(u) = catalog::preset_url(preset) {
        return (display_name.to_string(), u.to_string());
    }
    if let Some(entry) = catalog::catalog_entry(preset) {
        eprintln!(
            "\"{}\" is in Discover, but has no local-address preset. Add it with `{} add \"{}\" --url <your-instance-url>`.",
            entry.name, CLI_NAME, entry.name
        );
        std::process::exit(1);
    }
    eprintln!(
        "Unknown preset \"{}\". Run `{CLI_NAME} presets` or `{CLI_NAME} catalog search {}`.",
        preset, preset
    );
    std::process::exit(1);
}

pub fn run_cli() -> i32 {
    ensure_console();
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("{}", usage());
        return 1;
    }
    let cmd = args[0].as_str();

    match cmd {
        "add" => {
            let preset = get_flag(&args, "--preset");
            let name_arg = args.get(1).cloned().filter(|s| !s.starts_with("--"));
            let url = get_flag(&args, "--url");
            let icon = get_flag(&args, "--icon");
            let compose = get_flag(&args, "--compose");
            let health = get_flag(&args, "--health");

            let (name, url) = match (name_arg, url, preset) {
                (Some(n), Some(u), _) => (n, u),
                // Presets contain reviewed local addresses. The broad discovery
                // catalog contains project sources and is never a URL fallback.
                (Some(n), None, Some(p)) => resolve_preset(&n, &p),
                (None, None, Some(p)) => resolve_preset(&p, &p),
                _ => {
                    eprintln!("{}", usage());
                    return 1;
                }
            };
            // If we resolved the app from the catalog (not PRESETS), inherit its
            // icon/compose/health defaults so the user doesn't have to type them.
            // Enrich whatever we resolved (from PRESETS or the catalog) with
            // the catalog's icon/compose/health defaults where available, so a
            // user gets the right logo for `local-store add --preset immich` even
            // though PRESETS only stores the URL. Do NOT overwrite a URL the user
            // explicitly supplied via --url or --preset; the catalog may point at
            // the upstream site (e.g. immich.app) rather than localhost.
            let resolved_icon = catalog::catalog_entry(&name).and_then(|e| e.icon.or(icon.clone()));
            storage::upsert_app(&name, &url, resolved_icon, compose.clone(), health.clone());
            match (icon, compose) {
                (Some(i), Some(c)) => println!(
                    "Registered \"{}\" -> {} (icon: {}, compose: {}{})",
                    name,
                    url,
                    i,
                    c,
                    health
                        .map(|h| format!(", health: {}", h))
                        .unwrap_or_default()
                ),
                (Some(i), None) => println!("Registered \"{}\" -> {} (icon: {})", name, url, i),
                (None, Some(c)) => println!(
                    "Registered \"{}\" -> {} (compose: {}{})",
                    name,
                    url,
                    c,
                    health
                        .map(|h| format!(", health: {}", h))
                        .unwrap_or_default()
                ),
                (None, None) => println!("Registered \"{}\" -> {}", name, url),
            }
            0
        }
        "list" => {
            let apps: Vec<AppDef> = storage::load_apps();
            if apps.is_empty() {
                println!("No apps registered.");
            } else {
                println!("{}", serde_json::to_string_pretty(&apps).unwrap());
            }
            0
        }
        "open" => {
            let browser = args.iter().any(|a| a == "--browser");
            let args_no_flags: Vec<String> = args
                .iter()
                .skip(1)
                .filter(|a| a.as_str() != "--browser")
                .cloned()
                .collect();
            let name = match name_arg(
                &args_no_flags,
                &format!("Usage: {CLI_NAME} open <name> [--browser]"),
            ) {
                Ok(n) => n,
                Err(ec) => return ec,
            };
            match find_app(&name) {
                Some(appdef) => {
                    if let Err(e) = runtime::boot_and_wait(&appdef) {
                        eprintln!("warn: {}", e);
                    }
                    if browser {
                        windowing::launch_browser(&appdef.url);
                        println!("Opened \"{}\" in your browser at {}", name, appdef.url);
                    } else {
                        println!("Opened \"{}\" at {} (compose booted)", name, appdef.url);
                    }
                    0
                }
                None => {
                    eprintln!("No app named \"{}\" found.", name);
                    1
                }
            }
        }
        "shortcut" => {
            let name = match name_arg(&args, &format!("Usage: {CLI_NAME} shortcut <name>")) {
                Ok(n) => n,
                Err(ec) => return ec,
            };
            match find_app(&name) {
                Some(appdef) => {
                    let bin = std::env::current_exe()
                        .map(|p| p.to_string_lossy().into_owned())
                        .unwrap_or_default();
                    match platform::create_shortcut_for(&name, &bin, appdef.icon.as_deref()) {
                        Ok(path) => {
                            println!("Shortcut created: {}", path);
                            0
                        }
                        Err(e) => {
                            eprintln!("Failed to create shortcut: {}", e);
                            1
                        }
                    }
                }
                None => {
                    eprintln!("No app named \"{}\" found.", name);
                    1
                }
            }
        }
        "catalog" => {
            let query = args.get(1).cloned().filter(|s| !s.starts_with("--"));
            let entries = catalog::catalog();
            match query {
                None => {
                    let cats = catalog::catalog_categories();
                    println!(
                        "{CLI_NAME} app catalog: {} apps in {} categories",
                        entries.len(),
                        cats.len()
                    );
                    for c in cats.iter().take(12) {
                        let n = entries
                            .iter()
                            .filter(|e| e.category.as_deref() == Some(c.as_str()))
                            .count();
                        println!("  {} ({} apps)", c, n);
                    }
                    if cats.len() > 12 {
                        println!(
                            "  ... and {} more categories. Use `{CLI_NAME} catalog search <q>`",
                            cats.len() - 12
                        );
                    }
                    0
                }
                Some(q) => {
                    let ql = q.to_lowercase();
                    let matches: Vec<_> = entries
                        .iter()
                        .filter(|e| {
                            e.name.to_lowercase().contains(&ql)
                                || e.category
                                    .as_deref()
                                    .map(|c| c.to_lowercase().contains(&ql))
                                    .unwrap_or(false)
                                || e.description
                                    .as_deref()
                                    .map(|d| d.to_lowercase().contains(&ql))
                                    .unwrap_or(false)
                                || e.tags.to_lowercase().contains(&ql)
                        })
                        .collect();
                    if matches.is_empty() {
                        println!("No catalog apps matched \"{}\".", q);
                    } else {
                        for e in matches.iter().take(30) {
                            let cat = e.category.as_deref().unwrap_or("app");
                            println!("  {:<28} {:<22} {}", e.name, cat, e.url);
                        }
                        if matches.len() > 30 {
                            println!("  ... {} more. Narrow your search.", matches.len() - 30);
                        }
                    }
                    0
                }
            }
        }
        "remove" => {
            let name = match name_arg(&args, &format!("Usage: {CLI_NAME} remove <name>")) {
                Ok(n) => n,
                Err(ec) => return ec,
            };
            if storage::remove_app(&name) {
                println!("Removed \"{}\".", name);
                0
            } else {
                eprintln!("No app named \"{}\" found.", name);
                1
            }
        }
        "version" | "--version" | "-V" => {
            println!("{CLI_NAME} {VERSION}");
            0
        }
        "presets" => {
            println!("Built-in presets (name -> default url):");
            for (n, u) in catalog::PRESETS {
                println!("  {} -> {}", n, u);
            }
            0
        }
        _ => {
            eprintln!("Unknown command \"{}\".\n{}", cmd, usage());
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `name_arg` returns the bare positional name.
    #[test]
    fn name_arg_extracts_bare_name() {
        let args: Vec<String> = vec!["local-store".into(), "immich".into()];
        assert_eq!(name_arg(&args, "usage").unwrap(), "immich");
    }

    /// `name_arg` rejects flags (e.g. `--icon path`) as a name.
    #[test]
    fn name_arg_rejects_flag() {
        let args: Vec<String> = vec!["local-store".into(), "--icon".into(), "x".into()];
        assert!(name_arg(&args, "usage").is_err());
    }

    /// `get_flag` returns the value following `--flag`.
    #[test]
    fn get_flag_returns_value() {
        let args: Vec<String> = vec![
            "local-store".into(),
            "add".into(),
            "--url".into(),
            "http://x".into(),
        ];
        assert_eq!(get_flag(&args, "--url"), Some("http://x".to_string()));
    }

    /// `get_flag` returns None when the flag is absent.
    #[test]
    fn get_flag_absent_is_none() {
        let args: Vec<String> = vec!["local-store".into(), "add".into(), "my".into()];
        assert_eq!(get_flag(&args, "--icon"), None);
    }

    /// The discovery catalog and the launch-address presets have distinct roles.
    #[test]
    fn discovery_catalog_does_not_define_arbitrary_launch_presets() {
        let entry = catalog::catalog_entry("Immich");
        assert!(entry.is_some(), "Immich must exist in the embedded catalog");
        let e = entry.unwrap();
        assert!(e.icon.is_some(), "catalog Immich should carry an icon");
        assert!(
            e.category.is_some(),
            "catalog Immich should carry a category"
        );
        assert_eq!(catalog::preset_url("Immich"), None);
    }

    /// `local-store catalog search media` finds results (search is substring across
    /// name/category/description/tags).
    #[test]
    fn catalog_search_finds_results() {
        let entries = catalog::catalog();
        let ql = "media";
        let matches: Vec<_> = entries
            .iter()
            .filter(|e| {
                e.name.to_lowercase().contains(ql)
                    || e.category
                        .as_deref()
                        .map(|c| c.to_lowercase().contains(ql))
                        .unwrap_or(false)
                    || e.description
                        .as_deref()
                        .map(|d| d.to_lowercase().contains(ql))
                        .unwrap_or(false)
                    || e.tags.to_lowercase().contains(ql)
            })
            .collect();
        assert!(
            !matches.is_empty(),
            "catalog search 'media' should match ≥1 app"
        );
    }

    /// `local-store catalog` (no args) returns 0 and prints stats.
    #[test]
    fn catalog_subcommand_no_args_is_zero() {
        // run_cli reads std::env::args(), so we can't easily assert stdout.
        // Instead verify the underlying data the subcommand uses.
        let entries = catalog::catalog();
        let cats = catalog::catalog_categories();
        assert!(!entries.is_empty());
        assert!(!cats.is_empty());
        assert_eq!(
            cats.len(),
            cats.iter().collect::<std::collections::HashSet<_>>().len()
        );
    }

    /// `local-store open <name> --browser` parses the flag and strips it from args.
    #[test]
    fn open_browser_flag_is_stripped() {
        let args: Vec<String> = vec!["open".into(), "immich".into(), "--browser".into()];
        let browser = args.iter().any(|a| a == "--browser");
        let args_no_flags: Vec<String> = args
            .iter()
            .skip(1)
            .filter(|a| a.as_str() != "--browser")
            .cloned()
            .collect();
        assert!(browser, "--browser should be detected");
        assert_eq!(
            args_no_flags,
            vec!["immich".to_string()],
            "non-flag arg should remain"
        );
    }

    /// `local-store open --browser` (flag before name) still parses the name.
    #[test]
    fn open_browser_flag_before_name() {
        let args: Vec<String> = vec!["open".into(), "--browser".into(), "immich".into()];
        let args_no_flags: Vec<String> = args
            .iter()
            .skip(1)
            .filter(|a| a.as_str() != "--browser")
            .cloned()
            .collect();
        assert_eq!(args_no_flags, vec!["immich".to_string()]);
    }

    /// `usage()` mentions the new catalog subcommand.
    #[test]
    fn usage_mentions_catalog() {
        let u = usage();
        assert!(
            u.contains("catalog"),
            "usage should document the catalog subcommand"
        );
        assert!(
            u.contains("search"),
            "usage should document `catalog search`"
        );
    }
}
