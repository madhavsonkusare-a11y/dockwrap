//! Curated self-hosted app catalog, baked into the binary at compile time.
//! Ships as `src/catalog_full.json` (also lives in the webview bundle via
//! `frontendDist: src`). Lets the GUI wizard offer a "pick a popular app"
//! catalog instead of typing URLs, and backs `local-store add --preset`.

use crate::model::CatalogEntry;
use serde::Serialize;
use std::sync::OnceLock;

/// Curated, most-used self-hosted web apps (default localhost ports).
/// Lets a user run `local-store add --preset n8n` instead of typing the URL.
pub const PRESETS: &[(&str, &str)] = &[
    ("n8n", "http://localhost:5678"),
    ("open-webui", "http://localhost:3000"),
    ("immich", "http://localhost:2283"),
    ("stirling-pdf", "http://localhost:8080"),
    ("uptime-kuma", "http://localhost:3001"),
    ("memos", "http://localhost:5230"),
    ("coolify", "http://localhost:8000"),
    ("glance", "http://localhost:8080"),
    ("filebrowser", "http://localhost:80"),
    ("changedetection", "http://localhost:5000"),
];

/// The raw catalog, embedded at build time so the binary is self-contained
/// (no runtime file read, no extra network at startup). The full awesome-selfhosted
/// list (1257 apps) ships here; the 12 hand-curated apps (with compose/health snippets)
/// are a separate quick-start subset used by `local-store add --preset`.
const CATALOG_JSON: &str = include_str!("catalog_full.json");

/// The embedded catalog is parsed exactly once, with invalid data failing loudly.
fn cached_catalog() -> &'static [CatalogEntry] {
    static CATALOG: OnceLock<Vec<CatalogEntry>> = OnceLock::new();
    CATALOG.get_or_init(|| serde_json::from_str(CATALOG_JSON).expect("valid embedded catalog"))
}

pub fn catalog() -> Vec<CatalogEntry> {
    cached_catalog().to_vec()
}

pub fn catalog_categories() -> Vec<String> {
    let mut categories: Vec<_> = cached_catalog()
        .iter()
        .filter_map(|e| e.category.clone())
        .collect();
    categories.sort();
    categories.dedup();
    categories
}

pub fn catalog_entry(name: &str) -> Option<CatalogEntry> {
    cached_catalog()
        .iter()
        .find(|e| e.name.eq_ignore_ascii_case(name))
        .cloned()
}

#[derive(Serialize)]
pub struct DiscoveryEntry {
    pub name: String,
    pub source_url: String,
    pub description: String,
    pub category: String,
    pub license: String,
    pub icon: Option<String>,
    pub warning: bool,
    pub capability: &'static str,
    pub recipe_id: Option<String>,
}

#[derive(Serialize)]
pub struct CatalogPage {
    pub entries: Vec<DiscoveryEntry>,
    pub total: usize,
    pub catalog_total: usize,
    pub offset: usize,
    pub limit: usize,
    pub categories: Vec<String>,
}

pub fn search_catalog(query: &str, category: &str, offset: usize, limit: usize) -> CatalogPage {
    let query = query.trim().to_lowercase();
    let limit = limit.clamp(1, 48);
    let mut matches: Vec<_> = cached_catalog()
        .iter()
        .filter(|entry| {
            (category.is_empty() || entry.category.as_deref() == Some(category))
                && (query.is_empty()
                    || format!(
                        "{} {} {}",
                        entry.name,
                        entry.description.as_deref().unwrap_or(""),
                        entry.tags
                    )
                    .to_lowercase()
                    .contains(&query))
        })
        .collect();
    matches.sort_by_key(|e| e.name.to_lowercase());
    let total = matches.len();
    let entries = matches
        .into_iter()
        .skip(offset)
        .take(limit)
        .map(|e| DiscoveryEntry {
            name: e.name.clone(),
            source_url: e.url.clone(),
            description: e.description.clone().unwrap_or_default(),
            category: e.category.clone().unwrap_or_else(|| "Other".into()),
            license: e.tags.replace('`', ""),
            icon: e
                .icon
                .clone()
                .filter(|s| !s.is_empty())
                .or_else(|| e.favicon_url.clone()),
            warning: e.warning,
            capability: if crate::recipes::recipe(&e.name.to_lowercase()).is_some() {
                "verified_install"
            } else {
                "connect"
            },
            recipe_id: crate::recipes::recipe(&e.name.to_lowercase()).map(|recipe| recipe.id),
        })
        .collect();
    CatalogPage {
        entries,
        total,
        catalog_total: cached_catalog().len(),
        offset,
        limit,
        categories: catalog_categories(),
    }
}

