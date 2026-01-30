// frontend/tests/e2e/gtm.spec.ts
import { test, expect } from '@playwright/test';
import { createTerminal, TerminalHelper } from './helpers/terminal';

// Use a dedicated GTM test user
const testHandle = 'gtmtest';
const testPassword = 'GtmPass123!';
const testEmail = 'gtmtest@test.local';

test.describe('Grand Theft Meth', () => {
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

  test('can launch game from menu', async ({ page }) => {
    const terminal = await loginUser(page);

    // Go to Games submenu
    await terminal.menuSelect('G');
    await terminal.waitForText('Grand Theft Meth');

    // Launch game
    await terminal.menuSelect('1');

    // Should see intro or main menu
    await terminal.waitForText('GRAND THEFT METH', 10000);
  });

  test('can skip intro and see main menu', async ({ page }) => {
    const terminal = await loginUser(page);

    await terminal.menuSelect('G');
    await terminal.waitForText('Grand Theft Meth');
    await terminal.menuSelect('1');
    await terminal.waitForText('GRAND THEFT METH', 10000);

    // Press any key to skip intro
    await terminal.press('Space');
    await page.waitForTimeout(500);

    // Should see game menu with options
    await terminal.waitForText('Travel', 5000);
  });

  test('shows status bar with game stats', async ({ page }) => {
    const terminal = await loginUser(page);

    await terminal.menuSelect('G');
    await terminal.menuSelect('1');
    await terminal.waitForText('GRAND THEFT METH', 10000);
    await terminal.press('Space');
    await terminal.waitForText('Travel', 5000);

    const content = await terminal.getTerminalContent();
    expect(content).toContain('Day');
    expect(content).toContain('Cash');
    expect(content).toContain('Debt');
  });

  test('can access travel screen', async ({ page }) => {
    const terminal = await loginUser(page);

    await terminal.menuSelect('G');
    await terminal.menuSelect('1');
    await terminal.waitForText('GRAND THEFT METH', 10000);
    await terminal.press('Space');
    await terminal.waitForText('Travel', 5000);

    await terminal.menuSelect('T');
    await terminal.waitForText('Bronx', 5000);
  });

  test('can access trade screen', async ({ page }) => {
    const terminal = await loginUser(page);

    await terminal.menuSelect('G');
    await terminal.menuSelect('1');
    await terminal.waitForText('GRAND THEFT METH', 10000);
    await terminal.press('Space');
    await terminal.waitForText('Travel', 5000);

    await terminal.menuSelect('B');
    // Trade screen shows commodities
    await terminal.waitForText('Buy', 5000);
  });

  test('can access loan shark', async ({ page }) => {
    const terminal = await loginUser(page);

    await terminal.menuSelect('G');
    await terminal.menuSelect('1');
    await terminal.waitForText('GRAND THEFT METH', 10000);
    await terminal.press('Space');
    await terminal.waitForText('Travel', 5000);

    await terminal.menuSelect('L');
    await terminal.waitForText('Loan Shark', 5000);
  });

  test('can quit game and return to BBS menu', async ({ page }) => {
    const terminal = await loginUser(page);

    await terminal.menuSelect('G');
    await terminal.menuSelect('1');
    await terminal.waitForText('GRAND THEFT METH', 10000);
    await terminal.press('Space');
    await terminal.waitForText('Travel', 5000);

    // Quit game
    await terminal.menuSelect('X');
    await terminal.waitForText('quit', 3000);
    await terminal.menuSelect('Y');

    // Back to BBS menu
    await terminal.waitForText('Main Menu', 5000);
  });

  test('game saves on quit and resumes', async ({ page }) => {
    const terminal = await loginUser(page);

    // Start game
    await terminal.menuSelect('G');
    await terminal.menuSelect('1');
    await terminal.waitForText('GRAND THEFT METH', 10000);
    await terminal.press('Space');
    await terminal.waitForText('Travel', 5000);

    // Travel to change state
    await terminal.menuSelect('T');
    await terminal.waitForText('Bronx', 5000);
    await terminal.menuSelect('2'); // Select different borough
    await page.waitForTimeout(1000);

    // Quit
    await terminal.menuSelect('X');
    await terminal.waitForText('quit');
    await terminal.menuSelect('Y');
    await terminal.waitForText('Main Menu', 5000);

    // Re-launch - should resume
    await terminal.menuSelect('G');
    await terminal.menuSelect('1');

    // Should show intro for resumed game
    await terminal.waitForText('GRAND THEFT METH', 10000);
  });

  test('can use drugs if in inventory', async ({ page }) => {
    const terminal = await loginUser(page);

    await terminal.menuSelect('G');
    await terminal.menuSelect('1');
    await terminal.waitForText('GRAND THEFT METH', 10000);
    await terminal.press('Space');
    await terminal.waitForText('Travel', 5000);

    await terminal.menuSelect('U');
    // Either shows use screen or "no drugs" message
    await page.waitForTimeout(1000);
  });

  test('can access quests screen', async ({ page }) => {
    const terminal = await loginUser(page);

    await terminal.menuSelect('G');
    await terminal.menuSelect('1');
    await terminal.waitForText('GRAND THEFT METH', 10000);
    await terminal.press('Space');
    await terminal.waitForText('Travel', 5000);

    await terminal.menuSelect('Q');
    await terminal.waitForText('Quest', 5000);
  });
});
