// frontend/tests/e2e/chat.spec.ts
import { test, expect } from '@playwright/test';
import { createTerminal, TerminalHelper } from './helpers/terminal';

// Use a dedicated chat test user
const testHandle = 'chattest';
const testPassword = 'ChatPass123!';
const testEmail = 'chattest@test.local';

test.describe('Chat System', () => {
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

  test('can enter chat room', async ({ page }) => {
    const terminal = await loginUser(page);

    await terminal.menuSelect('C');
    await terminal.waitForText('Teleconference', 5000);
  });

  test('can send chat message', async ({ page }) => {
    const terminal = await loginUser(page);

    await terminal.menuSelect('C');
    await terminal.waitForText('Teleconference');

    await terminal.send('Hello from E2E test!');
    await terminal.waitForText('Hello from E2E test', 3000);
  });

  test('can use /quit to exit chat', async ({ page }) => {
    const terminal = await loginUser(page);

    await terminal.menuSelect('C');
    await terminal.waitForText('Teleconference');

    await terminal.send('/quit');
    await terminal.waitForText('Main Menu', 5000);
  });
});
