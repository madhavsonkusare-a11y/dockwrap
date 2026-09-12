//! Docker itself must accept what the plan renderer writes.
//!
//! The unit tests assert the rendered text, which proves the renderer is
//! consistent but not that Compose agrees with it. The three shipped recipes
//! cover the single-service shape, because a plan renders them byte-for-byte
//! and `scripts/validate-recipes.py` puts those through Docker. The
//! multi-service shape has no shipped example, so it is checked here.
//!
//! This parses a Compose file; it starts nothing, needs no daemon and pulls no
//! image. It skips with a message when the Docker CLI is absent.
use local_store::plan::{DeploymentPlan, PlanMount, PlanOverrides, PlanService, PublishedPort};
use std::{path::PathBuf, process::Command};

fn docker_available() -> bool {
    Command::new("docker")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// A web application beside a database: the shape phase 2 of the roadmap has to
/// install without anyone editing Compose by hand.
fn web_and_database() -> DeploymentPlan {
    DeploymentPlan {
        id: "example".into(),
        services: vec![
            PlanService {
                name: "web".into(),
                image: "example/web:1.2.3".into(),
                environment: vec![
                    ("DATABASE_HOST".into(), "db".into()),
                    ("DATABASE_PORT".into(), "5432".into()),
                    ("DEBUG".into(), "false".into()),
                ],
                published: Some(PublishedPort {
                    host: 8080,
                    container: 8080,
                }),
                mounts: vec![PlanMount::directory("data", "/var/lib/web")],
                depends_on: vec!["db".into()],
                overrides: PlanOverrides::default(),
            },
            PlanService {
                name: "db".into(),
                image: "example/postgres:16.2".into(),
                environment: vec![("POSTGRES_DB".into(), "app".into())],
                published: None,
                mounts: vec![PlanMount::Volume {
                    name: "db-data".into(),
                    target: "/var/lib/postgresql/data".into(),
                    read_only: false,
                }],
                depends_on: Vec::new(),
                overrides: PlanOverrides::default(),
            },
        ],
        named_volumes: vec!["db-data".into()],
    }
}

#[test]
fn docker_resolves_a_multi_service_plan_the_way_the_plan_describes_it() {
    if !docker_available() {
        eprintln!("skipped: the Docker CLI is not on PATH");
        return;
    }
    let mut plan = web_and_database();
    plan.services[0]
        .overrides
        .healthy_dependencies
        .insert("db".into());
    plan.services[0].overrides.healthcheck = Some(
        local_store::plan::PlanHealthcheck::from_compose(&serde_json::json!({
            "test": ["CMD", "curl", "-f", "http://localhost:8080"],
            "interval":"10s", "timeout":"2s", "start_period":"30s", "retries":3
        }))
        .unwrap(),
    );
    plan.services[1].overrides.healthcheck = Some(
        local_store::plan::PlanHealthcheck::from_compose(&serde_json::json!({
            "test": "pg_isready -U app || exit 1"
        }))
        .unwrap(),
    );
    let rendered = plan.to_compose().expect("plan should render");

    let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("plan-multi-service.yaml");
    std::fs::write(&path, &rendered).expect("write the rendered Compose file");
    let output = Command::new("docker")
        .args(["compose", "-f"])
        .arg(&path)
        .args(["config", "--format", "json"])
        .output()
        .expect("docker compose must run");
    assert!(
        output.status.success(),
        "Docker rejected the rendered Compose file:\n{}\n---\n{rendered}",
        String::from_utf8_lossy(&output.stderr)
    );

    let config: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("Compose config must be JSON");
    let services = config["services"]
        .as_object()
        .expect("services must be an object");
    assert_eq!(services.len(), 2, "both services must survive rendering");
    assert_eq!(
        services["web"]["depends_on"]["db"]["condition"],
        "service_healthy"
    );
    assert_eq!(
        services["web"]["healthcheck"]["test"],
        serde_json::json!(["CMD", "curl", "-f", "http://localhost:8080"])
    );
    assert_eq!(services["web"]["healthcheck"]["interval"], "10s");
    assert_eq!(services["web"]["healthcheck"]["retries"], 3);
    assert_eq!(
        services["db"]["healthcheck"]["test"],
        serde_json::json!(["CMD-SHELL", "pg_isready -U app || exit 1"])
    );

    // Only the web service reaches the host, and only on loopback. A database
    // published to the host would be the whole point of this test failing.
    let web_ports = services["web"]["ports"]
        .as_array()
        .expect("the web service publishes a port");
    assert_eq!(web_ports.len(), 1);
    assert_eq!(web_ports[0]["host_ip"], "127.0.0.1");
    assert_eq!(web_ports[0]["target"], 8080);
    assert_eq!(web_ports[0]["published"], "8080");
    assert!(
        services["db"]["ports"]
            .as_array()
            .is_none_or(|p| p.is_empty()),
        "the database must not be reachable from the host: {:?}",
        services["db"]["ports"]
    );

    assert_eq!(services["web"]["image"], "example/web:1.2.3");
    assert_eq!(services["db"]["image"], "example/postgres:16.2");
    assert!(
        services["web"]["depends_on"]["db"].is_object(),
        "the ordering dependency must survive rendering"
    );
    assert!(
        config["volumes"]["db-data"].is_object(),
        "the named volume must be declared"
    );

    // Values that would otherwise become a number or a boolean must reach the
    // container as strings; Compose rejects them otherwise.
    let environment = &services["web"]["environment"];
    assert_eq!(environment["DATABASE_PORT"], "5432");
    assert_eq!(environment["DEBUG"], "false");
    assert_eq!(environment["DATABASE_HOST"], "db");
}
