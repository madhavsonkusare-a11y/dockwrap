import { chromium } from "@playwright/test";
import { createHash } from "node:crypto";
import { performance } from "node:perf_hooks";

const origin = process.env.BRAND_MARK_ORIGIN ?? "http://127.0.0.1:4174";
const output = process.env.BRAND_MARK_SCREENSHOT ?? "v2-brand-mark-lab.png";
const stressOutput = process.env.BRAND_MARK_STRESS_SCREENSHOT ?? "v2-brand-mark-stress.png";
const failures = [];
const externalRequests = [];
const hash = (buffer) => createHash("sha256").update(buffer).digest("hex");

const browser = await chromium.launch({ channel: "msedge", headless: true });
const context = await browser.newContext({ viewport: { width: 1440, height: 900 }, deviceScaleFactor: 1 });
await context.route("**/*", async (route) => {
  const requestUrl = new URL(route.request().url());
  if (requestUrl.hostname === "127.0.0.1") return route.continue();
  externalRequests.push(requestUrl.href);
  return route.abort("blockedbyclient");
});

const page = await context.newPage();
page.on("pageerror", (error) => failures.push(`pageerror: ${error.message}`));
page.on("requestfailed", (request) => {
  const requestUrl = new URL(request.url());
  if (requestUrl.hostname === "127.0.0.1") failures.push(`requestfailed: ${request.url()}`);
});

await page.goto(`${origin}/docs/design/v2/brand-mark-lab.html`, { waitUntil: "networkidle" });
await page.evaluate(() => document.fonts.ready);
const labMetrics = await page.evaluate(() => ({
  title: document.title,
  viewportWidth: innerWidth,
  documentWidth: document.documentElement.scrollWidth,
  images: [...document.images].map((image) => ({
    source: image.getAttribute("src"),
    complete: image.complete,
    naturalWidth: image.naturalWidth,
    naturalHeight: image.naturalHeight,
  })),
}));

const labStart = performance.now();
const first = await page.screenshot({ path: output, fullPage: true });
const labScreenshotMs = performance.now() - labStart;
await page.reload({ waitUntil: "networkidle" });
await page.evaluate(() => document.fonts.ready);
const second = await page.screenshot({ fullPage: true });

const stressMarkup = Array.from({ length: 256 }, (_, index) =>
  `<img src="${origin}/docs/design/v2/assets/brand/mark-grain-approved.svg" alt="" data-index="${index}">`,
).join("");
const stressStart = performance.now();
await page.setContent(`<!doctype html><style>
  html,body{margin:0;background:#0c0a09}
  main{display:grid;grid-template-columns:repeat(16,48px);gap:8px;width:max-content;padding:24px}
  img{display:block;width:48px;height:48px}
</style><main>${stressMarkup}</main>`, { waitUntil: "load" });
await page.waitForFunction(() => [...document.images].every((image) => image.complete && image.naturalWidth > 0));
const stressLoadMs = performance.now() - stressStart;
const stressShotStart = performance.now();
await page.screenshot({ path: stressOutput, fullPage: true });
const stressScreenshotMs = performance.now() - stressShotStart;

const result = {
  browser: `Microsoft Edge ${browser.version()}`,
  origin,
  offlinePolicy: "non-127.0.0.1 requests blocked",
  externalRequests,
  failures,
  deterministic: Buffer.compare(first, second) === 0,
  screenshotSha256First: hash(first),
  screenshotSha256Second: hash(second),
  labScreenshotMs: Number(labScreenshotMs.toFixed(1)),
  stressMarkCount: 256,
  stressLoadMs: Number(stressLoadMs.toFixed(1)),
  stressScreenshotMs: Number(stressScreenshotMs.toFixed(1)),
  noHorizontalOverflow: labMetrics.documentWidth === labMetrics.viewportWidth,
  allImagesLoaded: labMetrics.images.every((image) => image.complete && image.naturalWidth > 0 && image.naturalHeight > 0),
  imageCount: labMetrics.images.length,
};

await browser.close();
console.log(JSON.stringify(result, null, 2));

if (
  failures.length ||
  externalRequests.length ||
  !result.deterministic ||
  !result.noHorizontalOverflow ||
  !result.allImagesLoaded
) process.exitCode = 1;
