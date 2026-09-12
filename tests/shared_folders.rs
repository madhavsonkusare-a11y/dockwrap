//! Sharing a folder with an app, end to end through the plan model.
//!
//! This is the only path that hands a container something outside the storage
//! Local Store manages, so what matters is not that it works but that the
//! refusals hold at every layer: a folder nobody chose, a folder that is not
//! allowed, and a plan that tries to carry a path instead of an answer.
use local_store::plan::{DeploymentPlan, PlanMount, PlanService, PublishedPort};
use local_store::setup::{FieldKind, PlanTemplate, SetupField};
use std::collections::BTreeMap;
use std::path::PathBuf;

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "local-store-shared-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn template() -> PlanTemplate {
    PlanTemplate {
        first_start: None,
        seeds: Vec::new(),
        plan: DeploymentPlan {
            id: "gallery".into(),
            services: vec![PlanService {
                name: "gallery".into(),
                image: "example/gallery:1.0".into(),
                digest: None,
                environment: Vec::new(),
                companion: None,
                networks: Vec::new(),
                published: Some(PublishedPort {
                    host: 8080,
                    container: 80,
                }),
                mounts: vec![
                    PlanMount::directory("data", "/app/data"),
                    PlanMount::Host {
                        source: "${PHOTOS}".into(),
                        target: "/photos".into(),
                        read_only: true,
                    },
                ],
                depends_on: Vec::new(),
                overrides: Default::default(),
            }],
            named_volumes: Vec::new(),
            internal_networks: Vec::new(),
        },
        fields: vec![SetupField {
            key: "PHOTOS".into(),
            label: "Photos folder".into(),
            kind: FieldKind::Folder { read_only: true },
            required: true,
            default: None,
            sensitive: false,
        }],
        secrets: Vec::new(),
    }
}

#[test]
fn a_chosen_folder_reaches_the_mount_as_the_path_that_was_checked() {
    let dir = scratch("ok");
    let template = template();
    let answers = BTreeMap::from([("PHOTOS".to_owned(), dir.to_string_lossy().into_owned())]);
    let resolved = template
        .resolve(&answers, &BTreeMap::new())
        .expect("a real folder should resolve");

    let compose = resolved.to_compose().expect("the plan should render");
    let expected = dir.canonicalize().unwrap();
    let text = expected.to_string_lossy().into_owned();
    // The mount source must be the resolved folder without the extended-length
    // prefix canonicalizing adds, because Docker cannot mount that form.
    let expected = text.strip_prefix(r"\\?\").unwrap_or(&text);
    assert!(
        compose.contains(expected),
        "expected {expected:?} in {compose}"
    );
    // Read-only survives, because the review promised it would.
    assert!(compose.contains(":/photos:ro"), "{compose}");
    // And the app's own storage is still a managed relative mount.
    assert!(compose.contains("./data:/app/data"), "{compose}");
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn a_folder_that_is_not_allowed_stops_the_install_rather_than_the_daemon() {
    let template = template();
    let system = if cfg!(windows) { r"C:\Windows" } else { "/etc" };
    let answers = BTreeMap::from([("PHOTOS".to_owned(), system.to_owned())]);
    let error = template
        .resolve(&answers, &BTreeMap::new())
        .expect_err("a system folder must be refused");
    assert!(error.contains("operating system"), "{error}");
}

#[test]
fn a_missing_answer_names_the_folder_nobody_chose() {
    let template = template();
    let error = template
        .resolve(&BTreeMap::new(), &BTreeMap::new())
        .expect_err("a required folder must be answered");
    assert!(error.contains("PHOTOS"), "{error}");
}

/// The guard that keeps this from becoming a way to mount anything: a plan may
/// not carry a literal path where an answer belongs.
#[test]
fn a_plan_cannot_carry_a_folder_nobody_was_asked_about() {
    let mut template = template();
    if let PlanMount::Host { source, .. } = &mut template.plan.services[0].mounts[1] {
        *source = if cfg!(windows) {
            r"C:\Users\someone\Photos".to_owned()
        } else {
            "/home/someone/Photos".to_owned()
        };
    }
    let error = template
        .validate()
        .expect_err("a literal path must be refused");
    // Either refusal is correct: the plan rejects the literal source, and the
    // template separately notices that the folder field it declares is now
    // referred to by nothing. What matters is that it cannot install.
    assert!(
        error.contains("declared folder answer") || error.contains("no service uses it"),
        "{error}"
    );
}

#[test]
fn a_shared_folder_is_never_treated_as_storage_the_installer_owns() {
    let resolved = template();
    // `data_directories` is what the installer creates and what deletion
    // removes. A folder the person already had must appear in neither.
    let directories = resolved.plan.data_directories();
    assert_eq!(directories, vec!["data"]);
}

#[test]
fn the_review_says_whether_the_app_may_change_what_is_in_there() {
    let review = template().setup_review().expect("it should project");
    let folder = review
        .fields
        .iter()
        .find(|field| field.key == "PHOTOS")
        .expect("the folder is offered");
    assert_eq!(folder.control, "folder");
    assert_eq!(folder.read_only, Some(true));
    assert!(!folder.sensitive);
}
