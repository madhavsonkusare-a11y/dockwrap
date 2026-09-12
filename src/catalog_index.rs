//! Versioned discovery metadata, generated offline from pinned upstream sources.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::OnceLock;

use crate::catalog_schema::Catalog;
pub use crate::catalog_schema::Project;

pub struct CatalogIndex {
    catalog: Catalog,
    search_text: Vec<String>,
}

impl std::ops::Deref for CatalogIndex {
    type Target = Catalog;
    fn deref(&self) -> &Self::Target {
        &self.catalog
    }
}

pub fn index() -> &'static CatalogIndex {
    static INDEX: OnceLock<CatalogIndex> = OnceLock::new();
    INDEX.get_or_init(|| {
        let catalog: Catalog = serde_json::from_str(include_str!("generated/catalog.json"))
            .expect("validated generated catalog");
        assert_eq!(catalog.schema_version, 1);
        let mut value = CatalogIndex {
            catalog,
            search_text: Vec::new(),
        };
        value.search_text = value
            .entries
            .iter()
            .map(|entry| {
                format!(
                    "{} {} {} {} {}",
                    entry.id,
                    entry.name,
                    entry.description,
                    entry.aliases.join(" "),
                    entry.tags.join(" ")
                )
                .to_lowercase()
            })
            .collect();
        value
    })
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Filters {
    pub capability: String,
    pub license: String,
    pub architecture: String,
    pub collection: String,
    pub hide_warnings: bool,
}

#[derive(Serialize)]
pub struct DiscoveryEntry {
    #[serde(flatten)]
    pub project: Project,
    pub license: String,
    pub capability: &'static str,
    pub recipe_id: Option<String>,
}

impl std::ops::Deref for DiscoveryEntry {
    type Target = Project;
    fn deref(&self) -> &Self::Target {
        &self.project
    }
}

#[derive(Serialize)]
pub struct Facet {
    pub value: String,
    pub count: usize,
}

#[derive(Serialize)]
pub struct CatalogPage {
    pub entries: Vec<DiscoveryEntry>,
    pub total: usize,
    pub catalog_total: usize,
    pub offset: usize,
    pub limit: usize,
    pub categories: Vec<String>,
    pub category_counts: Vec<Facet>,
    pub licenses: Vec<String>,
    pub architectures: Vec<String>,
    pub snapshot_date: String,
    pub source_count: usize,
}

/// Catalog entry id to offering id, read once.
///
/// Driven by our own allowlist rather than by upstream metadata, which is the
/// property the previous hardcoded match existed to guarantee: a catalog entry
/// cannot promote itself by claiming an id, because being listed upstream is
/// not what makes something installable — a review is. A withheld template is
/// absent from `offerings`, so it cannot appear here either.
///
/// The two ids are not always the same string. The catalog knows Node-RED as
/// `node-red`; the Runtipi definition describing it is `nodered`. The
/// catalog's own name and alias table is what reconciles them, so this goes
/// through `catalog::catalog_id` rather than comparing ids directly.
fn installable_ids() -> &'static std::collections::BTreeMap<String, String> {
    static IDS: std::sync::OnceLock<std::collections::BTreeMap<String, String>> =
        std::sync::OnceLock::new();
    IDS.get_or_init(|| {
        crate::offerings::offerings()
            .iter()
            .filter_map(|offering| {
                crate::catalog::catalog_id(offering.catalog_name())
                    .map(|catalog| (catalog, offering.id().to_owned()))
            })
            .collect()
    })
}

fn recipe_id(entry: &Project) -> Option<&str> {
    installable_ids()
        .get(&entry.id)
        .map(std::string::String::as_str)
}

fn capability(entry: &Project) -> &'static str {
    if recipe_id(entry).is_some() {
        "preview_install"
    } else if entry.web_ui {
        "connect"
    } else {
        "discover"
    }
}

fn in_collection(entry: &Project, collection: &str) -> bool {
    let tags = format!("{} {}", entry.category, entry.tags.join(" ")).to_lowercase();
    let terms: &[&str] = match collection {
        "" => return true,
        "writing" => &["note-taking", "wiki", "document", "office"],
        "automation" => &["automation", "workflow", "integration"],
        "media" => &["photo", "media", "video", "audio", "ebook"],
        "developer" => &["development", "developer", "source code", "ide", "api"],
        _ => return false,
    };
    terms.iter().any(|term| tags.contains(term))
}

fn distinct(values: impl Iterator<Item = String>) -> Vec<String> {
    let mut values: Vec<_> = values.collect();
    values.sort();
    values.dedup();
    values
}

