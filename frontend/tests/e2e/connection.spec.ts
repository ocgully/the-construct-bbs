// frontend/tests/e2e/connection.spec.ts
import { test, expect } from '@playwright/test';
import { createTerminal } from './helpers/terminal';

test.describe('Connection Ceremony', () => {
  test('displays press any key prompt on load', async ({ page }) => {
    await page.goto('/?e2e');
    // Wait for terminal to initialize
    await page.waitForSelector('.xterm-screen', { timeout: 15000 });
    const terminal = await createTerminal(page);
    // Should show "Press any key to connect" prompt
    await terminal.waitForText('Press any key', 10000);
  });

  test('shows modem connection and splash screen', async ({ page }) => {
    await page.goto('/?e2e');
    const terminal = await createTerminal(page);
    await terminal.connect();

    // Should show splash/login screen after ceremony
    await terminal.waitForText('THE CONSTRUCT', 15000);
  });

  test('shows login prompt after ceremony', async ({ page }) => {
    await page.goto('/?e2e');
    const terminal = await createTerminal(page);
    await terminal.connect();

    await terminal.waitForText('Enter your handle:', 15000);
  });
});
