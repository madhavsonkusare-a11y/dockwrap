//! Data types shared by build validation and runtime discovery.
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, path::Path};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub source: String,
    pub upstream_id: String,
    pub revision: String,
    pub url: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub aliases: Vec<String>,
    pub source_url: String,
    pub website_url: String,
    pub description: String,
    pub category: String,
    pub tags: Vec<String>,
    pub licenses: Vec<String>,
    pub platforms: Vec<String>,
    pub architectures: Vec<String>,
    pub updated_at: Option<String>,
    pub archived: bool,
    pub warning: bool,
    pub maintenance: String,
    pub web_ui: bool,
    pub icon: Option<String>,
    pub sources: Vec<Source>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Catalog {
    pub schema_version: u32,
    pub snapshot_date: String,
    pub entries: Vec<Project>,
}

impl Catalog {
    #[cfg_attr(not(test), allow(dead_code))] // Called by build.rs; source files are not required at runtime.
    pub fn validate(&self, assets: &Path) -> Result<(), String> {
        if self.schema_version != 1 || self.snapshot_date.len() != 10 || self.entries.is_empty() {
            return Err("Unsupported or empty catalog snapshot".into());
        }
        let mut ids = HashSet::new();
        for entry in &self.entries {
            if entry.id.is_empty()
                || entry.id.len() > 64
                || !entry
                    .id
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
                || !ids.insert(&entry.id)
            {
                return Err(format!("Invalid or duplicate catalog ID: {}", entry.id));
            }
            if entry.name.trim().is_empty()
                || entry.description.trim().is_empty()
                || entry.sources.is_empty()
            {
                return Err(format!("Missing project metadata: {}", entry.id));
            }
            if entry.sources.iter().any(|source| {
                source.upstream_id.is_empty()
                    || source.revision.len() != 40
                    || !source.url.starts_with("https://")
            }) {
                return Err(format!("Missing source provenance: {}", entry.id));
            }
            if let Some(icon) = &entry.icon {
                let stem = icon
                    .strip_prefix("assets/catalog/")
                    .and_then(|name| name.strip_suffix(".svg"));
                if !stem.is_some_and(|name| {
                    !name.is_empty()
                        && name
                            .bytes()
                            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
                }) || !assets.join(icon).is_file()
                {
                    return Err(format!("Invalid or missing local icon: {icon}"));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot() -> Catalog {
        serde_json::from_str(include_str!("generated/catalog.json")).unwrap()
    }

    #[test]
    fn rejects_duplicate_ids_and_missing_provenance() {
        let mut catalog = snapshot();
        catalog.entries[1].id = catalog.entries[0].id.clone();
        assert!(catalog
            .validate(Path::new("src"))
            .unwrap_err()
            .contains("duplicate"));
        let mut catalog = snapshot();
        catalog.entries[0].sources.clear();
        assert!(catalog
            .validate(Path::new("src"))
            .unwrap_err()
            .contains("metadata"));
    }

    #[test]
    fn rejects_remote_traversal_and_missing_artwork() {
        for path in [
            "https://example.com/icon.svg",
            "assets/catalog/../mark.svg",
            "assets/catalog/absent-test-icon.svg",
        ] {
            let mut catalog = snapshot();
            catalog.entries[0].icon = Some(path.into());
            assert!(catalog.validate(Path::new("src")).is_err());
        }
    }
}
