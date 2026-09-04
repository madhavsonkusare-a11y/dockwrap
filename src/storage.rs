//! Registry storage: where registered apps live and how the apps.json file is
//! loaded and saved. The single source of truth for wrapped apps, used by BOTH
//! the GUI (Tauri commands) and the CLI (standalone mode).

use crate::{
    brand::{CONFIG_SLUG, LEGACY_CONFIG_SLUG},
    model::{next_available_installed_app_id, AppDef, InstalledApp, RuntimeSpec},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fmt,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

/// Errors returned by the strict Registry V2 API.  The legacy `AppDef` facade
/// below intentionally keeps its historical best-effort behavior until callers
/// migrate in a later task.
#[derive(Debug)]
pub enum StorageError {
    Io(io::Error),
    Json(serde_json::Error),
    InvalidRegistryVersion(u32),
    MigrationRefused(String),
    InvalidComposeValue(String),
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "registry I/O error: {error}"),
            Self::Json(error) => write!(f, "registry JSON error: {error}"),
            Self::InvalidRegistryVersion(version) => {
                write!(f, "unsupported registry version {version}; expected 2")
            }
            Self::MigrationRefused(reason) => write!(f, "v1 migration refused: {reason}"),
            Self::InvalidComposeValue(value) => {
                write!(f, "v1 compose value is not a path: {value:?}")
            }
        }
    }
}

impl std::error::Error for StorageError {}

impl From<io::Error> for StorageError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for StorageError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

pub type StorageResult<T> = Result<T, StorageError>;

/// The versioned, losslessly serializable registry format.  V1's `health`
/// field has no V2 `InstalledApp` equivalent: the untouched source bytes are
/// retained in `migration-v1-backup.json` rather than inventing a value.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RegistryV2 {
    pub version: u32,
    pub apps: Vec<InstalledApp>,
}

impl RegistryV2 {
    pub const VERSION: u32 = 2;

    pub fn new(apps: Vec<InstalledApp>) -> Self {
        Self {
            version: Self::VERSION,
            apps,
        }
    }
}

/// Registry V2 roots are platform configuration *base* directories: `%APPDATA%`
/// on Windows, `$XDG_CONFIG_HOME` on Linux, else `$HOME/.config`. V2 files are
/// stored directly under that root; the historical source is
/// `<root>/<legacy-config-slug>/apps.json`. `LOCAL_STORE_CONFIG_DIR` is honored only in test builds, never by production.
fn registry_config_root() -> PathBuf {
    #[cfg(test)]
    if let Ok(root) = std::env::var("LOCAL_STORE_CONFIG_DIR") {
        if !root.is_empty() {
            return PathBuf::from(root);
        }
    }

    PathBuf::from(config_base())
}

fn config_base() -> String {
    std::env::var("APPDATA")
        .ok()
        .filter(|value| !value.is_empty())
        .or_else(|| {
            std::env::var("XDG_CONFIG_HOME")
                .ok()
                .filter(|value| !value.is_empty())
        })
        .or_else(|| {
            std::env::var("HOME")
                .ok()
                .map(|home| format!("{home}/.config"))
        })
        .unwrap_or_else(|| ".".to_string())
}

pub fn registry_v2_path_for_root(root: &Path) -> PathBuf {
    root.join("registry-v2.json")
}

pub fn migration_v1_backup_path_for_root(root: &Path) -> PathBuf {
    root.join("migration-v1-backup.json")
}

pub fn legacy_v1_path_for_root(root: &Path) -> PathBuf {
    legacy_v1_path_for_platform(root, std::env::consts::OS)
}

/// Platform-neutral helper deliberately takes a root, so Windows and Linux
/// legacy locations can be covered without relying on the host test platform.
pub fn legacy_v1_path_for_platform(root: &Path, _platform: &str) -> PathBuf {
    root.join(LEGACY_CONFIG_SLUG).join("apps.json")
}

pub fn load_registry_v2() -> StorageResult<RegistryV2> {
    load_registry_v2_at(&registry_config_root())
}

pub fn load_registry_v2_at(root: &Path) -> StorageResult<RegistryV2> {
    let data = fs::read(registry_v2_path_for_root(root))?;
    let registry: RegistryV2 = serde_json::from_slice(&data)?;
    if registry.version != RegistryV2::VERSION {
        return Err(StorageError::InvalidRegistryVersion(registry.version));
    }
    Ok(registry)
}

