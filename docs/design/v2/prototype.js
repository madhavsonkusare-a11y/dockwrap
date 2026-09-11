import { conditions, fixtures, screens } from "./prototype-fixtures.js";
import { stressCatalog } from "./stress-fixtures.js";

const mount = document.querySelector("#screen");
const appShell = document.querySelector(".app-shell");
const routeLabel = document.querySelector("#route-label");
const panel = document.querySelector("#prototype-panel");
const scrim = document.querySelector(".panel-scrim");
const panelTrigger = document.querySelector(".lab-trigger");
const panelClose = document.querySelector(".panel-close");
const screenJump = document.querySelector("#screen-jump");
const toastRegion = document.querySelector("#toast-region");
const overviewConceptToggle = document.querySelector("#show-overview-concepts");
const environmentCard = document.querySelector(".environment-card");

const prototype = {
  screen: "overview",
  condition: "default",
  selectedApp: "memos",
  appSection: "overview",
  appSubstate: "default",
  appQuery: "",
  appDialog: null,
  installRecipe: "memos",
  installOrigin: "discover",
  installPort: 5230,
  installPortError: "",
  installFailedPort: null,
  savedLinkName: null,
  installStage: "starting_containers",
  installFailure: "port_in_use",
  installOutcome: null,
  installSetupStudy: false,
  installAdvanced: false,
  installAnswers: { SITE_NAME: "My notes" },
  firstRunStep: "welcome",
  firstRunPreflight: "checking",
  firstRunRecipe: "memos",
  firstRunCompletion: "installed",
  activityFilter: "all",
  activityApp: "all",
  activityQuery: "",
  activityExpanded: null,
  recoveryMode: "candidate",
  selectedProject: null,
  discoverDrawer: null,
  discoverFilterOpen: false,
  discoverQuickFilter: "all",
  discoverQuery: "",
  discoverFilters: { category: "all", license: "all", architecture: "all", warnings: false },
  panelOpen: false,
  showOverviewConcepts: false,
};

const escapeHtml = (value) => String(value)
  .replaceAll("&", "&amp;")
  .replaceAll("<", "&lt;")
  .replaceAll(">", "&gt;")
  .replaceAll('"', "&quot;")
  .replaceAll("'", "&#039;");

const icon = (name) => `<svg aria-hidden="true"><use href="#i-${name}"></use></svg>`;
const appIcon = (app, large = false) => `<span class="app-icon${large ? " large" : ""}"><img src="${escapeHtml(app.icon)}" alt=""></span>`;
const appType = (app) => app.runtime.kind === "compose" ? "Managed app" : "Linked app";
const statusClass = (status) => status === "running" || status === "ready" || status === "succeeded" ? "success" : status === "unreachable" || status === "failed" || status === "error" ? "danger" : "warning";
const statusLabel = (status) => status.charAt(0).toUpperCase() + status.slice(1);
const starterRecipe = (id = prototype.firstRunRecipe) => fixtures.contract.starterRecipes.find((recipe) => recipe.id === id) || fixtures.contract.starterRecipes[0];
const activeRecipe = () => prototype.installRecipe === "memos" ? fixtures.contract.recipe : starterRecipe(prototype.installRecipe);
const activeRecipeFacts = (recipe = activeRecipe()) => ({ restartPolicy: "unless-stopped", containerName: `local-store-${recipe.id}`, managedItems: 3 });

function header(actions = "", overrides = {}) {
  const meta = screens[prototype.screen];
  return `<header class="screen-header">
    <div class="screen-title">
      <p class="eyebrow">${escapeHtml(overrides.eyebrow || meta.eyebrow)}</p>
      <h1 tabindex="-1">${escapeHtml(overrides.label || meta.label)}</h1>
      <p>${escapeHtml(overrides.description || meta.description)}</p>
    </div>
    ${actions ? `<div class="header-actions">${actions}</div>` : ""}
  </header>`;
}

function conditionBanner() {
  const content = {
    busy: ["sliders", "Working in the background", "Controls stay in place while the operation finishes."],
    success: ["check", "Action completed", "The affected screen remains selected and ready for the next step."],
    failure: ["alert", "Something needs attention", "Inputs and selection are preserved so the problem can be corrected."],
  }[prototype.condition];
  if (!content) return "";
  return `<div class="condition-banner ${prototype.condition}" role="status">${icon(content[0])}<span><strong>${content[1]}.</strong> ${content[2]}</span></div>`;
}

function loadingScreen() {
  return `${header()}<div class="skeleton-stack" role="status" aria-label="Loading ${escapeHtml(screens[prototype.screen].label)}" aria-busy="true">
    <div class="skeleton hero"></div>
    <div class="skeleton-row"><div class="skeleton"></div><div class="skeleton"></div><div class="skeleton"></div></div>
    <div class="skeleton" style="min-height:220px"></div>
  </div>`;
}

function emptyScreen() {
  if (prototype.screen === "overview") {
    return `${header()}<section class="panel overview-empty">
      <div class="empty-promise"><span class="empty-symbol">${icon("grid")}</span><p class="eyebrow">Nothing to monitor yet</p><h2>Bring one app into your workspace.</h2><p>Link something already running, preview a reviewed local install, or browse the offline catalog. Local Store will keep its status and next action close.</p></div>
      <div class="empty-routes" aria-label="Ways to add an app">
        <button class="empty-route primary-route" type="button" data-prototype-action="connect"><span>${icon("plus")}</span><strong>Connect an app</strong><small>Save the address of something you already host.</small>${icon("arrow")}</button>
        <button class="empty-route" type="button" data-go-screen="install"><span>${icon("discover")}</span><strong>Install a starter</strong><small>Review Memos before anything changes.</small>${icon("arrow")}</button>
        <button class="empty-route" type="button" data-go-screen="discover"><span>${icon("search")}</span><strong>Browse catalog</strong><small>Explore 1,672 projects and their options.</small>${icon("arrow")}</button>
      </div>
    </section>`;
  }
  if (prototype.screen === "discover") {
    return `${header(`<button class="button ghost" type="button" data-open-connect="">${icon("plus")} Connect app</button>`)}
      ${discoverToolbar("desktop wiki")}
      <section class="panel catalog-empty-panel"><span class="empty-symbol">${icon("search")}</span><p class="eyebrow">0 projects</p><h2>No project matches “desktop wiki”</h2><p>Try fewer words, switch the capability filter, or clear the advanced filters. Your catalog and saved apps have not changed.</p><button class="button primary" type="button" data-clear-discover>Clear search and filters ${icon("arrow")}</button></section>`;
  }
  if (prototype.screen === "my-apps") {
    return `${header(`<button class="button primary" type="button" data-open-connect="">${icon("plus")} Connect app</button>`)}<section class="panel apps-workspace empty-apps-workspace"><div class="apps-list"><div class="list-head"><strong>Saved apps</strong><span>0 total</span></div><label class="search-field"><span class="field-label" hidden>Filter saved apps</span>${icon("search")}<input class="input" type="search" placeholder="Filter saved apps" disabled></label><div class="empty-list-note">Your saved apps will appear here.</div></div><div class="empty-apps-detail"><span class="empty-symbol">${icon("grid")}</span><p class="eyebrow">Your workspace is empty</p><h2>Add the first app you want close.</h2><p>Connect something already running, browse the offline catalog, or preview a reviewed local install.</p><div><button class="button primary" type="button" data-open-connect="">Connect an app ${icon("arrow")}</button><button class="button ghost" type="button" data-go-screen="discover">Browse catalog</button><button class="text-button" type="button" data-go-screen="install">Review Memos install</button></div></div></section>`;
  }
  const empty = {
    activity: ["activity", "No operations yet", "Install, start, stop, or open an app and its result will appear here when history is implemented.", "Go to My Apps", "my-apps"],
    settings: ["settings", "Nothing needs recovery", "System checks are ready and no retained install files were found.", "Run Docker check", "settings"],
    install: ["discover", "Recipe unavailable", "This project does not currently have a reviewed local installation recipe.", "Back to Discover", "discover"],
    recovery: ["check", "No retained setup", "The recovery scan did not find any interrupted reviewed installs.", "Back to Settings", "settings"],
    "first-run": ["grid", "No starter recipe selected", "You can still connect an existing app or browse the offline catalog.", "Browse catalog", "discover"],
  }[prototype.screen];
  return `${header()}<section class="panel empty-state"><div><span class="empty-symbol">${icon(empty[0])}</span><h2>${empty[1]}</h2><p>${empty[2]}</p><button class="button primary" type="button" data-go-screen="${empty[4]}">${empty[3]} ${icon("arrow")}</button></div></section>`;
}

function overview() {
  const { doctor, apps, recovery, sessionOperations } = fixtures.contract;
  const allHealthy = prototype.condition === "success";
  const environmentFailed = prototype.condition === "failure";
  const checking = prototype.condition === "busy";
  const visibleApps = allHealthy ? apps.map((app) => ({ ...app, status: app.runtime.kind === "compose" ? "running" : "ready" })) : apps;
  const summary = {
    total: visibleApps.length,
    managed: visibleApps.filter((app) => app.runtime.kind === "compose").length,
    linked: visibleApps.filter((app) => app.runtime.kind === "external").length,
  };
  const attention = visibleApps.filter((app) => ["stopped", "unreachable", "error"].includes(app.status));
  const doctorChecks = doctor.checks.map((check) => {
    if (checking) return { ...check, detail: "Checking" };
    if (environmentFailed && check.id === "docker") return { ...check, ok: false, detail: "Unavailable" };
    if (environmentFailed && check.id === "compose") return { ...check, ok: false, detail: "Not checked" };
    return check;
  });
  const hero = environmentFailed
    ? { tone: "danger", label: "Local installs are paused", title: "Docker Engine needs your attention.", body: "Linked apps remain available. Restore Docker before starting or installing managed apps." }
    : checking
      ? { tone: "warning", label: "Checking local environment", title: "Your apps stay available while checks run.", body: "Local Store is checking Docker Engine and Docker Compose. Existing results remain visible until the check completes." }
      : allHealthy
        ? { tone: "success", label: "No action needed", title: "Everything in this workspace looks ready.", body: "Docker and Compose are available. Managed apps are running and linked addresses answered their latest checks." }
        : { tone: "success", label: "Docker and Compose ready", title: `${attention.length} apps need a quick look.`, body: "One managed app is stopped, one reports an error, and one linked address did not answer its latest check. Everything else is ready." };
  const actions = attention.length
    ? `<button class="button ghost" type="button" data-go-screen="discover">${icon("plus")} Add app</button><button class="button primary" type="button" data-go-screen="my-apps">Review ${attention.length} items ${icon("arrow")}</button>`
    : `<button class="button ghost" type="button" data-go-screen="discover">${icon("plus")} Add app</button><button class="button primary" type="button" data-go-screen="my-apps">Open My Apps ${icon("arrow")}</button>`;
  const recentActivity = [
    { ...sessionOperations[1], source: "Session operation" },
    { ...sessionOperations[0], source: "Session operation" },
    { app: fixtures.derived.recent[2].app, detail: fixtures.derived.recent[2].detail, time: fixtures.derived.recent[2].time, source: "Registry change" },
  ];
  const telemetry = fixtures.concept.overviewTelemetry;
  const conceptStudy = prototype.showOverviewConcepts ? `<section class="concept-telemetry grain-surface" aria-labelledby="telemetry-title" aria-describedby="telemetry-notice">
    <div class="telemetry-head"><div><span class="lab-tag">Concept telemetry · backend work</span><p id="telemetry-title">${escapeHtml(telemetry.label)}</p><strong>${escapeHtml(telemetry.value)}</strong></div><span class="telemetry-peak">${escapeHtml(telemetry.peak)}</span></div>
    <div class="telemetry-plot"><svg viewBox="0 0 800 190" role="img" aria-label="Concept line chart showing container memory rising and falling over six hours" preserveAspectRatio="none"><defs><linearGradient id="telemetry-area" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#fff3e7" stop-opacity=".30"/><stop offset="1" stop-color="#ff7a22" stop-opacity=".03"/></linearGradient></defs><path class="telemetry-grid" d="M8 46 H792 M8 96 H792 M8 146 H792"/><path class="telemetry-area" d="${telemetry.path} L792 184 L8 184 Z"/><path class="telemetry-line" d="${telemetry.path}"/><circle cx="500" cy="66" r="5"/></svg><span>${escapeHtml(telemetry.window)}</span></div>
    <p class="telemetry-notice" id="telemetry-notice">${escapeHtml(telemetry.notice)}</p>
  </section>` : "";
  const stateNotice = checking
    ? `<div class="condition-banner busy" role="status">${icon("sliders")}<span><strong>System check in progress.</strong> The prior app summary remains usable.</span></div>`
    : environmentFailed
      ? `<div class="condition-banner failure" role="alert">${icon("alert")}<span><strong>Docker Engine did not answer.</strong> No automatic repair or retry has started.</span></div>`
      : "";
  const attentionContent = allHealthy
    ? `<div class="calm-state"><span class="calm-glyph">${icon("check")}</span><div><strong>No app needs attention</strong><small>Stopped, unreachable, error, and recovery signals will appear here.</small></div><button class="text-button" type="button" data-go-screen="my-apps">View all apps ${icon("arrow")}</button></div>`
    : `${environmentFailed ? `<div class="recovery-callout"><span>${icon("alert")}</span><div><strong>Retained setup can be reviewed safely</strong><small>${escapeHtml(recovery.display_name)} · ownership ${escapeHtml(recovery.ownership_status)}</small></div><button class="text-button" type="button" data-go-screen="recovery">Review recovery ${icon("arrow")}</button></div>` : ""}
      ${attention.map((app) => `<div class="attention-row">${appIcon(app)}<div><strong title="${escapeHtml(app.display_name)}">${escapeHtml(app.display_name)}</strong>${((message) => `<small title="${escapeHtml(message)}">${escapeHtml(message)}</small>`)(app.status === "stopped" ? "Managed app · stopped — start it only when needed" : app.status === "error" ? `Managed app · ${app.status_error || "runtime error"}` : "Linked app · address did not answer the latest check")}</div><button class="text-button" type="button" data-go-screen="my-apps" data-select-app="${app.id}">Review ${icon("arrow")}</button></div>`).join("")}`;
  return `${header(actions)}
    ${stateNotice}
    <section class="panel health-hero ${hero.tone}">
      <div class="hero-copy">
        <div class="status-row"><span class="status-dot ${hero.tone}"></span>${escapeHtml(hero.label)}</div>
        <h2>${escapeHtml(hero.title)}</h2>
        <p>${escapeHtml(hero.body)}</p>
      </div>
      <div class="doctor-summary" aria-label="Docker checks">
        <p class="doctor-label">Local prerequisites <span class="truth-label">Current data</span></p>
        ${doctorChecks.map((check) => `<div class="doctor-row"><span class="status-dot ${checking ? "warning" : check.ok ? "success" : "danger"}"></span><span>${escapeHtml(check.label)}</span><code>${escapeHtml(check.detail)}</code></div>`).join("")}
        <button class="text-button" type="button" data-go-screen="settings">View diagnostics ${icon("arrow")}</button>
      </div>
    </section>
    <section class="metric-grid" aria-label="Workspace summary">
      <article class="panel metric-card"><strong>${summary.total}</strong><span>Saved apps</span><small>Registry records · available</small></article>
      <article class="panel metric-card"><strong>${summary.managed}</strong><span>Managed locally</span><small>Runtime type · derived</small></article>
      <article class="panel metric-card"><strong>${summary.linked}</strong><span>Linked apps</span><small>Runtime type · derived</small></article>
      <article class="panel metric-card ${attention.length ? "attention" : ""}"><strong>${attention.length}</strong><span>Need attention</span><small>Status + readiness · derived</small></article>
    </section>
    ${conceptStudy}
    <div class="content-grid">
      <section class="panel section-card">
        <div class="section-heading"><h2>Needs attention</h2><span>${attention.length ? `${attention.length} apps` : "All clear"}</span></div>
        ${attentionContent}
      </section>
      <section class="panel section-card">
        <div class="section-heading"><h2>Recent activity</h2><span>This session + registry</span></div>
        ${recentActivity.map((item) => `<div class="recent-row"><div><strong>${escapeHtml(item.app)}</strong><small>${escapeHtml(item.source)} · ${escapeHtml(item.detail)}</small></div><time>${escapeHtml(item.time)}</time></div>`).join("")}
      </section>
    </div>`;
}

