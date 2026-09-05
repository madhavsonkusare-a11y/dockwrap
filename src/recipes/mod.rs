use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Recipe {
    pub schema_version: u32,
    pub id: String,
    pub display_name: String,
    pub catalog_name: String,
    pub description: String,
    pub category: String,
    pub license: String,
    pub version: String,
    pub image: String,
    pub source_url: String,
    pub documentation_url: String,
    pub verified_at: String,
    pub launch_url: String,
    pub health_url: String,
    pub port: u16,
    pub data_directories: Vec<String>,
    pub data_storage: String,
    pub risk_notes: Vec<String>,
    pub compose: String,
}

const MEMOS: &str = include_str!("memos.json");
const N8N: &str = include_str!("n8n.json");
const UPTIME_KUMA: &str = include_str!("uptime-kuma.json");

pub fn verified_recipes() -> Vec<Recipe> {
    [MEMOS, N8N, UPTIME_KUMA]
        .into_iter()
        .map(|source| {
            let recipe: Recipe =
                serde_json::from_str(source).expect("reviewed recipe JSON must be valid");
            recipe.validate().expect("reviewed recipe must pass policy");
            recipe
        })
        .collect()
}

pub fn recipe(id: &str) -> Option<Recipe> {
    verified_recipes()
        .into_iter()
        .find(|recipe| recipe.id == id)
}

pub fn recipe_for_catalog_name(name: &str) -> Option<Recipe> {
    verified_recipes()
        .into_iter()
        .find(|recipe| recipe.catalog_name.eq_ignore_ascii_case(name))
}

impl Recipe {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 {
            return Err("unsupported recipe schema".into());
        }
        if !valid_audit_date(&self.verified_at)
            || !self.source_url.starts_with("https://")
            || !self.documentation_url.starts_with("https://")
        {
            return Err("recipe audit metadata is missing".into());
        }
        if !crate::model::is_valid_installed_app_id(&self.id)
            || self.display_name.trim().is_empty()
            || self.catalog_name.trim().is_empty()
            || self.description.trim().is_empty()
            || self.category.trim().is_empty()
            || self.license.trim().is_empty()
        {
            return Err("invalid recipe identity".into());
        }
        let expected_suffix = format!(":{}", self.version);
        if self.version.trim().is_empty()
            || !self.image.ends_with(&expected_suffix)
            || self.image.ends_with(":latest")
            || self.image.ends_with(":stable")
            || self.image.ends_with(":main")
            || !self.compose.contains(&format!("image: {}", self.image))
            || !self
                .compose
                .contains(&format!("\"127.0.0.1:{}:{}\"", self.port, self.port))
        {
            return Err("recipe image and port must be pinned exactly".into());
        }
        if self.data_storage.trim().is_empty() {
            return Err("recipe data storage description is missing".into());
        }
        for directory in &self.data_directories {
            let path = Path::new(directory);
            if path.is_absolute()
                || directory.is_empty()
                || path.components().any(|component| {
                    matches!(
                        component,
                        std::path::Component::ParentDir | std::path::Component::RootDir
                    )
                })
            {
                return Err("recipe data directory escapes its managed project".into());
            }
        }
        for forbidden in [
            "privileged:",
            "/var/run/docker.sock",
            "network_mode: host",
            "pid: host",
            "ipc: host",
            "cap_add:",
            "devices:",
            "- /:/",
        ] {
            if self.compose.contains(forbidden) {
                return Err(format!("recipe contains forbidden setting: {forbidden}"));
            }
        }
        Ok(())
    }
}

fn valid_audit_date(value: &str) -> bool {
    if value.len() != 10 || !value.is_ascii() {
        return false;
    }
    let parts: Vec<_> = value.split('-').collect();
    if parts.len() != 3 || parts[0].len() != 4 || parts[1].len() != 2 || parts[2].len() != 2 {
        return false;
    }
    let (Ok(year), Ok(month), Ok(day)) = (
        parts[0].parse::<u32>(),
        parts[1].parse::<u32>(),
        parts[2].parse::<u32>(),
    ) else {
        return false;
    };
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days = match month {
        2 => {
            if leap {
                29
            } else {
                28
            }
        }
        4 | 6 | 9 | 11 => 30,
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        _ => return false,
    };
    year >= 2020 && day > 0 && day <= days
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reviewed_recipes_are_pinned_persistent_and_unprivileged() {
        for recipe in verified_recipes() {
            recipe.validate().unwrap();
            assert!(recipe.compose.contains("./data:") || recipe.compose.contains("n8n-data:"));
        }
    }

    #[test]
    fn only_explicitly_reviewed_recipes_are_installable() {
        assert_eq!(verified_recipes().len(), 3);
        assert!(recipe("memos").is_some());
        assert!(recipe("n8n").is_some());
        assert!(recipe("uptime-kuma").is_some());
        assert!(recipe("immich").is_none());
    }

    #[test]
    fn unsafe_recipe_changes_fail_closed() {
        let mut candidate = recipe("memos").unwrap();
        candidate.compose.push_str("    privileged: true\n");
        assert!(candidate.validate().is_err());
        candidate = recipe("memos").unwrap();
        candidate.image = "neosmemo/memos:latest".into();
        assert!(candidate.validate().is_err());
        candidate = recipe("memos").unwrap();
        candidate.data_directories = vec!["../escape".into()];
        assert!(candidate.validate().is_err());
        candidate = recipe("memos").unwrap();
        candidate.compose = candidate.compose.replace("127.0.0.1:", "");
        assert!(candidate.validate().is_err());
    }

    #[test]
    fn audit_date_accepts_future_reviews_and_rejects_invalid_dates() {
        assert!(valid_audit_date("2026-10-01"));
        assert!(valid_audit_date("2028-02-29"));
        for invalid in ["", "2026-02-29", "2026-09-31", "2026-13-01", "2026-00-01"] {
            assert!(!valid_audit_date(invalid));
        }
    }
}
