// The V2 prototype's full state matrix, shared by every verification script so the
// evaluation gates (Phase 14) and visual verification (Phase 15) cover the same set.
// Each entry: [name, url suffix, optional async setup(page)].

export const base = "http://127.0.0.1:8765/docs/design/v2/index.html";
export const viewports = [{ width: 1440, height: 900 }, { width: 1280, height: 800 }];
export const conditions = ["default", "loading", "empty", "busy", "success", "failure"];
const workspaceScreens = ["overview", "discover", "my-apps", "activity", "settings"];

export const states = [
  ...workspaceScreens.flatMap((screen) => conditions.map((state) => [`${screen}:${state}`, `?state=${state}#${screen}`])),
  ["overview:concepts", "?concepts=1#overview"],
  ["discover:detail", "?project=memos&drawer=detail#discover"],
  ["discover:connect", "?project=immich&drawer=connect#discover"],
  ["discover:filters", "?filters=1#discover"],
  ["discover:no-results", "?q=zzzz-no-match#discover"],
  ["my-apps:linked", "?app=immich#my-apps"],
  ["my-apps:error-row", "?app=linkding#my-apps"],
  ["my-apps:unreachable", "?app=actual-budget#my-apps"],
  ["my-apps:logs", "?section=logs#my-apps"],
  ["my-apps:logs-loading", "?section=logs&substate=loading#my-apps"],
  ["my-apps:logs-empty", "?section=logs&substate=empty#my-apps"],
  ["my-apps:logs-failure", "?section=logs&substate=failure#my-apps"],
  ["my-apps:manage", "?section=manage#my-apps"],
  ["my-apps:uninstall", "?section=manage&dialog=uninstall#my-apps"],
  ["my-apps:delete-data", "?section=manage&dialog=delete-data#my-apps"],
  ["my-apps:remove-link", "?app=immich&section=manage&dialog=remove#my-apps"],
  ["activity:failed-filter", "?activity-filter=failed#activity"],
  ["activity:expanded", "?event=op-102#activity"],
  ...conditions.map((state) => [`install:${state}`, `?state=${state}#install`]),
  ["install:evidence", "?evidence=1#install"],
  ["install:setup-fields", "?setup=fields#install"],
  ["install:cancelled", "?outcome=cancelled#install"],
  ...["checking_system", "preparing_files", "validating_recipe", "starting_containers", "waiting_for_health", "saving_app", "rolling_back"]
    .map((stage) => [`install:stage-${stage}`, `?state=busy&stage=${stage}#install`]),
  ...["port_in_use", "prerequisite_unavailable", "invalid_input", "timed_out", "rollback_failed"]
    .map((failure) => [`install:fail-${failure}`, `?state=failure&failure=${failure}#install`]),
  ...["candidate", "no-containers", "confirm-keep", "confirm-delete", "working", "success-keep", "success-delete", "mismatch", "scan-failure", "loading", "empty"]
    .map((mode) => [`recovery:${mode}`, `?recovery=${mode}#recovery`]),
  ...["checking", "ready", "missing"].map((preflight) => [`first-run:welcome-${preflight}`, `?step=welcome&preflight=${preflight}#first-run`]),
  ["first-run:choose", "?step=choose&preflight=ready#first-run"],
  ["first-run:choose-n8n", "?step=choose&preflight=ready&starter=n8n#first-run"],
  ["first-run:ready-installed", "?step=ready&completion=installed#first-run"],
  ["first-run:ready-connected", "?step=ready&completion=connected#first-run"],
  ["panel:open", "#overview", async (page) => { await page.click(".lab-trigger"); await page.waitForSelector("#prototype-panel:not([hidden])"); }],
];

// Adds a query parameter to a state suffix that may or may not already have a query.
export function withParam(suffix, param) {
  const [path, hash = ""] = suffix.split("#");
  const query = path.startsWith("?") ? `${path}&${param}` : `?${param}`;
  return `${query}${hash ? `#${hash}` : ""}`;
}
