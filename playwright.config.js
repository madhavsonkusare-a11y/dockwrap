import { defineConfig } from '@playwright/test';
export default defineConfig({
  testDir: './tests/ui', snapshotPathTemplate: '{testDir}/{testFilePath}-snapshots/{arg}{ext}', timeout: 20_000, fullyParallel: true,
  use: { baseURL: 'http://127.0.0.1:4173', viewport: { width: 1280, height: 800 }, trace: 'retain-on-failure', channel: 'chromium' },
  expect: { toHaveScreenshot: { maxDiffPixelRatio: 0.01 } },
  webServer: { command: 'node scripts/preview.mjs', url: 'http://127.0.0.1:4173', reuseExistingServer: true }
});

