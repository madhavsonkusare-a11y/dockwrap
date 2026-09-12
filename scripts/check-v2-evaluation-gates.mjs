#!/usr/bin/env node
// Phase 14 evaluation gates for the V2 prototype.
//
// Serve the repository root first:  python -m http.server 8765 --bind 127.0.0.1
// Then run:                          node scripts/check-v2-evaluation-gates.mjs
//
// Hard gates (non-zero exit on failure):
//   axe            WCAG 2.0/2.1/2.2 A+AA rules, every state, both laptop viewports
//   focus          a real Tab walk reaches every control, each shows an indicator
//                  with >= 3:1 contrast, and focus never leaves an open modal
//   boundaries     text inputs and selects have a >= 3:1 boundary (WCAG 1.4.11)
//   legibility     no operational text below 13px unless it is a letter-spaced label
//   token drift    raw radius / font-size / colour / spacing values outside the
//                  documented exception list, and any var(--v2-*) that is never defined
//
// Pass --report to print every finding instead of stopping at the summary.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { chromium } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

import { base, states, viewports } from "./v2-states.mjs";

const verbose = process.argv.includes("--report");

// ---------- browser-side probes ----------
const probeLegibility = () => [...document.querySelectorAll("body *")].filter((node) => {
  if (node.closest('[aria-hidden="true"], .svg-sprite') || ![...node.childNodes].some((child) => child.nodeType === Node.TEXT_NODE && child.textContent.trim())) return false;
  const style = getComputedStyle(node);
  if (style.display === "none" || style.visibility === "hidden" || Number(style.opacity) === 0) return false;
  const rect = node.getBoundingClientRect();
  if (!rect.width || !rect.height) return false;
  const size = Number.parseFloat(style.fontSize);
  const spacing = style.letterSpacing === "normal" ? 0 : Number.parseFloat(style.letterSpacing);
  // 0.08em computes to exactly size*0.08, which floating point makes a hair larger.
  return size < 13 && spacing + 0.005 < size * 0.08;
}).map((node) => ({
  where: `${node.tagName.toLowerCase()}${[...node.classList].slice(0, 2).map((c) => "." + c).join("")}${node.parentElement ? " < " + node.parentElement.tagName.toLowerCase() + [...node.parentElement.classList].slice(0, 1).map((c) => "." + c).join("") : ""}`,
  text: node.textContent.trim().replace(/\s+/g, " ").slice(0, 60),
  size: getComputedStyle(node).fontSize,
}));

// Composite an rgba() over the nearest opaque ancestor background, then return WCAG contrast.
const probeNonText = async () => {
  const parse = (value) => {
    const m = value.match(/rgba?\(([^)]+)\)/);
    if (!m) return null;
    const p = m[1].split(/[\s,/]+/).filter(Boolean).map(Number);
    return { r: p[0], g: p[1], b: p[2], a: p.length > 3 ? p[3] : 1 };
  };
  const over = (top, under) => ({ r: top.r * top.a + under.r * (1 - top.a), g: top.g * top.a + under.g * (1 - top.a), b: top.b * top.a + under.b * (1 - top.a), a: 1 });
  const backdrop = (node) => {
    const layers = [];
    for (let n = node; n; n = n.parentElement) {
      const c = parse(getComputedStyle(n).backgroundColor);
      if (c && c.a > 0) { layers.push(c); if (c.a >= 1) break; }
    }
    return layers.reverse().reduce((acc, c) => over(c, acc), { r: 12, g: 10, b: 9, a: 1 });
  };
  const lum = ({ r, g, b }) => [r, g, b].map((v) => { v /= 255; return v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4; }).reduce((s, v, i) => s + v * [0.2126, 0.7152, 0.0722][i], 0);
  const ratio = (a, b) => { const [x, y] = [lum(a), lum(b)].sort((p, q) => q - p); return (x + 0.05) / (y + 0.05); };
  const visible = (el) => { const s = getComputedStyle(el); const r = el.getBoundingClientRect(); return s.display !== "none" && s.visibility !== "hidden" && r.width > 0 && r.height > 0 && !el.closest("[hidden], [inert]"); };

  const boundaryFails = [];
  for (const el of [...document.querySelectorAll('input:not([type="checkbox"]):not([type="radio"]):not([type="hidden"]), select, textarea')].filter(visible)) {
    const s = getComputedStyle(el);
    const bg = backdrop(el.parentElement || el);
    const border = Number.parseFloat(s.borderTopWidth) > 0 ? parse(s.borderTopColor) : null;
    const fill = backdrop(el);
    const edge = border ? ratio(over(border, fill), bg) : 0;
    const surface = ratio(fill, bg);
    const best = Math.max(edge, surface);
    if (best < 3) boundaryFails.push({ control: `${el.tagName.toLowerCase()}${el.id ? "#" + el.id : ""}${el.placeholder ? ` "${el.placeholder.slice(0, 28)}"` : ""}`, best: `${best.toFixed(2)}:1` });
  }
  return { boundaryFails };
};

