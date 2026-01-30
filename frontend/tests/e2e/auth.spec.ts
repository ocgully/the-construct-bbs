// frontend/tests/e2e/auth.spec.ts
import { test, expect } from '@playwright/test';
import { createTerminal } from './helpers/terminal';

// Use unique handles to avoid conflicts
const testHandle = `test${Date.now()}`;
const testPassword = 'TestPass123!';
const testEmail = `${testHandle}@test.local`;

test.describe('Authentication', () => {
  test('can register new user', async ({ page }) => {
    await page.goto('/');
    const terminal = await createTerminal(page);
    await terminal.connect();

    await terminal.waitForText('Handle:');
    await terminal.send('new');

    // Registration flow
    await terminal.waitForText('handle');
    await terminal.send(testHandle);

    await terminal.waitForText('email', 5000);
    await terminal.send(testEmail);

    await terminal.waitForText('password', 5000);
    await terminal.send(testPassword);

    // Should reach verification or main menu (depending on email config)
    // In dev mode without SMTP, it auto-verifies
    await terminal.waitForText('Main Menu', 30000);
  });

  test('can login with existing credentials', async ({ page }) => {
    await page.goto('/');
    const terminal = await createTerminal(page);
    await terminal.connect();

    await terminal.waitForText('Handle:');
    await terminal.send(testHandle);

    await terminal.waitForText('Password:');
    await terminal.send(testPassword);

    await terminal.waitForText('Main Menu', 10000);
  });

  test('shows error on invalid login', async ({ page }) => {
    await page.goto('/');
    const terminal = await createTerminal(page);
    await terminal.connect();

    await terminal.waitForText('Handle:');
    await terminal.send('nonexistent_user_xyz');

    await terminal.waitForText('Password:');
    await terminal.send('wrongpassword');

    // Should show error and re-prompt
    await terminal.waitForText('Invalid', 5000);
  });

  test('can quit and disconnect', async ({ page }) => {
    await page.goto('/');
    const terminal = await createTerminal(page);
    await terminal.connect();

    await terminal.waitForText('Handle:');
    await terminal.send(testHandle);
    await terminal.waitForText('Password:');
    await terminal.send(testPassword);
    await terminal.waitForText('Main Menu', 10000);

    // Quit
    await terminal.menuSelect('Q');
    await terminal.waitForText('goodbye', 5000);
  });
});
