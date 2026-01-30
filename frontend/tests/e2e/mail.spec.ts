// frontend/tests/e2e/mail.spec.ts
import { test, expect } from '@playwright/test';
import { createTerminal, TerminalHelper } from './helpers/terminal';

// Use a dedicated mail test user
const testHandle = 'mailtest';
const testPassword = 'MailPass123!';
const testEmail = 'mailtest@test.local';

test.describe('Mail System', () => {
  // Ensure test user exists
  test.beforeAll(async ({ browser }) => {
    const page = await browser.newPage();
    await page.goto('/');
    const terminal = await createTerminal(page);
    await terminal.connect();

    await terminal.waitForText('Handle:', 15000);
    await terminal.send('new');

    await terminal.waitForText('handle', 5000);
    await terminal.send(testHandle);

    await terminal.waitForText('email', 5000);
    await terminal.send(testEmail);

    await terminal.waitForText('password', 5000);
    await terminal.send(testPassword);

    await page.waitForTimeout(5000);
    await page.close();
  });

  async function loginUser(page: any): Promise<TerminalHelper> {
    await page.goto('/');
    const terminal = await createTerminal(page);
    await terminal.connect();

    await terminal.waitForText('Handle:', 15000);
    await terminal.send(testHandle);
    await terminal.waitForText('Password:');
    await terminal.send(testPassword);
    await terminal.waitForText('Main Menu', 15000);

    return terminal;
  }

  test('can access mail inbox', async ({ page }) => {
    const terminal = await loginUser(page);

    // Access mail
    await terminal.menuSelect('M');
    await terminal.waitForText('Inbox', 5000);
  });

  test('can compose new message', async ({ page }) => {
    const terminal = await loginUser(page);

    await terminal.menuSelect('M');
    await terminal.waitForText('Inbox');

    // Compose
    await terminal.menuSelect('C');
    await terminal.waitForText('To:', 5000);
  });

  test('can quit mail and return to menu', async ({ page }) => {
    const terminal = await loginUser(page);

    await terminal.menuSelect('M');
    await terminal.waitForText('Inbox');

    await terminal.menuSelect('Q');
    await terminal.waitForText('Main Menu', 5000);
  });
});