// Measures whatever the keyboard just focused. Programmatic .focus() does not
// reliably match :focus-visible, so the walk below uses real Tab presses.
const probeActiveFocus = () => {
  const el = document.activeElement;
  if (!el || el === document.body) return { key: "body" };
  const parse = (value) => { const m = value.match(/rgba?\(([^)]+)\)/); if (!m) return null; const p = m[1].split(/[\s,/]+/).filter(Boolean).map(Number); return { r: p[0], g: p[1], b: p[2], a: p.length > 3 ? p[3] : 1 }; };
  const over = (top, under) => ({ r: top.r * top.a + under.r * (1 - top.a), g: top.g * top.a + under.g * (1 - top.a), b: top.b * top.a + under.b * (1 - top.a), a: 1 });
  const backdrop = (node) => { const layers = []; for (let n = node; n; n = n.parentElement) { const c = parse(getComputedStyle(n).backgroundColor); if (c && c.a > 0) { layers.push(c); if (c.a >= 1) break; } } return layers.reverse().reduce((acc, c) => over(c, acc), { r: 12, g: 10, b: 9, a: 1 }); };
  const lum = ({ r, g, b }) => [r, g, b].map((v) => { v /= 255; return v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4; }).reduce((s, v, i) => s + v * [0.2126, 0.7152, 0.0722][i], 0);
  const ratio = (a, b) => { const [x, y] = [lum(a), lum(b)].sort((p, q) => q - p); return (x + 0.05) / (y + 0.05); };
  const s = getComputedStyle(el);
  const bg = backdrop(el.parentElement || el);
  let ring = null;
  if (s.outlineStyle !== "none" && Number.parseFloat(s.outlineWidth) >= 1) ring = parse(s.outlineColor);
  if (!ring && s.boxShadow && s.boxShadow !== "none") { const m = s.boxShadow.match(/rgba?\([^)]+\)/); if (m) ring = parse(m[0]); }
  const modal = [...document.querySelectorAll('[aria-modal="true"]')].find((d) => !d.hidden && getComputedStyle(d).display !== "none");
  const label = `${el.tagName.toLowerCase()}${el.className ? "." + String(el.className).split(" ")[0] : ""} "${(el.getAttribute("aria-label") || el.textContent || el.value || "").trim().replace(/\s+/g, " ").slice(0, 32)}"`;
  if (!el.dataset.v2Probe) el.dataset.v2Probe = String(Math.random()).slice(2);
  return { key: el.dataset.v2Probe, label, ratio: ring ? ratio(over(ring, bg), bg) : 0, escapedModal: Boolean(modal && !modal.contains(el)) };
};

