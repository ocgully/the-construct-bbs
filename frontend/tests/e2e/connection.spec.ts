// frontend/tests/e2e/connection.spec.ts
import { test, expect } from '@playwright/test';
import { createTerminal } from './helpers/terminal';

test.describe('Connection Ceremony', () => {
  test('displays connect button on load', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('text=Connect to The Construct')).toBeVisible();
  });

  test('shows modem connection and splash screen', async ({ page }) => {
    await page.goto('/');
    const terminal = await createTerminal(page);
    await terminal.connect();

    // Should show splash/login screen after ceremony
    await terminal.waitForText('THE CONSTRUCT', 15000);
  });

  test('shows login prompt after ceremony', async ({ page }) => {
    await page.goto('/');
    const terminal = await createTerminal(page);
    await terminal.connect();

    await terminal.waitForText('Handle:', 15000);
  });
});
