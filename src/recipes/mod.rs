use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Recipe {
    pub schema_version: u32,
    pub id: String,
    pub display_name: String,
    pub catalog_name: String,
    pub version: String,
    pub image: String,
    pub source_url: String,
    pub documentation_url: String,
    pub launch_url: String,
    pub health_url: String,
    pub port: u16,
    pub data_directories: Vec<String>,
    pub risk_notes: Vec<String>,
    pub compose: String,
}

const MEMOS: &str = include_str!("memos.json");

pub fn verified_recipes() -> Vec<Recipe> {
    vec![serde_json::from_str(MEMOS).expect("reviewed Memos recipe must be valid")]
}

pub fn recipe(id: &str) -> Option<Recipe> {
    verified_recipes()
        .into_iter()
        .find(|recipe| recipe.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reviewed_recipe_is_pinned_persistent_and_unprivileged() {
        let recipe = recipe("memos").unwrap();
        assert_eq!(recipe.schema_version, 1);
        assert_eq!(recipe.version, "0.30.0");
        assert!(recipe.image.ends_with(":0.30.0"));
        assert!(!recipe.image.contains(":latest"));
        assert!(!recipe.image.contains(":stable"));
        assert!(recipe.compose.contains("./data:/var/opt/memos"));
        assert!(!recipe.compose.contains("/var/run/docker.sock"));
        assert!(!recipe.compose.contains("/:"));
        assert!(!recipe.compose.contains("privileged:"));
        assert!(recipe.compose.contains("5230:5230"));
    }

    #[test]
    fn only_explicitly_reviewed_recipes_are_installable() {
        assert_eq!(verified_recipes().len(), 1);
        assert!(recipe("memos").is_some());
        assert!(recipe("n8n").is_none());
    }
}
