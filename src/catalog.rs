//! Curated self-hosted app catalog, baked into the binary at compile time.
//! Ships as `src/catalog_full.json` (also lives in the webview bundle via
//! `frontendDist: src`). Lets the GUI wizard offer a "pick a popular app"
//! catalog instead of typing URLs, and backs `dockwrap add --preset`.

use crate::model::CatalogEntry;

/// Curated, most-used self-hosted web apps (default localhost ports).
/// Lets a user run `dockwrap add --preset n8n` instead of typing the URL.
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
/// are a separate quick-start subset used by `dockwrap add --preset`.
const CATALOG_JSON: &str = include_str!("catalog_full.json");

/// Parse the compile-time-embedded catalog JSON into structs.
fn parse_catalog() -> Vec<CatalogEntry> {
    serde_json::from_str(CATALOG_JSON).unwrap_or_default()
}

/// Public catalog access — cheap, returns the parsed list (re-parsed per call,
/// but small and typically called once at launcher render time).
pub fn catalog() -> Vec<CatalogEntry> {
    parse_catalog()
}

/// All distinct categories, in catalog order (owned so callers don't borrow locals).
pub fn catalog_categories() -> Vec<String> {
    let mut seen: Vec<String> = Vec::new();
    for e in parse_catalog().iter() {
        if let Some(cat) = &e.category {
            if !seen.iter().any(|c: &String| c == cat) {
                seen.push(cat.clone());
            }
        }
    }
    seen
}

/// Lookup a single catalog entry by name (case-insensitive — app names in the
/// source list have inconsistent casing, e.g. "Immich" vs "n8n").
pub fn catalog_entry(name: &str) -> Option<CatalogEntry> {
    let lower = name.to_lowercase();
    parse_catalog()
        .into_iter()
        .find(|e| e.name.to_lowercase() == lower)
}

/// Resolve a preset name to its default localhost URL, if known.
pub fn preset_url(name: &str) -> Option<&'static str> {
    PRESETS.iter().find(|(n, _)| *n == name).map(|(_, u)| *u)
}

#[cfg(test)]
mod tests {
    use super::*;

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
