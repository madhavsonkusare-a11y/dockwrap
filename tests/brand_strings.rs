use std::{
    fs,
    path::{Path, PathBuf},
};

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn shipping_files(root: &Path) -> Vec<PathBuf> {
    let mut files = vec![
        root.join("Cargo.toml"),
        root.join("tauri.conf.json"),
        root.join(".github/workflows/build.yml"),
        root.join("README.md"),
        root.join("CONTRIBUTING.md"),
    ];
    let publish = root.join("PUBLISH.md");
    if publish.is_file() {
        files.push(publish);
    }
    collect_shipping_source_files(&root.join("src"), &mut files);
    files
}

fn collect_shipping_source_files(directory: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(directory).expect("read shipping source directory") {
        let path = entry.expect("read shipping source entry").path();
        if path.is_dir() {
            collect_shipping_source_files(&path, files);
        } else if matches!(
            path.extension().and_then(|extension| extension.to_str()),
            Some("rs" | "js" | "html" | "plist")
        ) {
            files.push(path);
        }
    }
}

#[test]
fn product_identity_is_centralized_and_consistent() {
    let root = repository_root();
    let brand = fs::read_to_string(root.join("src/brand.rs")).expect("src/brand.rs must exist");
    for required in [
        "pub const PRODUCT_NAME: &str = \"Local Store\";",
        "pub const CLI_NAME: &str = \"local-store\";",
        "pub const CONFIG_SLUG: &str = \"local-store\";",
        "pub const URL_SCHEME: &str = \"localstore\";",
        "pub const LEGACY_CONFIG_SLUG: &str = \"dockwrap\";",
        "pub const LEGACY_URL_SCHEME: &str = \"dockwrap\";",
    ] {
        assert!(
            brand.contains(required),
            "missing identity constant: {required}"
        );
    }

    let cargo = fs::read_to_string(root.join("Cargo.toml")).expect("read Cargo.toml");
    let tauri = fs::read_to_string(root.join("tauri.conf.json")).expect("read tauri.conf.json");
    assert!(cargo.contains("name = \"local-store\""));
    assert!(cargo.contains("version = \"0.5.0-1\""));
    assert!(tauri.contains("\"productName\": \"Local Store\""));
    assert!(tauri.contains("\"version\": \"0.5.0-1\""));
    assert!(tauri.contains("\"identifier\": \"io.github.madhavsonkusare.localstore\""));
}

#[test]
fn shipping_user_interfaces_do_not_leak_the_legacy_brand() {
    let root = repository_root();
    let mut leaks = Vec::new();

    for path in shipping_files(&root) {
        let relative = path
            .strip_prefix(&root)
            .expect("shipping file is inside the repository")
            .to_string_lossy()
            .replace('\\', "/");
        let content = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
        for (line_number, line) in content.lines().enumerate() {
            if line.to_ascii_lowercase().contains("dockwrap")
                && !allowed_legacy_reference(&relative, line)
            {
                leaks.push(format!("{}:{}: {}", relative, line_number + 1, line.trim()));
            }
        }
    }

    assert!(
        leaks.is_empty(),
        "legacy brand leaked into shipping user interfaces:\n{}",
        leaks.join("\n")
    );
}

fn allowed_legacy_reference(relative: &str, line: &str) -> bool {
    // One-release migration compatibility only: named legacy constants, config
    // migration, and old URI scheme handling/registration.
    if (relative == "src/brand.rs" && line.starts_with("pub const LEGACY_"))
        || (relative == "Cargo.toml" && line.contains("repository ="))
        || (relative == "src/Info.plist" && line.contains("<string>dockwrap</string>"))
        || (relative == "README.md"
            && line.trim()
                == "Compatibility (one release): legacy dockwrap registry and dockwrap:// deep links are imported/recognized.")
    {
        return true;
    }

    // This is an implementation-only JavaScript bridge marker; changing it would
    // be an unrelated behavior change.
    relative == "src/windowing.rs" && line.contains("window.__dockwrapBridge")
}

#[test]
fn legacy_allowlist_is_limited_to_explicit_compatibility_references() {
    assert!(!allowed_legacy_reference(
        "README.md",
        "this is a legacy note about dockwrap"
    ));
    assert!(allowed_legacy_reference(
        "README.md",
        "Compatibility (one release): legacy dockwrap registry and dockwrap:// deep links are imported/recognized."
    ));
    assert!(!allowed_legacy_reference(
        "README.md",
        "LEGACY_URL_SCHEME must not excuse this stale dockwrap product name"
    ));
}
