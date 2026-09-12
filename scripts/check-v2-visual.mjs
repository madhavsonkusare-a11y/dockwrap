#!/usr/bin/env node
// Phase 15 laptop visual verification for the V2 prototype.
//
// Serve the repository root first:  python -m http.server 8765 --bind 127.0.0.1
// Then run:                          node scripts/check-v2-visual.mjs
//
// Captures every state at 1440x900 and 1280x800, proves the captures are
// deterministic, re-renders every state with worst-case real content (?stress=1),
// renders every state in a short window (1280x640 at 150%: a maximised 1920x1080
// laptop at Windows' default scaling), renders key screens at 125% / 150% / 200%
// display scaling and at the CSS viewports real Windows laptops produce, and checks
// interaction states.
//
// Hard failures (non-zero exit):
//   overflow      the document scrolls horizontally
//   truncation    text is cut off with no way to read the full value
//   overlap       two interactive controls overlap
//   obscured      a primary action is covered by something else
//   dialogs       a dialog or drawer does not fit, or hides a commit / irreversible action
//   determinism   two captures of the same state differ
//   commitBelowFold  in the short window, a focused task's commit is off screen
//   states        hover / active / focus / selected / disabled / busy are not distinct
//
// Writes docs/design/v2/visual-manifest.json (SHA-256 of every normal capture) and
// docs/design/v2/visual-report.json. Full-resolution PNGs go to .cache/v2-visual/.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { chromium } from "@playwright/test";
import { probeLayout } from "./v2-layout-probe.mjs";
import { base, states, viewports, withParam } from "./v2-states.mjs";

const cache = ".cache/v2-visual";
const quick = process.argv.includes("--quick");
const sha = (file) => createHash("sha256").update(readFileSync(file)).digest("hex");
const slug = (name) => name.replace(/[:/]/g, "-");

// ---------------- run ----------------
mkdirSync(cache, { recursive: true });
const browser = await chromium.launch({ channel: "chrome", headless: true });
const report = { generated: new Date().toISOString().slice(0, 10), passes: {}, findings: { overflow: [], truncation: [], glyphClip: [], overlap: [], obscured: [], dialogs: [], determinism: [], states: [], commitBelowFold: [] }, review: { belowFold: [], recoverableTruncation: [], scaling: [] } };
const manifest = {};

async function load(page, suffix, setup) {
  await page.goto("about:blank");
  await page.goto(`${base}${suffix}`, { waitUntil: "networkidle" });
  await page.evaluate(() => document.fonts.ready);
  if (setup) await setup(page);
  // Decode every image first. Chrome decodes off-thread, and under load a logo at the
  // viewport edge was once captured mid-decode (7 pixels differed).
  await page.evaluate(() => Promise.all([...document.images].map((img) => (img.complete ? img.decode() : new Promise((resolve) => { img.onload = img.onerror = resolve; })).catch(() => {}))));
  await page.evaluate(() => new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve))));
}

// Each context first visits the pages that reference every logo, so later captures
// decode logos from a warm cache before first paint, as the app's bundled assets do.
// From a cold cache Chrome may repaint a strip around a late-decoding logo: stable,
// visually identical, but not byte-identical to a warm render.
async function warm(page) {
  for (const suffix of ["?stress=1#discover", "#my-apps", "?step=choose&preflight=ready#first-run"]) await load(page, suffix);
}

function record(mode, viewportLabel, name, layout) {
  const where = `${mode} ${viewportLabel} ${name}`;
  if (layout.overflowX) report.findings.overflow.push(where);
  for (const t of layout.truncated) (t.recoverable ? report.review.recoverableTruncation : report.findings.truncation).push(`${where}: ${t.kind} ${t.where} "${t.text}"`);
  for (const o of layout.overlaps) report.findings.overlap.push(`${where}: ${o}`);
  for (const o of layout.obscured) report.findings.obscured.push(`${where}: ${o}`);
  for (const d of layout.dialogs) if (!d.fits || d.hiddenActions.length) report.findings.dialogs.push(`${where}: ${d.dialog} fits=${d.fits} hidden=[${d.hiddenActions.join(", ")}] height=${d.height}`);
  for (const b of layout.belowFold) report.review.belowFold.push(`${where}: ${b}`);
  // In a focused task the commit belongs on screen without scrolling, down to 1280x640.
  if (mode === "short" && /^(install|recovery|first-run):/.test(name)) for (const b of layout.belowFold) report.findings.commitBelowFold.push(`${where}: ${b}`);
  for (const g of layout.glyphClip) report.findings.glyphClip.push(`${where}: ${g.where} "${g.text}" shaved ${g.px}px (font ${g.fontSize} / line ${g.lineHeight})`);
}

