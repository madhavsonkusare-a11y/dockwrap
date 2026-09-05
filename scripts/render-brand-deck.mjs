import { chromium } from '@playwright/test';
import { mkdir } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';

const output = new URL('../test-results/branding/', import.meta.url);
await mkdir(output, { recursive: true });
const browser = await chromium.launch({ channel: 'chromium' });
try {
  const page = await browser.newPage({ viewport: { width: 1280, height: 900 }, deviceScaleFactor: 1 });
  await page.goto(new URL('../branding/brand-deck.html', import.meta.url).href);
  await page.evaluate(() => document.fonts.ready);
  for (const id of ['foundation', 'geometry', 'reduction', 'palette', 'product']) {
    await page.locator(`#${id}`).screenshot({ path: fileURLToPath(new URL(`${id}.png`, output)) });
  }
  console.log(`Brand deck previews: ${fileURLToPath(output)}`);
} finally { await browser.close(); }
