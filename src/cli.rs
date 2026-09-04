//! Standalone command-line interface for Local Store.
use local_store::{
    brand::CLI_NAME,
    catalog,
    model::{next_available_installed_app_id, InstalledApp, RuntimeSpec},
    platform, recipes, runtime, storage, windowing,
};
use std::time::{SystemTime, UNIX_EPOCH};
const VERSION: &str = env!("CARGO_PKG_VERSION");

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

fn usage() -> String {
    format!("Usage:\n  {CLI_NAME} add <name> --url <url>\n  {CLI_NAME} list\n  {CLI_NAME} open <id-or-name> --browser\n  {CLI_NAME} shortcut <id-or-name>\n  {CLI_NAME} remove <id-or-name>\n  {CLI_NAME} doctor\n  {CLI_NAME} install memos\n  {CLI_NAME} start|stop|status|logs <id-or-name>\n  {CLI_NAME} uninstall <id-or-name> [--delete-data]\n  {CLI_NAME} catalog [search]\n  {CLI_NAME} version")
}
fn get_flag(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|arg| arg == flag)
        .and_then(|index| args.get(index + 1).cloned())
}
fn positional(args: &[String], index: usize, line: &str) -> Result<String, i32> {
    args.get(index)
        .filter(|value| !value.starts_with("--"))
        .cloned()
        .ok_or_else(|| {
            eprintln!("{line}");
            1
        })
}
fn registry() -> Result<Vec<InstalledApp>, String> {
    storage::load_or_migrate_registry()
        .map(|registry| registry.apps)
        .map_err(|error| error.to_string())
}
fn find_app(value: &str) -> Result<InstalledApp, String> {
    registry()?
        .into_iter()
        .find(|app| app.id == value || app.display_name.eq_ignore_ascii_case(value))
        .ok_or_else(|| format!("No app with ID or name \"{value}\" found."))
}
fn now() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_secs())
        .map_err(|error| error.to_string())
}
fn report(result: Result<(), String>, success: &str) -> i32 {
    match result {
        Ok(()) => {
            println!("{success}");
            0
        }
        Err(error) => {
            eprintln!("{error}");
            1
        }
    }
}
fn catalog_command(query: Option<&String>) -> i32 {
    let entries = catalog::catalog();
    if let Some(query) = query {
        let query = query.to_lowercase();
        for entry in entries
            .iter()
            .filter(|entry| {
                format!(
                    "{} {} {} {}",
                    entry.name,
                    entry.category.as_deref().unwrap_or(""),
                    entry.description.as_deref().unwrap_or(""),
                    entry.tags
                )
                .to_lowercase()
                .contains(&query)
            })
            .take(30)
        {
            println!(
                "  {:<28} {:<22} {}",
                entry.name,
                entry.category.as_deref().unwrap_or("app"),
                entry.url
            );
        }
    } else {
        println!(
            "{CLI_NAME} catalog: {} projects; reviewed installs: Memos",
            entries.len()
        );
    }
    0
}