pub fn search(
    query: &str,
    category: &str,
    offset: usize,
    limit: usize,
    filters: &Filters,
) -> CatalogPage {
    let index = index();
    let query = query.trim().to_lowercase();
    let terms: Vec<_> = query.split_whitespace().collect();
    let limit = limit.clamp(1, 48);
    let mut counts = BTreeMap::<String, usize>::new();
    let entries: Vec<_> = index
        .entries
        .iter()
        .zip(&index.search_text)
        .filter_map(|(entry, text)| {
            let matches = terms.iter().all(|term| text.contains(term))
                && (filters.capability.is_empty()
                    || capability(entry) == filters.capability
                    || (filters.capability == "connect" && entry.web_ui))
                && (filters.license.is_empty() || entry.licenses.contains(&filters.license))
                && (filters.architecture.is_empty()
                    || entry.architectures.contains(&filters.architecture))
                && (!filters.hide_warnings || !entry.warning)
                && in_collection(entry, &filters.collection);
            if !matches {
                return None;
            }
            *counts.entry(entry.category.clone()).or_default() += 1;
            (category.is_empty() || entry.category == category).then_some(entry)
        })
        .collect();
    let total = entries.len();
    // Matching is not finding. Every entry whose text contains the words is a
    // match, in browse order, so searching "Sim" returned forty-eight apps
    // that merely mention it and not the app named Sim. An exact name comes
    // first, then a name that starts with what was typed; everything else
    // keeps the order it had.
    let mut entries = entries;
    if !query.is_empty() {
        entries.sort_by_key(|entry| {
            let name = entry.name.to_lowercase();
            let named = |value: &String| value.to_lowercase() == query;
            if name == query || entry.aliases.iter().any(named) {
                0
            } else if name.starts_with(&query) {
                1
            } else {
                2
            }
        });
    }
    let entries = entries
        .into_iter()
        .skip(offset)
        .take(limit)
        .map(|entry| DiscoveryEntry {
            project: entry.clone(),
            license: entry.licenses.join(", "),
            capability: capability(entry),
            recipe_id: recipe_id(entry).map(str::to_string),
        })
        .collect();
    CatalogPage {
        entries,
        total,
        catalog_total: index.entries.len(),
        offset,
        limit,
        categories: distinct(index.entries.iter().map(|entry| entry.category.clone())),
        category_counts: counts
            .into_iter()
            .map(|(value, count)| Facet { value, count })
            .collect(),
        licenses: distinct(
            index
                .entries
                .iter()
                .flat_map(|entry| entry.licenses.clone()),
        ),
        architectures: distinct(
            index
                .entries
                .iter()
                .flat_map(|entry| entry.architectures.clone()),
        ),
        snapshot_date: index.snapshot_date.clone(),
        source_count: distinct(
            index
                .entries
                .iter()
                .flat_map(|entry| entry.sources.iter().map(|source| source.source.clone()))
                .filter(|name| name != "legacy"),
        )
        .len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn catalog_ids_provenance_and_assets_are_valid() {
        let mut ids = HashSet::new();
        for entry in &index().entries {
            assert!(crate::model::is_valid_installed_app_id(&entry.id));
            assert!(ids.insert(&entry.id));
            assert!(!entry.sources.is_empty());
            assert!(crate::windowing::validated_external_url(&entry.source_url).is_ok());
            if let Some(path) = &entry.icon {
                assert!(path.starts_with("assets/catalog/") && !path.contains(".."));
                assert!(std::path::Path::new("src").join(path).is_file());
            }
        }
    }

    #[test]
    fn filters_compose_and_facets_respect_search() {
        let filters = Filters {
            capability: "preview_install".into(),
            ..Default::default()
        };
        let all = search("", "", 0, 48, &filters);
        // Derived from the allowlist rather than written down, so approving an
        // app updates this instead of breaking it. Every offering has to reach
        // the catalog: one that does not is installable but undiscoverable.
        let offered = crate::offerings::offerings().len();
        assert_eq!(all.total, offered);
        assert_eq!(
            all.category_counts
                .iter()
                .map(|facet| facet.count)
                .sum::<usize>(),
            offered
        );
        assert_eq!(search("Memos", "", 0, 12, &filters).total, 1);
        assert_eq!(search("Memos", "Analytics", 0, 12, &filters).total, 0);
        let none = Filters {
            architecture: "not-an-architecture".into(),
            ..Default::default()
        };
        assert_eq!(search("", "", 0, 12, &none).total, 0);
    }

    #[test]
    fn full_catalog_paging_has_no_holes_or_repeated_projects() {
        let mut ids = HashSet::new();
        for offset in (0..index().entries.len()).step_by(48) {
            for entry in search("", "", offset, 48, &Filters::default()).entries {
                assert!(ids.insert(entry.project.id));
            }
        }
        assert_eq!(ids.len(), index().entries.len());
    }
}
