// Opt-in flatnotes first-use probe, using the Playwright already in this repo.
//
// flatnotes is the first offered app whose install asks a person questions, so
// what has to be proved is that the answers took effect: the username and
// password typed into the setup form are the ones that sign in, and a wrong
// password is refused. The note is created through the app's own API rather
// than by dropping a file into the managed folder, so this probe stands on its
// own and the harness needs to know nothing about how flatnotes stores things.
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
const BODY = 'This note was written through the app itself.';

async function token(secret) {
  const response = await fetch(`${base}/api/token`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ username, password: secret }),
  });
  if (!response.ok) return null;
  const { access_token: value } = await response.json();
  return value ?? null;
}

async function signIn(page, secret) {
  await page.goto(base);
  await page.locator('#password').waitFor({ state: 'visible' });
  await page.locator('#username').fill(username);
  await page.locator('#password').fill(secret);
  await page.getByRole('button', { name: /log in/i }).click();
}

// A password nobody chose must not work, or the sign-in below proves only that
// some password works.
if (await token(`${password}-wrong`)) {
  throw new Error('a wrong password was accepted');
}
const bearer = await token(password);
if (!bearer) throw new Error('the password chosen at install does not sign in');

if (mode === 'first-use') {
  const created = await fetch(`${base}/api/notes`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', Authorization: `Bearer ${bearer}` },
    body: JSON.stringify({ title: NOTE, content: BODY }),
  });
  if (!created.ok) throw new Error(`the note could not be created: ${created.status}`);
}

// The note has to still be there, whichever phase this is.
const read = await fetch(`${base}/api/notes/${NOTE}`, {
  headers: { Authorization: `Bearer ${bearer}` },
});
if (!read.ok) throw new Error(`the note is gone: ${read.status}`);
const note = await read.json();
if (note.content !== BODY) throw new Error('the note came back with different content');

// And a person has to be able to sign in and read it, not just an API client.
const browser = await chromium.launch({ headless: true, channel: 'msedge' });
try {
  const page = await browser.newPage();
  page.setDefaultTimeout(30000);
  await signIn(page, `${password}-wrong`);
  await page.waitForTimeout(2500);
  if (!(await page.locator('#password').isVisible()) || !page.url().includes('/login')) {
    throw new Error('the sign-in page accepted a wrong password');
  }
  await signIn(page, password);
  await page.locator('#password').waitFor({ state: 'hidden' });
  await page.goto(`${base}/note/${NOTE}`);
  await page.getByText('written through the app itself').first().waitFor({ state: 'visible' });
} finally {
  await browser.close();
}

console.log(`flatnotes ${mode} probe passed`);