pub fn save_registry_v2(registry: &RegistryV2) -> StorageResult<()> {
    save_registry_v2_at(&registry_config_root(), registry)
}

pub fn save_registry_v2_at(root: &Path, registry: &RegistryV2) -> StorageResult<()> {
    if registry.version != RegistryV2::VERSION {
        return Err(StorageError::InvalidRegistryVersion(registry.version));
    }
    let encoded = serde_json::to_vec_pretty(registry)?;
    atomic_replace_with_previous(&registry_v2_path_for_root(root), &encoded)
}

fn atomic_replace_with_previous(path: &Path, bytes: &[u8]) -> StorageResult<()> {
    if path.exists() {
        // A read failure means the old state is not known-good: do not replace it.
        let previous = fs::read(path)?;
        atomic_write(&path.with_extension("previous.json"), &previous)?;
    }
    atomic_write(path, bytes)
}

fn atomic_write(path: &Path, bytes: &[u8]) -> StorageResult<()> {
    let parent = path.parent().ok_or_else(|| {
        StorageError::MigrationRefused(format!("registry path has no parent: {}", path.display()))
    })?;
    fs::create_dir_all(parent)?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| StorageError::MigrationRefused(error.to_string()))?
        .as_nanos();
    let mut temp = None;
    for attempt in 0_u32..100 {
        let candidate = parent.join(format!(
            ".{}-{nonce}-{attempt}.tmp",
            path.file_name().unwrap().to_string_lossy()
        ));
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&candidate)
        {
            Ok(file) => {
                temp = Some((candidate, file));
                break;
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error.into()),
        }
    }
    let (temp_path, mut file) = temp.ok_or_else(|| {
        StorageError::MigrationRefused("could not allocate unique temporary registry file".into())
    })?;
    let result = (|| -> StorageResult<()> {
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        if path.exists() {
            // Windows cannot rename over a target. The prior bytes have already
            // been atomically retained as `.previous.json` before this removal.
            fs::remove_file(path)?;
        }
        fs::rename(&temp_path, path)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp_path);
    }
    result
}

#[derive(Deserialize)]
struct LegacyAppDef {
    name: String,
    url: String,
    icon: Option<String>,
    compose: Option<String>,
    health: Option<String>,
}

/// Import once only when no V2 file exists. Valid V1 bytes are parsed and fully
/// converted before either destination file is written. `health` remains in the
/// exact backup because InstalledApp intentionally has no health field.
pub fn migrate_v1_registry() -> StorageResult<()> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| StorageError::MigrationRefused(error.to_string()))?
        .as_secs();
    migrate_v1_registry_at(&registry_config_root(), now)
}

pub fn migrate_v1_registry_at(root: &Path, timestamp: u64) -> StorageResult<()> {
    let v2_path = registry_v2_path_for_root(root);
    if v2_path.exists() {
        return Err(StorageError::MigrationRefused(format!(
            "{} already exists; migration never overwrites V2",
            v2_path.display()
        )));
    }
    let source = fs::read(legacy_v1_path_for_root(root))?;
    let legacy: Vec<LegacyAppDef> = serde_json::from_slice(&source)?;
    let apps = convert_legacy_apps(legacy, timestamp)?;
    let registry = RegistryV2::new(apps);

    // Back up original valid bytes first. Both writes use same-directory atomic
    // replacement; no parse/validation failure can change either destination.
    atomic_write(&migration_v1_backup_path_for_root(root), &source)?;
    save_registry_v2_at(root, &registry)
}

fn convert_legacy_apps(
    legacy: Vec<LegacyAppDef>,
    timestamp: u64,
) -> StorageResult<Vec<InstalledApp>> {
    let mut used = HashSet::new();
    legacy
        .into_iter()
        .map(|app| {
            let base = ascii_slug(&app.name);
            let id = next_available_installed_app_id(&base, |candidate| !used.contains(candidate))
                .expect("ascii_slug always creates a valid installed-app ID");
            used.insert(id.clone());
            let runtime = match app.compose.filter(|value| !value.trim().is_empty()) {
                None => RuntimeSpec::External,
                Some(compose) => compose_runtime(&compose, &id)?,
            };
            // Read the V1 health field so deserialization remains deliberately
            // five-field and its source value is retained in the byte backup.
            let _health = app.health;
            Ok(InstalledApp {
                id,
                catalog_id: None,
                display_name: app.name,
                launch_url: app.url,
                icon_path: app.icon.map(PathBuf::from),
                runtime,
                created_at_unix: timestamp,
                updated_at_unix: timestamp,
            })
        })
        .collect()
}

