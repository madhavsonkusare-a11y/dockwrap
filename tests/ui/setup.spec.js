import {test, expect} from '@playwright/test';
import {installAdapter} from './fixtures.js';

// The install review for an app that asks questions. No reviewed recipe
// declares typed setup yet, so the fixture supplies the projection the backend
// would return for one; what is under test here is the form, not the values.
test.beforeEach(async ({page}) => {
  await installAdapter(page, {setup: 'memos'});
  await page.goto('/');
});

async function openReview(page) {
  await page.locator('[data-featured="memos"]').click();
  await expect(page.getByRole('heading', {name:'Setup'})).toBeVisible();
}
const confirm = page => page.getByRole('button', {name:'Install Memos', exact:true});

test('every declared field is rendered with its control and its blank behaviour', async ({page}) => {
  await openReview(page);
  const dialog = page.locator('#install-dialog');

  // A required field says so; an optional one says what leaving it blank does.
  await expect(dialog.getByText('Site name (required)')).toBeVisible();
  await expect(dialog.getByText('Leave blank to use admin@example.com.')).toBeVisible();
  // A sensitive default is never sent to the interface, so it is described
  // rather than shown.
  await expect(dialog.getByText('Leave blank to use the app’s own default.')).toBeVisible();
  await expect(dialog.locator('#setup-api-key')).toHaveAttribute('type', 'password');
  await expect(dialog.locator('#setup-api-key')).not.toHaveValue('admin@example.com');

  // Typed controls keep their type rather than collapsing to text.
  await expect(dialog.locator('#setup-max-uploads')).toHaveAttribute('type', 'number');
  await expect(dialog.locator('#setup-max-uploads')).toHaveAttribute('min', '1');
  await expect(dialog.locator('#setup-max-uploads')).toHaveAttribute('max', '100');
  await expect(dialog.locator('#setup-allow-signups')).toHaveAttribute('type', 'checkbox');
  await expect(dialog.locator('#setup-allow-signups')).not.toBeChecked();
  await expect(dialog.locator('#setup-theme')).toHaveValue('dark');

  // Generated credentials are stated, never displayed or asked for.
  await expect(dialog.getByText('One password is generated for this app and kept with its data.')).toBeVisible();
  await expect(dialog.locator('[data-setup-key="DB_PASSWORD"]')).toHaveCount(0);
});

test('an app label cannot write markup into the review', async ({page}) => {
  // Stand in for an upstream definition whose label is an injection attempt.
  // Labels are arbitrary upstream text on purpose, so the form has to be the
  // thing that refuses to treat them as markup.
  await page.evaluate(() => {
    const original = window.__TAURI__.core.invoke;
    window.__TAURI__.core.invoke = async (name, args) => {
      const result = await original(name, args);
      if (name === 'recipe_details' && result?.setup_review) {
        result.setup_review.fields[0].label = '<img src=x onerror="window.__pwned=1">';
      }
      return result;
    };
  });
  await openReview(page);
  await expect(page.locator('#setup-fields .setup-label').first()).toContainText(
    '<img src=x onerror="window.__pwned=1">'
  );
  expect(await page.evaluate(() => window.__pwned)).toBeUndefined();
  expect(await page.locator('#setup-fields img').count()).toBe(0);
});

test('a required answer is asked for before Docker is', async ({page}) => {
  await openReview(page);
  await confirm(page).click();
  await expect(page.locator('#install-error')).toHaveText('Fill in the required setup fields.');
  await expect(page.locator('#setup-site-name')).toBeFocused();
  await expect(page.locator('#setup-site-name')).toHaveAttribute('aria-invalid', 'true');
  // Nothing was attempted: the install command was never called.
  expect(await page.evaluate(() => window.__calls.filter(c => c.command === 'install_app').length)).toBe(0);
});

test('only answered fields are sent, and a blank one is left to its default', async ({page}) => {
  await openReview(page);
  await page.locator('#setup-site-name').fill('  My notes  ');
  await page.locator('#setup-max-uploads').fill('40');
  await page.locator('#setup-allow-signups').check();
  await page.locator('#setup-theme').selectOption('light');
  await confirm(page).click();
  await expect(page.locator('#install-dialog')).not.toBeVisible();

  const call = await page.evaluate(() => window.__calls.find(c => c.command === 'install_app'));
  expect(call.args.answers).toEqual({
    SITE_NAME: 'My notes',
    MAX_UPLOADS: '40',
    ALLOW_SIGNUPS: 'true',
    THEME: 'light',
  });
  // ADMIN_EMAIL and API_KEY were left blank, so they are absent and the
  // backend applies its own defaults rather than receiving an empty string.
  expect('ADMIN_EMAIL' in call.args.answers).toBe(false);
  expect('API_KEY' in call.args.answers).toBe(false);
});

test('a rejected answer points at the field it names and can be corrected', async ({page}) => {
  await page.evaluate(() => {
    const original = window.__TAURI__.core.invoke;
    window.__refused = true;
    window.__TAURI__.core.invoke = async (name, args) => {
      if (name === 'install_app' && window.__refused) {
        window.__refused = false;
        throw {code:'InvalidInput', message:'SITE_NAME: Use between 1 and 60 characters.'};
      }
      return original(name, args);
    };
  });
  await openReview(page);
  await page.locator('#setup-site-name').fill('x');
  await confirm(page).click();

  await expect(page.locator('#install-error')).toContainText('Use between 1 and 60 characters.');
  await expect(page.locator('#setup-site-name')).toHaveAttribute('aria-invalid', 'true');
  await expect(page.locator('#setup-site-name')).toBeFocused();
  // The dialog stays open with the answers intact, so a correction is one edit
  // rather than a re-entry of the whole form.
  await expect(page.locator('#setup-site-name')).toHaveValue('x');

  await page.locator('#setup-site-name').fill('My notes');
  await confirm(page).click();
  await expect(page.locator('#install-dialog')).not.toBeVisible();
});

test('answers do not survive the dialog they were typed into', async ({page}) => {
  await openReview(page);
  await page.locator('#setup-site-name').fill('First attempt');
  await page.locator('#setup-api-key').fill('a-secret-value');
  await page.getByRole('button', {name:'Cancel'}).click();
  await expect(page.locator('#install-dialog')).not.toBeVisible();

  await openReview(page);
  await expect(page.locator('#setup-site-name')).toHaveValue('');
  await expect(page.locator('#setup-api-key')).toHaveValue('');
  // And nothing that was typed reached anywhere it could be read back.
  const leaked = await page.evaluate(() => {
    const calls = JSON.stringify(window.__calls);
    const stored = [...Array(localStorage.length).keys()]
      .map(index => localStorage.getItem(localStorage.key(index))).join('');
    return (calls + stored + document.body.innerHTML).includes('a-secret-value');
  });
  expect(leaked).toBe(false);
});

test('a recipe that asks nothing renders no setup section at all', async ({page}) => {
  await page.locator('[data-featured="n8n"]').click();
  await expect(page.locator('#install-dialog')).toBeVisible();
  await expect(page.locator('#setup-fields')).toHaveCount(0);

  await page.getByRole('button', {name:'Install n8n', exact:true}).click();
  const call = await page.evaluate(() => window.__calls.find(c => c.command === 'install_app'));
  // No answers key at all, so the ordinary install sends exactly what it
  // always sent.
  expect('answers' in call.args).toBe(false);
});