function discoverToolbar(query = prototype.discoverQuery) {
  const summary = fixtures.derived.catalogSummary;
  const filters = prototype.discoverFilters;
  const activeAdvanced = [filters.category !== "all", filters.license !== "all", filters.architecture !== "all", filters.warnings].filter(Boolean).length;
  return `<div class="discover-toolbar">
    <label class="search-field"><span class="field-label" hidden>Search catalog</span>${icon("search")}<input class="input" id="catalog-search" type="search" value="${escapeHtml(query)}" placeholder="Search ${summary.total.toLocaleString()} projects" autocomplete="off"></label>
    <div class="capability-tabs" aria-label="Capability filter">
      ${[["all", "All"], ["install", "Reviewed install"], ["connect", "Connect"], ["explore", "Explore"]].map(([value, label]) => `<button type="button" data-quick-filter="${value}" aria-pressed="${prototype.discoverQuickFilter === value}">${label}</button>`).join("")}
    </div>
    <div class="filter-anchor">
      <button class="button ghost filter-trigger" type="button" data-toggle-filters aria-expanded="${prototype.discoverFilterOpen}" aria-controls="catalog-filter-panel">${icon("sliders")} Filters${activeAdvanced ? `<span>${activeAdvanced}</span>` : ""}</button>
      <div id="catalog-filter-panel" class="filter-popover" ${prototype.discoverFilterOpen ? "" : "hidden"}>
        <div class="filter-popover-head"><div><p class="eyebrow">Narrow the catalog</p><strong>More filters</strong></div><button class="icon-button" type="button" data-toggle-filters aria-label="Close filters">${icon("close")}</button></div>
        <label class="field-label">Category<select class="input" data-catalog-filter="category"><option value="all">All categories</option><option value="Monitoring" ${filters.category === "Monitoring" ? "selected" : ""}>Monitoring</option><option value="Photo Galleries" ${filters.category === "Photo Galleries" ? "selected" : ""}>Photo galleries</option><option value="Document Management" ${filters.category === "Document Management" ? "selected" : ""}>Document management</option></select></label>
        <label class="field-label">License<select class="input" data-catalog-filter="license"><option value="all">Any license</option><option value="MIT" ${filters.license === "MIT" ? "selected" : ""}>MIT</option><option value="AGPL-3.0" ${filters.license === "AGPL-3.0" ? "selected" : ""}>AGPL-3.0</option><option value="GPL-3.0" ${filters.license === "GPL-3.0" ? "selected" : ""}>GPL-3.0</option></select></label>
        <label class="field-label">Architecture<select class="input" data-catalog-filter="architecture"><option value="all">Any architecture</option><option value="amd64" ${filters.architecture === "amd64" ? "selected" : ""}>amd64</option><option value="arm64" ${filters.architecture === "arm64" ? "selected" : ""}>arm64</option></select></label>
        <label class="warning-filter"><input type="checkbox" data-warning-filter ${filters.warnings ? "checked" : ""}><span><strong>Maintenance warnings only</strong><small>Projects with stale or caution metadata</small></span></label>
        <button class="text-button" type="button" data-clear-advanced>Clear advanced filters</button>
      </div>
    </div>
  </div>`;
}

function projectDrawer() {
  if (!prototype.discoverDrawer) return "";
  const project = fixtures.contract.catalog.find((item) => item.id === prototype.selectedProject) || null;
  const connectMode = prototype.discoverDrawer === "connect";
  const name = project?.name || "an existing app";
  const detailAction = project?.capability === "install"
    ? `<button class="button primary" type="button" data-go-screen="install">Review install ${icon("arrow")}</button><button class="button ghost" type="button" data-open-connect="${project.id}">Connect running app</button>`
    : project?.capability === "connect"
      ? `<button class="button primary" type="button" data-open-connect="${project.id}">Connect this app ${icon("arrow")}</button>`
      : `<button class="button primary" type="button" data-visit-project="${project?.id || ""}">Visit project ${icon("external")}</button>`;
  const drawerBody = connectMode ? `<div class="connect-intro"><span class="drawer-kicker">Linked app</span><h2 id="discover-drawer-title">Connect ${escapeHtml(name)}</h2><p>Save an HTTP(S) address in Local Store. This creates a shortcut and readiness check; it does not install, start, stop, update, or back up the server.</p></div>
    <form class="connect-form" data-connect-form novalidate>
      <label class="field-label" for="connect-name">Name<input class="input" id="connect-name" name="name" value="${escapeHtml(project?.name || "")}" required></label>
      <label class="field-label" for="connect-address">HTTP(S) address<input class="input" id="connect-address" name="address" type="url" value="" placeholder="${project ? `e.g. http://${escapeHtml(project.id)}.local` : "e.g. http://photos.home:2283"}" required aria-describedby="connect-consequence connect-feedback"></label>
      <p id="connect-consequence" class="form-note">The address check is advisory. Saving adds a linked app even when the server is temporarily offline.</p>
      <div id="connect-feedback" class="address-feedback" role="status" hidden></div>
      <div class="drawer-actions"><button class="button ghost" type="button" data-check-address>Check address</button><button class="button primary" type="submit">Save linked app ${icon("arrow")}</button></div>
    </form>` : `<div class="drawer-identity">${appIcon(project, true)}<div><span class="drawer-kicker">${escapeHtml(project.category)}</span><h2 id="discover-drawer-title">${escapeHtml(project.name)}</h2><p>${escapeHtml(project.description)}</p></div></div>
    <div class="capability-statement ${project.capability}"><span>${icon(project.capability === "install" ? "docker" : project.capability === "connect" ? "plus" : "external")}</span><div><strong>${escapeHtml(project.action)}</strong><small>${project.capability === "install" ? "A reviewed Local Store recipe is available. Nothing runs until its review is confirmed." : project.capability === "connect" ? "A web interface is known. Local Store can save its address but will not manage the server." : "Discovery only. Local Store does not claim an install recipe or supported connection."}</small></div></div>
    ${project.warning ? `<div class="project-warning">${icon("alert")}<p>${escapeHtml(project.warning)}</p></div>` : ""}
    <dl class="project-facts"><div><dt>License</dt><dd>${escapeHtml(project.license)}</dd></div><div><dt>Architecture</dt><dd>${escapeHtml(project.architectures.join(", "))}</dd></div><div><dt>Maintenance</dt><dd>${escapeHtml(project.maintenance)}</dd></div><div><dt>Catalog updated</dt><dd>${escapeHtml(project.updated_at)}</dd></div></dl>
    <section class="provenance-block"><p class="eyebrow">Provenance</p><div><span>Catalog source</span><code>${escapeHtml(project.source)}</code></div><div><span>Source record</span><code>${escapeHtml(project.source_url)}</code></div><div><span>Upstream project</span><code>${escapeHtml(project.project_url)}</code></div></section>
    <div class="drawer-actions">${detailAction}</div>`;
  return `<div class="discover-scrim" data-dismiss-discover></div><aside class="project-drawer" role="dialog" aria-modal="true" aria-labelledby="discover-drawer-title"><div class="drawer-top"><span class="lab-tag">Catalog facts · available</span><button class="icon-button" type="button" data-dismiss-discover aria-label="Close project panel">${icon("close")}</button></div>${drawerBody}</aside>`;
}

function discover() {
  const summary = fixtures.derived.catalogSummary;
  const apps = fixtures.contract.catalog;
  const headerAction = `<button class="button ghost" type="button" data-open-connect="">${icon("plus")} Connect app</button>`;
  if (prototype.condition === "failure") return `${header(headerAction)}${discoverToolbar()}<section class="panel catalog-error"><span class="empty-symbol">${icon("alert")}</span><p class="eyebrow">Local snapshot unavailable</p><h2>The catalog could not be read.</h2><p>Saved apps are unaffected. Retry the local catalog file or open Settings to inspect the snapshot details.</p><div><button class="button primary" type="button" data-prototype-action="catalog-retry">Retry catalog ${icon("arrow")}</button><button class="button ghost" type="button" data-go-screen="settings">Catalog details</button></div><code>catalog_index · read_failed · no network fallback</code></section>`;
  const stateNotice = prototype.condition === "success" ? `<div class="condition-banner success" role="status">${icon("check")}<span><strong>Catalog snapshot ready.</strong> 1,672 local project records and icons are available offline.</span></div>` : "";
  const pagination = prototype.condition === "busy"
    ? `<div id="catalog-pagination" class="catalog-pagination busy" role="status"><span>Showing 6 of ${summary.total.toLocaleString()}</span><button class="button ghost" type="button" disabled>Loading next page…</button></div>`
    : `<div id="catalog-pagination" class="catalog-pagination"><span id="catalog-page-count">Showing 6 of ${summary.total.toLocaleString()} · Page 1</span><button class="button ghost" type="button" data-prototype-action="next-page">Load next 24 ${icon("arrow")}</button></div>`;
  const reviewedRecipeCards = fixtures.contract.starterRecipes.map((recipe) => `<button class="featured-recipe-card" type="button" data-reviewed-recipe="${recipe.id}" aria-label="Review ${escapeHtml(recipe.display_name)} install"><span class="featured-icon">${appIcon(recipe)}</span><strong>${escapeHtml(recipe.display_name)}</strong><small>${escapeHtml(recipe.version)}</small><small>localhost:${recipe.host_port}</small>${icon("arrow")}</button>`).join("");
  return `${header(headerAction)}
    ${stateNotice}
    <section class="featured-recipe grain-surface" aria-labelledby="featured-recipes-title"><div class="featured-recipe-copy"><span>Verified recipes</span><h2 id="featured-recipes-title">Three you can install<br>without reading a compose file.</h2><p>Every image, port, storage path, health check, and rollback step is shown before anything runs.</p></div><div class="featured-recipe-cards">${reviewedRecipeCards}</div></section>
    ${discoverToolbar()}
    <div class="catalog-note"><span><strong id="catalog-visible">${apps.length}</strong> representative results · ${summary.total.toLocaleString()} projects in the offline catalog</span><span>${summary.localIcons.toLocaleString()} local icons · catalog snapshot</span></div>
    <section class="catalog-grid" aria-label="Catalog projects">
      ${apps.map((app) => `<article class="panel catalog-card" data-search-value="${escapeHtml(`${app.name} ${app.category} ${app.description} ${app.tags.join(" ")}`.toLowerCase())}" data-capability="${app.capability}" data-category="${escapeHtml(app.category)}" data-license="${escapeHtml(app.license)}" data-architectures="${escapeHtml(app.architectures.join(" "))}" data-warning="${app.maintenance === "Warning"}">
        <div class="app-line">${appIcon(app)}<div><h2 title="${escapeHtml(app.name)}">${escapeHtml(app.name)}</h2><span class="category" title="${escapeHtml(app.category)}">${escapeHtml(app.category)}</span></div>${app.maintenance === "Warning" ? `<span class="maintenance-mark" title="Maintenance warning">${icon("alert")}</span>` : ""}</div>
        <p>${escapeHtml(app.description)}</p>
        <div class="card-facts">${[app.license, app.architectures.join(" · ")].filter(Boolean).map((fact) => `<span>${escapeHtml(fact)}</span>`).join("")}</div>
        <footer><button class="text-button secondary-card-action" type="button" data-open-project="${app.id}">Details</button>${app.capability === "install" ? `<button class="button compact primary" type="button" data-go-screen="install">Review install</button>` : app.capability === "connect" ? `<button class="button compact connect-action" type="button" data-open-connect="${app.id}">Connect</button>` : `<button class="button compact ghost explore-action" type="button" data-open-project="${app.id}">Explore ${icon("external")}</button>`}</footer>
      </article>`).join("")}
    </section>
    <section id="catalog-no-results" class="panel inline-no-results" hidden><span>${icon("search")}</span><div><strong>No matching projects</strong><small>Try another name or tag, or clear search and filters to browse all ${summary.total.toLocaleString()} projects.</small></div><button class="text-button" type="button" data-clear-discover>Clear everything ${icon("arrow")}</button></section>
    ${pagination}
    ${projectDrawer()}`;
}

const formatUnixDate = (timestamp) => new Intl.DateTimeFormat("en", { month: "short", day: "numeric", year: "numeric" }).format(new Date(timestamp * 1000));

function appStatus(app) {
  if (prototype.condition === "busy" && app.id === prototype.selectedApp && app.runtime.kind === "compose") return "starting";
  return app.status;
}

function appStatusDetails(app, status) {
  const managed = app.runtime.kind === "compose";
  if (status === "running") return ["Running locally", "The Compose project is running. Reachability is checked separately when the app is opened."];
  if (status === "starting") return ["Starting managed app", "The operation is active. Selection and controls remain in place while Compose starts the project."];
  if (status === "stopped") return ["Stopped by Local Store", "The Compose project is not running. Its managed data remains on disk."];
  if (status === "error") return ["Runtime needs attention", app.status_error || "The Compose project returned an error."];
  if (status === "unreachable") return ["Address did not answer", "This linked server may be offline or its saved address may have changed. Local Store does not manage it."];
  if (status === "ready") return ["Address is ready", "The latest bounded check reached this linked app. Local Store does not manage its server."];
  return [managed ? "Status not checked" : "Address not checked", "Refresh to request a current bounded status check."];
}

function appOverview(app, status) {
  const managed = app.runtime.kind === "compose";
  const [title, detail] = appStatusDetails(app, status);
  return `<div class="detail-status ${statusClass(status)}"><div><span class="status-dot ${statusClass(status)}"></span><strong>${escapeHtml(title)}</strong></div><p>${escapeHtml(detail)}</p></div>
    <div class="facts app-facts">
      <div class="fact wide"><span>Address</span><code>${escapeHtml(app.launch_url)}</code></div>
      <div class="fact"><span>Runs as</span><code>${managed ? "Managed · Compose" : "Linked · External"}</code></div>
      <div class="fact"><span>From catalog</span><code>${escapeHtml(app.catalog_id)}</code></div>
      <div class="fact"><span>Added</span><code>${formatUnixDate(app.created_at_unix)}</code></div>
      <div class="fact"><span>Last changed</span><code>${formatUnixDate(app.updated_at_unix)}</code></div>
    </div>
    ${managed ? `<details class="path-disclosure"><summary>Managed project paths</summary><div><span>Project folder</span><code>${escapeHtml(app.runtime.project_dir)}</code></div><div><span>Compose file</span><code>${escapeHtml(app.runtime.compose_file)}</code></div></details>` : `<div class="linked-boundary">${icon("external")}<p><strong>Server boundary</strong> Start, stop, logs, updates, and data management stay with the external server.</p></div>`}`;
}

function appLogs(app) {
  if (app.runtime.kind !== "compose") return `<section class="logs-unavailable"><span class="empty-symbol">${icon("external")}</span><h3>Logs stay with the linked server</h3><p>Local Store only stores this address. Open the server's own administration surface to inspect its logs.</p></section>`;
  const state = prototype.appSubstate;
  if (state === "loading") return `<div class="logs-loading" role="status" aria-busy="true" aria-label="Loading recent logs"><div class="log-skeleton"></div><div class="log-skeleton short"></div><div class="log-skeleton"></div><div class="log-skeleton medium"></div></div>`;
  if (state === "empty") return `<section class="logs-unavailable"><span class="empty-symbol">${icon("activity")}</span><h3>No recent output</h3><p>The bounded log snapshot returned no lines. This does not mean the app has never produced logs.</p><button class="button ghost" type="button" data-app-action="refresh-logs">Refresh logs</button></section>`;
  if (state === "failure") return `<section class="logs-error"><span>${icon("alert")}</span><div><strong>Recent logs could not be read</strong><p>The selected app and its last known status are preserved. Retry after checking Docker.</p><code>app_logs · compose process failed</code></div><button class="button ghost" type="button" data-app-action="refresh-logs">Retry</button></section>`;
  const lines = fixtures.contract.logs[app.id] || [];
  return `<div class="logs-toolbar"><div><strong>Recent Compose output</strong><small>Bounded snapshot · newest last</small></div><div><button class="button ghost compact" type="button" data-app-action="refresh-logs">Refresh</button><button class="button ghost compact" type="button" data-app-action="copy-logs">Copy</button></div></div><pre class="log-view" tabindex="0" aria-label="Recent logs for ${escapeHtml(app.display_name)}">${lines.map((line) => `<code>${escapeHtml(line)}</code>`).join("\n")}</pre><p class="logs-footnote">Logs are fetched on request and are not retained as Local Store activity history.</p>`;
}