fn compose_runtime(value: &str, project_name: &str) -> StorageResult<RuntimeSpec> {
    if value.contains('\n')
        || value.contains('\r')
        || value.contains("services:")
        || value.contains("version:")
        || value.contains(": ")
        || value.trim_start().starts_with("---")
    {
        return Err(StorageError::InvalidComposeValue(value.to_owned()));
    }
    let compose_file = PathBuf::from(value);
    let project_dir = compose_file
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .ok_or_else(|| StorageError::InvalidComposeValue(value.to_owned()))?
        .to_path_buf();
    Ok(RuntimeSpec::Compose {
        project_name: project_name.to_owned(),
        project_dir,
        compose_file,
    })
}

fn ascii_slug(name: &str) -> String {
    let mut slug = String::new();
    let mut pending_hyphen = false;
    for byte in name.bytes() {
        if byte.is_ascii_alphanumeric() {
            if pending_hyphen && !slug.is_empty() {
                slug.push('-');
            }
            slug.push(char::from(byte.to_ascii_lowercase()));
            pending_hyphen = false;
        } else if !slug.is_empty() {
            pending_hyphen = true;
        }
    }
    if slug.is_empty() {
        "app".to_owned()
    } else {
        slug
    }
}

/// Where registered apps live. Same location on every platform via APPDATA
/// (Windows) or XDG/HOME fallback on Linux/macOS.
fn config_path(slug: &str) -> PathBuf {
    let base = std::env::var("APPDATA")
        .ok()
        .filter(|v| !v.is_empty())
        .or_else(|| {
            std::env::var("XDG_CONFIG_HOME")
                .ok()
                .filter(|v| !v.is_empty())
        })
        .or_else(|| std::env::var("HOME").ok().map(|h| format!("{}/.config", h)))
        .unwrap_or_else(|| ".".to_string());
    let mut p = PathBuf::from(base);
    p.push(slug);
    p.push("apps.json");
    p
}

pub fn resolve_config() -> String {
    config_path(CONFIG_SLUG).to_string_lossy().into_owned()
}

fn legacy_config() -> String {
    config_path(LEGACY_CONFIG_SLUG)
        .to_string_lossy()
        .into_owned()
}

pub fn load_apps() -> Vec<AppDef> {
    let primary = PathBuf::from(resolve_config());
    let legacy = PathBuf::from(legacy_config());
    load_apps_from_paths(&primary, &legacy)
}

fn load_apps_from_paths(primary: &std::path::Path, legacy: &std::path::Path) -> Vec<AppDef> {
    match std::fs::read_to_string(primary) {
        Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            std::fs::read_to_string(legacy)
                .ok()
                .and_then(|data| serde_json::from_str(&data).ok())
                .unwrap_or_default()
        }
        Err(_) => Vec::new(),
    }
}

