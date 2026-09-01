use std::{fs, path::Path};

#[test]
fn package_metadata_is_present_in_cargo_manifest() {
    let manifest = fs::read_to_string(Path::new("Cargo.toml")).expect("read Cargo.toml");
    assert!(manifest.contains("name = \"dockwrap\""));
    assert!(manifest.contains("version = \"0.4.0\""));
    assert!(manifest.contains("edition = \"2021\""));
    assert!(manifest.contains("rust-version"));
    assert!(manifest.contains("license = \"MIT\""));
    assert!(manifest.contains("repository = \"https://github.com/madhavsonkusare-a11y/dockwrap\""));
    assert!(manifest.contains("readme = \"README.md\""));
    assert!(manifest.contains("description ="));
}