pub fn run_cli() -> i32 {
    ensure_console();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(command) = args.first().map(String::as_str) else {
        eprintln!("{}", usage());
        return 1;
    };
    match command {
        "add" => {
            let name = match positional(
                &args,
                1,
                &format!("Usage: {CLI_NAME} add <name> --url <url>"),
            ) {
                Ok(value) => value,
                Err(code) => return code,
            };
            let Some(url) = get_flag(&args, "--url") else {
                eprintln!("Usage: {CLI_NAME} add <name> --url <url>");
                return 1;
            };
            let url = match windowing::validated_external_url(&url) {
                Ok(value) => value,
                Err(error) => {
                    eprintln!("{error}");
                    return 1;
                }
            };
            let apps = match registry() {
                Ok(value) => value,
                Err(error) => {
                    eprintln!("{error}");
                    return 1;
                }
            };
            let base = storage::slug_for_display_name(&name);
            let Some(id) = next_available_installed_app_id(&base, |candidate| {
                !apps.iter().any(|app| app.id == candidate)
            }) else {
                eprintln!("Could not create a safe app ID.");
                return 1;
            };
            let timestamp = match now() {
                Ok(value) => value,
                Err(error) => {
                    eprintln!("{error}");
                    return 1;
                }
            };
            let app = InstalledApp {
                id: id.clone(),
                catalog_id: None,
                display_name: name,
                launch_url: url,
                icon_path: None,
                runtime: RuntimeSpec::External,
                created_at_unix: timestamp,
                updated_at_unix: timestamp,
            };
            report(
                storage::insert_installed_app(app).map_err(|error| error.to_string()),
                &format!("Connected app as \"{id}\"."),
            )
        }
        "list" => match registry() {
            Ok(apps) => {
                println!("{}", serde_json::to_string_pretty(&apps).unwrap());
                0
            }
            Err(error) => {
                eprintln!("{error}");
                1
            }
        },
        "doctor" => {
            let result = runtime::doctor();
            for check in result.checks {
                println!(
                    "{} {:<18} {}",
                    if check.ok { "ok" } else { "fail" },
                    check.label,
                    check.detail
                );
            }
            if result.ready {
                0
            } else {
                1
            }
        }
        "install" => {
            let id = match positional(&args, 1, &format!("Usage: {CLI_NAME} install memos")) {
                Ok(value) => value,
                Err(code) => return code,
            };
            let Some(recipe) = recipes::recipe(&id) else {
                eprintln!("No reviewed install recipe named \"{id}\".");
                return 1;
            };
            match runtime::install_recipe(&recipe) {
                Ok(app) => report(
                    storage::insert_installed_app(app).map_err(|error| error.to_string()),
                    "Installed and started.",
                ),
                Err(error) => {
                    eprintln!("{error}");
                    1
                }
            }
        }
        "open" => {
            let value = match positional(
                &args,
                1,
                &format!("Usage: {CLI_NAME} open <id-or-name> --browser"),
            ) {
                Ok(value) => value,
                Err(code) => return code,
            };
            if !args.iter().any(|arg| arg == "--browser") {
                eprintln!("The CLI opens app pages only with --browser. Use the Local Store launcher for a desktop app window.");
                return 1;
            }
            match find_app(&value) {
                Ok(app) => {
                    if app.is_managed() {
                        if let Err(error) = runtime::start(&app) {
                            eprintln!("{error}");
                            return 1;
                        }
                    }
                    windowing::launch_browser(&app.launch_url);
                    println!("Opened {}.", app.display_name);
                    0
                }
                Err(error) => {
                    eprintln!("{error}");
                    1
                }
            }
        }
        "shortcut" => {
            let value = match positional(
                &args,
                1,
                &format!("Usage: {CLI_NAME} shortcut <id-or-name>"),
            ) {
                Ok(value) => value,
                Err(code) => return code,
            };
            match find_app(&value) {
                Ok(app) => match std::env::current_exe()
                    .map_err(|error| error.to_string())
                    .and_then(|bin| {
                        platform::create_shortcut_for(
                            &app.id,
                            &bin.to_string_lossy(),
                            app.icon_path.as_ref().and_then(|path| path.to_str()),
                        )
                    }) {
                    Ok(path) => {
                        println!("Shortcut created: {path}");
                        0
                    }
                    Err(error) => {
                        eprintln!("{error}");
                        1
                    }
                },
                Err(error) => {
                    eprintln!("{error}");
                    1
                }
            }
        }
        "start" | "stop" | "status" | "logs" | "remove" | "uninstall" => {
            let value = match positional(
                &args,
                1,
                &format!("Usage: {CLI_NAME} {command} <id-or-name>"),
            ) {
                Ok(value) => value,
                Err(code) => return code,
            };
            let app = match find_app(&value) {
                Ok(value) => value,
                Err(error) => {
                    eprintln!("{error}");
                    return 1;
                }
            };
            match command {
                "start" => report(runtime::start(&app), "Started."),
                "stop" => report(runtime::stop(&app), "Stopped."),
                "status" => match runtime::status(&app) {
                    Ok(status) => {
                        println!("{status:?}");
                        0
                    }
                    Err(error) => {
                        eprintln!("{error}");
                        1
                    }
                },
                "logs" => match runtime::logs(&app) {
                    Ok(logs) => {
                        print!("{logs}");
                        0
                    }
                    Err(error) => {
                        eprintln!("{error}");
                        1
                    }
                },
                "remove" if app.is_managed() => {
                    eprintln!("Managed apps must be removed with uninstall.");
                    1
                }
                "remove" => report(
                    storage::remove_installed_app(&app.id)
                        .map(|_| ())
                        .map_err(|error| error.to_string()),
                    "Connection removed.",
                ),
                "uninstall" if !app.is_managed() => {
                    eprintln!("Connected apps must be removed with remove.");
                    1
                }
                "uninstall" => {
                    let delete_data = args.iter().any(|arg| arg == "--delete-data");
                    if let Err(error) = runtime::uninstall(&app, delete_data) {
                        eprintln!("{error}");
                        return 1;
                    }
                    report(
                        storage::remove_installed_app(&app.id)
                            .map(|_| ())
                            .map_err(|error| error.to_string()),
                        if delete_data {
                            "App and data removed."
                        } else {
                            "App removed; data preserved."
                        },
                    )
                }
                _ => unreachable!(),
            }
        }
        "catalog" => catalog_command(args.get(1)),
        "version" | "--version" | "-V" => {
            println!("{CLI_NAME} {VERSION}");
            0
        }
        _ => {
            eprintln!("Unknown command \"{command}\".\n{}", usage());
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn get_flag_returns_value() {
        assert_eq!(
            get_flag(
                &["add".into(), "x".into(), "--url".into(), "http://x".into()],
                "--url"
            ),
            Some("http://x".into())
        );
    }
    #[test]
    fn get_flag_absent_is_none() {
        assert_eq!(get_flag(&["add".into()], "--url"), None);
    }
    #[test]
    fn positional_rejects_flag() {
        assert!(positional(&["open".into(), "--browser".into()], 1, "usage").is_err());
    }
    #[test]
    fn usage_mentions_managed_commands() {
        let value = usage();
        for command in [
            "doctor",
            "install",
            "start",
            "stop",
            "logs",
            "uninstall",
            "catalog",
        ] {
            assert!(value.contains(command));
        }
    }
    #[test]
    fn catalog_has_reviewed_memos() {
        assert_eq!(
            catalog::search_catalog("Memos", "", 0, 10).entries[0]
                .recipe_id
                .as_deref(),
            Some("memos")
        );
    }
}
