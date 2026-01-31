// frontend/tests/e2e/auth.spec.ts
import { test, expect } from '@playwright/test';
import { createTerminal } from './helpers/terminal';

test.describe('Authentication', () => {
  test('can start registration flow', async ({ page }) => {
    await page.goto('/?e2e');
    const terminal = await createTerminal(page);
    await terminal.connect();

    // Terminal shows "Enter your handle:"
    await terminal.send('new');

    // Registration flow should start
    await terminal.waitForText('NEW USER REGISTRATION', 10000);
    await terminal.waitForText('Choose your handle:', 10000);
  });

  test('registration validates handle', async ({ page }) => {
    await page.goto('/?e2e');
    const terminal = await createTerminal(page);
    await terminal.connect();

    await terminal.send('new');
    await terminal.waitForText('Choose your handle:', 10000);

    // Try an invalid handle (too short)
    await terminal.send('ab');
    await terminal.waitForText('must be', 10000); // Error about length requirements
  });

  test('shows handle not found for invalid login', async ({ page }) => {
    await page.goto('/?e2e');
    const terminal = await createTerminal(page);
    await terminal.connect();

    await terminal.send('fakeuserxyz');
    await terminal.waitForText('not found', 10000);
  });

  test('shows prompt to type new for registration', async ({ page }) => {
    await page.goto('/?e2e');
    const terminal = await createTerminal(page);
    await terminal.connect();

    await terminal.send('nonexistentuser');
    // Should show hint to register
    await terminal.waitForText("Type 'new' to register", 10000);
  });
});
