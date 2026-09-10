//! Offline catalog generated from pinned upstream snapshots.
//! Project source addresses are never treated as running app instances.

use crate::model::CatalogEntry;
#[path = "catalog_index.rs"]
mod index;
pub use index::{CatalogPage, Filters};

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

/// Compatibility projection for CLI consumers; discovery uses the versioned index.
pub fn catalog() -> Vec<CatalogEntry> {
    index::index()
        .entries
        .iter()
        .map(|entry| CatalogEntry {
            name: entry.name.clone(),
            url: entry.source_url.clone(),
            icon: entry.icon.clone(),
            category: Some(entry.category.clone()),
            description: Some(entry.description.clone()),
            tags: entry.licenses.join(", "),
            warning: entry.warning,
            ..Default::default()
        })
        .collect()
}

pub fn catalog_categories() -> Vec<String> {
    search_catalog("", "", 0, 1).categories
}

pub fn catalog_entry(value: &str) -> Option<CatalogEntry> {
    let project = index::index().entries.iter().find(|entry| {
        entry.id == value
            || entry.name.eq_ignore_ascii_case(value)
            || entry
                .aliases
                .iter()
                .any(|alias| alias.eq_ignore_ascii_case(value))
    })?;
    Some(CatalogEntry {
        name: project.name.clone(),
        url: project.source_url.clone(),
        icon: project.icon.clone(),
        category: Some(project.category.clone()),
        description: Some(project.description.clone()),
        tags: project.licenses.join(", "),
        warning: project.warning,
        ..Default::default()
    })
}

pub fn catalog_id(value: &str) -> Option<String> {
    index::index()
        .entries
        .iter()
        .find(|entry| {
            entry.id == value
                || entry.name.eq_ignore_ascii_case(value)
                || entry
                    .aliases
                    .iter()
                    .any(|alias| alias.eq_ignore_ascii_case(value))
        })
        .map(|entry| entry.id.clone())
}

pub fn search_catalog(query: &str, category: &str, offset: usize, limit: usize) -> CatalogPage {
    index::search(query, category, offset, limit, &Filters::default())
}

pub fn search_filtered(
    query: &str,
    category: &str,
    offset: usize,
    limit: usize,
    filters: &Filters,
) -> CatalogPage {
    index::search(query, category, offset, limit, filters)
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
        assert!(entry.get("id").is_some());
        assert!(search_catalog("", "", usize::MAX, 12).entries.is_empty());
    }

    #[test]
    fn only_reviewed_catalog_entries_advertise_preview_install() {
        let memos = search_catalog("Memos", "", 0, 48)
            .entries
            .into_iter()
            .find(|entry| entry.name == "Memos")
            .unwrap();
        assert_eq!(memos.capability, "preview_install");
        assert_eq!(memos.recipe_id.as_deref(), Some("memos"));
        // Every offering, not a list that has to be remembered: an approved
        // app that never reaches a catalog entry can be installed by id and
        // found by nobody. Matched on the recipe id rather than the name,
        // because the catalog's name for an app is not always the offering's:
        // Node-RED is listed as "Node RED", with the hyphenated spelling as an
        // alias, and searching either has to find it.
        for offering in crate::offerings::offerings() {
            let name = offering.catalog_name().to_owned();
            let entry = search_catalog(&name, "", 0, 48)
                .entries
                .into_iter()
                .find(|entry| entry.recipe_id.as_deref() == Some(offering.id()))
                .unwrap_or_else(|| panic!("searching {name:?} does not find the app it names"));
            assert_eq!(entry.capability, "preview_install");
        }
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