function appManage(app, status) {
  const managed = app.runtime.kind === "compose";
  if (!managed) return `<div class="manage-stack"><section class="manage-card"><span class="manage-icon">${icon("external")}</span><div><h3>Create a desktop shortcut</h3><p>Open this saved address directly from Windows without changing the remote server.</p></div><button class="button ghost" type="button" data-app-action="shortcut">Create shortcut</button></section><section class="manage-card danger-zone"><span class="manage-icon">${icon("close")}</span><div><h3>Remove linked app</h3><p>Removes only this Local Store record. The external server and its data are untouched.</p></div><button class="button danger" type="button" data-open-app-dialog="remove">Remove link</button></section></div>`;
  return `<div class="manage-stack"><section class="manage-card"><span class="manage-icon">${icon(status === "running" ? "close" : "arrow")}</span><div><h3>${status === "running" ? "Stop managed app" : "Start managed app"}</h3><p>${status === "running" ? "Stops the Compose project without removing its registration or data." : "Starts the existing Compose project using its saved configuration."}</p></div><button class="button ghost" type="button" data-app-action="${status === "running" ? "stop" : "start"}">${status === "running" ? "Stop app" : "Start app"}</button></section><section class="manage-card safe-uninstall"><span class="manage-icon">${icon("docker")}</span><div><h3>Uninstall and keep data</h3><p>Removes the Compose project and Local Store record. Managed data stays in its folder for recovery or manual reuse.</p></div><button class="button ghost" type="button" data-open-app-dialog="uninstall">Review uninstall</button></section><section class="manage-card danger-zone"><span class="manage-icon">${icon("alert")}</span><div><h3>Delete app and managed data</h3><p>Permanently removes the Compose project, registration, and the managed data folder.</p></div><button class="button danger" type="button" data-open-app-dialog="delete-data">Delete app &amp; data</button></section></div>`;
}

function appDialog(app) {
  if (!prototype.appDialog) return "";
  const mode = prototype.appDialog;
  const deleteData = mode === "delete-data";
  const linkedRemove = mode === "remove";
  const title = deleteData ? `Delete ${app.display_name} and its data?` : linkedRemove ? `Remove ${app.display_name}?` : `Uninstall ${app.display_name}?`;
  const body = deleteData ? "This permanently deletes the managed data folder. It cannot be recovered by Local Store." : linkedRemove ? "Only the saved link and shortcut record are removed. The external server is untouched." : "The Compose project and Local Store record are removed. The managed data folder is preserved by default.";
  return `<div class="app-dialog-scrim" data-close-app-dialog></div><section class="app-dialog ${deleteData ? "irreversible" : ""}" role="dialog" aria-modal="true" aria-labelledby="app-dialog-title"><div class="dialog-mark">${icon(deleteData ? "alert" : linkedRemove ? "external" : "docker")}</div><p class="eyebrow">${deleteData ? "Irreversible action" : linkedRemove ? "Linked app" : "Safe default · data kept"}</p><h2 id="app-dialog-title">${escapeHtml(title)}</h2><p>${escapeHtml(body)}</p>${deleteData ? `<label class="field-label" for="delete-confirmation"><span>Type <strong>${escapeHtml(app.display_name)}</strong> to confirm</span><input class="input" id="delete-confirmation" autocomplete="off" data-delete-confirmation></label><div class="delete-scope"><span>Compose project</span><strong>Deleted</strong><span>Local Store record</span><strong>Deleted</strong><span>Managed data</span><strong>Deleted permanently</strong></div>` : !linkedRemove ? `<div class="preserve-summary"><span>${icon("check")}</span><div><strong>Managed data remains on disk</strong><small>${escapeHtml(app.runtime.project_dir)}</small></div></div>` : ""}<div class="dialog-actions"><button class="button ghost" type="button" data-close-app-dialog>Cancel</button><button class="button ${deleteData || linkedRemove ? "danger" : "primary"}" type="button" data-confirm-app-action="${mode}" ${deleteData ? "disabled" : ""}>${deleteData ? "Delete permanently" : linkedRemove ? "Remove linked app" : "Uninstall, keep data"}</button></div></section>`;
}

function appDetail(app) {
  const status = appStatus(app);
  const managed = app.runtime.kind === "compose";
  const primary = status === "stopped" ? ["start", "Start app", "arrow"] : status === "error" ? ["show-logs", "View logs", "activity"] : status === "unreachable" ? ["check-address", "Check address", "activity"] : ["open", "Open app", "external"];
  return `<div class="detail-head"><div class="detail-identity">${appIcon(app, true)}<div><h2>${escapeHtml(app.display_name)}</h2><div><span class="type-pill ${managed ? "managed" : "linked"}">${appType(app)}</span><span class="badge ${statusClass(status)}">${statusLabel(status)}</span></div></div></div><div class="detail-actions">${managed && status === "running" ? `<button class="button ghost" type="button" data-app-action="stop">Stop</button>` : ""}<button class="button primary" type="button" data-app-action="${primary[0]}" ${status === "starting" ? "disabled" : ""}>${primary[1]} ${icon(primary[2])}</button></div></div>
    <div class="detail-tabs" role="tablist" aria-label="${escapeHtml(app.display_name)} sections">${[["overview", "Overview"], ["logs", "Logs"], ["manage", "Manage"]].map(([section, label]) => `<button class="detail-tab" role="tab" aria-selected="${prototype.appSection === section}" tabindex="${prototype.appSection === section ? "0" : "-1"}" type="button" data-app-section="${section}" ${section === "logs" && !managed ? "aria-disabled=\"true\"" : ""}>${label}</button>`).join("")}</div>
    <div class="detail-body" role="tabpanel">${prototype.appSection === "logs" ? appLogs(app) : prototype.appSection === "manage" ? appManage(app, status) : appOverview(app, status)}</div>`;
}

function myAppsLoading() {
  return `${header()}<section class="panel apps-workspace apps-loading" aria-busy="true"><div class="apps-list"><div class="list-head"><strong>Saved apps</strong><span>Loading</span></div>${Array.from({ length: 5 }, (_, index) => `<div class="app-row-skeleton"><i></i><span style="--skeleton-width:${74 - index * 7}%"></span></div>`).join("")}</div><div class="apps-detail"><div class="detail-head-skeleton"></div><div class="detail-tabs-skeleton"></div><div class="detail-body-skeleton"></div></div></section>`;
}

function myApps() {
  const apps = fixtures.contract.apps;
  const selectedBase = apps.find((app) => app.id === prototype.selectedApp) || apps[0];
  const selected = prototype.condition === "busy" && selectedBase.runtime.kind === "compose" ? { ...selectedBase, status: "starting" } : selectedBase;
  const notice = prototype.savedLinkName ? `<div class="condition-banner success" role="status">${icon("check")}<span><strong>${escapeHtml(prototype.savedLinkName)} saved as a linked app.</strong> Production selects it at the top of Saved apps; this prototype has no row to add.</span></div>` : prototype.condition === "failure" ? `<div class="condition-banner failure" role="alert">${icon("alert")}<span><strong>Refresh failed.</strong> Last known app records and selection are still shown.</span><button class="text-button" type="button" data-app-action="refresh-list">Retry</button></div>` : prototype.condition === "busy" ? `<div class="condition-banner busy" role="status">${icon("sliders")}<span><strong>${escapeHtml(selected.display_name)} is starting.</strong> Other saved apps remain available.</span></div>` : prototype.condition === "success" ? `<div class="condition-banner success" role="status">${icon("check")}<span><strong>Action completed.</strong> ${escapeHtml(selected.display_name)} remains selected.</span></div>` : "";
  return `${header(`<button class="button ghost" type="button" data-open-connect="">${icon("plus")} Connect app</button><button class="button ghost" type="button" data-go-screen="discover">Discover ${icon("arrow")}</button>`)}${notice}
    <section class="panel apps-workspace"><div class="apps-list"><div class="list-head"><strong>Saved apps</strong><span>${apps.length} total</span></div><label class="search-field"><span class="field-label" hidden>Filter saved apps</span>${icon("search")}<input class="input" id="apps-search" type="search" value="${escapeHtml(prototype.appQuery)}" placeholder="Filter saved apps" autocomplete="off"></label>
      <div role="listbox" aria-label="Saved apps" data-app-list>${apps.map((app) => { const status = appStatus(app); return `<button class="app-row ${app.runtime.kind === "compose" ? "managed" : "linked"}" type="button" role="option" aria-selected="${app.id === selected.id}" data-app-id="${app.id}" data-search-value="${escapeHtml(`${app.display_name} ${appType(app)} ${status}`.toLowerCase())}">${appIcon(app)}<span class="app-row-copy"><strong title="${escapeHtml(app.display_name)}">${escapeHtml(app.display_name)}</strong><small>${appType(app)}</small></span><span class="row-status ${statusClass(status)}">${statusLabel(status)}</span></button>`; }).join("")}</div>
      <div class="apps-filter-empty" hidden><strong>No saved apps match</strong><small>Clear the filter to restore all ${apps.length} apps.</small><button class="text-button" type="button" data-clear-app-filter>Clear filter</button></div></div>
      <div class="apps-detail">${appDetail(selected)}</div></section>${appDialog(selected)}`;
}

function activity() {
  if (prototype.condition === "loading") return `${header()}<div class="activity-loading" aria-busy="true"><div class="skeleton activity-toolbar-skeleton"></div>${Array.from({ length: 5 }, () => `<div class="skeleton activity-row-skeleton"></div>`).join("")}</div>`;
  const events = (prototype.condition === "empty" ? [] : fixtures.concept.events).filter((event) => {
    const query = prototype.activityQuery.trim().toLowerCase();
    const matchesText = !query || `${event.app} ${event.kind} ${event.state} ${event.detail} ${event.error || ""}`.toLowerCase().includes(query);
    const matchesApp = prototype.activityApp === "all" || event.app_id === prototype.activityApp;
    const matchesFilter = prototype.activityFilter === "all"
      || (prototype.activityFilter === "install" && event.kind === "Install")
      || (prototype.activityFilter === "lifecycle" && ["Start", "Stop", "Uninstall"].includes(event.kind))
      || (prototype.activityFilter === "failed" && event.state === "Failed");
    return matchesText && matchesApp && matchesFilter;
  });
  const groups = [...new Set(events.map((event) => event.group))];
  const live = prototype.condition === "busy" ? `<section class="panel live-operation" aria-live="polite"><span class="busy-orbit" aria-hidden="true"></span>${appIcon(fixtures.contract.apps[0])}<div><span class="lab-tag available-tag">Live event</span><strong>Memos · Install</strong><small>Waiting for the app to respond</small></div><span class="badge warning">In progress</span></section>` : `<div class="session-idle"><span class="status-dot success"></span><span><strong>No operation is running</strong><small>New launcher events will appear here immediately.</small></span><span class="truth-label">Current session</span></div>`;
  const failure = prototype.condition === "failure" ? `<div class="condition-banner failure" role="alert">${icon("alert")}<span><strong>History could not be refreshed.</strong> Last loaded concept rows remain visible; live operations still use their in-context feedback.</span><button class="text-button" type="button" data-activity-action="retry">Retry</button></div>` : "";
  const emptyCopy = prototype.condition === "empty" ? ["No operations yet", "Install, start, stop, uninstall, or open an app and its result will appear here once history is persisted.", "Go to My Apps"] : ["No operations match", "Change the search, app, or type filters. No saved app state has changed.", "Clear filters"];
  return `${header()}${failure}${live}<div class="concept-banner activity-contract">${icon("alert")}<div><span class="lab-tag">Backend work</span><p>${escapeHtml(fixtures.concept.activityNotice)}</p></div></div><section class="panel activity-surface"><div class="activity-toolbar"><label class="search-field"><span class="field-label" hidden>Search activity</span>${icon("search")}<input class="input" id="activity-search" type="search" value="${escapeHtml(prototype.activityQuery)}" placeholder="Search operations" autocomplete="off"></label><div class="activity-filter-chips" aria-label="Activity type">${[["all", "All"], ["install", "Installs"], ["lifecycle", "Lifecycle"], ["failed", "Failures"]].map(([id, label]) => `<button class="filter-chip" type="button" aria-pressed="${prototype.activityFilter === id}" data-activity-filter="${id}">${label}</button>`).join("")}</div><label class="activity-app-filter"><span>App</span><span class="select-wrap"><select data-activity-app><option value="all">All apps</option>${fixtures.contract.apps.map((app) => `<option value="${app.id}" ${prototype.activityApp === app.id ? "selected" : ""}>${escapeHtml(app.display_name)}</option>`).join("")}</select>${icon("chevron")}</span></label></div><div class="activity-list-head"><strong>Operation history</strong><span><b id="activity-count">${events.length}</b> concept events</span></div><div class="activity-timeline" aria-label="Prototype operation history">${events.length ? groups.map((group) => `<section class="activity-group"><h2>${escapeHtml(group)}</h2>${events.filter((event) => event.group === group).map((event) => { const app = fixtures.contract.apps.find((item) => item.id === event.app_id); const expanded = prototype.activityExpanded === event.operation_id; return `<article class="activity-event ${expanded ? "expanded" : ""}"><button type="button" aria-expanded="${expanded}" data-activity-event="${event.operation_id}">${app ? appIcon(app) : `<span class="activity-glyph">${icon("activity")}</span>`}<span class="activity-event-copy"><strong>${escapeHtml(event.app)} · ${escapeHtml(event.kind)}</strong><small>${escapeHtml(event.detail)}</small></span><time>${escapeHtml(event.time.replace(`${group} · `, ""))}</time><span class="badge ${statusClass(event.state.toLowerCase())}">${escapeHtml(event.state)}</span>${icon("chevron")}</button>${expanded ? `<div class="activity-event-detail"><div><span>Operation ID</span><code>${escapeHtml(event.operation_id)}</code></div><div><span>App ID</span><code>${escapeHtml(event.app_id)}</code></div><div><span>Last stage</span><code>${escapeHtml(event.stage || "Not applicable")}</code></div><div><span>Error code</span><code class="${event.error ? "danger-text" : ""}">${escapeHtml(event.error || "None")}</code></div></div>` : ""}</article>`; }).join("")}</section>`).join("") : `<div class="activity-empty"><span class="empty-symbol">${icon("activity")}</span><h2>${emptyCopy[0]}</h2><p>${emptyCopy[1]}</p><button class="button ghost" type="button" ${prototype.condition === "empty" ? `data-go-screen="my-apps"` : `data-activity-action="clear"`}>${emptyCopy[2]}</button></div>`}</div></section>`;
}

