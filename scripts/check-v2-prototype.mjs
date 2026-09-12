#!/usr/bin/env node
import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const design = join(root, "docs", "design", "v2");
const required = ["index.html", "tokens.css", "components.css", "prototype.css", "prototype.js", "prototype-fixtures.js", "MOTION-SPEC.md"];
for (const file of required) assert.ok(existsSync(join(design, file)), `missing ${file}`);

const html = readFileSync(join(design, "index.html"), "utf8");
const css = readFileSync(join(design, "prototype.css"), "utf8") + readFileSync(join(design, "components.css"), "utf8");
const controller = readFileSync(join(design, "prototype.js"), "utf8");
const fixturesText = readFileSync(join(design, "prototype-fixtures.js"), "utf8");
const { conditions, fixtures, screens } = await import(pathToFileURL(join(design, "prototype-fixtures.js")));

assert.deepEqual(Object.keys(screens), ["overview", "discover", "my-apps", "activity", "settings", "install", "recovery", "first-run"]);
assert.deepEqual([...conditions], ["default", "loading", "empty", "busy", "success", "failure"]);
assert.match(html, /<nav class="primary-nav"/);
assert.match(html, /<main id="screen"/);
assert.match(html, /aria-live="polite"/);
assert.match(html, /<fieldset class="state-picker">/);
assert.match(html, /role="dialog" aria-modal="true"/);
assert.match(css, /:focus-visible/);
assert.match(css, /prefers-reduced-motion/);
assert.doesNotMatch(css, /transition:\s*all\b/);
assert.doesNotMatch(css, /\bease-in\b/);
assert.doesNotMatch(controller, /location\.reload|window\.open/);
assert.match(controller, /appShell\.inert = true/);
assert.match(controller, /appShell\.inert = false/);
assert.match(controller, /showOverviewConcepts/);
assert.match(controller, /finishTransientExit/);
assert.match(css, /@starting-style/);
assert.match(css, /--v2-ease-drawer/);
assert.match(controller, /URLSearchParams\(location\.search\)/);
assert.match(controller, /discoverDrawer/);
assert.match(controller, /applyCatalogFilters/);
assert.match(controller, /appSection/);
assert.match(controller, /applyAppFilter/);
assert.match(controller, /data-delete-confirmation/);
assert.match(css, /\.app-dialog/);
assert.match(controller, /validInstallPort/);
assert.match(controller, /\["saving_app", "rolling_back"\]/);
assert.match(controller, /installFailure/);
assert.match(css, /\.stage-list/);
assert.match(controller, /data-activity-filter/);
assert.match(controller, /data-settings-action/);
assert.match(controller, /data-recovery-action/);
assert.match(controller, /recovery-confirmation/);
assert.match(css, /\.activity-timeline/);
assert.match(css, /\.system-status/);
assert.match(css, /\.recovery-consequences/);

for (const source of [html, css, controller]) {
  assert.doesNotMatch(source, /(?:src|href|url\()=["']?https?:\/\//i, "prototype has a remote runtime dependency");
  assert.doesNotMatch(source, /mark-grain-strong|Ember Paper/i, "removed brand treatment was referenced");
}

const iconPaths = [
  ...fixtures.contract.apps.map((app) => app.icon),
  ...fixtures.contract.catalog.map((app) => app.icon),
  fixtures.contract.recipe.icon,
  ...fixtures.contract.starterRecipes.map((recipe) => recipe.icon),
];
for (const icon of new Set(iconPaths)) {
  assert.ok(existsSync(resolve(design, icon)), `fixture icon does not exist: ${icon}`);
}

assert.ok(fixtures.contract.doctor.checks.every((check) => ["id", "label", "ok", "detail"].every((field) => field in check)));
assert.ok(fixtures.contract.apps.every((app) => ["id", "catalog_id", "display_name", "launch_url", "runtime", "status"].every((field) => field in app)));
assert.ok(fixtures.contract.apps.some((app) => app.runtime.kind === "compose"));
assert.ok(fixtures.contract.apps.some((app) => app.runtime.kind === "external"));
assert.ok(fixtures.contract.apps.filter((app) => app.runtime.kind === "external").every((app) => !["running", "stopped"].includes(app.status)));
assert.ok(fixtures.contract.logs && Array.isArray(fixtures.contract.logs.memos));
assert.ok(["image", "version", "source_url", "documentation_url", "health_url", "host_port", "container_port", "requirements", "data_directories", "data_storage", "risk_notes", "setup_review"].every((field) => field in fixtures.contract.recipe));
assert.equal(fixtures.contract.starterRecipes.length, 3);
assert.deepEqual(fixtures.contract.starterRecipes.map((recipe) => recipe.id), ["memos", "n8n", "uptime-kuma"]);
assert.ok(fixtures.contract.starterRecipes.every((recipe) => ["id", "display_name", "description", "version", "image", "host_port", "container_port", "data_storage", "icon"].every((field) => field in recipe)));
assert.deepEqual(fixtures.contract.installStages.map((stage) => stage.id), ["checking_system", "preparing_files", "validating_recipe", "starting_containers", "waiting_for_health", "saving_app"]);
assert.ok(["port_in_use", "prerequisite_unavailable", "invalid_input", "timed_out", "rollback_failed"].every((code) => code in fixtures.contract.installErrors));
assert.ok(fixtures.contract.sessionOperations.every((operation) => ["id", "app", "kind", "state", "detail", "time"].every((field) => field in operation)));
assert.match(fixtures.concept.overviewTelemetry.path, /^M/);
assert.ok(["label", "value", "peak", "window", "notice"].every((field) => field in fixtures.concept.overviewTelemetry));
assert.ok(fixtures.contract.catalog.every((project) => ["capability", "license", "architectures", "source", "source_url", "project_url", "updated_at", "warning"].every((field) => field in project)));
assert.ok(fixtures.concept.activityNotice && fixtures.concept.events.length > 0);
assert.ok(fixtures.concept.events.every((event) => ["operation_id", "app_id", "time", "group", "app", "kind", "state", "detail", "stage", "error"].every((field) => field in event)));
assert.ok(fixtures.concept.recoveryCleanupNotice);
assert.match(fixturesText, /contract:/);
assert.match(fixturesText, /derived:/);
assert.match(fixturesText, /concept:/);

console.log(`Verified V2 foundation: ${Object.keys(screens).length} screens, ${conditions.length} conditions, ${new Set(iconPaths).size} offline fixture icons.`);
