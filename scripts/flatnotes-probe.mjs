// Opt-in flatnotes first-use probe, using the Playwright already in this repo.
//
// This is the first app whose install asks a person questions, so the thing to
// prove is that the answers took effect: the username and password typed into
// the setup form are the ones that sign in, and a wrong password does not.
import { chromium } from '@playwright/test';

const [mode, endpoint, username, password] = process.argv.slice(2);
if (!['first-use', 'verify'].includes(mode) || !endpoint || !username || !password) {
  throw new Error('first-use|verify URL USERNAME PASSWORD required');
}
const target = new URL(endpoint);
if (target.protocol !== 'http:' || !['127.0.0.1', 'localhost', '[::1]'].includes(target.hostname)) {
  throw new Error('loopback only');
}
const base = target.origin;
const NOTE = 'local-store-probe';

async function signIn(page, secret) {
  await page.goto(base);
  await page.locator('#password').waitFor({ state: 'visible' });
  await page.locator('#username').fill(username);
  await page.locator('#password').fill(secret);
  await page.getByRole('button', { name: /log in/i }).click();
}

const browser = await chromium.launch({ headless: true, channel: 'msedge' });
try {
  const page = await browser.newPage();
  page.setDefaultTimeout(30000);

  // A password nobody chose must not work, or the answer proved nothing.
  await signIn(page, `${password}-wrong`);
  await page.waitForTimeout(2500);
  if (!(await page.locator('#password').isVisible()) || !page.url().includes('/login')) {
    throw new Error('a wrong password was accepted');
  }

  // The password somebody typed into the setup form must work.
  await signIn(page, password);
  await page.locator('#password').waitFor({ state: 'hidden' });

  // The note is a markdown file this test put in the app's managed folder.
  // Seeing it here is what proves that folder really is the notes folder.
  await page.goto(`${base}/note/${NOTE}`);
  await page.getByText('written into the managed folder').first()
    .waitFor({ state: 'visible' });

  console.log(`flatnotes ${mode} probe passed`);
} finally {
  await browser.close();
}
