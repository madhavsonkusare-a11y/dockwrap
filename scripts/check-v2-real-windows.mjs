#!/usr/bin/env node
// Runs the V2 layout probe at the window sizes Local Store actually gets, which
// are smaller than the 1280x800 the design was verified at:
//
//   1180x760   the launcher's own default window (src/main.rs inner_size)
//   1366x688   a 1366x768 panel at 100%, maximised, minus taskbar and title bar
//   1280x640   a 1920x1080 panel at 150% (Windows' default), maximised, likewise
//
// This is a report. 1280x640 is also gated in check-v2-visual.mjs (the short sweep),
// and the 1180x760 launcher default is a Phase 16 decision.
// Serve the repository root first:  python -m http.server 8765 --bind 127.0.0.1
import { writeFileSync } from "node:fs";
import { chromium } from "@playwright/test";
import { probeLayout } from "./v2-layout-probe.mjs";
import { base, states } from "./v2-states.mjs";

const windows = [
  { label: "launcher default 1180x760", viewport: { width: 1180, height: 760 }, scale: 1 },
  { label: "1366x768 laptop, maximised", viewport: { width: 1366, height: 688 }, scale: 1 },
  { label: "1920x1080 laptop at 150%, maximised", viewport: { width: 1280, height: 640 }, scale: 1.5 },
];
const browser = await chromium.launch({ channel: "chrome", headless: true });
const results = [];
try {
  for (const win of windows) {
    const context = await browser.newContext({ viewport: win.viewport, deviceScaleFactor: win.scale, reducedMotion: "reduce" });
    const page = await context.newPage();
    const found = { overflow: [], truncation: [], overlap: [], obscured: [], dialogs: [], commitBelowFold: [] };
    for (const [name, suffix, setup] of states) {
      await page.goto("about:blank");
      await page.goto(`${base}${suffix}`, { waitUntil: "networkidle" });
      await page.evaluate(() => document.fonts.ready);
      if (setup) await setup(page);
      const layout = await page.evaluate(probeLayout);
      if (layout.overflowX) found.overflow.push(name);
      for (const t of layout.truncated) if (!t.recoverable) found.truncation.push(`${name}: ${t.where} "${t.text.slice(0, 50)}"`);
      for (const o of layout.overlaps) found.overlap.push(`${name}: ${o}`);
      for (const o of layout.obscured) found.obscured.push(`${name}: ${o}`);
      for (const d of layout.dialogs) if (!d.fits || d.hiddenActions.length) found.dialogs.push(`${name}: fits=${d.fits} hidden=[${d.hiddenActions.join(", ")}] height=${d.height}`);
      // In a focused task the commit belongs on screen without scrolling.
      if (/^(install|recovery|first-run):/.test(name)) for (const b of layout.belowFold) found.commitBelowFold.push(`${name}: ${b}`);
    }
    results.push({ window: win.label, viewport: `${win.viewport.width}x${win.viewport.height}`, devicePixelRatio: win.scale, counts: Object.fromEntries(Object.entries(found).map(([k, v]) => [k, v.length])), found });
    await context.close();
  }
} finally {
  await browser.close();
}
writeFileSync("docs/design/v2/visual-real-windows.json", `${JSON.stringify(results, null, 2)}\n`);
for (const r of results) console.log(`${r.window.padEnd(38)} ${r.viewport.padEnd(10)} ${JSON.stringify(r.counts)}`);
