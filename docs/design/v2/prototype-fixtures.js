/*
 * Prototype fixtures are split by truth status on purpose.
 * `contract` mirrors fields returned by current commands/models.
 * `derived` is deterministic presentation computed from those fields.
 * `concept` is visibly labeled and must not ship as backend truth.
 */

export const fixtures = Object.freeze({
  contract: {
    doctor: {
      ready: true,
      checks: [
        { id: "docker", label: "Docker Engine", ok: true, detail: "29.3.1" },
        { id: "compose", label: "Docker Compose", ok: true, detail: "2.39.4" },
      ],
    },
    apps: [
      {
        id: "memos",
        catalog_id: "memos",
        display_name: "Memos",
        launch_url: "http://127.0.0.1:5230",
        icon: "assets/logos/memos.png",
        status: "running",
        runtime: { kind: "compose", project_name: "local-store-memos", project_dir: "Local Store/apps/memos", compose_file: "Local Store/apps/memos/compose.yaml" },
        created_at_unix: 1788537600,
        updated_at_unix: 1788796800,
      },
      {
        id: "immich",
        catalog_id: "immich",
        display_name: "Immich",
        launch_url: "http://photos.home:2283",
        icon: "assets/logos/immich.svg",
        status: "ready",
        runtime: { kind: "external" },
        created_at_unix: 1788278400,
        updated_at_unix: 1788278400,
      },
      {
        id: "uptime-kuma",
        catalog_id: "uptime-kuma",
        display_name: "Uptime Kuma",
        launch_url: "http://127.0.0.1:3001",
        icon: "assets/logos/uptime-kuma.svg",
        status: "stopped",
        runtime: { kind: "compose", project_name: "local-store-uptime-kuma", project_dir: "Local Store/apps/uptime-kuma", compose_file: "Local Store/apps/uptime-kuma/compose.yaml" },
        created_at_unix: 1787846400,
        updated_at_unix: 1788703200,
      },
      {
        id: "actual-budget",
        catalog_id: "actual",
        display_name: "Actual Budget",
        launch_url: "http://budget.home:5006",
        icon: "assets/logos/actual.svg",
        status: "unreachable",
        runtime: { kind: "external" },
        created_at_unix: 1787414400,
        updated_at_unix: 1787414400,
      },
      {
        id: "linkding",
        catalog_id: "linkding",
        display_name: "Linkding",
        launch_url: "http://127.0.0.1:9090",
        icon: "assets/logos/linkding.svg",
        status: "error",
        status_error: "Compose reported an unhealthy container.",
        runtime: { kind: "compose", project_name: "local-store-linkding", project_dir: "Local Store/apps/linkding", compose_file: "Local Store/apps/linkding/compose.yaml" },
        created_at_unix: 1787068800,
        updated_at_unix: 1788624000,
      },
    ],
    catalog: [
      { id: "memos", name: "Memos", category: "Note-taking & Editors", description: "Knowledge base that works with a SQLite database file.", icon: "assets/logos/memos.png", license: "MIT", capability: "install", action: "Install preview", maintenance: "Tracked", architectures: ["amd64", "arm64"], tags: ["notes", "knowledge base"], source: "awesome-selfhosted", source_url: "github.com/awesome-selfhosted/awesome-selfhosted", project_url: "github.com/usememos/memos", updated_at: "2026-09-05", warning: "" },
      { id: "immich", name: "Immich", category: "Photo Galleries", description: "Photo and video backup solution directly from your phone.", icon: "assets/logos/immich.svg", license: "AGPL-3.0", capability: "connect", action: "Connect", maintenance: "Tracked", architectures: ["amd64", "arm64"], tags: ["photos", "backup"], source: "awesome-selfhosted", source_url: "github.com/awesome-selfhosted/awesome-selfhosted", project_url: "github.com/immich-app/immich", updated_at: "2026-08-28", warning: "Local Store can save its web address but does not manage the Immich server." },
      { id: "uptime-kuma", name: "Uptime Kuma", category: "Monitoring", description: "A focused self-hosted monitoring tool with status pages.", icon: "assets/logos/uptime-kuma.svg", license: "MIT", capability: "connect", action: "Connect", maintenance: "Tracked", architectures: ["amd64", "arm64"], tags: ["monitoring", "status"], source: "awesome-selfhosted", source_url: "github.com/awesome-selfhosted/awesome-selfhosted", project_url: "github.com/louislam/uptime-kuma", updated_at: "2026-08-22", warning: "" },
      { id: "paperless-ngx", name: "Paperless-ngx", category: "Document Management", description: "Scan, index and archive documents with full-text search.", icon: "assets/logos/paperless-ngx.svg", license: "GPL-3.0", capability: "explore", action: "Explore project", maintenance: "Tracked", architectures: ["amd64", "arm64"], tags: ["documents", "archive"], source: "awesome-selfhosted", source_url: "github.com/awesome-selfhosted/awesome-selfhosted", project_url: "github.com/paperless-ngx/paperless-ngx", updated_at: "2026-08-19", warning: "No reviewed install or supported web connection is claimed by Local Store." },
      { id: "home-assistant", name: "Home Assistant", category: "Internet of Things", description: "Open-source home automation that puts local control first.", icon: "assets/logos/home-assistant.svg", license: "Apache-2.0", capability: "connect", action: "Connect", maintenance: "Tracked", architectures: ["amd64", "arm64"], tags: ["automation", "smart home"], source: "awesome-selfhosted", source_url: "github.com/awesome-selfhosted/awesome-selfhosted", project_url: "github.com/home-assistant/core", updated_at: "2026-08-11", warning: "Local Store only saves the address of an existing Home Assistant instance." },
      { id: "nextcloud", name: "Nextcloud", category: "File Transfer & Synchronization", description: "A safe home for files, calendars, contacts and collaboration.", icon: "assets/logos/nextcloud.svg", license: "AGPL-3.0", capability: "explore", action: "Explore project", maintenance: "Warning", architectures: ["amd64", "arm64"], tags: ["files", "collaboration"], source: "awesome-selfhosted", source_url: "github.com/awesome-selfhosted/awesome-selfhosted", project_url: "github.com/nextcloud/server", updated_at: "2026-06-14", warning: "Catalog metadata is older than the current review window; verify upstream requirements." },
    ],
    recipe: {
      schema_version: 3,
      id: "memos",
      display_name: "Memos",
      catalog_name: "Memos",
      description: "A lightweight, self-hosted memo hub.",
      category: "Note-taking & Editors",
      license: "MIT",
      version: "0.30.0",
      image: "neosmemo/memos:0.30.0",
      source_url: "github.com/usememos/memos",
      documentation_url: "usememos.com/docs/deploy/docker-compose",
      verified_at: "2026-09-05",
      launch_url: "http://localhost:5230",
      health_url: "http://localhost:5230",
      host_port: 5230,
      container_port: 5230,
      requirements: {
        docker_engine_os: "linux",
        compose_major: 2,
        local_storage_required: true,
        container_platforms: ["linux/amd64", "linux/arm/v7", "linux/arm64"],
        image_audit: {
          source_url: "hub.docker.com/v2/repositories/neosmemo/memos/tags/0.30.0",
          checked_at: "2026-09-07",
          index_digest: "sha256:71a5b4738d1bed96e92112004054f0888e92791b64eb78afd79077c96e6f9327",
        },
      },
      data_directories: ["data"],
      data_storage: "Local Store managed folder / memos / data",
      risk_notes: [
        "Creates a Docker container named local-store-memos.",
        "Publishes port 5230 on this computer's loopback interface only.",
        "Stores the SQLite database and uploaded assets in the Local Store app directory.",
        "Complete Memos setup and review its access settings before storing private notes.",
      ],
      setup_review: { fields: [], service_count: 1, generated_credential_count: 0 },
      icon: "assets/logos/memos.png",
    },
    starterRecipes: [
      {
        id: "memos", display_name: "Memos", description: "A lightweight, self-hosted memo hub.", category: "Note-taking & Editors", license: "MIT", version: "0.30.0", image: "neosmemo/memos:0.30.0", verified_at: "2026-09-05", launch_url: "http://localhost:5230", health_url: "http://localhost:5230", host_port: 5230, container_port: 5230, data_storage: "Local Store managed folder / memos / data", icon: "assets/logos/memos.png",
        requirements: { container_platforms: ["linux/amd64", "linux/arm/v7", "linux/arm64"], image_audit: { index_digest: "sha256:71a5b4738d1bed96e92112004054f0888e92791b64eb78afd79077c96e6f9327" } },
        risk_notes: ["Creates a Docker container named local-store-memos.", "Publishes port 5230 on this computer's loopback interface only.", "Stores the SQLite database and uploaded assets in the Local Store app directory.", "Complete Memos setup and review its access settings before storing private notes."],
      },
      {
        id: "n8n", display_name: "n8n", description: "A workflow automation platform with a visual editor and hundreds of integrations.", category: "Automation", license: "Sustainable Use License", version: "2.37.10", image: "docker.n8n.io/n8nio/n8n:2.37.10", verified_at: "2026-09-05", launch_url: "http://localhost:5678", health_url: "http://localhost:5678/healthz/readiness", host_port: 5678, container_port: 5678, data_storage: "Docker volume local-store-n8n_n8n-data", icon: "assets/logos/n8n.svg",
        requirements: { container_platforms: ["linux/amd64", "linux/arm64"], image_audit: { index_digest: "sha256:307d6065be25619aa24cfc63a7c2f04ca56d084a08c05c8e9f189a89f353b1ec" } },
        risk_notes: ["Creates a Docker container named local-store-n8n.", "Publishes port 5678 on this computer's loopback interface only.", "Stores workflows, credentials, and the encryption key in the Docker volume local-store-n8n_n8n-data.", "n8n uses the Sustainable Use License and is not an OSI-approved open-source product.", "External webhook delivery requires a separately configured HTTPS endpoint; this recipe stays local."],
      },
      {
        id: "uptime-kuma", display_name: "Uptime Kuma", description: "A friendly monitoring dashboard for websites, services, and status pages.", category: "Status / Uptime pages", license: "MIT", version: "2.5.3", image: "louislam/uptime-kuma:2.5.3", verified_at: "2026-09-05", launch_url: "http://localhost:3001", health_url: "http://localhost:3001", host_port: 3001, container_port: 3001, data_storage: "Local Store managed folder / uptime-kuma / data", icon: "assets/logos/uptime-kuma.svg",
        requirements: { container_platforms: ["linux/amd64", "linux/arm/v7", "linux/arm64"], image_audit: { index_digest: "sha256:3e24e96c89efff0e3a4b0698cbdd36c15ad3022371db57166e5588853002ee5c" } },
        risk_notes: ["Creates a Docker container named local-store-uptime-kuma.", "Publishes port 3001 on this computer's loopback interface only.", "Stores the SQLite database and uploaded assets in the Local Store app directory.", "Monitoring private network services may reveal their addresses inside Uptime Kuma's database."],
      },
    ],
    installStages: [
      { id: "checking_system", label: "Checking Docker and Compose", detail: "Confirms the two required local tools are available." },
      { id: "preparing_files", label: "Preparing app storage", detail: "Creates the managed folder and writes the reviewed Compose file." },
      { id: "validating_recipe", label: "Validating app configuration", detail: "Runs Docker Compose validation before starting anything." },
      { id: "starting_containers", label: "Starting the app", detail: "Downloads the pinned image if needed and starts the container." },
      { id: "waiting_for_health", label: "Waiting for the app to respond", detail: "Checks the reviewed local health address for up to 60 seconds." },
      { id: "saving_app", label: "Saving to My Apps", detail: "Commits the healthy app to the Local Store registry." },
    ],
    installErrors: {
      port_in_use: { title: "Port 5230 is already in use", detail: "Choose an unused port or stop the app using it before trying again.", recovery: "port" },
      prerequisite_unavailable: { title: "Docker is not ready", detail: "Docker and Docker Compose must be running before installation.", recovery: "doctor" },
      invalid_input: { title: "The reviewed configuration is invalid", detail: "No container was started. Review the port and setup answers before trying again.", recovery: "review" },
      timed_out: { title: "Memos did not become ready", detail: "The incomplete setup was rolled back. Check Docker and retry when the cause is resolved.", recovery: "retry" },
      rollback_failed: { title: "Cleanup could not finish safely", detail: "Containers or files may remain. Review the retained setup before installing again.", recovery: "recovery" },
    },
    recovery: {
      recipe_id: "memos",
      display_name: "Memos",
      compose_file: "Local Store/apps/memos/compose.yaml",
      project_name: "local-store-memos",
      docker_ownership_verified: true,
      ownership_status: "verified",
    },
    sessionOperations: [
      { id: "op-stop-uptime-kuma", app: "Uptime Kuma", kind: "Stop", state: "succeeded", detail: "Compose project stopped cleanly.", time: "2 days ago" },
      { id: "op-install-memos", app: "Memos", kind: "Install", state: "succeeded", detail: "Recipe 0.30.0 installed and registered.", time: "Yesterday" },
    ],
    logs: {
      memos: [
        "2026-09-09T09:41:52.118Z  INFO  server  listening on 0.0.0.0:5230",
        "2026-09-09T09:41:52.203Z  INFO  store   database ready at /var/opt/memos/memos_prod.db",
        "2026-09-09T09:41:52.247Z  INFO  http    GET /healthz 200 3ms",
        "2026-09-09T09:42:17.906Z  INFO  http    GET / 200 12ms",
        "2026-09-09T09:43:02.441Z  INFO  http    GET /api/v1/memos 200 8ms",
      ],
      "uptime-kuma": ["No recent output. The managed app is stopped."],
      linkding: [
        "2026-09-09T08:16:04.012Z ERROR  health  dependency check failed",
        "2026-09-09T08:16:04.015Z ERROR  worker  database connection refused",
      ],
    },
  },

  derived: {
    appSummary: { total: 5, running: 1, stopped: 1, linked: 2, attention: 3 },
    catalogSummary: { total: 1672, categories: 96, reviewedInstalls: 3, localIcons: 1672 },
    recipeFacts: { restartPolicy: "unless-stopped", containerName: "local-store-memos", managedItems: 3 },
    recent: [
      { app: "Memos", detail: "Managed app added to this workspace", time: "Yesterday" },
      { app: "Uptime Kuma", detail: "Stopped successfully", time: "2 days ago" },
      { app: "Immich", detail: "Linked address saved", time: "5 days ago" },
    ],
  },

  concept: {
    installSetupVariant: {
      notice: "Component study: the current reviewed Memos recipe asks for no setup values.",
      fields: [
        { key: "SITE_NAME", label: "Workspace name", required: false, sensitive: false, control: "text", default: "My notes", server_validated: true },
        { key: "API_KEY", label: "Existing API key", required: false, sensitive: true, control: "password", has_default: false, server_validated: true },
      ],
      generated_credential_count: 1,
    },
    overviewTelemetry: {
      notice: "Container memory is not collected by the current backend. This chart is a labeled visual study for a future bounded telemetry endpoint.",
      label: "Container memory · all managed apps",
      value: "412 MB",
      peak: "Peak 640 MB · 04:12",
      window: "Last 6 hours · concept fixture",
      path: "M8 154 C54 151 70 137 119 143 C170 149 185 117 231 119 C278 122 301 91 349 96 C393 101 421 106 458 83 C500 57 526 76 567 75 C616 74 642 112 690 114 C735 116 756 91 792 98",
    },
    activityNotice: "Live operation events are available only for the current launcher session. Persisted history, timestamps, and cross-session filters require backend work; the rows below evaluate that future model.",
    events: [
      { operation_id: "op-104", app_id: "memos", time: "Today · 09:42", group: "Today", app: "Memos", kind: "Start", state: "Succeeded", detail: "Container reached its readiness check.", stage: null, error: null },
      { operation_id: "op-103", app_id: "memos", time: "Today · 09:38", group: "Today", app: "Memos", kind: "Install", state: "Succeeded", detail: "Recipe 0.30.0 installed and registered.", stage: "saving_app", error: null },
      { operation_id: "op-102", app_id: "actual-budget", time: "Yesterday · 16:17", group: "Yesterday", app: "Actual Budget", kind: "Open", state: "Failed", detail: "The linked address could not be reached.", stage: null, error: "browser_open_failed" },
      { operation_id: "op-101", app_id: "uptime-kuma", time: "Yesterday · 11:28", group: "Yesterday", app: "Uptime Kuma", kind: "Stop", state: "Succeeded", detail: "Compose project stopped cleanly.", stage: null, error: null },
      { operation_id: "op-100", app_id: "linkding", time: "Sep 06 · 10:04", group: "Earlier", app: "Linkding", kind: "Start", state: "Failed", detail: "Compose reported an unhealthy container.", stage: null, error: "process_failed" },
    ],
    recoveryCleanupNotice: "Recovery cleanup is implemented in the Rust library but is not exposed as a launcher command. These controls specify the required UI and re-verification behavior without mutating files.",
  },
});

export const conditions = Object.freeze(["default", "loading", "empty", "busy", "success", "failure"]);

export const screens = Object.freeze({
  overview: { label: "Overview", eyebrow: "Workspace health", description: "What changed, what is running, and what needs your attention." },
  discover: { label: "Discover", eyebrow: "Offline catalog", description: "Explore self-hosted projects, reviewed installs, and apps you can connect." },
  "my-apps": { label: "My Apps", eyebrow: "Your workspace", description: "Open, inspect, and manage every app saved in Local Store." },
  activity: { label: "Activity", eyebrow: "Operations over time", description: "Every install, start, stop and failure, newest first." },
  settings: { label: "Settings", eyebrow: "System & support", description: "Docker diagnostics, recovery, catalog facts and product information." },
  install: { label: "Install", eyebrow: "Focused task · Memos", description: "Review exactly what Local Store will create before anything runs." },
  recovery: { label: "Review recovery", eyebrow: "Focused task · retained setup", description: "Verify ownership and consequences before clearing interrupted setup files." },
  "first-run": { label: "First Run", eyebrow: "Introduction", description: "Choose the most useful way to begin without blocking alternate routes." },
});