pub fn save_apps(apps: &[AppDef]) {
    let path = resolve_config();
    if let Some(parent) = std::path::Path::new(&path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(&path, serde_json::to_string(apps).unwrap());
}

/// Insert or replace an app by name (dedup on name).
pub fn upsert_app(
    name: &str,
    url: &str,
    icon: Option<String>,
    compose: Option<String>,
    health: Option<String>,
) {
    let mut apps = load_apps();
    apps.retain(|a| a.name != name);
    apps.push(AppDef {
        name: name.to_string(),
        url: url.to_string(),
        icon,
        compose,
        health,
    });
    save_apps(&apps);
}

pub fn remove_app(name: &str) -> bool {
    let mut apps = load_apps();
    let before = apps.len();
    apps.retain(|a| a.name != name);
    if apps.len() != before {
        save_apps(&apps);
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn fixture_paths() -> (PathBuf, PathBuf, PathBuf) {
        let root = std::env::temp_dir().join(format!(
            "local-store-storage-test-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&root).unwrap();
        (
            root.clone(),
            root.join("primary.json"),
            root.join("legacy.json"),
        )
    }

    fn app_json(name: &str) -> String {
        format!(
            r#"[{{"name":"{name}","url":"http://{name}","icon":null,"compose":null,"health":null}}]"#
        )
    }

    #[test]
    fn config_path_uses_native_separator() {
        let p = resolve_config();
        if cfg!(windows) {
            assert!(p.ends_with("local-store\\apps.json"), "got: {}", p);
        } else {
            assert!(!p.contains('\\'), "path must use native separator: {}", p);
            assert!(p.ends_with("local-store/apps.json"), "got: {}", p);
        }
    }

    #[test]
    fn upsert_dedups_by_name() {
        let mut apps: Vec<AppDef> = vec![];
        apps.push(AppDef {
            name: "x".into(),
            url: "http://a".into(),
            icon: None,
            compose: None,
            health: None,
        });
        apps.push(AppDef {
            name: "x".into(),
            url: "http://b".into(),
            icon: None,
            compose: None,
            health: None,
        });
        apps.retain(|a| a.name != "x");
        apps.push(AppDef {
            name: "x".into(),
            url: "http://c".into(),
            icon: None,
            compose: None,
            health: None,
        });
        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0].url, "http://c");
    }

    #[test]
    fn primary_registry_wins_over_legacy_registry() {
        let (root, primary, legacy) = fixture_paths();
        fs::write(&primary, app_json("primary")).unwrap();
        fs::write(&legacy, app_json("legacy")).unwrap();

        assert_eq!(load_apps_from_paths(&primary, &legacy)[0].name, "primary");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn missing_primary_registry_falls_back_to_legacy_registry() {
        let (root, primary, legacy) = fixture_paths();
        fs::write(&legacy, app_json("legacy")).unwrap();

        assert_eq!(load_apps_from_paths(&primary, &legacy)[0].name, "legacy");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn corrupt_primary_registry_does_not_fall_back_to_legacy_registry() {
        let (root, primary, legacy) = fixture_paths();
        fs::write(&primary, "not json").unwrap();
        fs::write(&legacy, app_json("legacy")).unwrap();

        assert!(load_apps_from_paths(&primary, &legacy).is_empty());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn unreadable_primary_registry_does_not_fall_back_to_legacy_registry() {
        let (root, primary, legacy) = fixture_paths();
        fs::create_dir(&primary).unwrap();
        fs::write(&legacy, app_json("legacy")).unwrap();

        assert!(load_apps_from_paths(&primary, &legacy).is_empty());
        fs::remove_dir_all(root).unwrap();
    }

    fn v2_paths(root: &std::path::Path) -> (PathBuf, PathBuf, PathBuf) {
        (
            registry_v2_path_for_root(root),
            migration_v1_backup_path_for_root(root),
            legacy_v1_path_for_root(root),
        )
    }

    fn fixture_v1() -> &'static [u8] {
        include_bytes!("../tests/fixtures/apps-v1.json")
    }

    #[test]
    fn registry_v2_atomic_save_and_load_round_trip() {
        let (root, registry_path, _, _) = {
            let (root, _, _) = fixture_paths();
            let (registry, backup, legacy) = v2_paths(&root);
            (root, registry, backup, legacy)
        };
        let registry = RegistryV2::new(vec![InstalledApp {
            id: "external-dashboard".into(),
            catalog_id: None,
            display_name: "External Dashboard".into(),
            launch_url: "https://dashboard.example.test".into(),
            icon_path: None,
            runtime: RuntimeSpec::External,
            created_at_unix: 7,
            updated_at_unix: 7,
        }]);

        save_registry_v2_at(&root, &registry).unwrap();
        assert_eq!(load_registry_v2_at(&root).unwrap(), registry);
        assert!(registry_path.exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn migration_imports_fixture_writes_exact_backup_and_preserves_runtime_boundary() {
        let (root, registry_path, backup_path, legacy_path) = {
            let (root, _, _) = fixture_paths();
            let (registry, backup, legacy) = v2_paths(&root);
            (root, registry, backup, legacy)
        };
        fs::create_dir_all(legacy_path.parent().unwrap()).unwrap();
        fs::write(&legacy_path, fixture_v1()).unwrap();

        migrate_v1_registry_at(&root, 1_700_000_000).unwrap();

        assert_eq!(fs::read(&backup_path).unwrap(), fixture_v1());
        let migrated = load_registry_v2_at(&root).unwrap();
        assert_eq!(migrated.version, 2);
        assert_eq!(migrated.apps.len(), 2);
        assert_eq!(migrated.apps[0].id, "external-dashboard");
        assert_eq!(migrated.apps[0].display_name, "External Dashboard");
        assert_eq!(
            migrated.apps[0].icon_path,
            Some(PathBuf::from("C:/icons/dashboard.png"))
        );
        assert!(matches!(migrated.apps[0].runtime, RuntimeSpec::External));
        match &migrated.apps[1].runtime {
            RuntimeSpec::Compose {
                project_name,
                project_dir,
                compose_file,
            } => {
                assert_eq!(project_name, "paperless-ngx");
                assert_eq!(
                    project_dir,
                    &PathBuf::from("C:/Users/example/compose/paperless")
                );
                assert_eq!(
                    compose_file,
                    &PathBuf::from("C:/Users/example/compose/paperless/compose.yaml")
                );
            }
            RuntimeSpec::External => panic!("compose path must migrate to compose runtime"),
        }
        assert!(registry_path.exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn migration_assigns_collision_safe_ascii_ids() {
        let (root, _, _, legacy_path) = {
            let (root, _, _) = fixture_paths();
            let (registry, backup, legacy) = v2_paths(&root);
            (root, registry, backup, legacy)
        };
        fs::create_dir_all(legacy_path.parent().unwrap()).unwrap();
        fs::write(
            &legacy_path,
            r#"[{"name":"Café!","url":"https://one.test","icon":null,"compose":null,"health":null},{"name":"CAFÉ?","url":"https://two.test","icon":null,"compose":null,"health":null},{"name":"---","url":"https://three.test","icon":null,"compose":null,"health":null}]"#,
        )
        .unwrap();

        migrate_v1_registry_at(&root, 9).unwrap();
        let ids: Vec<_> = load_registry_v2_at(&root)
            .unwrap()
            .apps
            .into_iter()
            .map(|app| app.id)
            .collect();
        assert_eq!(ids, vec!["caf", "caf-2", "app"]);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn malformed_v1_leaves_v2_and_backup_absent() {
        let (root, registry_path, backup_path, legacy_path) = {
            let (root, _, _) = fixture_paths();
            let (registry, backup, legacy) = v2_paths(&root);
            (root, registry, backup, legacy)
        };
        fs::create_dir_all(legacy_path.parent().unwrap()).unwrap();
        fs::write(&legacy_path, b"not valid json").unwrap();

        assert!(migrate_v1_registry_at(&root, 1).is_err());
        assert!(!registry_path.exists());
        assert!(!backup_path.exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn migration_refuses_nonempty_v2_without_changing_it() {
        let (root, registry_path, backup_path, legacy_path) = {
            let (root, _, _) = fixture_paths();
            let (registry, backup, legacy) = v2_paths(&root);
            (root, registry, backup, legacy)
        };
        fs::create_dir_all(legacy_path.parent().unwrap()).unwrap();
        fs::write(&legacy_path, fixture_v1()).unwrap();
        let existing = RegistryV2::new(vec![InstalledApp {
            id: "existing".into(),
            catalog_id: None,
            display_name: "Existing".into(),
            launch_url: "https://existing.test".into(),
            icon_path: None,
            runtime: RuntimeSpec::External,
            created_at_unix: 1,
            updated_at_unix: 1,
        }]);
        save_registry_v2_at(&root, &existing).unwrap();
        let original = fs::read(&registry_path).unwrap();

        assert!(migrate_v1_registry_at(&root, 2).is_err());
        assert_eq!(fs::read(&registry_path).unwrap(), original);
        assert!(!backup_path.exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn inline_compose_yaml_is_rejected_without_writing_migration_files() {
        let (root, registry_path, backup_path, legacy_path) = {
            let (root, _, _) = fixture_paths();
            let (registry, backup, legacy) = v2_paths(&root);
            (root, registry, backup, legacy)
        };
        fs::create_dir_all(legacy_path.parent().unwrap()).unwrap();
        fs::write(&legacy_path, r#"[{"name":"unsafe","url":"https://unsafe.test","icon":null,"compose":"services:\n  web:\n    image: unsafe","health":null}]"#).unwrap();

        assert!(migrate_v1_registry_at(&root, 1).is_err());
        assert!(!registry_path.exists());
        assert!(!backup_path.exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn legacy_path_helpers_match_root_semantics_on_windows_and_linux() {
        assert_eq!(
            legacy_v1_path_for_platform(PathBuf::from("C:/Config").as_path(), "windows"),
            PathBuf::from("C:/Config")
                .join(LEGACY_CONFIG_SLUG)
                .join("apps.json")
        );
        assert_eq!(
            legacy_v1_path_for_platform(PathBuf::from("/home/user/.config").as_path(), "linux"),
            PathBuf::from("/home/user/.config")
                .join(LEGACY_CONFIG_SLUG)
                .join("apps.json")
        );
    }
}
