// The App Store standard, as a check that works without knowing the app.
//
// Umbrel states the bar in prose: an app "should open to a web UI, setup flow,
// login page, or status page that gives users a clear next step without SSH,
// CLI access, log scraping, or manual file edits." That is nearly word for word
// this product's own goal — install with one action, open to a useful browser
// screen — and unlike "can you create a paste" it can be checked for any app.
//
// What it proves: the address opens something a person could act on. What it
// does not prove: that they could finish a real task. An app passing this is
// *tested*, not *verified*, and the two must not be conflated.
import { chromium } from '@playwright/test';
import { readFile, writeFile } from 'node:fs/promises';

const [mode, endpoint, statePath] = process.argv.slice(2);
if (!['first-use', 'verify'].includes(mode) || !statePath) {
  throw new Error('first-use|verify URL STATE required');
}
const target = new URL(endpoint);
if (target.protocol !== 'http:' || !['127.0.0.1', 'localhost', '[::1]'].includes(target.hostname)) {
  throw new Error('loopback only');
}

// Pages that answer without being an app: a web server's "it works" placeholder
// or a bare API error. Matched on the whole visible text so a page merely
// mentioning one of these is not caught.
const PLACEHOLDERS = [
  /^welcome to nginx!/i,
  /^apache2 (ubuntu |debian )?default page/i,
  /^it works!$/i,
  /^\{"detail":/,
  /^404 not found$/i,
  /^403 forbidden$/i,
];

const browser = await chromium.launch({ headless: true, channel: 'msedge' });
try {
  const page = await browser.newPage();
  page.setDefaultTimeout(30000);

  const response = await page.goto(endpoint, { waitUntil: 'domcontentloaded' });
  if (response && response.status() >= 400) {
    throw new Error(`the address answered ${response.status()}`);
  }
  // Answering is not the same as being ready. A single-page app paints after
  // load, and one holding a websocket — Node-RED does — never reaches network
  // idle at all, so waiting for that alone judged an empty document. Poll
  // until the page has something on it, and only then decide.
  const read = () =>
    page.evaluate(() => {
    const text = (document.body?.innerText || '').trim();
    const count = (selector) => document.querySelectorAll(selector).length;
    return {
      title: (document.title || '').trim(),
      text: text.slice(0, 400),
      length: text.length,
      inputs: count('input:not([type=hidden]), textarea, select'),
      buttons: count('button, [role=button], input[type=submit]'),
      links: count('a[href]'),
      forms: count('form'),
      password: count('input[type=password]') > 0,
    };
    });

  let seen = await read();
  const settled = Date.now() + 45000;
  while (Date.now() < settled) {
    const enough = seen.inputs + seen.buttons + seen.forms > 0 || seen.links > 2 || seen.length >= 40;
    if (enough) break;
    await page.waitForTimeout(1500);
    seen = await read();
  }

  if (PLACEHOLDERS.some((pattern) => pattern.test(seen.text))) {
    throw new Error(`the address serves a placeholder page: ${seen.text.slice(0, 60)}`);
  }
  // A clear next step is something to read or something to do. An app with a
  // rich UI and little text passes on its controls; a status page with no
  // controls passes on its text.
  const canAct = seen.inputs + seen.buttons + seen.forms > 0 || seen.links > 2;
  if (!canAct && seen.length < 40) {
    throw new Error(
      `the address opened nothing to act on (${seen.length} characters, no controls)`,
    );
  }

  if (mode === 'first-use') {
    await writeFile(statePath, JSON.stringify({ title: seen.title, password: seen.password }));
  } else {
    const before = JSON.parse(await readFile(statePath, 'utf8'));
    // An app that lost its data comes back as a setup wizard: the page it
    // opens to changes, and a sign-in form appears where there was none. This
    // is a signal rather than a proof, so only a *new* sign-in prompt or a
    // changed title is treated as failure.
    if (before.title && seen.title && before.title !== seen.title) {
      throw new Error(
        `it opened a different page than before: ${before.title} then ${seen.title}`,
      );
    }
    if (!before.password && seen.password) {
      throw new Error('it now asks to be set up again, which is what losing data looks like');
    }
  }
  console.log(`standard ${mode} probe passed`);
} finally {
  await browser.close();
}
