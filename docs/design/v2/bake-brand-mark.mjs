import { chromium } from "@playwright/test";
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";

const sourcePath = resolve("docs/design/v2/assets/brand/mark-grain-approved.svg");
const outputPath = resolve("docs/design/v2/assets/brand/mark-grain-baked.png");
const source = await readFile(sourcePath, "utf8");

const browser = await chromium.launch({ channel: "msedge", headless: true });
const page = await browser.newPage({ viewport: { width: 512, height: 512 }, deviceScaleFactor: 1 });
await page.setContent(`<!doctype html><style>
  html,body{width:512px;height:512px;margin:0;background:transparent;overflow:hidden}
  svg{display:block;width:512px;height:512px}
</style>${source}`);
await page.screenshot({ path: outputPath, omitBackground: true });
await browser.close();

console.log(outputPath);