function settings() {
  if (prototype.condition === "loading") return `${header()}<div class="settings-loading" aria-busy="true"><div class="skeleton settings-hero-skeleton"></div><div class="settings-loading-grid"><div class="skeleton"></div><div class="skeleton"></div></div></div>`;
  const busy = prototype.condition === "busy";
  const failed = prototype.condition === "failure";
  const checks = fixtures.contract.doctor.checks.map((check, index) => failed && index === 0 ? { ...check, ok: false, detail: "Docker engine is not responding" } : check);
  const recoveryEmpty = prototype.condition === "empty";
  return `${header(`<button class="button primary" type="button" data-settings-action="doctor" ${busy ? "disabled" : ""}>${busy ? "Checking…" : "Run Docker check"} ${busy ? "" : icon("arrow")}</button>`)}<section class="panel system-status ${failed ? "failed" : ""}"><div class="system-summary"><span class="system-mark">${busy ? `<span class="busy-orbit" aria-hidden="true"></span>` : icon(failed ? "alert" : "check")}</span><div><p class="eyebrow">System readiness</p><h2>${busy ? "Checking Docker" : failed ? "Docker needs attention" : "Ready for managed apps"}</h2><p>${busy ? "Running the two bounded prerequisite checks." : failed ? "Linked apps still work. Reviewed installs and managed lifecycle actions wait for Docker." : "Docker Engine and Compose passed the checks required by reviewed installs."}</p></div><span class="truth-label">Available now</span></div><div class="doctor-results">${checks.map((check, index) => `<div><span class="status-dot ${busy ? "warning" : check.ok ? "success" : "danger"}"></span><span><strong>${escapeHtml(check.label)}</strong><small>${busy ? "Checking local command" : check.ok ? "Ready" : index === 0 ? "Start Docker Desktop, then run the check again." : "Waiting for Docker Engine"}</small></span><code>${busy ? "…" : escapeHtml(check.detail)}</code></div>`).join("")}</div></section><div class="settings-grid settings-v2"><div class="settings-stack"><section class="panel settings-card"><div class="section-heading"><div><p class="eyebrow">Safety</p><h2>Diagnostics &amp; Recovery</h2></div><span>Read-only scan</span></div>${recoveryEmpty ? `<div class="settings-empty-row">${icon("check")}<div><strong>No retained setups found</strong><small>Supported managed app folders are clear.</small></div></div>` : `<div class="settings-row recovery-settings-row">${appIcon(fixtures.contract.recipe)}<div><strong>Retained Memos setup</strong><small>Matching Docker ownership snapshot verified</small></div><span class="badge warning">Review</span><button class="text-button" type="button" data-go-screen="recovery">Open ${icon("arrow")}</button></div>`}<div class="settings-card-footer"><p>Scanning reads managed setup paths and Docker labels. It never starts, stops, or deletes anything.</p><button class="button ghost" type="button" data-settings-action="scan">Scan again</button></div></section><section class="panel settings-card"><div class="section-heading"><div><p class="eyebrow">Workspace</p><h2>Introduction</h2></div><span class="lab-tag">Concept</span></div><div class="settings-row">${icon("overview")}<div><strong>Run the introduction again</strong><small>Revisit the welcome and starter choices without changing saved apps.</small></div><button class="text-button" type="button" data-first-action="restart">Start ${icon("arrow")}</button></div></section></div><aside class="settings-stack"><section class="panel settings-card"><div class="section-heading"><div><p class="eyebrow">Offline bundle</p><h2>Catalog</h2></div><span class="badge success">Local</span></div><div class="settings-metrics"><div><strong>1,672</strong><span>projects</span></div><div><strong>3</strong><span>reviewed installs</span></div><div><strong>100%</strong><span>local icons</span></div></div><p class="settings-footnote">Catalog data and icons ship with this build. Browsing does not require a network connection.</p></section><section class="panel settings-card"><div class="section-heading"><div><p class="eyebrow">About</p><h2>Build information</h2></div><span>Desktop</span></div><div class="build-fact"><span>Version</span><code>0.5.0-1</code></div><div class="build-fact"><span>Platform</span><code>Windows · x86_64</code></div><div class="build-fact"><span>Catalog source</span><code>Bundled snapshot</code></div><div class="build-fact"><span>Prototype</span><code>V2 · Phase 12</code></div></section></aside></div>`;
}

function installSetupFields() {
  if (!prototype.installSetupStudy) return `<div class="setup-none">${icon("check")}<div><strong>No setup answers required</strong><p>This reviewed ${escapeHtml(activeRecipe().display_name)} recipe generates no credentials and asks for no secrets.</p></div></div>`;
  const study = fixtures.concept.installSetupVariant;
  return `<section class="setup-study"><div class="setup-study-head"><span class="lab-tag">Component study</span><p>${escapeHtml(study.notice)}</p></div>${study.fields.map((field) => `<label class="field-label" for="setup-${field.key}">${escapeHtml(field.label)}${field.required ? " · Required" : " · Optional"}<input class="input" id="setup-${field.key}" data-setup-key="${field.key}" type="${field.control}" value="${field.sensitive ? "" : escapeHtml(prototype.installAnswers[field.key] ?? field.default ?? "")}" placeholder="${field.sensitive ? "Secret value is masked" : "Optional"}" autocomplete="off"></label>`).join("")}<div class="generated-secret-note">${icon("check")}<span><strong>${study.generated_credential_count} credential generated during install</strong> Generated values are never shown. Sensitive fields remain masked and are not retained by this prototype.</span></div></section>`;
}

function installReview() {
  const recipe = activeRecipe();
  const facts = activeRecipeFacts(recipe);
  return `<div class="install-layout review-layout">
    <section class="panel install-card"><div class="recipe-head">${appIcon(recipe, true)}<div><span class="badge success">Reviewed install</span><h2>${escapeHtml(recipe.display_name)}</h2><p>${escapeHtml(recipe.description)}</p></div></div>
      <div class="install-not-run">${icon("check")}<div><strong>Nothing has run yet</strong><p>Changing these fields only updates the review. Docker is contacted after you press Install.</p></div></div>
      <section class="install-section" aria-labelledby="configuration-title"><div class="install-section-head"><div><p class="eyebrow">Configuration</p><h3 id="configuration-title">Choose the local address</h3></div><span class="truth-label">Editable now</span></div>
        <label class="port-editor field-label" for="install-port"><span>Published port</span><span class="port-control"><code>127.0.0.1:</code><input class="input" id="install-port" type="number" min="1024" max="65535" inputmode="numeric" value="${prototype.installPort}" aria-describedby="port-help port-error"><code>→ ${recipe.container_port}</code></span></label>
        <p id="port-help" class="field-help">Only the host-side port changes. The image’s internal port stays fixed.</p><p id="port-error" class="field-error" ${prototype.installPortError ? "" : "hidden"}>${escapeHtml(prototype.installPortError)}</p>
        ${installSetupFields()}
      </section>
      <section class="install-section" aria-labelledby="changes-title"><div class="install-section-head"><div><p class="eyebrow">Machine changes</p><h3 id="changes-title">What Local Store will create</h3></div><span>${facts.managedItems} managed items</span></div>
        <div class="change-list"><div>${icon("docker")}<span><strong>One container</strong><code>${escapeHtml(facts.containerName)}</code></span></div><div>${icon("activity")}<span><strong>One loopback port</strong><code>127.0.0.1:${prototype.installPort} → ${recipe.container_port}</code></span></div><div>${icon("grid")}<span><strong>One managed data folder</strong><code>${escapeHtml(recipe.data_storage)}</code></span></div></div>
      </section>
      <details class="install-evidence" ${prototype.installAdvanced ? "open" : ""}><summary><span>Recipe evidence &amp; compatibility</span><small>Image audit, platforms, health, restart, and rollback</small></summary><div class="evidence-grid"><div><span>Exact image</span><code>${escapeHtml(recipe.image)}</code></div><div><span>Version</span><code>${escapeHtml(recipe.version)}</code></div><div><span>Health address</span><code>http://localhost:${prototype.installPort}</code></div><div><span>Restart policy</span><code>${escapeHtml(facts.restartPolicy)}</code></div><div><span>Verified</span><code>${escapeHtml(recipe.verified_at)}</code></div><div><span>License</span><code>${escapeHtml(recipe.license)}</code></div><div class="wide"><span>Container platforms</span><code>${escapeHtml(recipe.requirements.container_platforms.join(" · "))}</code></div><div class="wide"><span>Image digest</span><code>${escapeHtml(recipe.requirements.image_audit.index_digest)}</code></div></div><ul class="risk-notes">${recipe.risk_notes.map((note) => `<li>${escapeHtml(note.replaceAll(String(recipe.host_port), String(prototype.installPort)))}</li>`).join("")}</ul><p class="rollback-note"><strong>Rollback:</strong> if startup or health checks fail, Local Store stops the incomplete Compose project and restores or removes files it created. A cleanup failure is surfaced for manual review.</p></details>
    </section>
    <aside class="panel install-summary"><p class="eyebrow">Ready to install</p><h3>One reviewed app, locally managed.</h3><p>${escapeHtml(recipe.display_name)} will be reachable only from this computer at <code>localhost:${prototype.installPort}</code>.</p><div class="doctor-compact"><div><span class="status-dot success"></span><span>Docker Engine</span><code>${escapeHtml(fixtures.contract.doctor.checks[0].detail)}</code></div><div><span class="status-dot success"></span><span>Docker Compose</span><code>${escapeHtml(fixtures.contract.doctor.checks[1].detail)}</code></div></div><button class="button primary" type="button" data-install-action="start">Install ${escapeHtml(recipe.display_name)} ${icon("arrow")}</button><button class="button ghost" type="button" data-install-action="leave">Cancel review</button><small class="summary-note">Install starts with a fresh system and port check. No percentage is estimated.</small></aside>
  </div>`;
}

function installProgress() {
  const recipe = activeRecipe();
  const rollingBack = prototype.installStage === "rolling_back";
  const normalStages = fixtures.contract.installStages;
  const stages = rollingBack ? [...normalStages.slice(0, 5), { id: "rolling_back", label: "Cleaning up incomplete setup", detail: "Stops owned containers and restores or removes files created by this attempt." }] : normalStages;
  const activeIndex = stages.findIndex((stage) => stage.id === prototype.installStage);
  const cancellable = !["saving_app", "rolling_back"].includes(prototype.installStage);
  return `<div class="install-progress-layout"><section class="panel progress-card" aria-busy="true"><div class="progress-identity">${appIcon(recipe, true)}<div><p class="eyebrow">Install in progress</p><h2>${rollingBack ? `Cleaning up ${escapeHtml(recipe.display_name)}` : `Installing ${escapeHtml(recipe.display_name)}`}</h2><p>${rollingBack ? "The original failure is retained while cleanup finishes." : "Keep Local Store open. Completed steps remain visible."}</p></div><span class="busy-orbit" aria-hidden="true"></span></div>
    <ol class="stage-list">${stages.map((stage, index) => { const state = rollingBack && index === 4 ? "failed" : index < activeIndex ? "complete" : index === activeIndex ? (rollingBack ? "rollback" : "active") : "pending"; return `<li class="stage ${state}" ${state === "active" || state === "rollback" ? "aria-current=\"step\"" : ""}><span class="stage-mark">${state === "complete" ? icon("check") : state === "failed" ? icon("alert") : index + 1}</span><div><strong>${escapeHtml(stage.label)}</strong><p>${escapeHtml(stage.detail)}</p></div><small>${state === "complete" ? "Done" : state === "active" ? "In progress" : state === "rollback" ? "Required cleanup" : state === "failed" ? "Failed" : "Waiting"}</small></li>`; }).join("")}</ol>
    ${!cancellable ? `<div class="commit-boundary ${rollingBack ? "danger" : ""}">${icon(rollingBack ? "alert" : "check")}<p><strong>${rollingBack ? "Cleanup cannot be cancelled" : "Final registration cannot be cancelled"}</strong>${rollingBack ? " Interrupting cleanup could leave the machine in an unknown state." : " The container is healthy; Local Store is committing it to My Apps so it never becomes invisible."}</p></div>` : ""}</section>
    <aside class="panel install-summary progress-summary"><p class="eyebrow">Current operation</p><h3>${escapeHtml(stages[activeIndex]?.label || "Installing")}</h3><p>No download percentage is shown because the backend reports stages, not byte progress.</p><div class="summary-facts"><span>Address</span><code>localhost:${prototype.installPort}</code><span>Data</span><code>Managed folder</code><span>Operation</span><code>install · ${escapeHtml(recipe.id)}</code></div>${cancellable ? `<button class="button ghost" type="button" data-install-action="cancel">Cancel installation</button><small class="summary-note">Cancel requests cleanup. ${escapeHtml(recipe.display_name)} will not be added to My Apps.</small>` : `<div class="no-cancel-note">${icon("check")}<span>Cancel is unavailable at this stage.</span></div>`}</aside></div>`;
}

function installSuccess() {
  const recipe = activeRecipe();
  const next = prototype.installOrigin === "first-run" ? `<button class="button ghost" type="button" data-first-action="finish-install">Finish setup</button>` : `<button class="button ghost" type="button" data-install-action="my-apps">View in My Apps</button>`;
  return `<section class="panel install-result success-result"><span class="result-mark">${icon("check")}</span><p class="eyebrow">Installed successfully</p><h2>${escapeHtml(recipe.display_name)} is ready at localhost:${prototype.installPort}</h2><p>The reviewed container is healthy, its managed data is in place, and the app is registered in My Apps.</p><div class="result-facts"><div><span>Image</span><code>${escapeHtml(recipe.image)}</code></div><div><span>Data</span><code>Preserved by default on uninstall</code></div></div><div class="result-actions"><button class="button primary" type="button" data-install-action="open">Open ${escapeHtml(recipe.display_name)} ${icon("external")}</button>${next}</div></section>`;
}

function installCancelled() {
  return `<section class="panel install-result cancelled-result"><span class="result-mark">${icon("close")}</span><p class="eyebrow">Installation cancelled</p><h2>Nothing was added to My Apps.</h2><p>Local Store requested cleanup of the incomplete setup. Your chosen port and valid setup values remain available for another review.</p><div class="result-actions"><button class="button primary" type="button" data-install-action="review">Return to review ${icon("arrow")}</button><button class="button ghost" type="button" data-install-action="leave">Leave install</button></div></section>`;
}

