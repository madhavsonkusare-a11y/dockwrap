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
    assert!(package.contains("rust-version = \"1.77.2\""));
    assert!(package.contains("license = \"MIT\""));
    assert!(package.contains("repository = \"https://github.com/madhavsonkusare-a11y/dockwrap\""));
    assert!(package.contains("readme = \"README.md\""));
    assert!(package.contains("description ="));
}
