use std::{fs, path::Path};

#[test]
fn package_metadata_is_present_in_cargo_manifest() {
    let manifest_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let manifest = fs::read_to_string(&manifest_path).expect("read Cargo.toml");

    // Scope assertions to the [package] table so dependency lines can never
    // satisfy them.
    let package = manifest
        .split("[dependencies]")
        .next()
        .expect("[package] table present");

    let version = env!("CARGO_PKG_VERSION");
    assert!(package.contains("name = \"local-store\""));
    let version_line = format!("version = \"{version}\"");
    assert!(package.contains(&version_line));
    assert!(package.contains("edition = \"2021\""));
    // Asserted against what Cargo parsed rather than a literal, so a bump stays
    // honest here. Whether the value is *high enough* for the dependency graph
    // is scripts/check-msrv.py, and whether our own code builds there is CI.
    let rust_version = env!("CARGO_PKG_RUST_VERSION");
    assert!(!rust_version.is_empty(), "rust-version must be declared");
    assert!(package.contains(&format!("rust-version = \"{rust_version}\"")));
    assert!(package.contains("license = \"MIT\""));
    assert!(
        package.contains("repository = \"https://github.com/madhavsonkusare-a11y/local-store\"")
    );
    assert!(package.contains("readme = \"README.md\""));
    assert!(package.contains("description ="));
}
