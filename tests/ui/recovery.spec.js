import {test, expect} from '@playwright/test';
import {installAdapter} from './fixtures.js';

test('recovery scan supports empty, verified, error and late replies', async ({page}) => {
  await page.emulateMedia({reducedMotion:'reduce'});
  await installAdapter(page);
  await page.goto('/');
  await page.evaluate(() => {
    const original = window.__TAURI__.core.invoke;
    window.__recoveryAnswer = [];
    window.__TAURI__.core.invoke = (name, args) => {
      if (name !== 'inspect_recovery') return original(name, args);
      if (window.__recoveryAnswer === 'error') return Promise.reject(new Error('Docker is unavailable'));
      if (window.__recoveryAnswer === 'pending') return new Promise(resolve => { window.__resolveRecovery = resolve; });
      return Promise.resolve(window.__recoveryAnswer);
    };
  });
  await page.locator('#settings').click();
  const scan = page.getByRole('button', {name:'Scan for setups'});
  await scan.click();
  await expect(page.locator('#recovery-output')).toHaveText('No retained setups found for supported apps.');
  await page.evaluate(() => { window.__recoveryAnswer = [{display_name:'<img onerror=alert(1)>', ownership_status:'verified', compose_file:'D:/private/compose.yaml'}]; });
  await scan.click();
  await expect(page.locator('#recovery-output')).toContainText('Matching Docker setup found');
  await expect(page.locator('#recovery-output img')).toHaveCount(0);
  await page.getByText('Setup location', {exact:true}).click();
  await expect(page.locator('#recovery-output')).toContainText('D:/private/compose.yaml');
  await page.evaluate(() => { window.__recoveryAnswer = 'error'; });
  await scan.click();
  await expect(page.locator('#recovery-error')).toHaveText('Docker is unavailable');
  await expect(scan).toBeEnabled();
  await page.evaluate(() => { window.__recoveryAnswer = 'pending'; });
  await scan.click();
  await expect(page.getByRole('button', {name:'Scanning…'})).toBeDisabled();
  await page.keyboard.press('Escape');
  await page.evaluate(() => window.__resolveRecovery([{display_name:'Stale', ownership_status:'verified'}]));
  await page.locator('#settings').click();
  await expect(page.locator('#recovery-output')).not.toContainText('Stale');
  await expect(scan).toBeEnabled();
});