/// Resolve a preset name to its default localhost URL, if known.
pub fn preset_url(name: &str) -> Option<&'static str> {
    PRESETS.iter().find(|(n, _)| *n == name).map(|(_, u)| *u)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovery_is_bounded_and_does_not_expose_source_as_launch_url() {
        let page = search_catalog("", "", 0, 5000);
        assert_eq!(page.entries.len(), 48);
        assert_eq!(page.catalog_total, catalog().len());
        let entry = serde_json::to_value(&page.entries[0]).unwrap();
        assert!(entry.get("url").is_none());
        assert!(entry.get("source_url").is_some());
        assert_eq!(entry["capability"], "connect");
        assert!(search_catalog("", "", usize::MAX, 12).entries.is_empty());
    }

    #[test]
    fn only_reviewed_catalog_entry_advertises_verified_install() {
        let memos = search_catalog("Memos", "", 0, 48)
            .entries
            .into_iter()
            .find(|entry| entry.name == "Memos")
            .unwrap();
        assert_eq!(memos.capability, "verified_install");
        assert_eq!(memos.recipe_id.as_deref(), Some("memos"));
        assert!(search_catalog("Immich", "", 0, 48)
            .entries
            .into_iter()
            .find(|entry| entry.name == "Immich")
            .unwrap()
            .recipe_id
            .is_none());
    }

    #[test]
    fn discovery_combines_search_category_and_pagination() {
        let all = search_catalog("", "Analytics", 0, 48);
        assert!(all.total > 1);
        assert!(all.entries.iter().all(|e| e.category == "Analytics"));
        let second = search_catalog("", "Analytics", 1, 1);
        assert_eq!(second.entries[0].name, all.entries[1].name);
        assert_eq!(
            search_catalog("  IMMICH  ", "", 0, 12).entries[0].name,
            "Immich"
        );
        assert_eq!(search_catalog("no-such-app-xyzxyz", "", 0, 12).total, 0);
    }

    #[test]
    fn preset_lookup_works() {
        assert_eq!(preset_url("n8n"), Some("http://localhost:5678"));
        assert_eq!(preset_url("penpot"), None);
    }

    #[test]
    fn catalog_loads_and_looks_up() {
        let apps = catalog();
        assert!(
            !apps.is_empty(),
            "catalog should embed the full self-hosted list"
        );
        let by_name: Vec<&str> = apps.iter().map(|e| e.name.as_str()).collect();
        assert!(by_name.contains(&"Immich"), "Immich should be cataloged");
        assert!(
            by_name.contains(&"Jellyfin"),
            "Jellyfin should be cataloged"
        );
        let entry = catalog_entry("Immich");
        assert!(entry.is_some(), "Immich lookup should return an entry");
        // The 12 curated apps carry compose snippets; the full list may not.
        let _ = entry.unwrap().compose.is_some();
    }

    #[test]
    fn catalog_categories_distinct() {
        let cats = catalog_categories();
        assert!(!cats.is_empty());
        let mut sorted = cats.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), cats.len(), "categories must be distinct");
    }

    /// `--preset immich` should resolve even though PRESETS and the catalog
    /// both use different casing — the CLI passes lowercase.
    #[test]
    fn catalog_entry_case_insensitive() {
        assert!(catalog_entry("immich").is_some());
        assert!(catalog_entry("IMMICH").is_some());
        assert!(catalog_entry("penpot").is_some());
        assert!(catalog_entry("PENPOT").is_some());
    }
}