function installFailure() {
  const recipe = activeRecipe();
  const error = fixtures.contract.installErrors[prototype.installFailure] || fixtures.contract.installErrors.invalid_input;
  const unsafe = prototype.installFailure === "rollback_failed";
  const portFailure = prototype.installFailure === "port_in_use";
  const status = unsafe ? "Machine state needs review" : prototype.installFailure === "prerequisite_unavailable" || portFailure ? "No files or containers were created" : "Incomplete setup rolled back";
  if (portFailure && prototype.installFailedPort == null) prototype.installFailedPort = prototype.installPort;
  const failedPort = prototype.installFailedPort;
  const action = error.recovery === "doctor" ? `<button class="button primary" type="button" data-go-screen="settings">Open system check ${icon("arrow")}</button><button class="button ghost" type="button" data-install-action="review">Back to review</button>` : error.recovery === "recovery" ? `<button class="button primary" type="button" data-go-screen="recovery">Review retained setup ${icon("arrow")}</button><button class="button ghost" type="button" data-go-screen="discover">Leave install</button>` : `<button class="button primary" type="button" data-install-action="review">${portFailure ? "Review new port" : "Review and retry"} ${icon("arrow")}</button><button class="button ghost" type="button" data-go-screen="discover">Back to Discover</button>`;
  const title = portFailure ? `Port ${prototype.installPort} is already in use` : error.title.replaceAll("Memos", recipe.display_name);
  return `<section class="panel install-result failure-result ${unsafe ? "unsafe" : ""}"><span class="result-mark">${icon("alert")}</span><p class="eyebrow">${unsafe ? "Manual review required" : "Install did not complete"}</p><h2>${escapeHtml(title)}</h2><p>${escapeHtml(error.detail)}</p><div class="failure-status">${icon(unsafe ? "alert" : "check")}<div><strong>${escapeHtml(status)}</strong><p>${unsafe ? "Retry is deliberately unavailable until retained resources are inspected." : "Your selected configuration remains available."}</p></div></div>${portFailure ? `<label class="field-label failure-port" for="failure-port">Try another published port<input class="input" id="failure-port" type="number" min="1024" max="65535" value="${prototype.installPort}" data-install-port aria-describedby="failure-port-error" ${prototype.installPort === failedPort ? 'aria-invalid="true"' : ""}></label><p id="failure-port-error" class="field-error" ${prototype.installPort === failedPort ? "" : "hidden"}>Port ${failedPort} was refused. Choose a different port.</p>` : ""}<code class="error-code">${escapeHtml(prototype.installFailure)} · install · ${escapeHtml(recipe.id)}</code><div class="result-actions">${action}</div></section>`;
}

function installLoading() {
  return `<div class="install-layout" aria-busy="true"><section class="panel install-card"><div class="install-review-skeleton identity"></div><div class="install-review-skeleton notice"></div><div class="install-review-skeleton body"></div></section><aside class="panel install-summary"><div class="install-review-skeleton side"></div></aside></div>`;
}

function installUnavailable() {
  const recipe = activeRecipe();
  return `<section class="panel install-result failure-result"><span class="result-mark">${icon("alert")}</span><p class="eyebrow">Recipe unavailable</p><h2>${escapeHtml(recipe.display_name)} could not be reviewed.</h2><p>No installation can begin without a complete local recipe. Saved apps and Docker remain untouched.</p><div class="result-actions"><button class="button primary" type="button" data-install-action="reload-recipe">Retry local recipe ${icon("arrow")}</button><button class="button ghost" type="button" data-install-action="leave">Leave install</button></div></section>`;
}

function install() {
  const recipe = activeRecipe();
  const state = prototype.condition === "busy" ? "installing" : prototype.condition === "success" ? "success" : prototype.condition === "failure" ? "failure" : prototype.condition === "loading" ? "loading" : prototype.condition === "empty" ? "unavailable" : prototype.installOutcome === "cancelled" ? "cancelled" : "review";
  const headings = {
    review: ["Review install", "Review exactly what Local Store will create before anything runs."],
    installing: [`Installing ${recipe.display_name}`, "Follow each backend stage without leaving the task."],
    success: [`${recipe.display_name} is ready`, "Open the app now or continue to its operational details."],
    failure: ["Install needs attention", "The result, cleanup state, and safest next action stay together."],
    cancelled: ["Install cancelled", "Your valid configuration is retained for another review."],
    loading: ["Loading install review", "Reading the local reviewed recipe. Nothing can run yet."],
    unavailable: ["Install unavailable", "A complete reviewed recipe is required before machine changes are allowed."],
  };
  const [label, description] = headings[state];
  const content = state === "installing" ? installProgress() : state === "success" ? installSuccess() : state === "failure" ? installFailure() : state === "cancelled" ? installCancelled() : state === "loading" ? installLoading() : state === "unavailable" ? installUnavailable() : installReview();
  const backLabel = prototype.installOrigin === "first-run" ? "← Back to starter choices" : `← Back to ${escapeHtml(recipe.display_name)} in Discover`;
  const back = state === "review" || state === "loading" || state === "unavailable" ? `<button class="text-button task-back" type="button" data-install-action="leave">${backLabel}</button>` : "";
  return `<div class="focused-wrap install-flow">${back}${header("", { eyebrow: `Focused task · ${recipe.display_name}`, label, description })}${content}</div>`;
}

function recovery() {
  const candidate = fixtures.contract.recovery;
  const mode = prototype.condition === "loading" ? "loading" : prototype.condition === "empty" ? "empty" : prototype.condition === "failure" ? "mismatch" : prototype.condition === "success" ? "success-keep" : prototype.recoveryMode;
  const frame = (content, back = true) => `<div class="focused-wrap recovery-flow">${back ? `<button class="text-button task-back" type="button" data-go-screen="settings">← Back to Diagnostics &amp; Recovery</button>` : ""}${header()}${content}</div>`;
  if (mode === "loading") return frame(`<section class="panel recovery-loading" aria-busy="true"><span class="busy-orbit" aria-hidden="true"></span><p class="eyebrow">Read-only scan</p><h2>Inspecting retained setups</h2><p>Checking managed recipe paths, Compose project labels, and ownership. Nothing can be changed during this scan.</p><div class="recovery-skeletons"><span class="skeleton"></span><span class="skeleton"></span><span class="skeleton"></span></div></section>`);
  if (mode === "empty") return frame(`<section class="panel install-result recovery-result"><span class="result-mark">${icon("check")}</span><p class="eyebrow">Scan complete</p><h2>No retained setups found.</h2><p>Supported app folders contain no unregistered Compose setup requiring review.</p><div class="result-actions"><button class="button primary" type="button" data-recovery-action="scan">Scan again ${icon("arrow")}</button><button class="button ghost" type="button" data-go-screen="settings">Back to Settings</button></div></section>`);
  if (mode === "mismatch" || mode === "scan-failure") {
    const mismatch = mode === "mismatch";
    return frame(`<section class="panel install-result failure-result recovery-result unsafe"><span class="result-mark">${icon("alert")}</span><p class="eyebrow">${mismatch ? "Ownership mismatch" : "Scan did not complete"}</p><h2>${mismatch ? "Local Store will not touch this setup." : "Recovery status is unknown."}</h2><p>${mismatch ? "Containers using this Compose project name do not match the retained setup file. Acting on them could stop somebody else’s service." : "Docker ownership inspection failed or returned incomplete output. Nothing was changed."}</p><div class="failure-status">${icon("alert")}<div><strong>All cleanup actions are blocked</strong><p>${mismatch ? "Review the project in Docker, resolve the ownership conflict, then scan again." : "Start Docker and retry the read-only inspection."}</p></div></div><code class="error-code">${mismatch ? "unsafe_path · ownership_mismatch" : "process_failed · recovery_scan"}</code><div class="result-actions"><button class="button primary" type="button" data-recovery-action="scan">Scan again ${icon("arrow")}</button><button class="button ghost" type="button" data-go-screen="settings">Back to Settings</button></div></section>`);
  }
  if (["confirm-keep", "confirm-delete"].includes(mode)) {
    const deletes = mode === "confirm-delete";
    return frame(`<section class="panel recovery-confirm ${deletes ? "delete-confirm" : ""}"><span class="first-run-state-mark ${deletes ? "danger" : ""}">${icon(deletes ? "alert" : "docker")}</span><p class="eyebrow">${deletes ? "Permanent deletion" : "Container cleanup"}</p><h2>${deletes ? "Delete the retained setup and its data?" : "Stop the retained Compose project?"}</h2><p>${deletes ? "Local Store will re-verify ownership, stop matching containers, and permanently delete the Memos managed directory." : "Local Store will re-verify ownership and stop matching containers. The Compose file and every data file remain on this computer."}</p><div class="recovery-consequences ${deletes ? "danger" : ""}"><div>${icon("docker")}<span><strong>Matching containers</strong><small>Stopped only after ownership is verified again</small></span></div><div>${icon("grid")}<span><strong>Setup and app data</strong><small>${deletes ? "Permanently deleted after Docker succeeds" : "Preserved in the managed Memos folder"}</small></span></div></div>${deletes ? `<label class="field-label delete-recovery-label" for="recovery-confirmation">Type <strong>Memos</strong> to confirm<input class="input" id="recovery-confirmation" type="text" autocomplete="off"></label>` : ""}<div class="recovery-confirm-actions"><button class="button ghost" type="button" data-recovery-action="cancel">Cancel</button><button class="button ${deletes ? "danger" : "primary"}" type="button" data-recovery-action="${deletes ? "delete" : "keep"}" ${deletes ? "disabled" : ""}>${deletes ? "Delete setup and data" : "Stop matching containers"}</button></div></section>`);
  }
  if (mode === "working") return frame(`<section class="panel recovery-loading recovery-working" aria-busy="true"><span class="busy-orbit" aria-hidden="true"></span><p class="eyebrow">Protected operation</p><h2>Re-verifying before cleanup</h2><p>The candidate is being scanned again under its operation lock. Containers are handled before any file can be deleted.</p><div class="commit-boundary danger">${icon("alert")}<p><strong>Do not close Local Store.</strong> This safety check and any required Docker cleanup cannot be interrupted.</p></div></section>`);
  if (["success-keep", "success-delete"].includes(mode)) {
    const deleted = mode === "success-delete";
    return frame(`<section class="panel install-result success-result recovery-result"><span class="result-mark">${icon("check")}</span><p class="eyebrow">Recovery complete</p><h2>${deleted ? "Retained setup and data deleted." : "Owned containers stopped."}</h2><p>${deleted ? "One matching container was removed before the managed Memos directory was deleted." : "One matching container was removed. The Compose file and app data remain in the managed Memos folder."}</p><div class="result-facts"><div><span>Containers removed</span><code>1</code></div><div><span>Data deleted</span><code>${deleted ? "Yes" : "No"}</code></div></div><div class="result-actions"><button class="button primary" type="button" data-go-screen="settings">Return to Settings ${icon("arrow")}</button>${deleted ? "" : `<button class="button ghost" type="button" data-recovery-action="scan">Review retained files</button>`}</div></section>`);
  }
  const ownership = mode === "no-containers" ? "No matching containers" : "Ownership snapshot verified";
  return frame(`<div class="recovery-truth"><span class="truth-label">Inspection available now</span><span class="lab-tag">Cleanup needs launcher wiring</span></div><section class="panel recovery-card"><div class="recovery-identity">${appIcon(fixtures.contract.recipe, true)}<div><p class="eyebrow">Retained reviewed setup</p><h2>${escapeHtml(candidate.display_name)}</h2><p>An install that never entered My Apps left a managed Compose file. This snapshot is evidence to review, not permission to act.</p></div><span class="badge ${mode === "no-containers" ? "" : "success"}">${ownership}</span></div><div class="recovery-assurance">${icon("check")}<p><strong>Every cleanup rechecks ownership under an operation lock.</strong> A changed, installed, or mismatched project is refused before Docker or file deletion.</p></div><div class="trust-list recovery-trust"><div class="trust-row"><span>Recipe</span><code>${escapeHtml(candidate.recipe_id)}</code></div><div class="trust-row"><span>Project</span><code>${escapeHtml(candidate.project_name)}</code></div><div class="trust-row"><span>Compose</span><code>${escapeHtml(candidate.compose_file)}</code></div><div class="trust-row"><span>Ownership</span><code>${mode === "no-containers" ? "no_containers" : escapeHtml(candidate.ownership_status)}</code></div></div><div class="recovery-actions"><div><span><strong>Stop owned containers</strong><small>Keep the Compose file and all existing app data.</small></span><button class="button ghost" type="button" data-recovery-action="confirm-keep">Review cleanup</button></div><div class="destructive-recovery"><span><strong>Delete setup and data</strong><small>Stop owned containers, then permanently remove the managed directory.</small></span><button class="button danger" type="button" data-recovery-action="confirm-delete">Review deletion</button></div></div></section><div class="concept-banner recovery-contract">${icon("alert")}<div><span class="lab-tag">Implementation boundary</span><p>${escapeHtml(fixtures.concept.recoveryCleanupNotice)}</p></div></div>`);
}

function firstRunSteps(current) {
  const labels = [["welcome", "Welcome"], ["choose", "Choose"], ["install", "Install"], ["ready", "Ready"]];
  const currentIndex = labels.findIndex(([id]) => id === current);
  return labels.map(([id, label], index) => `<div class="step ${index < currentIndex ? "complete" : id === current ? "current" : ""}"><i>${index < currentIndex ? icon("check") : index + 1}</i><span>${label}</span></div>`).join("");
}

function firstRunFrame(step, content) {
  return `<section class="panel first-run-shell"><aside class="first-run-steps"><a class="first-run-brand" href="#first-run" aria-label="Local Store introduction"><img src="assets/brand/mark-grain-approved.svg" alt=""><span><strong>Local Store</strong><small>Private workspace</small></span></a><nav aria-label="Introduction progress">${firstRunSteps(step)}</nav><div class="first-run-aside-note"><span class="status-dot success"></span><span>Local by design<br><small>No account required</small></span></div></aside><div class="first-run-content">${content}</div></section>`;
}

function firstRunPreflight() {
  if (prototype.firstRunPreflight === "ready" && !["busy", "loading"].includes(prototype.condition)) return `<div class="preflight-inline ready" role="status"><span class="status-dot success"></span><span><strong>Ready for reviewed installs</strong><small>Docker Engine and Compose checked in the background.</small></span><span class="badge success">Ready</span></div>`;
  return `<div class="preflight-inline checking" role="status" aria-busy="true"><span class="preflight-spinner" aria-hidden="true"></span><span><strong>Checking this computer</strong><small>Checking Docker and Compose while you read.</small></span><span class="badge">2 checks</span></div>`;
}

function firstRunWelcome() {
  return firstRunFrame("welcome", `<div class="welcome-layout"><div class="welcome-copy"><p class="eyebrow">A home for what you host</p><h1>Run and reach your self-hosted apps from one calm place.</h1><p class="welcome-lede">Local Store keeps reviewed installs and apps you already run together—without taking control away from you.</p><div class="welcome-principles"><div>${icon("check")}<span><strong>Review before run</strong><small>See ports, storage, and machine changes first.</small></span></div><div>${icon("grid")}<span><strong>One local workspace</strong><small>Open and inspect every saved app.</small></span></div><div>${icon("external")}<span><strong>Your services stay yours</strong><small>Connect existing apps without managing their server.</small></span></div></div><div class="welcome-actions"><button class="button primary" type="button" data-first-action="continue">Get started ${icon("arrow")}</button><button class="text-button" type="button" data-first-action="skip">Skip for now</button></div>${firstRunPreflight()}</div><div class="welcome-art" aria-hidden="true"><span class="welcome-orbit orbit-one"></span><span class="welcome-orbit orbit-two"></span><img src="assets/brand/mark-grain-approved.svg" alt=""><div class="welcome-window"><i></i><i></i><i></i><span></span><span></span><span></span></div></div></div>`);
}

function firstRunDockerMissing() {
  return firstRunFrame("welcome", `<div class="first-run-narrow"><span class="first-run-state-mark danger">${icon("docker")}</span><p class="eyebrow">Docker required for local installs</p><h1>Start Docker, then check again.</h1><p>Nothing is broken in your workspace. Reviewed installs wait for Docker Engine and Compose; connecting or browsing can continue now.</p><div class="preflight-checks" role="status"><div><span class="status-dot danger"></span><span><strong>Docker Engine</strong><small>Not available</small></span><span class="badge danger">Required</span></div><div><span class="status-dot warning"></span><span><strong>Docker Compose</strong><small>Waiting for Docker Engine</small></span><span class="badge">Pending</span></div></div><div class="docker-recovery"><strong>Recovery</strong><ol><li>Open Docker Desktop and wait until it reports that the engine is running.</li><li>Return here and run the two checks again.</li></ol></div><div class="welcome-actions"><button class="button primary" type="button" data-first-action="retry-doctor">Check again ${icon("arrow")}</button><button class="button ghost" type="button" data-first-action="connect">Connect an existing app</button></div><button class="text-button first-run-browse" type="button" data-go-screen="discover">Browse the catalog without Docker</button></div>`);
}

