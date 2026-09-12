#!/usr/bin/env node
// Phase 16 design freeze for Local Store V2.
//
// Records, in docs/design/v2/freeze.json:
//   tokens   every --v2-* custom property, per context (base, narrow laptop, reduced motion)
//   fonts    each @font-face: family, weight, file, SHA-256
//   screens  the navigable screens and every prototype state (scripts/v2-states.mjs)
//
//   node scripts/check-v2-freeze.mjs           fail if the prototype drifted from the freeze
//   node scripts/check-v2-freeze.mjs --write   re-record after an approved change
//
// Production implementation copies tokens.css verbatim; the same check then guards it.
import { createHash } from "node:crypto";
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { states } from "./v2-states.mjs";

const v2 = "docs/design/v2";
const file = `${v2}/freeze.json`;
const source = readFileSync(`${v2}/tokens.css`, "utf8").replace(/\/\*[\s\S]*?\*\//g, "");

function declarations(block) {
  return Object.fromEntries([...block.matchAll(/(--v2-[\w-]+)\s*:\s*([^;]+);/g)].map(([, name, value]) => [name, value.trim().replace(/\s+/g, " ")]));
}
const tokens = {};
for (const match of source.matchAll(/(@media[^{]+)\{\s*:root\s*\{([^}]*)\}\s*\}|:root\s*\{([^}]*)\}/g)) {
  const context = match[1] ? match[1].trim().replace(/\s+/g, " ") : "base";
  tokens[context] = { ...tokens[context], ...declarations(match[2] ?? match[3]) };
}
const fonts = [...source.matchAll(/@font-face\s*\{([^}]*)\}/g)].map(([, body]) => {
  const src = body.match(/url\("([^"]+)"\)/)[1];
  const path = `${v2}/${src}`;
  return {
    family: body.match(/font-family:\s*"([^"]+)"/)[1],
    weight: Number(body.match(/font-weight:\s*(\d+)/)[1]),
    file: src,
    sha256: existsSync(path) ? createHash("sha256").update(readFileSync(path)).digest("hex") : "MISSING",
  };
});
const screens = {};
for (const [name] of states) {
  const [screen, state] = name.split(":");
  (screens[screen] ||= []).push(state);
}
const current = { tokens, fonts, screens };

if (process.argv.includes("--write")) {
  const previous = existsSync(file) ? JSON.parse(readFileSync(file, "utf8")) : {};
  const record = {
    note: "Local Store V2 design freeze. Change only through an approved design decision, then re-run scripts/check-v2-freeze.mjs --write.",
    status: previous.status || "proposed",
    approvedBy: previous.approvedBy || null,
    approvedOn: previous.approvedOn || null,
    recordedOn: new Date().toISOString().slice(0, 10),
    counts: { tokens: Object.values(tokens).reduce((n, t) => n + Object.keys(t).length, 0), contexts: Object.keys(tokens).length, fonts: fonts.length, screens: Object.keys(screens).length, states: states.length },
    ...current,
  };
  writeFileSync(file, `${JSON.stringify(record, null, 2)}\n`);
  console.log(`Recorded ${file}: ${JSON.stringify(record.counts)} (status: ${record.status})`);
  process.exit(0);
}

if (!existsSync(file)) { console.error(`${file} is missing; run with --write.`); process.exit(1); }
const frozen = JSON.parse(readFileSync(file, "utf8"));
const drift = [];
for (const context of new Set([...Object.keys(frozen.tokens), ...Object.keys(tokens)])) {
  const a = frozen.tokens[context] || {}; const b = tokens[context] || {};
  for (const name of new Set([...Object.keys(a), ...Object.keys(b)])) {
    if (!(name in b)) drift.push(`token removed  ${context} ${name}`);
    else if (!(name in a)) drift.push(`token added    ${context} ${name}: ${b[name]}`);
    else if (a[name] !== b[name]) drift.push(`token changed  ${context} ${name}: ${a[name]} -> ${b[name]}`);
  }
}
const fontKey = (f) => `${f.family} ${f.weight}`;
const frozenFonts = new Map(frozen.fonts.map((f) => [fontKey(f), f]));
for (const f of fonts) {
  const was = frozenFonts.get(fontKey(f));
  if (!was) drift.push(`font added     ${fontKey(f)} ${f.file}`);
  else if (was.sha256 !== f.sha256 || was.file !== f.file) drift.push(`font changed   ${fontKey(f)} ${was.file}@${was.sha256.slice(0, 12)} -> ${f.file}@${f.sha256.slice(0, 12)}`);
  frozenFonts.delete(fontKey(f));
}
for (const key of frozenFonts.keys()) drift.push(`font removed   ${key}`);
for (const screen of new Set([...Object.keys(frozen.screens), ...Object.keys(screens)])) {
  const a = new Set(frozen.screens[screen] || []); const b = new Set(screens[screen] || []);
  for (const s of b) if (!a.has(s)) drift.push(`state added    ${screen}:${s}`);
  for (const s of a) if (!b.has(s)) drift.push(`state removed  ${screen}:${s}`);
}
if (drift.length) {
  console.error(`V2 design drifted from ${file} (status: ${frozen.status}):\n  ${drift.join("\n  ")}`);
  process.exit(1);
}
console.log(`V2 matches the ${frozen.status} freeze: ${JSON.stringify(frozen.counts)}`);
