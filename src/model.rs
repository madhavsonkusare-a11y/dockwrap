//! Core data models shared by storage, catalog, runtime, and the CLI.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Describes how an installed app is launched and, when applicable, managed locally.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RuntimeSpec {
    External,
    Compose {
        project_name: String,
        project_dir: PathBuf,
        compose_file: PathBuf,
    },
}

/// A versioned record for one installed application.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InstalledApp {
    pub id: String,
    pub catalog_id: Option<String>,
    pub display_name: String,
    pub launch_url: String,
    pub icon_path: Option<PathBuf>,
    pub runtime: RuntimeSpec,
    pub created_at_unix: u64,
    pub updated_at_unix: u64,
}

/// Returns whether `id` is a stable lowercase ASCII slug.
///
/// Valid IDs are non-empty, begin and end with an ASCII alphanumeric character,
/// and contain only lowercase ASCII letters, digits, or single hyphens.
pub fn is_valid_installed_app_id(id: &str) -> bool {
    let bytes = id.as_bytes();
    let Some((&first, rest)) = bytes.split_first() else {
        return false;
    };
    let Some(&last) = rest.last().or(Some(&first)) else {
        return false;
    };

    if !first.is_ascii_alphanumeric() || !last.is_ascii_alphanumeric() {
        return false;
    }

    !bytes.windows(2).any(|pair| pair == b"--")
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'-')
}

/// Returns the first available stable ID using `base`, then `base-2`, `base-3`, and so on.
///
/// The caller supplies availability so the helper remains pure and independent of storage.
pub fn next_available_installed_app_id(
    base: &str,
    mut is_available: impl FnMut(&str) -> bool,
) -> String {
    if is_available(base) {
        return base.to_owned();
    }

    for suffix in 2_u64.. {
        let candidate = format!("{base}-{suffix}");
        if is_available(&candidate) {
            return candidate;
        }
    }

    unreachable!("u64 suffix range is exhaustive")
}

#[derive(Deserialize, Clone, Serialize, Default)]
pub struct AppDef {
    pub name: String,
    pub url: String,
    pub icon: Option<String>,
    /// Optional path to a docker-compose.yml (or a directory containing one).
    /// When set, opening the app runs `docker compose up -d` first.
    #[serde(default)]
    pub compose: Option<String>,
    /// Optional health URL or host:port to poll after compose boot.
    /// Defaults to `url` when omitted.
    #[serde(default)]
    pub health: Option<String>,
}

/// One catalog entry — a fuller AppDef + a human description + category.
#[derive(Deserialize, Clone, Serialize, Default)]
pub struct CatalogEntry {
    pub name: String,
    pub url: String,
    pub icon: Option<String>,
    #[serde(default)]
    pub favicon_url: Option<String>,
    #[serde(default)]
    pub compose: Option<String>,
    #[serde(default)]
    pub health: Option<String>,
    pub category: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub tags: String,
}

#[cfg(test)]
mod tests {
    use std::{collections::HashSet, path::PathBuf};

    use super::*;

    #[test]
    fn installed_app_external_round_trips_to_exact_json() {
        let app = InstalledApp {
            id: "actual-budget".into(),
            catalog_id: Some("actual".into()),
            display_name: "Actual Budget".into(),
            launch_url: "http://127.0.0.1:5006".into(),
            icon_path: Some(PathBuf::from("icons/actual.png")),
            runtime: RuntimeSpec::External,
            created_at_unix: 1_700_000_000,
            updated_at_unix: 1_700_000_001,
        };

        let json = serde_json::to_string(&app).unwrap();
        assert_eq!(
            json,
            r#"{"id":"actual-budget","catalog_id":"actual","display_name":"Actual Budget","launch_url":"http://127.0.0.1:5006","icon_path":"icons/actual.png","runtime":{"kind":"external"},"created_at_unix":1700000000,"updated_at_unix":1700000001}"#
        );
        assert_eq!(serde_json::from_str::<InstalledApp>(&json).unwrap(), app);
    }

    #[test]
    fn installed_app_compose_round_trips_to_exact_json() {
        let app = InstalledApp {
            id: "paperless-ngx".into(),
            catalog_id: None,
            display_name: "Paperless-ngx".into(),
            launch_url: "http://localhost:8000".into(),
            icon_path: None,
            runtime: RuntimeSpec::Compose {
                project_name: "paperless".into(),
                project_dir: PathBuf::from("projects/paperless"),
                compose_file: PathBuf::from("projects/paperless/compose.yaml"),
            },
            created_at_unix: 42,
            updated_at_unix: 99,
        };

        let json = serde_json::to_string(&app).unwrap();
        assert_eq!(
            json,
            r#"{"id":"paperless-ngx","catalog_id":null,"display_name":"Paperless-ngx","launch_url":"http://localhost:8000","icon_path":null,"runtime":{"kind":"compose","project_name":"paperless","project_dir":"projects/paperless","compose_file":"projects/paperless/compose.yaml"},"created_at_unix":42,"updated_at_unix":99}"#
        );
        assert_eq!(serde_json::from_str::<InstalledApp>(&json).unwrap(), app);
    }

    #[test]
    fn installed_app_ids_accept_only_stable_lowercase_ascii_slugs() {
        for id in ["a", "z9", "actual-budget", "a0-b9-c"] {
            assert!(is_valid_installed_app_id(id), "{id} should be valid");
        }

        for id in [
            "",
            "-leading",
            "trailing-",
            "double--hyphen",
            "UPPER",
            "has_space",
            "café",
            "under_score",
        ] {
            assert!(!is_valid_installed_app_id(id), "{id:?} should be invalid");
        }
    }

    #[test]
    fn next_available_installed_app_id_uses_deterministic_numeric_suffixes() {
        let used = HashSet::from([
            "actual".to_owned(),
            "actual-2".to_owned(),
            "actual-3".to_owned(),
        ]);

        assert_eq!(
            next_available_installed_app_id("actual", |candidate| !used.contains(candidate)),
            "actual-4"
        );
        assert_eq!(
            next_available_installed_app_id("vikunja", |candidate| !used.contains(candidate)),
            "vikunja"
        );
    }

    #[test]
    fn installed_app_display_names_preserve_arbitrary_unicode_text() {
        let display_name = "📚 Café — Привет 世界".to_owned();
        let app = InstalledApp {
            id: "library".into(),
            catalog_id: None,
            display_name: display_name.clone(),
            launch_url: "https://example.test".into(),
            icon_path: None,
            runtime: RuntimeSpec::External,
            created_at_unix: 0,
            updated_at_unix: 0,
        };

        assert_eq!(app.display_name, display_name);
        assert_eq!(
            serde_json::from_str::<InstalledApp>(&serde_json::to_string(&app).unwrap()).unwrap(),
            app
        );
    }
}