function firstRunChoose(empty = false) {
  const selected = starterRecipe();
  const recipeCards = fixtures.contract.starterRecipes.map((recipe) => `<button class="starter-card ${recipe.id === selected.id ? "selected" : ""}" type="button" role="radio" aria-checked="${recipe.id === selected.id}" data-first-recipe="${recipe.id}">${appIcon(recipe)}<span><strong>${escapeHtml(recipe.display_name)}</strong><small>${escapeHtml(recipe.description)}</small><em>${escapeHtml(recipe.version)} · ${escapeHtml(recipe.license)}</em></span><i>${recipe.id === selected.id ? icon("check") : ""}</i></button>`).join("");
  const starter = empty ? `<div class="starter-empty"><span class="first-run-state-mark">${icon("discover")}</span><div><strong>Starter recipes are unavailable</strong><p>Connecting and browsing still work. No install can begin without a complete local recipe.</p></div></div>` : `<div class="starter-grid" role="radiogroup" aria-label="Reviewed starter recipes">${recipeCards}</div><div class="starter-config"><label class="field-label" for="first-run-port">Published port<span class="first-port-control"><code>127.0.0.1:</code><input class="input" id="first-run-port" type="number" min="1024" max="65535" value="${prototype.installPort}" aria-describedby="first-port-help first-port-error"></span></label><div><span>Internal port</span><code>${selected.container_port}</code></div><p id="first-port-help">Editable now. A complete review appears before Docker runs.</p><p id="first-port-error" class="field-error" ${prototype.installPortError ? "" : "hidden"}>${escapeHtml(prototype.installPortError)}</p></div>`;
  return firstRunFrame("choose", `<div class="choose-layout"><div class="choose-head"><p class="eyebrow">A useful first step</p><h1>${empty ? "Your workspace can still begin." : "Start with one reviewed app."}</h1><p>${empty ? "The local starter list could not be read." : "Memos is recommended for a quick, transparent first install. You can change the recipe and local port before review."}</p></div>${starter}<div class="choose-footer"><div class="alternate-routes"><span>Not installing today?</span><button class="text-button" type="button" data-first-action="connect">Connect something running</button><i>or</i><button class="text-button" type="button" data-go-screen="discover">Browse all projects</button></div><div class="choose-actions"><button class="text-button" type="button" data-first-action="back">Back</button>${empty ? "" : `<button class="button primary" type="button" data-first-action="review-install">Review ${escapeHtml(selected.display_name)} install ${icon("arrow")}</button>`}</div></div></div>`);
}

function firstRunReady() {
  const recipe = starterRecipe(prototype.installRecipe);
  const connected = prototype.firstRunCompletion === "connected";
  return firstRunFrame("ready", `<div class="first-run-narrow ready-content"><span class="first-run-state-mark success">${icon("check")}</span><p class="eyebrow">Workspace ready</p><h1>${connected ? "Your first app is connected." : `${escapeHtml(recipe.display_name)} has a home.`}</h1><p>${connected ? "The saved address is ready in My Apps. Local Store will not manage the service behind it." : `${escapeHtml(recipe.display_name)} is healthy at localhost:${prototype.installPort} and ready to use.`}</p><div class="ready-summary"><div>${appIcon(recipe, true)}<span><strong>${connected ? "Connected app" : escapeHtml(recipe.display_name)}</strong><small>${connected ? "Linked address" : `Reviewed install · ${escapeHtml(recipe.version)}`}</small></span><span class="badge success">Ready</span></div><p>${connected ? "Open it from My Apps whenever you need it." : "Its managed data stays on this computer and is preserved by default if you uninstall."}</p></div><div class="welcome-actions">${connected ? "" : `<button class="button primary" type="button" data-install-action="open">Open ${escapeHtml(recipe.display_name)} ${icon("external")}</button>`}<button class="button ${connected ? "primary" : "ghost"}" type="button" data-first-action="workspace">Enter workspace ${icon("arrow")}</button></div><button class="text-button first-run-browse" type="button" data-go-screen="discover">Explore more projects</button></div>`);
}

function firstRun() {
  if (prototype.condition === "failure" || prototype.firstRunPreflight === "missing") return firstRunDockerMissing();
  if (prototype.condition === "success" || prototype.firstRunStep === "ready") return firstRunReady();
  if (prototype.condition === "empty") return firstRunChoose(true);
  if (["busy", "loading"].includes(prototype.condition)) return firstRunWelcome();
  if (prototype.firstRunStep === "choose") return firstRunChoose();
  return firstRunWelcome();
}

const renderers = { overview, discover, "my-apps": myApps, activity, settings, install, recovery, "first-run": firstRun };

function render({ focusHeading = false, suppressMotion = false } = {}) {
  if (!screens[prototype.screen]) prototype.screen = "overview";
  routeLabel.textContent = screens[prototype.screen].label;
  document.body.dataset.screen = prototype.screen;
  screenJump.value = prototype.screen;
  document.title = `${screens[prototype.screen].label} · Local Store V2`;
  const parent = prototype.screen === "recovery" ? "settings" : prototype.screen === "install" && prototype.installOrigin !== "first-run" ? "discover" : null;
  document.querySelectorAll("[data-route]").forEach((link) => {
    if (link.dataset.route === prototype.screen) link.setAttribute("aria-current", "page");
    else if (link.dataset.route === parent && link.classList.contains("nav-item")) link.setAttribute("aria-current", "location");
    else link.removeAttribute("aria-current");
  });
  if (parent) routeLabel.textContent = `${screens[parent].label} / ${screens[prototype.screen].label}`;
  document.querySelectorAll("[data-state]").forEach((button) => button.setAttribute("aria-pressed", String(button.dataset.state === prototype.condition)));
  overviewConceptToggle.checked = prototype.showOverviewConcepts;
  const environmentDot = environmentCard.querySelector(".status-dot");
  const environmentTitle = environmentCard.querySelector("strong");
  const environmentDetail = environmentCard.querySelector("small");
  const overviewFailure = prototype.screen === "overview" && prototype.condition === "failure";
  const overviewBusy = prototype.screen === "overview" && prototype.condition === "busy";
  environmentDot.className = `status-dot ${overviewFailure ? "danger" : overviewBusy ? "warning" : "success"}`;
  environmentTitle.textContent = overviewFailure ? "Docker unavailable" : overviewBusy ? "Checking Docker" : "Docker ready";
  environmentDetail.textContent = overviewFailure ? "No repair started" : overviewBusy ? "Prior result retained" : "Engine 29.3.1";
  mount.dataset.screen = prototype.screen;
  mount.dataset.condition = prototype.condition;
  mount.classList.toggle("motion-instant", suppressMotion);
  mount.innerHTML = ["install", "first-run", "activity", "settings", "recovery"].includes(prototype.screen) ? renderers[prototype.screen]() : prototype.condition === "loading" && prototype.screen === "my-apps" ? myAppsLoading() : prototype.condition === "loading" ? loadingScreen() : prototype.condition === "empty" ? emptyScreen() : renderers[prototype.screen]();
  if (suppressMotion) requestAnimationFrame(() => mount.classList.remove("motion-instant"));
  bindLocalControls();
  const discoverModalOpen = prototype.screen === "discover" && Boolean(prototype.discoverDrawer);
  const appModalOpen = prototype.screen === "my-apps" && Boolean(prototype.appDialog);
  const modalOpen = discoverModalOpen || appModalOpen;
  document.querySelector(".app-rail").inert = modalOpen;
  document.querySelector(".workspace-bar").inert = modalOpen;
  mount.querySelectorAll(":scope > :not(.project-drawer):not(.discover-scrim):not(.app-dialog):not(.app-dialog-scrim)").forEach((element) => { element.inert = modalOpen; });
  if (appModalOpen && !document.querySelector(".app-dialog")?.contains(document.activeElement)) document.querySelector(".app-dialog [data-close-app-dialog]")?.focus();
  if (focusHeading) mount.querySelector("h1")?.focus({ preventScroll: true });
}

function navigate(screen, { focusHeading = true } = {}) {
  if (!screens[screen]) return;
  if (screen !== "discover") {
    prototype.discoverDrawer = null;
    prototype.discoverFilterOpen = false;
  }
  if (screen !== "my-apps") { prototype.appDialog = null; prototype.savedLinkName = null; }
  prototype.screen = screen;
  history.pushState(null, "", `#${screen}`);
  render({ focusHeading });
  document.querySelector(".workspace")?.scrollTo({ top: 0 });
}

function bindLocalControls() {
  document.querySelectorAll("[data-go-screen]").forEach((button) => button.addEventListener("click", () => {
    if (button.dataset.selectApp) prototype.selectedApp = button.dataset.selectApp;
    if (button.dataset.goScreen === "install" && prototype.screen !== "first-run") {
      prototype.installOrigin = "discover";
      prototype.installRecipe = "memos";
      prototype.installPort = starterRecipe("memos").host_port;
    }
    if (button.dataset.prototypeAction) showToast("Prototype surface", "A contextual project drawer is specified for a later phase.");
    else navigate(button.dataset.goScreen);
  }));

  document.querySelectorAll("[data-prototype-action]").forEach((button) => button.addEventListener("click", () => {
    if (!button.dataset.prototypeAction) return;
    const labels = {
      palette: ["Global search concept", "The Ctrl+K aggregation contract still requires backend work."],
      connect: ["Connect flow", "The contextual form will be designed in the install-flow phase."],
      "project-detail": ["Project detail", "The contextual drawer is part of the Discover screen phase."],
      start: ["Start requested", "This fixture demonstrates immediate feedback without changing real apps."],
      open: ["Open requested", "Prototype actions never launch external application windows."],
      logs: ["Logs section", "The bounded log snapshot state is specified for the My Apps phase."],
      manage: ["Manage section", "Lifecycle and destructive states will be completed with My Apps."],
      filter: ["Activity filters", "Filtering is visible here but its history model is still concept data."],
      doctor: ["Docker check complete", "Docker Engine 29.3.1 and Compose 2.39.4 are ready."],
      install: ["Install prototype", "No container was started. Use the state controls to inspect progress and results."],
      "clear-recovery": ["Destructive action withheld", "This prototype does not mutate retained files."],
      "catalog-retry": ["Catalog retry started", "Saved apps remain untouched while the local snapshot is read again."],
      "next-page": ["Next page requested", "The current query and filters remain in place while the next bounded page loads."],
    };
    const message = labels[button.dataset.prototypeAction] || ["Prototype action", "This control is intentionally non-mutating."];
    showToast(...message);
  }));

  document.querySelectorAll("[data-first-action]").forEach((button) => button.addEventListener("click", () => handleFirstRunAction(button.dataset.firstAction)));
  document.querySelectorAll("[data-first-recipe]").forEach((button) => button.addEventListener("click", () => {
    const recipe = starterRecipe(button.dataset.firstRecipe);
    prototype.firstRunRecipe = recipe.id;
    prototype.installPort = recipe.host_port;
    prototype.installPortError = "";
    render();
    document.querySelector(`[data-first-recipe="${recipe.id}"]`)?.focus();
  }));
  document.querySelectorAll("[data-reviewed-recipe]").forEach((button) => button.addEventListener("click", () => {
    const recipe = starterRecipe(button.dataset.reviewedRecipe);
    prototype.installOrigin = "discover";
    prototype.installRecipe = recipe.id;
    prototype.installPort = recipe.host_port;
    prototype.installPortError = "";
    navigate("install");
  }));
  document.querySelector("#first-run-port")?.addEventListener("input", (event) => {
    prototype.installPort = Number(event.currentTarget.value);
    prototype.installPortError = "";
    event.currentTarget.removeAttribute("aria-invalid");
    document.querySelector("#first-port-error")?.setAttribute("hidden", "");
  });
  document.querySelectorAll("[data-activity-filter]").forEach((button) => button.addEventListener("click", () => {
    prototype.activityFilter = button.dataset.activityFilter;
    render();
    document.querySelector(`[data-activity-filter="${prototype.activityFilter}"]`)?.focus();
  }));
  document.querySelector("[data-activity-app]")?.addEventListener("change", (event) => {
    prototype.activityApp = event.currentTarget.value;
    render();
    document.querySelector("[data-activity-app]")?.focus();
  });
  document.querySelector("#activity-search")?.addEventListener("input", (event) => {
    prototype.activityQuery = event.currentTarget.value;
    render();
    const input = document.querySelector("#activity-search");
    input?.focus();
    input?.setSelectionRange(input.value.length, input.value.length);
  });
  document.querySelectorAll("[data-activity-event]").forEach((button) => button.addEventListener("click", () => {
    prototype.activityExpanded = prototype.activityExpanded === button.dataset.activityEvent ? null : button.dataset.activityEvent;
    render();
    document.querySelector(`[data-activity-event="${button.dataset.activityEvent}"]`)?.focus();
  }));
  document.querySelectorAll("[data-activity-action]").forEach((button) => button.addEventListener("click", () => handleActivityAction(button.dataset.activityAction)));
  document.querySelectorAll("[data-settings-action]").forEach((button) => button.addEventListener("click", () => handleSettingsAction(button.dataset.settingsAction)));
  document.querySelectorAll("[data-recovery-action]").forEach((button) => button.addEventListener("click", () => handleRecoveryAction(button.dataset.recoveryAction)));
  document.querySelector("#recovery-confirmation")?.addEventListener("input", (event) => {
    document.querySelector('[data-recovery-action="delete"]').disabled = event.currentTarget.value !== fixtures.contract.recovery.display_name;
  });

  document.querySelectorAll("[data-open-project]").forEach((button) => button.addEventListener("click", () => openDiscoverDrawer("detail", button.dataset.openProject)));
  document.querySelectorAll("[data-open-connect]").forEach((button) => button.addEventListener("click", () => openDiscoverDrawer("connect", button.dataset.openConnect || null)));
  document.querySelectorAll("[data-dismiss-discover]").forEach((button) => button.addEventListener("click", closeDiscoverDrawer));
  document.querySelectorAll("[data-visit-project]").forEach((button) => button.addEventListener("click", () => {
    const project = fixtures.contract.catalog.find((item) => item.id === button.dataset.visitProject);
    showToast("Project link withheld", `${project?.project_url || "The upstream URL"} would open outside the prototype.`);
  }));
  document.querySelectorAll("[data-toggle-filters]").forEach((button) => button.addEventListener("click", () => {
    if (prototype.discoverFilterOpen) {
      closeDiscoverFilters();
      return;
    }
    prototype.discoverFilterOpen = true;
    render();
    document.querySelector("#catalog-filter-panel select")?.focus({ preventScroll: true });
    revealPopover(document.querySelector("#catalog-filter-panel"));
  }));
  document.querySelectorAll("[data-quick-filter]").forEach((button) => button.addEventListener("click", () => {
    prototype.discoverQuickFilter = button.dataset.quickFilter;
    render();
    document.querySelector(`[data-quick-filter="${prototype.discoverQuickFilter}"]`)?.focus();
  }));
  document.querySelectorAll("[data-catalog-filter]").forEach((control) => control.addEventListener("change", () => {
    prototype.discoverFilters[control.dataset.catalogFilter] = control.value;
    render({ suppressMotion: true });
    document.querySelector(`[data-catalog-filter="${control.dataset.catalogFilter}"]`)?.focus();
  }));
  document.querySelector("[data-warning-filter]")?.addEventListener("change", (event) => {
    prototype.discoverFilters.warnings = event.currentTarget.checked;
    render({ suppressMotion: true });
    document.querySelector("[data-warning-filter]")?.focus();
  });
  document.querySelectorAll("[data-clear-discover]").forEach((button) => button.addEventListener("click", resetDiscoverFilters));
  document.querySelector("[data-clear-advanced]")?.addEventListener("click", () => {
    prototype.discoverFilters = { category: "all", license: "all", architecture: "all", warnings: false };
    render();
    document.querySelector("[data-clear-advanced]")?.focus();
  });
  document.querySelector("[data-check-address]")?.addEventListener("click", checkConnectAddress);
  document.querySelector("[data-connect-form]")?.addEventListener("submit", saveConnectedApp);

  document.querySelectorAll("[data-app-id]").forEach((row) => row.addEventListener("click", () => {
    prototype.selectedApp = row.dataset.appId;
    prototype.appSection = "overview";
    prototype.appSubstate = "default";
    render();
    document.querySelector(`[data-app-id="${prototype.selectedApp}"]`)?.focus();
  }));

  document.querySelectorAll("[data-app-section]").forEach((tab) => tab.addEventListener("click", () => {
    if (tab.getAttribute("aria-disabled") === "true") return;
    prototype.appSection = tab.dataset.appSection;
    prototype.appSubstate = "default";
    render();
    document.querySelector(`[data-app-section="${prototype.appSection}"]`)?.focus();
  }));
  document.querySelectorAll("[data-app-action]").forEach((button) => button.addEventListener("click", () => handleAppAction(button.dataset.appAction)));
  document.querySelectorAll("[data-open-app-dialog]").forEach((button) => button.addEventListener("click", () => {
    prototype.appDialog = button.dataset.openAppDialog;
    render();
    document.querySelector(".app-dialog [data-close-app-dialog]")?.focus();
  }));
  document.querySelectorAll("[data-close-app-dialog]").forEach((button) => button.addEventListener("click", closeAppDialog));
  document.querySelector("[data-delete-confirmation]")?.addEventListener("input", (event) => {
    const selected = fixtures.contract.apps.find((app) => app.id === prototype.selectedApp);
    const confirmButton = document.querySelector("[data-confirm-app-action=\"delete-data\"]");
    confirmButton.disabled = event.currentTarget.value !== selected.display_name;
  });
  document.querySelector("[data-confirm-app-action]")?.addEventListener("click", (event) => {
    const action = event.currentTarget.dataset.confirmAppAction;
    const selected = fixtures.contract.apps.find((app) => app.id === prototype.selectedApp);
    const message = action === "delete-data" ? "App and data deletion withheld" : action === "remove" ? "Linked app removal withheld" : "Safe uninstall simulated";
    const detail = action === "uninstall" ? `${selected.display_name} data would remain in its managed folder.` : "The prototype does not mutate apps or files.";
    closeAppDialog(() => showToast(message, detail));
  });
  document.querySelector("[data-clear-app-filter]")?.addEventListener("click", () => {
    prototype.appQuery = "";
    render();
    document.querySelector("#apps-search")?.focus();
  });
  document.querySelector("[data-app-list]")?.addEventListener("keydown", handleAppListKeys);
  document.querySelector(".detail-tabs")?.addEventListener("keydown", handleAppTabKeys);
  document.querySelectorAll("[data-install-action]").forEach((button) => button.addEventListener("click", () => handleInstallAction(button.dataset.installAction)));
  document.querySelectorAll("#install-port, [data-install-port]").forEach((input) => input.addEventListener("input", () => {
    prototype.installPort = Number(input.value);
    prototype.installPortError = "";
    input.removeAttribute("aria-invalid");
    document.querySelector("#port-error")?.setAttribute("hidden", "");
    const refused = prototype.installFailedPort === prototype.installPort;
    document.querySelector("#failure-port-error")?.toggleAttribute("hidden", !refused);
    if (refused && input.id === "failure-port") input.setAttribute("aria-invalid", "true");
  }));
  document.querySelectorAll("[data-setup-key]").forEach((input) => input.addEventListener("input", () => {
    if (input.type !== "password") prototype.installAnswers[input.dataset.setupKey] = input.value;
  }));
  document.querySelector(".install-evidence")?.addEventListener("toggle", (event) => {
    prototype.installAdvanced = event.currentTarget.open;
  });

  const catalogSearch = document.querySelector("#catalog-search");
  catalogSearch?.addEventListener("input", () => {
    prototype.discoverQuery = catalogSearch.value;
    applyCatalogFilters();
  });
  if (catalogSearch) applyCatalogFilters();
  const appsSearch = document.querySelector("#apps-search");
  appsSearch?.addEventListener("input", () => {
    prototype.appQuery = appsSearch.value;
    applyAppFilter();
  });
  if (appsSearch) applyAppFilter();
}

