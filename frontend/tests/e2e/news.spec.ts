// frontend/tests/e2e/news.spec.ts
import { test, expect } from '@playwright/test';
import { createTerminal, TerminalHelper } from './helpers/terminal';

// Use a dedicated news test user
const testHandle = 'newstest';
const testPassword = 'NewsPass123!';
const testEmail = 'newstest@test.local';

test.describe('News System', () => {
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

  test('can access news', async ({ page }) => {
    const terminal = await loginUser(page);

    await terminal.menuSelect('N');
    // Either shows news or loading/error
    await terminal.waitForText('WIRE', 10000);
  });

  test('can navigate news with arrow keys', async ({ page }) => {
    const terminal = await loginUser(page);

    await terminal.menuSelect('N');
    await terminal.waitForText('WIRE', 10000);

    // Navigate down
    await terminal.press('ArrowDown');
    await page.waitForTimeout(500);
  });

  test('can exit news with Q', async ({ page }) => {
    const terminal = await loginUser(page);

    await terminal.menuSelect('N');
    await terminal.waitForText('WIRE', 10000);

    await terminal.menuSelect('Q');
    await terminal.waitForText('Main Menu', 5000);
  });
});
