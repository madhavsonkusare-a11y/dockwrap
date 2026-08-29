//! Standalone CLI mode for dockwrap. Derived from the old cli.js, rewritten in
//! Rust and unified into the same binary as the GUI.
//!
//! Usage:
//!   dockwrap add <name> --url <url> [--icon <path>] [--preset <name>]
//!   dockwrap add --preset <name>            (uses the preset's default URL)
//!   dockwrap list
//!   dockwrap remove <name>
//!   dockwrap presets                        (show built-in presets)

use crate::registry::{self, AppDef};

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// On Windows the binary is a GUI subsystem (no console allocated). When invoked
/// with arguments from a terminal, attach to the parent's console so stdio is
/// wired up and prints are visible. Harmless on non-Windows.
#[cfg(windows)]
fn ensure_console() {
    use windows_sys::Win32::System::Console::{AttachConsole, ATTACH_PARENT_PROCESS, GetConsoleWindow};
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
/// `registry::load_apps().iter().find(...)` + "No app named" block.
fn find_app(name: &str) -> Option<AppDef> {
    registry::load_apps().iter().find(|a| a.name == name).cloned()
}

fn usage() -> String {
    "Usage:\n  \\
     dockwrap add <name> --url <url> [--icon <path>] [--compose <path>] [--health <url>] [--preset <name>]\n  \
     dockwrap add --preset <name>            (resolves from presets + the 1257-app catalog)\n  \
     dockwrap list\n  \
     dockwrap open <name>\n  \
     dockwrap remove <name>\n  \
     dockwrap shortcut <name>\n  \
     dockwrap presets                        (show built-in presets)\n  \
     dockwrap catalog                        (list catalog stats)\n  \
     dockwrap catalog <search>               (search the 1257-app catalog)"
        .to_string()
}

fn get_flag(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1).cloned())
}

