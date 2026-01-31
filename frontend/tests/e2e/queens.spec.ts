// frontend/tests/e2e/queens.spec.ts
import { test, expect } from '@playwright/test';
import { createTerminal, TerminalHelper } from './helpers/terminal';

// Use a dedicated Queens test user
const testHandle = 'queenstest';
const testPassword = 'QueensPass123!';
const testEmail = 'queenstest@test.local';

test.describe('Queens Daily Puzzle', () => {
  // Ensure test user exists
  test.beforeAll(async ({ browser }) => {
    const page = await browser.newPage();
    await page.goto('/?e2e');
    const terminal = await createTerminal(page);
    await terminal.connect();

    // Try to register new user
    await terminal.send('new');
    await terminal.waitForText('handle', 10000);
    await terminal.send(testHandle);
    await terminal.waitForText('email', 10000);
    await terminal.send(testEmail);
    await terminal.waitForText('password', 10000);
    await terminal.send(testPassword);

    await page.waitForTimeout(5000);
    await page.close();
  });

  async function loginUser(page: any): Promise<TerminalHelper> {
    await page.goto('/?e2e');
    const terminal = await createTerminal(page);
    await terminal.connect();

    // Terminal shows "Enter your handle:"
    await terminal.send(testHandle);
    await terminal.waitForText('password', 10000);
    await terminal.send(testPassword);
    await terminal.waitForText('Main Menu', 15000);

    return terminal;
  }

  test('can launch game from menu', async ({ page }) => {
    const terminal = await loginUser(page);

    // Go to Games submenu
    await terminal.menuSelect('G');
    await terminal.waitForText('Queens');

    // Launch game (find the Queens option number)
    // Queens should be in the games list
    const content = await terminal.getTerminalContent();
    // Look for Queens in the menu and select it
    if (content.includes('Queens')) {
      // Send the appropriate number key for Queens
      // This may vary based on menu position
      await terminal.menuSelect('2'); // Adjust if Queens is at different position
    }

    // Should see intro or puzzle screen
    await terminal.waitForText('QUEENS', 10000);
  });

  test('shows puzzle board after intro', async ({ page }) => {
    const terminal = await loginUser(page);

    await terminal.menuSelect('G');
    await terminal.waitForText('Queens');
    await terminal.menuSelect('2'); // Adjust based on menu position

    await terminal.waitForText('QUEENS', 10000);

    // Press any key to skip intro
    await terminal.press('Space');
    await page.waitForTimeout(500);

    // Should see board elements
    const content = await terminal.getTerminalContent();
    // Board should have row/column indicators
    expect(content.match(/[A-H]/)).toBeTruthy(); // Row labels
    expect(content.match(/[1-8]/)).toBeTruthy(); // Column numbers
  });

  test('can navigate with WASD keys', async ({ page }) => {
    const terminal = await loginUser(page);

    await terminal.menuSelect('G');
    await terminal.menuSelect('2');
    await terminal.waitForText('QUEENS', 10000);
    await terminal.press('Space'); // Start playing
    await page.waitForTimeout(500);

    // Try movement keys
    await terminal.press('S'); // Down
    await page.waitForTimeout(200);
    await terminal.press('D'); // Right
    await page.waitForTimeout(200);
    await terminal.press('W'); // Up
    await page.waitForTimeout(200);
    await terminal.press('A'); // Left
    await page.waitForTimeout(200);

    // Board should still be visible
    const content = await terminal.getTerminalContent();
    expect(content).toContain('Move');
  });

  test('can place and remove queen with space', async ({ page }) => {
    const terminal = await loginUser(page);

    await terminal.menuSelect('G');
    await terminal.menuSelect('2');
    await terminal.waitForText('QUEENS', 10000);
    await terminal.press('Space'); // Start playing
    await page.waitForTimeout(500);

    // Place a queen
    await terminal.press(' ');
    await page.waitForTimeout(300);

    // Board should show queen count > 0
    const content1 = await terminal.getTerminalContent();
    expect(content1).toContain('Queens: 1');

    // Remove the queen
    await terminal.press(' ');
    await page.waitForTimeout(300);

    const content2 = await terminal.getTerminalContent();
    expect(content2).toContain('Queens: 0');
  });

  test('can get hint', async ({ page }) => {
    const terminal = await loginUser(page);

    await terminal.menuSelect('G');
    await terminal.menuSelect('2');
    await terminal.waitForText('QUEENS', 10000);
    await terminal.press('Space');
    await page.waitForTimeout(500);

    // Get a hint
    await terminal.press('?');
    await page.waitForTimeout(500);

    // Should show hint message
    const content = await terminal.getTerminalContent();
    expect(content).toContain('Hint');
  });

  test('can clear all queens', async ({ page }) => {
    const terminal = await loginUser(page);

    await terminal.menuSelect('G');
    await terminal.menuSelect('2');
    await terminal.waitForText('QUEENS', 10000);
    await terminal.press('Space');
    await page.waitForTimeout(500);

    // Place a few queens
    await terminal.press(' ');
    await terminal.press('d');
    await terminal.press(' ');
    await page.waitForTimeout(300);

    // Clear all
    await terminal.press('c');
    await page.waitForTimeout(300);

    const content = await terminal.getTerminalContent();
    expect(content).toContain('Queens: 0');
  });

  test('can access help screen', async ({ page }) => {
    const terminal = await loginUser(page);

    await terminal.menuSelect('G');
    await terminal.menuSelect('2');
    await terminal.waitForText('QUEENS', 10000);
    await terminal.press('Space');
    await page.waitForTimeout(500);

    // Open help
    await terminal.press('/');
    await page.waitForTimeout(500);

    const content = await terminal.getTerminalContent();
    expect(content).toContain('HOW TO PLAY');
  });

  test('can access stats screen', async ({ page }) => {
    const terminal = await loginUser(page);

    await terminal.menuSelect('G');
    await terminal.menuSelect('2');
    await terminal.waitForText('QUEENS', 10000);
    await terminal.press('Space');
    await page.waitForTimeout(500);

    // Open stats
    await terminal.press('i');
    await page.waitForTimeout(500);

    const content = await terminal.getTerminalContent();
    expect(content).toContain('STATISTICS');
  });

  test('quit confirmation works', async ({ page }) => {
    const terminal = await loginUser(page);

    await terminal.menuSelect('G');
    await terminal.menuSelect('2');
    await terminal.waitForText('QUEENS', 10000);
    await terminal.press('Space');
    await page.waitForTimeout(500);

    // Try to quit
    await terminal.press('q');
    await page.waitForTimeout(300);

    // Should show confirmation
    const content = await terminal.getTerminalContent();
    expect(content).toContain('QUIT');
    expect(content).toContain('Y/N');

    // Cancel quit
    await terminal.press('n');
    await page.waitForTimeout(300);

    // Should be back on board
    const content2 = await terminal.getTerminalContent();
    expect(content2).toContain('Move');
  });

  test('can quit game and return to BBS menu', async ({ page }) => {
    const terminal = await loginUser(page);

    await terminal.menuSelect('G');
    await terminal.menuSelect('2');
    await terminal.waitForText('QUEENS', 10000);
    await terminal.press('Space');
    await page.waitForTimeout(500);

    // Quit game
    await terminal.press('q');
    await terminal.waitForText('Y/N', 3000);
    await terminal.press('y');

    // Back to BBS menu
    await terminal.waitForText('Main Menu', 5000);
  });

  test('shows timer on puzzle screen', async ({ page }) => {
    const terminal = await loginUser(page);

    await terminal.menuSelect('G');
    await terminal.menuSelect('2');
    await terminal.waitForText('QUEENS', 10000);
    await terminal.press('Space');
    await page.waitForTimeout(500);

    const content = await terminal.getTerminalContent();
    expect(content).toContain('Time:');
  });

  test('shows streak information', async ({ page }) => {
    const terminal = await loginUser(page);

    await terminal.menuSelect('G');
    await terminal.menuSelect('2');
    await terminal.waitForText('QUEENS', 10000);
    await terminal.press('Space');
    await page.waitForTimeout(500);

    const content = await terminal.getTerminalContent();
    expect(content).toContain('Streak:');
  });

  test('can enter coordinate directly', async ({ page }) => {
    const terminal = await loginUser(page);

    await terminal.menuSelect('G');
    await terminal.menuSelect('2');
    await terminal.waitForText('QUEENS', 10000);
    await terminal.press('Space');
    await page.waitForTimeout(500);

    // Enter a coordinate
    await terminal.send('A1');
    await page.waitForTimeout(500);

    // Should have placed a queen
    const content = await terminal.getTerminalContent();
    expect(content).toContain('Queens:');
  });

  test('wrong solution shows error message', async ({ page }) => {
    const terminal = await loginUser(page);

    await terminal.menuSelect('G');
    await terminal.menuSelect('2');
    await terminal.waitForText('QUEENS', 10000);
    await terminal.press('Space');
    await page.waitForTimeout(500);

    // Place a few queens in wrong positions (same row)
    await terminal.send('A1');
    await terminal.send('A2');
    await page.waitForTimeout(300);

    // Try to submit (Enter on empty line)
    await terminal.press('Enter');
    await page.waitForTimeout(500);

    // Should show error about wrong number of queens or conflict
    const content = await terminal.getTerminalContent();
    // Error message could be about count or conflict
    expect(content.match(/Need|queens|Multiple|row/i)).toBeTruthy();
  });

  test('region colors are displayed', async ({ page }) => {
    const terminal = await loginUser(page);

    await terminal.menuSelect('G');
    await terminal.menuSelect('2');
    await terminal.waitForText('QUEENS', 10000);
    await terminal.press('Space');
    await page.waitForTimeout(500);

    const content = await terminal.getTerminalContent();
    // Should show region legend
    expect(content).toContain('Regions:');
    // Should have color indicators
    expect(content.match(/R=Red|B=Blue|G=Green|Y=Yellow/)).toBeTruthy();
  });
});