function openDiscoverDrawer(mode, projectId) {
  if (prototype.screen !== "discover") {
    prototype.screen = "discover";
    history.pushState(null, "", "#discover");
  }
  prototype.selectedProject = projectId;
  prototype.discoverDrawer = mode;
  prototype.discoverFilterOpen = false;
  render();
  const addressField = mode === "connect" ? document.querySelector("#connect-address") : null;
  (addressField || document.querySelector(".project-drawer .icon-button"))?.focus();
}

function handleAppAction(action) {
  const selected = fixtures.contract.apps.find((app) => app.id === prototype.selectedApp);
  if (action === "show-logs") {
    prototype.appSection = "logs";
    prototype.appSubstate = "default";
    render();
    document.querySelector("[data-app-section=\"logs\"]")?.focus();
    return;
  }
  const messages = {
    open: ["Open requested", `The prototype will not launch ${selected.display_name}.`],
    start: ["Start requested", `${selected.display_name} remains selected while the Compose operation begins.`],
    stop: ["Stop requested", `${selected.display_name} remains registered and its data stays on disk.`],
    "check-address": ["Address check requested", "The bounded check is advisory and does not manage the linked server."],
    "refresh-logs": ["Logs refreshed", "A new bounded Compose log snapshot would replace the current lines."],
    "copy-logs": ["Logs copied", "The visible bounded snapshot would be copied to the clipboard."],
    "refresh-list": ["Refresh requested", "Last known app records remain visible until a new response arrives."],
    shortcut: ["Shortcut requested", `A Windows shortcut would open ${selected.launch_url}.`],
  };
  showToast(...(messages[action] || ["App action", "This prototype does not mutate or launch apps."]));
}

function validInstallPort(input = document.querySelector("#install-port, #first-run-port, [data-install-port]")) {
  const port = Number(input?.value ?? prototype.installPort);
  prototype.installPort = port;
  if (!Number.isInteger(port) || port < 1024 || port > 65535) {
    prototype.installPortError = "Choose a whole-number port between 1024 and 65535.";
    if (input) {
      input.setAttribute("aria-invalid", "true");
      if (input.id === "install-port") input.setAttribute("aria-describedby", "port-help port-error");
      else if (input.id === "first-run-port") input.setAttribute("aria-describedby", "first-port-help first-port-error");
      else input.removeAttribute("aria-describedby");
      input.focus();
    }
    const error = document.querySelector(input?.id === "first-run-port" ? "#first-port-error" : "#port-error");
    if (error) {
      error.hidden = false;
      error.textContent = prototype.installPortError;
    }
    return false;
  }
  prototype.installPortError = "";
  return true;
}

function handleActivityAction(action) {
  if (action === "clear") {
    prototype.activityFilter = "all";
    prototype.activityApp = "all";
    prototype.activityQuery = "";
    prototype.activityExpanded = null;
    render();
    document.querySelector("#activity-search")?.focus();
    return;
  }
  if (action === "retry") {
    prototype.condition = "default";
    render();
    showToast("History refreshed", "The last available activity model is visible again.");
  }
}

function handleSettingsAction(action) {
  if (action === "doctor") {
    prototype.condition = "busy";
    render({ focusHeading: true });
    window.setTimeout(() => {
      if (prototype.screen !== "settings" || prototype.condition !== "busy") return;
      prototype.condition = "success";
      render();
      showToast("Docker is ready", "Docker Engine and Compose passed both checks.");
    }, 520);
    return;
  }
  if (action === "scan") {
    prototype.recoveryMode = "loading";
    prototype.condition = "default";
    navigate("recovery");
    window.setTimeout(() => {
      if (prototype.screen !== "recovery" || prototype.recoveryMode !== "loading") return;
      prototype.recoveryMode = "candidate";
      render({ focusHeading: true });
    }, 520);
  }
}

function handleRecoveryAction(action) {
  if (action === "scan") {
    prototype.condition = "default";
    prototype.recoveryMode = "loading";
    render({ focusHeading: true });
    window.setTimeout(() => {
      if (prototype.screen !== "recovery" || prototype.recoveryMode !== "loading") return;
      prototype.recoveryMode = "candidate";
      render({ focusHeading: true });
    }, 520);
    return;
  }
  if (action === "confirm-keep" || action === "confirm-delete") {
    prototype.recoveryMode = action;
    render({ focusHeading: true });
    return;
  }
  if (action === "cancel") {
    prototype.recoveryMode = "candidate";
    render({ focusHeading: true });
    return;
  }
  if (action === "keep" || action === "delete") {
    const deletes = action === "delete";
    prototype.recoveryMode = "working";
    render({ focusHeading: true });
    window.setTimeout(() => {
      if (prototype.screen !== "recovery" || prototype.recoveryMode !== "working") return;
      prototype.recoveryMode = deletes ? "success-delete" : "success-keep";
      render({ focusHeading: true });
    }, 620);
  }
}

function handleFirstRunAction(action) {
  if (action === "continue") {
    prototype.firstRunPreflight = "ready";
    prototype.firstRunStep = "choose";
    prototype.condition = "default";
    render({ focusHeading: true });
    return;
  }
  if (action === "back") {
    prototype.firstRunStep = "welcome";
    render({ focusHeading: true });
    return;
  }
  if (action === "retry-doctor") {
    prototype.firstRunPreflight = "ready";
    prototype.firstRunStep = "choose";
    prototype.condition = "default";
    render({ focusHeading: true });
    showToast("Docker is ready", "Docker Engine and Compose passed the two preflight checks.");
    return;
  }
  if (action === "connect") {
    openDiscoverDrawer("connect", null);
    return;
  }
  if (action === "review-install") {
    if (!validInstallPort(document.querySelector("#first-run-port"))) return;
    prototype.installRecipe = prototype.firstRunRecipe;
    prototype.installOrigin = "first-run";
    prototype.condition = "default";
    prototype.installOutcome = null;
    navigate("install");
    return;
  }
  if (action === "finish-install") {
    prototype.condition = "default";
    prototype.firstRunCompletion = "installed";
    prototype.firstRunStep = "ready";
    navigate("first-run");
    return;
  }
  if (action === "workspace") {
    const app = fixtures.contract.apps.find((item) => item.id === prototype.installRecipe);
    if (app) prototype.selectedApp = app.id;
    navigate("my-apps");
    return;
  }
  if (action === "skip") {
    prototype.firstRunStep = "welcome";
    navigate("overview");
    showToast("Introduction skipped", "You can run it again from Settings. Saved apps were not changed.");
    return;
  }
  if (action === "restart") {
    prototype.firstRunStep = "welcome";
    prototype.firstRunPreflight = "checking";
    prototype.firstRunRecipe = "memos";
    prototype.installRecipe = "memos";
    prototype.installPort = starterRecipe("memos").host_port;
    prototype.condition = "default";
    navigate("first-run");
  }
}

function handleInstallAction(action) {
  if (action === "start") {
    if (!validInstallPort()) return;
    if (prototype.installFailedPort !== prototype.installPort) prototype.installFailedPort = null;
    prototype.installOutcome = null;
    prototype.installStage = "checking_system";
    prototype.condition = "busy";
    render();
    document.querySelector("h1")?.focus();
    return;
  }
  if (action === "cancel") {
    prototype.condition = "default";
    prototype.installOutcome = "cancelled";
    render();
    document.querySelector("h1")?.focus();
    return;
  }
  if (action === "review") {
    const portInput = document.querySelector("[data-install-port]");
    if (portInput && !validInstallPort(portInput)) return;
    if (portInput && prototype.installFailedPort === prototype.installPort) {
      portInput.setAttribute("aria-invalid", "true");
      document.querySelector("#failure-port-error")?.removeAttribute("hidden");
      portInput.focus();
      return;
    }
    prototype.condition = "default";
    prototype.installOutcome = null;
    prototype.installPortError = "";
    render();
    document.querySelector("#install-port")?.focus();
    return;
  }
  if (action === "my-apps") {
    if (fixtures.contract.apps.some((app) => app.id === prototype.installRecipe)) prototype.selectedApp = prototype.installRecipe;
    navigate("my-apps");
    return;
  }
  if (action === "leave") {
    prototype.condition = "default";
    prototype.installOutcome = null;
    if (prototype.installOrigin === "first-run") {
      prototype.firstRunStep = "choose";
      navigate("first-run");
    } else navigate("discover");
    return;
  }
  if (action === "reload-recipe") {
    prototype.condition = "default";
    prototype.installOutcome = null;
    render();
    document.querySelector("h1")?.focus();
    return;
  }
  if (action === "open") {
    showToast("Open requested", `The prototype will not launch ${activeRecipe().display_name} at localhost:${prototype.installPort}.`);
  }
}

function closeAppDialog(afterClose, instant = false) {
  const mode = prototype.appDialog;
  prototype.appDialog = null;
  finishTransientExit(document.querySelector(".app-dialog"), document.querySelector(".app-dialog-scrim"), () => {
    render({ suppressMotion: true });
    document.querySelector(`[data-open-app-dialog="${mode}"]`)?.focus();
    if (typeof afterClose === "function") afterClose();
  }, instant);
}

function applyAppFilter() {
  const query = prototype.appQuery.trim().toLowerCase();
  let visible = 0;
  document.querySelectorAll(".app-row").forEach((row) => {
    const match = !query || row.dataset.searchValue.includes(query);
    row.hidden = !match;
    if (match) visible += 1;
  });
  const empty = document.querySelector(".apps-filter-empty");
  if (empty) empty.hidden = visible !== 0;
}

function handleAppListKeys(event) {
  if (!["ArrowDown", "ArrowUp", "Home", "End"].includes(event.key)) return;
  const rows = [...event.currentTarget.querySelectorAll(".app-row:not([hidden])")];
  if (!rows.length) return;
  event.preventDefault();
  const current = rows.indexOf(document.activeElement);
  const target = event.key === "Home" ? rows[0] : event.key === "End" ? rows.at(-1) : event.key === "ArrowDown" ? rows[(current + 1 + rows.length) % rows.length] : rows[(current - 1 + rows.length) % rows.length];
  prototype.selectedApp = target.dataset.appId;
  prototype.appSection = "overview";
  render();
  document.querySelector(`[data-app-id="${prototype.selectedApp}"]`)?.focus();
}

function handleAppTabKeys(event) {
  if (!["ArrowLeft", "ArrowRight", "Home", "End"].includes(event.key)) return;
  const tabs = [...event.currentTarget.querySelectorAll("[data-app-section]:not([aria-disabled=\"true\"])")];
  event.preventDefault();
  const current = tabs.findIndex((tab) => tab.dataset.appSection === prototype.appSection);
  const target = event.key === "Home" ? tabs[0] : event.key === "End" ? tabs.at(-1) : event.key === "ArrowRight" ? tabs[(current + 1) % tabs.length] : tabs[(current - 1 + tabs.length) % tabs.length];
  prototype.appSection = target.dataset.appSection;
  prototype.appSubstate = "default";
  render();
  document.querySelector(`[data-app-section="${prototype.appSection}"]`)?.focus();
}

