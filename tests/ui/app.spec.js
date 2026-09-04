import { test, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import { installAdapter } from './fixtures.js';

test.beforeEach(async ({page}) => { await installAdapter(page); await page.goto('/'); });
test('discover search, category, detail and source actions are honest', async ({page}) => {
 await expect(page.getByRole('heading',{name:'Good software. Your space.'})).toBeVisible();
 await expect(page.getByText('1,257 projects to discover')).toBeVisible();
 await page.getByRole('searchbox').fill('memo'); await expect(page.getByRole('heading',{name:'Memos'})).toBeVisible();
 await expect(page.getByRole('heading',{name:'Immich'})).toHaveCount(0);
 await page.getByRole('searchbox').fill(''); await page.getByLabel('Category').selectOption('Photo Galleries');
 await expect(page.getByRole('heading',{name:'Immich'})).toBeVisible();
 await page.getByRole('button',{name:'View Immich details'}).click();
 await expect(page.getByText('Connect existing instance')).toBeVisible();
 await expect(page.getByText(/Automatic installation isn’t available/)).toBeVisible();
 await page.getByRole('button',{name:/Visit project website/}).click();
 expect(await page.evaluate(() => window.__calls.at(-1))).toEqual({command:'open_project',args:{url:'https://immich.app'}});
});

test('connect validates, preserves input after error, and appears in My Apps', async ({page}) => {
 await page.getByRole('button',{name:'Connect an app'}).first().click();
 await page.getByLabel('App name').fill('Home photos'); await page.getByLabel('Instance address').fill('file:///etc/passwd');
 await page.getByRole('button',{name:/Add to My Apps/}).click(); await expect(page.getByRole('alert')).toContainText('http:// or https://');
 await expect(page.getByLabel('App name')).toHaveValue('Home photos');
 await page.getByLabel('Instance address').fill('http://192.168.1.5:2283'); await page.getByRole('button',{name:/Add to My Apps/}).click();
 await expect(page.getByRole('heading',{name:'Right where you left them.'})).toBeVisible(); await expect(page.getByRole('heading',{name:'Home photos'})).toBeVisible();
 expect(await page.evaluate(() => window.__calls.find(c=>c.command==='add_app'))).toEqual({command:'add_app',args:{name:'Home photos',url:'http://192.168.1.5:2283'}});
});

test('My Apps handles open errors and confirms non-destructive removal', async ({page}) => {
 await page.getByRole('button',{name:/My Apps/}).click(); await page.getByRole('button',{name:/Open/}).click();
 expect(await page.evaluate(() => window.__calls.at(-1).command)).toBe('open_app');
 await page.getByRole('button',{name:'Remove Studio notes'}).click();
 await expect(page.getByText('The server and its data stay untouched.')).toBeVisible(); await page.getByRole('button',{name:'Remove connection'}).click();
 await expect(page.getByRole('heading',{name:'Your apps belong here.'})).toBeVisible();
});

test('keyboard shortcut and accessibility', async ({page}) => {
 await page.keyboard.press('Control+k'); await expect(page.getByRole('searchbox')).toBeFocused();
 await page.keyboard.press('Escape');
 const results = await new AxeBuilder({page}).analyze(); expect(results.violations).toEqual([]);
});

test('responsive visual surfaces', async ({page}) => {
 await expect(page).toHaveScreenshot('discover-1280x800.png', {animations:'disabled'});
 await page.setViewportSize({width:800,height:600}); await expect(page).toHaveScreenshot('discover-800x600.png', {animations:'disabled'});
});
