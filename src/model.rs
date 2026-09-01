//! Core data models shared by storage, catalog, runtime, and the CLI.

use serde::{Deserialize, Serialize};

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
