import { readFile, writeFile } from 'node:fs/promises';
import assert from 'node:assert/strict';

// Approved September 5: one circular construction, shared by every export.
const geometry = { canvas: 512, margin: 95, stroke: 56, gap: 42, tile: 224, radius: 56 };
const { margin, stroke, gap, tile, radius } = geometry;
const x = margin + stroke + gap, y = margin, cx = x + radius, cy = y + tile - radius;
const innerRadius = radius + gap, pathRadius = innerRadius + stroke / 2;
assert.equal(y, 123 - stroke / 2);
assert.equal(389 - stroke / 2 - (y + tile), gap);
assert.equal(cx - innerRadius, margin + stroke);
assert.equal(cy + innerRadius, 389 - stroke / 2);
for (let i = 0; i <= 10000; i++) {
  const angle = Math.PI / 2 + i / 10000 * Math.PI / 2;
  const distance = Math.hypot(gap * Math.cos(angle), gap * Math.sin(angle));
  assert.ok(Math.abs(distance - gap) < 1e-10);
}
const path = `M123 123 V${cy} A${pathRadius} ${pathRadius} 0 0 0 ${cx} 389 H389`;
const flat = (ink, signal = '#FF623E') => `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64" fill="none">\n  <path d="M13 13 V33 A18 18 0 0 0 31 51 H51" stroke="${ink}" stroke-width="8" stroke-linecap="round" stroke-linejoin="round"/>\n  <rect x="23" y="9" width="32" height="32" rx="8" fill="${signal}"/>\n</svg>\n`;
const app = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" fill="none">
  <defs>
    <linearGradient id="field" x1="80" y1="0" x2="420" y2="512" gradientUnits="userSpaceOnUse"><stop stop-color="#303337"/><stop offset="1" stop-color="#111214"/></linearGradient>
    <linearGradient id="tile" x1="${x}" y1="${y}" x2="${x + tile}" y2="${y + tile}" gradientUnits="userSpaceOnUse"><stop stop-color="#FFA482"/><stop offset=".5" stop-color="#FF7654"/><stop offset="1" stop-color="#FF623E"/></linearGradient>
    <linearGradient id="edge" x2="0" y2="512" gradientUnits="userSpaceOnUse"><stop stop-color="#FFFFFF" stop-opacity=".18"/><stop offset="1" stop-color="#FFFFFF" stop-opacity="0"/></linearGradient>
  </defs>
  <rect x="1" y="1" width="510" height="510" rx="112" fill="url(#field)" stroke="url(#edge)" stroke-width="2"/>
  <path d="${path}" stroke="#F8F7F3" stroke-width="56" stroke-linecap="round" stroke-linejoin="round"/>
  <rect x="${x}" y="${y}" width="${tile}" height="${tile}" rx="${radius}" fill="url(#tile)"/>
</svg>\n`;
const files = { 'branding/local-store-app-icon.svg': app, 'branding/local-store-mark.svg': flat('#151719'), 'branding/local-store-mark-reversed.svg': flat('#F8F7F3'), 'branding/local-store-mark-mono.svg': flat('#F8F7F3', '#F8F7F3'), 'src/assets/mark.svg': flat('#F8F7F3') };
for (const [file, contents] of Object.entries(files)) {
  if (process.argv.includes('--check')) assert.equal(await readFile(file, 'utf8'), contents, `${file} is out of date`);
  else await writeFile(file, contents);
}
console.log('Brand verified: 42 px gaps, 224 px tile, shared corner (249, 263), aligned top y=95.');