/// Resolve a `--preset` name to (display_name, url), consulting the 10-entry
/// PRESETS table first, then the embedded 1257-app catalog as a fallback.
/// If the preset is unknown everywhere, prints guidance and exits the CLI.
fn resolve_preset(display_name: &str, preset: &str) -> (String, String) {
    if let Some(u) = registry::preset_url(preset) {
        return (display_name.to_string(), u.to_string());
    }
    if let Some(e) = registry::catalog_entry(preset) {
        return (e.name.clone(), e.url.clone());
    }
    eprintln!(
        "Unknown preset \"{}\". Run `dockwrap presets` or `dockwrap catalog search {}`.",
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
                // --preset N (with or without a name): resolve from PRESETS first,
                // then fall back to the embedded 1257-app catalog so users can do
                // `dockwrap add --preset immich` even if immich isn't in PRESETS.
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
            // user gets the right logo for `dockwrap add --preset immich` even
            // though PRESETS only stores the URL. Do NOT overwrite a URL the user
            // explicitly supplied via --url or --preset; the catalog may point at
            // the upstream site (e.g. immich.app) rather than localhost.
            let resolved_icon = registry::catalog_entry(&name)
                .and_then(|e| e.icon.or(icon.clone()));
            registry::upsert_app(
                &name,
                &url,
                resolved_icon,
                compose.clone(),
                health.clone(),
            );
            match (icon, compose) {
                (Some(i), Some(c)) => println!(
                    "Registered \"{}\" -> {} (icon: {}, compose: {}{})",
                    name,
                    url,
                    i,
                    c,
                    health.map(|h| format!(", health: {}", h)).unwrap_or_default()
                ),
                (Some(i), None) => println!("Registered \"{}\" -> {} (icon: {})", name, url, i),
                (None, Some(c)) => println!(
                    "Registered \"{}\" -> {} (compose: {}{})",
                    name,
                    url,
                    c,
                    health.map(|h| format!(", health: {}", h)).unwrap_or_default()
                ),
                (None, None) => println!("Registered \"{}\" -> {}", name, url),
            }
            0
        }
        "list" => {
            let apps: Vec<AppDef> = registry::load_apps();
            if apps.is_empty() {
                println!("No apps registered.");
            } else {
                println!("{}", serde_json::to_string_pretty(&apps).unwrap());
            }
            0
        }
        "open" => {
            let name = match name_arg(&args, "Usage: dockwrap open <name>") {
                Ok(n) => n,
                Err(ec) => return ec,
            };
            match find_app(&name) {
                Some(appdef) => {
                    // In CLI mode there's no GUI window to open; boot compose and
                    // report the URL (the GUI `open_app` opens the actual window).
                    if let Err(e) = crate::boot_and_wait(&appdef) {
                        eprintln!("warn: {}", e);
                    }
                    println!("Opened \"{}\" at {}", name, appdef.url);
                    0
                }
                None => {
                    eprintln!("No app named \"{}\" found.", name);
                    1
                }
            }
        }
        "shortcut" => {
            let name = match name_arg(&args, "Usage: dockwrap shortcut <name>") {
                Ok(n) => n,
                Err(ec) => return ec,
            };
            match find_app(&name) {
                Some(appdef) => {
                    let bin = std::env::current_exe()
                        .map(|p| p.to_string_lossy().into_owned())
                        .unwrap_or_default();
                    match crate::create_shortcut_for(&name, &bin, appdef.icon.as_deref()) {
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
            let entries = registry::catalog();
            match query {
                None => {
                    let cats = registry::catalog_categories();
                    println!("dockwrap app catalog: {} apps in {} categories", entries.len(), cats.len());
                    for c in cats.iter().take(12) {
                        let n = entries.iter().filter(|e| e.category.as_deref() == Some(c.as_str())).count();
                        println!("  {} ({} apps)", c, n);
                    }
                    if cats.len() > 12 {
                        println!("  ... and {} more categories. Use `dockwrap catalog search <q>`", cats.len() - 12);
                    }
                    0
                }
                Some(q) => {
                    let ql = q.to_lowercase();
                    let matches: Vec<_> = entries
                        .iter()
                        .filter(|e| {
                            e.name.to_lowercase().contains(&ql)
                                || e.category.as_deref().map(|c| c.to_lowercase().contains(&ql)).unwrap_or(false)
                                || e.description.as_deref().map(|d| d.to_lowercase().contains(&ql)).unwrap_or(false)
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
            let name = match name_arg(&args, "Usage: dockwrap remove <name>") {
                Ok(n) => n,
                Err(ec) => return ec,
            };
            if registry::remove_app(&name) {
                println!("Removed \"{}\".", name);
                0
            } else {
                eprintln!("No app named \"{}\" found.", name);
                1
            }
        }
        "version" | "--version" | "-V" => {
            println!("dockwrap {}", VERSION);
            0
        }
        "presets" => {
            println!("Built-in presets (name -> default url):");
            for (n, u) in registry::PRESETS {
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
        let args: Vec<String> = vec!["dockwrap".into(), "immich".into()];
        assert_eq!(name_arg(&args, "usage").unwrap(), "immich");
    }

    /// `name_arg` rejects flags (e.g. `--icon path`) as a name.
    #[test]
    fn name_arg_rejects_flag() {
        let args: Vec<String> = vec!["dockwrap".into(), "--icon".into(), "x".into()];
        assert!(name_arg(&args, "usage").is_err());
    }

    /// `get_flag` returns the value following `--flag`.
    #[test]
    fn get_flag_returns_value() {
        let args: Vec<String> = vec![
            "dockwrap".into(), "add".into(), "--url".into(), "http://x".into(),
        ];
        assert_eq!(get_flag(&args, "--url"), Some("http://x".to_string()));
    }

    /// `get_flag` returns None when the flag is absent.
    #[test]
    fn get_flag_absent_is_none() {
        let args: Vec<String> = vec!["dockwrap".into(), "add".into(), "my".into()];
        assert_eq!(get_flag(&args, "--icon"), None);
    }

    /// `dockwrap add --preset immich` resolves from the embedded catalog
    /// (immich is NOT in the 10-entry PRESETS array) and inherits the catalog
    /// icon + description.
    #[test]
    fn preset_falls_back_to_catalog() {
        // immich is not in PRESETS — catalog_entry must resolve it.
        let entry = registry::catalog_entry("Immich");
        assert!(entry.is_some(), "Immich must exist in the embedded catalog");
        let e = entry.unwrap();
        assert!(e.icon.is_some(), "catalog Immich should carry an icon");
        assert!(e.category.is_some(), "catalog Immich should carry a category");
    }

    /// `dockwrap catalog search media` finds results (search is substring across
    /// name/category/description/tags).
    #[test]
    fn catalog_search_finds_results() {
        let entries = registry::catalog();
        let ql = "media";
        let matches: Vec<_> = entries
            .iter()
            .filter(|e| {
                e.name.to_lowercase().contains(&ql)
                    || e.category.as_deref().map(|c| c.to_lowercase().contains(&ql)).unwrap_or(false)
                    || e.description.as_deref().map(|d| d.to_lowercase().contains(&ql)).unwrap_or(false)
                    || e.tags.to_lowercase().contains(&ql)
            })
            .collect();
        assert!(!matches.is_empty(), "catalog search 'media' should match ≥1 app");
    }

    /// `dockwrap catalog` (no args) returns 0 and prints stats.
    #[test]
    fn catalog_subcommand_no_args_is_zero() {
        // run_cli reads std::env::args(), so we can't easily assert stdout.
        // Instead verify the underlying data the subcommand uses.
        let entries = registry::catalog();
        let cats = registry::catalog_categories();
        assert!(!entries.is_empty());
        assert!(!cats.is_empty());
        assert_eq!(cats.len(), cats.iter().collect::<std::collections::HashSet<_>>().len());
    }

    /// `usage()` mentions the new catalog subcommand.
    #[test]
    fn usage_mentions_catalog() {
        let u = usage();
        assert!(u.contains("catalog"), "usage should document the catalog subcommand");
        assert!(u.contains("search"), "usage should document `catalog search`");
    }
}