function finishTransientExit(surface, backdrop, onFinish, instant = false) {
  if (!surface || instant) {
    onFinish();
    return;
  }
  let finished = false;
  const finish = () => {
    if (finished) return;
    finished = true;
    surface.removeEventListener("transitionend", onTransitionEnd);
    onFinish();
  };
  const onTransitionEnd = (event) => {
    if (event.target === surface && ["opacity", "transform"].includes(event.propertyName)) finish();
  };
  surface.addEventListener("transitionend", onTransitionEnd);
  surface.classList.add("is-closing");
  backdrop?.classList.add("is-closing");
  window.setTimeout(finish, 280);
}

function closeDiscoverDrawer(instantRequest = false) {
  const instant = instantRequest === true;
  const projectId = prototype.selectedProject;
  prototype.discoverDrawer = null;
  finishTransientExit(document.querySelector(".project-drawer"), document.querySelector(".discover-scrim"), () => {
    render({ suppressMotion: true });
    document.querySelector(`[data-open-project="${projectId}"]`)?.focus() || document.querySelector("#catalog-search")?.focus();
  }, instant);
}

function closeDiscoverFilters(instantRequest = false) {
  const instant = instantRequest === true;
  prototype.discoverFilterOpen = false;
  finishTransientExit(document.querySelector(".filter-popover"), null, () => {
    render({ suppressMotion: true });
    document.querySelector("[data-toggle-filters]")?.focus();
  }, instant);
}

function resetDiscoverFilters() {
  prototype.condition = "default";
  prototype.discoverQuery = "";
  prototype.discoverQuickFilter = "all";
  prototype.discoverFilters = { category: "all", license: "all", architecture: "all", warnings: false };
  render();
  document.querySelector("#catalog-search")?.focus();
}

function applyCatalogFilters() {
  const query = prototype.discoverQuery.trim().toLowerCase();
  const filters = prototype.discoverFilters;
  let visible = 0;
  document.querySelectorAll(".catalog-card").forEach((card) => {
    const match = (!query || card.dataset.searchValue.includes(query))
      && (prototype.discoverQuickFilter === "all" || card.dataset.capability === prototype.discoverQuickFilter)
      && (filters.category === "all" || card.dataset.category === filters.category)
      && (filters.license === "all" || card.dataset.license === filters.license)
      && (filters.architecture === "all" || card.dataset.architectures.split(" ").includes(filters.architecture))
      && (!filters.warnings || card.dataset.warning === "true");
    card.hidden = !match;
    if (match) visible += 1;
  });
  const count = document.querySelector("#catalog-visible");
  if (count) count.textContent = visible;
  const noResults = document.querySelector("#catalog-no-results");
  if (noResults) noResults.hidden = visible !== 0;
  // Nothing to page through when nothing matches; a narrowed list says what it counts.
  const pagination = document.querySelector("#catalog-pagination");
  if (pagination) pagination.hidden = visible === 0;
  const narrowed = Boolean(query) || prototype.discoverQuickFilter !== "all" || filters.category !== "all" || filters.license !== "all" || filters.architecture !== "all" || filters.warnings;
  const pageCount = document.querySelector("#catalog-page-count");
  if (pageCount) pageCount.textContent = narrowed
    ? `Showing ${visible} matching ${visible === 1 ? "project" : "projects"} · Page 1`
    : `Showing ${visible} of ${fixtures.derived.catalogSummary.total.toLocaleString()} · Page 1`;
}

// Keeps an anchored popover wholly on screen, scrolling the workspace only as far as needed.
function revealPopover(element) {
  if (!element) return;
  const reduce = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
  element.scrollIntoView({ block: "nearest", inline: "nearest", behavior: reduce ? "auto" : "smooth" });
}

function parsedConnectAddress() {
  const input = document.querySelector("#connect-address");
  try {
    const parsed = new URL(input.value.trim());
    if (!["http:", "https:"].includes(parsed.protocol)) throw new Error("protocol");
    input.removeAttribute("aria-invalid");
    return parsed;
  } catch {
    input.setAttribute("aria-invalid", "true");
    const feedback = document.querySelector("#connect-feedback");
    feedback.hidden = false;
    feedback.className = "address-feedback danger";
    feedback.textContent = "Enter a complete HTTP or HTTPS address, including the protocol.";
    input.focus();
    return null;
  }
}

function checkConnectAddress() {
  const address = parsedConnectAddress();
  if (!address) return;
  const feedback = document.querySelector("#connect-feedback");
  feedback.hidden = false;
  feedback.className = "address-feedback success";
  feedback.textContent = `Address format is valid for ${address.host}. A real reachability check remains advisory.`;
}

function saveConnectedApp(event) {
  event.preventDefault();
  const address = parsedConnectAddress();
  const name = document.querySelector("#connect-name");
  if (!name.value.trim()) {
    name.setAttribute("aria-invalid", "true");
    const feedback = document.querySelector("#connect-feedback");
    feedback.hidden = false;
    feedback.className = "address-feedback danger";
    feedback.textContent = "Give this app a name you will recognise in My Apps.";
    name.focus();
    return;
  }
  if (!address) return;
  showToast("Linked app saved", `${name.value.trim()} would be added without Local Store managing its server.`);
  const existing = fixtures.contract.apps.find((app) => app.id === prototype.selectedProject);
  if (existing) prototype.selectedApp = existing.id;
  // Production selects the new row. The prototype has no row to add, so it says so
  // instead of silently selecting whichever app happened to be selected before.
  prototype.savedLinkName = existing ? null : name.value.trim();
  navigate("my-apps");
}

function filterCards(selector, query, countSelector) {
  const normalized = query.trim().toLowerCase();
  let visible = 0;
  document.querySelectorAll(selector).forEach((item) => {
    const match = item.dataset.searchValue.includes(normalized);
    item.hidden = !match;
    if (match) visible += 1;
  });
  const count = countSelector ? document.querySelector(countSelector) : null;
  if (count) count.textContent = visible;
}

function showToast(title, detail, instant = false) {
  const toast = document.createElement("div");
  toast.className = `toast${instant ? " instant" : ""}`;
  toast.innerHTML = `<strong>${escapeHtml(title)}</strong><br>${escapeHtml(detail)}`;
  toastRegion.replaceChildren(toast);
  window.setTimeout(() => {
    toast.style.opacity = "0";
    toast.style.transform = "translateY(100%)";
    window.setTimeout(() => toast.remove(), 240);
  }, 3200);
}

function setPanel(open, instant = false) {
  panelTrigger.setAttribute("aria-expanded", String(open));
  if (open) {
    prototype.panelOpen = true;
    panel.hidden = false;
    scrim.hidden = false;
    appShell.inert = true;
    document.querySelector(".skip-link").inert = true;
    panelClose.focus();
    return;
  }
  prototype.panelOpen = false;
  finishTransientExit(panel, scrim, () => {
    panel.hidden = true;
    scrim.hidden = true;
    panel.classList.remove("is-closing");
    scrim.classList.remove("is-closing");
    appShell.inert = false;
    document.querySelector(".skip-link").inert = false;
    panelTrigger.focus();
  }, instant);
}

panelTrigger.addEventListener("click", () => setPanel(true));
panelClose.addEventListener("click", () => setPanel(false));
scrim.addEventListener("click", () => setPanel(false));
screenJump.addEventListener("change", () => navigate(screenJump.value));
document.querySelectorAll("[data-state]").forEach((button) => button.addEventListener("click", () => {
  if (!conditions.includes(button.dataset.state)) return;
  prototype.condition = button.dataset.state;
  if (prototype.screen === "install" && button.dataset.state === "default") prototype.installOutcome = null;
  render();
}));
overviewConceptToggle.addEventListener("change", () => {
  prototype.showOverviewConcepts = overviewConceptToggle.checked;
  render();
});

window.addEventListener("hashchange", () => {
  const route = location.hash.slice(1);
  if (screens[route] && route !== prototype.screen) {
    prototype.screen = route;
    render({ focusHeading: true });
  }
});

// Every aria-modal surface keeps Tab inside itself. The app dialog had its own
// trap; the prototype panel and the Discover drawer let focus fall out of the
// document, and the panel could reach the skip link behind it.
const focusableWithin = (root) => [...root.querySelectorAll('a[href], button:not([disabled]), input:not([disabled]):not([type="hidden"]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])')]
  .filter((element) => !element.closest("[hidden], [inert]") && element.getClientRects().length);
const openModal = () => (prototype.panelOpen && panel)
  || (prototype.appDialog && document.querySelector(".app-dialog"))
  || (prototype.discoverDrawer && document.querySelector(".project-drawer"))
  || null;

document.addEventListener("keydown", (event) => {
  const modal = event.key === "Tab" ? openModal() : null;
  if (modal) {
    const controls = focusableWithin(modal);
    if (controls.length) {
      const current = controls.indexOf(document.activeElement);
      const target = event.shiftKey ? controls[(current - 1 + controls.length) % controls.length] : controls[(current + 1) % controls.length];
      event.preventDefault();
      target.focus();
      return;
    }
  }
  if (event.key === "Escape") {
    if (prototype.panelOpen) {
      event.preventDefault();
      setPanel(false, true);
      return;
    }
    if (prototype.appDialog) {
      event.preventDefault();
      closeAppDialog(null, true);
      return;
    }
    if (prototype.discoverDrawer) {
      event.preventDefault();
      closeDiscoverDrawer(true);
      return;
    }
    if (prototype.discoverFilterOpen) {
      event.preventDefault();
      closeDiscoverFilters(true);
      return;
    }
  }
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "k") {
    event.preventDefault();
    showToast("Global search concept", "Saved apps, catalog projects, and commands will share this instant palette.", true);
  }
});

const previewParams = new URLSearchParams(location.search);
// ?stress=1 replaces the friendly fixture strings with worst-case real content so
// Phase 15 can verify the layout against it: long names, deep URLs, a digest-pinned
// image, verbatim Docker errors, and real catalog entries with extreme icon shapes.
if (previewParams.get("stress") === "1") {
  const contract = fixtures.contract;
  contract.catalog.push(...stressCatalog.filter((entry) => !contract.catalog.some((item) => item.id === entry.id)));
  const app = (id) => contract.apps.find((item) => item.id === id);
  Object.assign(app("memos"), { display_name: "Memos — Family Notes, Recipes & Household Archive", launch_url: "http://127.0.0.1:5230/explore/shared-with-family?view=timeline&tag=recipes" });
  Object.assign(app("immich"), { display_name: "Immich Photo Library (Family NAS, Upstairs Office)", launch_url: "https://photos.upstairs-office.internal.home.arpa:2283/albums/shared-with-everyone" });
  Object.assign(app("actual-budget"), { display_name: "Actual Budget — Shared Household Finances 2026" });
  Object.assign(app("linkding"), { status_error: "Error response from daemon: driver failed programming external connectivity on endpoint local-store-linkding (4f2a1c9e7b3d): Bind for 127.0.0.1:9090 failed: port is already allocated" });
  contract.recipe.image = "ghcr.io/usememos/memos:0.30.0-rc.2-alpine3.20@sha256:9f2c1b4e7d8a6f5e3c2b1a0987654321fedcba9876543210abcdef0123456789";
  for (const error of Object.values(contract.installErrors)) {
    error.detail += " Docker reported: Error response from daemon: failed to set up container networking: driver failed programming external connectivity on endpoint local-store-memos (9f2c1b4e7d8a): Bind for 127.0.0.1:5230 failed: port is already allocated.";
  }
}
const initialCondition = previewParams.get("state");
if (conditions.includes(initialCondition)) prototype.condition = initialCondition;
prototype.showOverviewConcepts = previewParams.get("concepts") === "1";
prototype.discoverQuery = previewParams.get("q") || "";
prototype.discoverFilterOpen = previewParams.get("filters") === "1";
const previewCapability = previewParams.get("capability");
if (["all", "install", "connect", "explore"].includes(previewCapability)) prototype.discoverQuickFilter = previewCapability;
const previewProject = previewParams.get("project");
if (fixtures.contract.catalog.some((project) => project.id === previewProject)) prototype.selectedProject = previewProject;
const previewDrawer = previewParams.get("drawer");
if (["detail", "connect"].includes(previewDrawer)) prototype.discoverDrawer = previewDrawer;
const previewApp = previewParams.get("app");
if (fixtures.contract.apps.some((app) => app.id === previewApp)) prototype.selectedApp = previewApp;
const previewAppSection = previewParams.get("section");
if (["overview", "logs", "manage"].includes(previewAppSection)) prototype.appSection = previewAppSection;
const previewAppSubstate = previewParams.get("substate");
if (["default", "loading", "empty", "failure"].includes(previewAppSubstate)) prototype.appSubstate = previewAppSubstate;
const previewAppDialog = previewParams.get("dialog");
if (["uninstall", "delete-data", "remove"].includes(previewAppDialog)) prototype.appDialog = previewAppDialog;
const previewInstallPort = Number(previewParams.get("port"));
if (Number.isInteger(previewInstallPort) && previewInstallPort > 0) prototype.installPort = previewInstallPort;
const previewFirstRunStep = previewParams.get("step");
if (["welcome", "choose", "ready"].includes(previewFirstRunStep)) prototype.firstRunStep = previewFirstRunStep;
const previewPreflight = previewParams.get("preflight");
if (["checking", "ready", "missing"].includes(previewPreflight)) prototype.firstRunPreflight = previewPreflight;
const previewStarter = previewParams.get("starter");
if (fixtures.contract.starterRecipes.some((recipe) => recipe.id === previewStarter)) {
  prototype.firstRunRecipe = previewStarter;
  prototype.installRecipe = previewStarter;
  if (!previewParams.has("port")) prototype.installPort = starterRecipe(previewStarter).host_port;
}
const previewCompletion = previewParams.get("completion");
if (["installed", "connected"].includes(previewCompletion)) prototype.firstRunCompletion = previewCompletion;
const previewActivityFilter = previewParams.get("activity-filter");
if (["all", "install", "lifecycle", "failed"].includes(previewActivityFilter)) prototype.activityFilter = previewActivityFilter;
const previewActivityApp = previewParams.get("activity-app");
if (previewActivityApp === "all" || fixtures.contract.apps.some((app) => app.id === previewActivityApp)) prototype.activityApp = previewActivityApp;
prototype.activityQuery = previewParams.get("activity-q") || "";
const previewActivityEvent = previewParams.get("event");
if (fixtures.concept.events.some((event) => event.operation_id === previewActivityEvent)) prototype.activityExpanded = previewActivityEvent;
const previewRecoveryMode = previewParams.get("recovery");
if (["candidate", "no-containers", "confirm-keep", "confirm-delete", "working", "success-keep", "success-delete", "mismatch", "scan-failure", "loading", "empty"].includes(previewRecoveryMode)) prototype.recoveryMode = previewRecoveryMode;
const previewInstallStage = previewParams.get("stage");
if ([...fixtures.contract.installStages.map((stage) => stage.id), "rolling_back"].includes(previewInstallStage)) prototype.installStage = previewInstallStage;
const previewInstallFailure = previewParams.get("failure");
if (previewInstallFailure in fixtures.contract.installErrors) prototype.installFailure = previewInstallFailure;
prototype.installOutcome = previewParams.get("outcome") === "cancelled" ? "cancelled" : null;
prototype.installSetupStudy = previewParams.get("setup") === "fields";
prototype.installAdvanced = previewParams.get("evidence") === "1";
const initialRoute = location.hash.slice(1);
prototype.screen = screens[initialRoute] ? initialRoute : "overview";
if (!initialRoute) history.replaceState(null, "", "#overview");
render();
if (prototype.screen === "discover" && prototype.discoverFilterOpen) revealPopover(document.querySelector("#catalog-filter-panel"));