// ---------- run ----------
const browser = await chromium.launch({ channel: "chrome", headless: true });
const results = { axe: [], legibility: [], focus: [], boundaries: [], statesChecked: 0, focusChecked: 0 };
try {
  for (const viewport of viewports) {
    const context = await browser.newContext({ viewport, reducedMotion: "reduce" });
    const page = await context.newPage();
    for (const [name, suffix, setup] of states) {
      await page.goto(`${base}${suffix}`, { waitUntil: "networkidle" });
      await page.evaluate(() => document.fonts.ready);
      if (setup) await setup(page);
      const vp = `${viewport.width}x${viewport.height}`;
      const axe = await new AxeBuilder({ page }).withTags(["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "wcag22aa"]).analyze();
      for (const v of axe.violations) results.axe.push({ state: name, viewport: vp, rule: v.id, impact: v.impact, nodes: v.nodes.slice(0, 3).map((n) => n.target.join(" ")) });
      for (const item of await page.evaluate(probeLegibility)) results.legibility.push({ state: name, viewport: vp, ...item });
      const nonText = await page.evaluate(probeNonText);
      for (const item of nonText.boundaryFails) results.boundaries.push({ state: name, viewport: vp, ...item });
      // Keyboard walk: stop once focus returns to a control already measured.
      const seen = new Set();
      for (let step = 0; step < 80; step += 1) {
        await page.keyboard.press("Tab");
        const f = await page.evaluate(probeActiveFocus);
        if (f.key === "body") continue;
        if (seen.has(f.key)) break;
        seen.add(f.key);
        results.focusChecked += 1;
        if (f.escapedModal) results.focus.push({ state: name, viewport: vp, control: f.label, reason: "focus left an open modal" });
        else if (f.ratio < 3) results.focus.push({ state: name, viewport: vp, control: f.label, reason: f.ratio ? `indicator ${f.ratio.toFixed(2)}:1` : "no focus indicator" });
      }
      results.statesChecked += 1;
    }
    await context.close();
  }
} finally {
  await browser.close();
}

// ---------- static token drift ----------
const read = (file) => readFileSync(resolve("docs/design/v2", file), "utf8");
const css = ["components.css", "prototype.css"].map(read).join("\n");
const allowed = JSON.parse(read("evaluation-exceptions.json"));
const uniq = (list) => [...new Set(list)];
const values = (property) => uniq([...css.matchAll(new RegExp(`(?:^|[;{\\s])${property}:\\s*([^;}\\n]+)`, "g"))].map((m) => m[1].trim()));
// The `font:` shorthand hides sizes from a font-size-only scan; the first pass missed 51 of them.
const shorthandSizes = values("font").flatMap((v) => (v.match(/(?:^|\s)(\d+(?:\.\d+)?px)(?=\/|\s)/) || []).slice(1));
const drift = {
  // 0 is the absence of rounding, not a value on or off the scale.
  radius: values("border-radius").filter((v) => !/var\(/.test(v) && !/^0(px)?$/.test(v) && !allowed.radius.includes(v)),
  fontSize: uniq([...values("font-size"), ...shorthandSizes]).filter((v) => !/var\(|clamp\(|inherit|em$/.test(v) && !allowed.fontSize.includes(v)),
  color: uniq([...css.matchAll(/#[0-9a-fA-F]{3,8}\b/g)].map((m) => m[0].toLowerCase())).filter((v) => !allowed.color.includes(v)),
  spacing: uniq(["margin", "padding", "gap", "margin-top", "margin-bottom", "margin-left", "margin-right", "padding-top", "padding-bottom", "padding-left", "padding-right", "row-gap", "column-gap"]
    .flatMap((p) => values(p)).flatMap((v) => v.split(/\s+/)).filter((v) => /^\d+(\.\d+)?px$/.test(v) && v !== "0px" && ![4, 8, 12, 16, 20, 24, 32, 40, 48, 64].includes(Number.parseFloat(v)) && !allowed.spacing.includes(v))),
};

// An undefined custom property invalidates its whole declaration and the value
// silently falls back to `initial`. Six --v2-space-7 margins rendered as 0 this way.
const defined = new Set([...["tokens.css", "components.css", "prototype.css"].map(read).join("\n").matchAll(/(--v2-[\w-]+)\s*:/g)].map((m) => m[1]));
const referenced = css + read("prototype.js");
const undefinedTokens = uniq([...referenced.matchAll(/var\((--v2-[\w-]+)\s*\)/g)].map((m) => m[1]).filter((name) => !defined.has(name)));

// ---------- report ----------
const legibilityByState = Object.entries(results.legibility.reduce((acc, item) => { acc[`${item.state} ${item.viewport}`] = (acc[`${item.state} ${item.viewport}`] || 0) + 1; return acc; }, {}));
const summary = {
  statesChecked: results.statesChecked,
  focusableControlsChecked: results.focusChecked,
  axeViolations: results.axe.length,
  focusIndicatorFailures: results.focus.length,
  inputBoundaryFailures: results.boundaries.length,
  smallTextNodes: results.legibility.length,
  worstLegibilityStates: legibilityByState.sort((a, b) => b[1] - a[1]).slice(0, 8),
  tokenDrift: drift,
  undefinedTokens,
};
console.log(JSON.stringify(verbose ? { summary, ...results } : summary, null, 2));

assert.equal(results.axe.length, 0, `axe: ${results.axe.slice(0, 5).map((v) => `${v.state} ${v.viewport} ${v.rule}`).join("; ")}`);
assert.equal(results.focus.length, 0, `focus indicator: ${results.focus.slice(0, 5).map((v) => `${v.state} ${v.control} ${v.reason}`).join("; ")}`);
assert.equal(results.boundaries.length, 0, `input boundary: ${results.boundaries.slice(0, 5).map((v) => `${v.state} ${v.control} ${v.best}`).join("; ")}`);
assert.equal(results.legibility.length, 0, `legibility: ${results.legibility.length} sub-13px unspaced text nodes`);
for (const [kind, list] of Object.entries(drift)) assert.deepEqual(list, [], `token drift (${kind}): ${list.join(", ")}`);
assert.deepEqual(undefinedTokens, [], `undefined tokens: ${undefinedTokens.join(", ")}`);
console.error(`\nAll Phase 14 gates passed across ${results.statesChecked} state renders.`);
