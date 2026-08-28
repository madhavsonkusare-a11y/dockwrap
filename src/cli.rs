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
    "Usage:\n  \
     dockwrap add <name> --url <url> [--icon <path>] [--compose <path>] [--health <url>] [--preset <name>]\n  \
     dockwrap add --preset <name>\n  \
     dockwrap list\n  \
     dockwrap remove <name>\n  \
     dockwrap open <name>\n  \
     dockwrap shortcut <name>\n  \
     dockwrap presets"
        .to_string()
}

fn get_flag(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1).cloned())
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
                (Some(n), None, Some(p)) => match registry::preset_url(&p) {
                    Some(u) => (n, u.to_string()),
                    None => {
                        eprintln!("Unknown preset \"{}\". Run `dockwrap presets`.", p);
                        return 1;
                    }
                },
                (None, None, Some(p)) => match registry::preset_url(&p) {
                    Some(u) => (p.to_string(), u.to_string()),
                    None => {
                        eprintln!("Unknown preset \"{}\". Run `dockwrap presets`.", p);
                        return 1;
                    }
                },
                _ => {
                    eprintln!("{}", usage());
                    return 1;
                }
            };

            registry::upsert_app(&name, &url, icon.clone(), compose.clone(), health.clone());
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