async function sweep(mode, viewportList, stateList, { capture = true, deviceScaleFactor = 1, param = null } = {}) {
  let count = 0;
  for (const viewport of viewportList) {
    const label = `${viewport.width}x${viewport.height}${deviceScaleFactor !== 1 ? `@${deviceScaleFactor}x` : ""}`;
    const context = await browser.newContext({ viewport, deviceScaleFactor, reducedMotion: "reduce" });
    const page = await context.newPage();
    await warm(page);
    const dir = `${cache}/${mode}-${label}`;
    mkdirSync(dir, { recursive: true });
    for (const [name, suffix, setup] of stateList) {
      await load(page, param ? withParam(suffix, param) : suffix, setup);
      record(mode, label, name, await page.evaluate(probeLayout));
      if (capture) {
        const file = `${dir}/${slug(name)}.png`;
        await page.screenshot({ path: file, animations: "disabled", caret: "hide" });
        if (mode === "normal") manifest[`${label}/${slug(name)}`] = sha(file);
      }
      count += 1;
    }
    await context.close();
  }
  const key = `${mode}${deviceScaleFactor !== 1 ? `@${deviceScaleFactor}x` : ""}`;
  report.passes[key] = (report.passes[key] || 0) + count;
}

try {
  const matrix = quick ? states.filter((_, i) => i % 6 === 0) : states;

  // 1. Deterministic capture of every state at both laptop viewports.
  await sweep("normal", viewports, matrix);

  // 2. Determinism: re-capture a sample in a fresh browser context and compare bytes.
  const sample = matrix.filter(([name]) => /discover:default|first-run:welcome-ready|overview:concepts|overview:default|my-apps:delete-data|install:stage-starting_containers|recovery:confirm-delete|activity:expanded|settings:default|first-run:choose|panel:open|install:fail-port_in_use/.test(name));
  for (const viewport of viewports) {
    const label = `${viewport.width}x${viewport.height}`;
    const context = await browser.newContext({ viewport, reducedMotion: "reduce" });
    const page = await context.newPage();
    await warm(page);
    mkdirSync(`${cache}/determinism-${label}`, { recursive: true });
    for (const [name, suffix, setup] of sample) {
      await load(page, suffix, setup);
      const file = `${cache}/determinism-${label}/${slug(name)}.png`;
      await page.screenshot({ path: file, animations: "disabled", caret: "hide" });
      if (sha(file) !== manifest[`${label}/${slug(name)}`]) report.findings.determinism.push(`${label} ${name}`);
    }
    await context.close();
  }
  report.passes.determinismSample = sample.length * viewports.length;

  // 3. Worst-case real content: long names, deep URLs, digest-pinned images, verbatim Docker errors.
  await sweep("stress", viewports, matrix, { capture: true, param: "stress=1" });

  // 3b. Short window: every state at 1280x640 CSS px and 1.5x, the maximised
  //     1920x1080 laptop at 150%. All probe checks apply, plus commitBelowFold.
  await sweep("short", [{ width: 1280, height: 640 }], matrix, { deviceScaleFactor: 1.5 });

  // 4. Display scaling. Same CSS viewport at 125% and 150%, plus the CSS viewports
  //    that common Windows laptop panels actually produce at their default scaling.
  const keyScreens = matrix.filter(([name]) => /^(overview|discover|my-apps|activity|settings|install):default$|first-run:welcome-ready|first-run:choose|recovery:candidate|my-apps:delete-data|discover:connect|install:fail-port_in_use/.test(name));
  for (const scale of [1.25, 1.5]) await sweep("scaled", [{ width: 1280, height: 800 }], keyScreens, { deviceScaleFactor: scale });
  const panels = [
    { label: "1920x1080 panel at 125%", viewport: { width: 1536, height: 864 }, scale: 1.25 },
    { label: "1920x1080 panel at 150%", viewport: { width: 1280, height: 720 }, scale: 1.5 },
    { label: "2560x1600 panel at 150%", viewport: { width: 1707, height: 1067 }, scale: 1.5 },
    { label: "2880x1800 panel at 200%", viewport: { width: 1440, height: 900 }, scale: 2 },
  ];
  for (const panel of panels) {
    const before = { ...report.findings };
    const counts = Object.fromEntries(Object.entries(before).map(([k, v]) => [k, v.length]));
    await sweep("laptop", [panel.viewport], keyScreens, { deviceScaleFactor: panel.scale });
    const added = Object.fromEntries(Object.entries(report.findings).map(([k, v]) => [k, v.length - counts[k]]).filter(([, n]) => n));
    report.review.scaling.push({ panel: panel.label, cssViewport: `${panel.viewport.width}x${panel.viewport.height}`, devicePixelRatio: panel.scale, newFindings: added });
  }

  // 5. Interaction states: each must be visibly distinct from rest.
  const context = await browser.newContext({ viewport: { width: 1440, height: 900 }, deviceScaleFactor: 2 });
  const page = await context.newPage();
  mkdirSync(`${cache}/states`, { recursive: true });
  const snapshot = (sel) => page.evaluate((s) => {
    const el = document.querySelector(s); const c = getComputedStyle(el);
    return [c.backgroundColor, c.borderTopColor, c.color, c.boxShadow, `${c.outlineStyle} ${c.outlineWidth} ${c.outlineColor}`, c.transform, c.opacity, c.textDecorationLine].join(" | ");
  }, sel);
  // Crop with a margin: focus rings and hover lifts are drawn outside the element box.
  const crop = async (sel, name) => {
    const box = await page.locator(sel).first().boundingBox();
    const pad = 10;
    await page.screenshot({ path: `${cache}/states/${name}.png`, clip: { x: Math.max(0, box.x - pad), y: Math.max(0, box.y - pad), width: box.width + pad * 2, height: box.height + pad * 2 } });
  };
  const tabTo = async (sel) => {
    for (let i = 0; i < 90; i += 1) {
      await page.keyboard.press("Tab");
      if (await page.evaluate((s) => document.activeElement === document.querySelector(s), sel)) return true;
    }
    return false;
  };
  const pressables = [
    ["Primary button", "#overview", ".screen .button.primary"],
    ["Ghost button", "#overview", ".screen .button.ghost"],
    ["Text button", "#overview", ".screen .text-button"],
    ["Icon button", "#overview", ".lab-trigger"],
    ["Navigation item", "#overview", '.nav-item[data-route="discover"]'],
    ["Docker status card", "#overview", ".environment-card"],
    ["Search command", "#overview", ".command-button"],
    ["Catalog compact action", "#discover", ".catalog-card .button.compact"],
    ["Filter chip", "#activity", ".filter-chip:not([aria-pressed='true'])"],
    ["Saved-app row", "#my-apps", ".app-row:not([aria-selected='true'])"],
    ["Starter card", "?step=choose&preflight=ready#first-run", ".starter-card:not(.selected)"],
    ["Danger button", "?section=manage&dialog=delete-data#my-apps", ".app-dialog .button.danger, .app-dialog button.danger"],
  ];
  const prepare = { "Danger button": async () => { await page.fill("#delete-confirmation", "Memos"); } };
  for (const [label, suffix, sel] of pressables) {
    await load(page, suffix);
    if (prepare[label]) await prepare[label]();
    const exists = await page.$(sel);
    if (!exists) { report.findings.states.push(`${label}: selector not found (${sel})`); continue; }
    await page.locator(sel).first().scrollIntoViewIfNeeded();
    const rest = await snapshot(sel);
    const box = await page.locator(sel).first().boundingBox();
    await crop(sel, `${slug(label)}-1-rest`);
    await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2); await page.waitForTimeout(260);
    const hover = await snapshot(sel);
    await crop(sel, `${slug(label)}-2-hover`);
    await page.mouse.down(); await page.waitForTimeout(140);
    const active = await snapshot(sel);
    await crop(sel, `${slug(label)}-3-active`);
    await page.mouse.up(); await page.mouse.move(2, 2);
    await load(page, suffix);
    if (prepare[label]) await prepare[label]();
    const reached = await tabTo(sel);
    const focus = reached ? await snapshot(sel) : rest;
    if (reached) await crop(sel, `${slug(label)}-4-focus`);
    if (hover === rest) report.findings.states.push(`${label}: hover is identical to rest`);
    if (active === rest) report.findings.states.push(`${label}: active is identical to rest`);
    if (!reached) report.findings.states.push(`${label}: not reachable by keyboard`);
    else if (focus === rest) report.findings.states.push(`${label}: focus is identical to rest`);
  }
  const selections = [
    ["Selected navigation item", "#overview", '.nav-item[aria-current="page"]', '.nav-item:not([aria-current])'],
    ["Selected saved-app row", "#my-apps", '.app-row[aria-selected="true"]', '.app-row[aria-selected="false"]'],
    ["Selected filter chip", "#activity", ".filter-chip[aria-pressed='true']", ".filter-chip[aria-pressed='false']"],
    ["Selected starter card", "?step=choose&preflight=ready#first-run", ".starter-card.selected", ".starter-card:not(.selected)"],
    ["Selected detail tab", "#my-apps", '.detail-tabs [aria-selected="true"]', '.detail-tabs [aria-selected="false"]'],
  ];
  for (const [label, suffix, on, off] of selections) {
    await load(page, suffix);
    if (!(await page.$(on)) || !(await page.$(off))) { report.findings.states.push(`${label}: selector not found`); continue; }
    await page.locator(on).first().scrollIntoViewIfNeeded();
    await crop(on, slug(label));
    if ((await snapshot(on)) === (await snapshot(off))) report.findings.states.push(`${label}: selected looks identical to unselected`);
  }
  // Disabled: the deletion button stays inert until the confirmation matches.
  await load(page, "?recovery=confirm-delete#recovery");
  const disabled = await page.evaluate(() => {
    const b = [...document.querySelectorAll(".screen button")].find((x) => x.disabled);
    if (!b) return null;
    const c = getComputedStyle(b);
    return { text: b.textContent.trim(), opacity: Number(c.opacity), cursor: c.cursor };
  });
  if (!disabled) report.findings.states.push("Disabled: no disabled control on the delete confirmation");
  else if (!(disabled.opacity < 1 || disabled.cursor === "not-allowed")) report.findings.states.push(`Disabled "${disabled.text}" is not visually distinct`);
  else await crop(".screen button[disabled]", "Disabled-button");
  // Busy: in-progress work is announced and visible.
  for (const [label, suffix] of [["Busy install", "?state=busy&stage=starting_containers#install"], ["Busy app start", "?state=busy#my-apps"], ["Busy recovery", "?recovery=working#recovery"]]) {
    await load(page, suffix);
    const busy = await page.evaluate(() => Boolean(document.querySelector('.screen [aria-busy="true"], .screen [role="status"], .screen [aria-live]')));
    if (!busy) report.findings.states.push(`${label}: no aria-busy, role=status or live region`);
  }
  report.passes.interactionStates = pressables.length * 4 + selections.length + 4;
  await context.close();
} finally {
  await browser.close();
}

writeFileSync("docs/design/v2/visual-manifest.json", `${JSON.stringify({ note: "SHA-256 of each normal capture; regenerate with scripts/check-v2-visual.mjs", captures: manifest }, null, 2)}\n`);
writeFileSync("docs/design/v2/visual-report.json", `${JSON.stringify(report, null, 2)}\n`);
const totals = Object.fromEntries(Object.entries(report.findings).map(([k, v]) => [k, v.length]));
console.log(JSON.stringify({ passes: report.passes, findings: totals, review: { belowFold: report.review.belowFold.length, recoverableTruncation: report.review.recoverableTruncation.length, scaling: report.review.scaling } }, null, 2));
for (const [kind, list] of Object.entries(report.findings)) assert.deepEqual(list, [], `${kind}: ${list.slice(0, 6).join(" ; ")}`);
console.error("\nAll Phase 15 visual checks passed.");
