// frontend/tests/e2e/navigation.spec.ts
import { test, expect } from '@playwright/test';
import { createTerminal, TerminalHelper } from './helpers/terminal';

// Use a dedicated navigation test user
const testHandle = 'navtest';
const testPassword = 'NavPass123!';
const testEmail = 'navtest@test.local';

test.describe('Navigation', () => {
  // Ensure test user exists
  test.beforeAll(async ({ browser }) => {
    const page = await browser.newPage();
    await page.goto('/');
    const terminal = await createTerminal(page);
    await terminal.connect();

    await terminal.waitForText('Handle:', 15000);
    await terminal.send('new');

    // Try to register
    await terminal.waitForText('handle', 5000);
    await terminal.send(testHandle);

    await terminal.waitForText('email', 5000);
    await terminal.send(testEmail);

    await terminal.waitForText('password', 5000);
    await terminal.send(testPassword);

    // Wait for registration to complete or fail (user might exist)
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

  test('shows main menu with options', async ({ page }) => {
    const terminal = await loginUser(page);
    const content = await terminal.getTerminalContent();

    expect(content).toContain('Mail');
    expect(content).toContain('Chat');
    expect(content).toContain('Games');
  });

  test('can enter Games submenu', async ({ page }) => {
    const terminal = await loginUser(page);
    await terminal.menuSelect('G');

    await terminal.waitForText('Grand Theft Meth', 5000);
  });

  test('can return from submenu with Q', async ({ page }) => {
    const terminal = await loginUser(page);
    await terminal.menuSelect('G');
    await terminal.waitForText('Grand Theft Meth');

    await terminal.menuSelect('Q');
    await terminal.waitForText('Main Menu', 5000);
  });

  test('can view profile with P', async ({ page }) => {
    const terminal = await loginUser(page);
    await terminal.menuSelect('P');

    await terminal.waitForText(testHandle, 5000);
    await terminal.waitForText('Member since', 5000);
  });

  test('can access Who\'s Online with W', async ({ page }) => {
    const terminal = await loginUser(page);
    await terminal.menuSelect('W');

    await terminal.waitForText('Online', 5000);
  });
});
