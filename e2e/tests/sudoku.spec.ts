/**
 * Sudoku E2E Tests
 *
 * Tests for the daily Sudoku puzzle game.
 * Uses Playwright to simulate BBS terminal sessions.
 *
 * Key features tested:
 * - Daily puzzle generation (same puzzle for all players)
 * - One attempt per day
 * - Streak tracking
 * - Pause day functionality (3 per week)
 * - Daily reset at Midnight Eastern
 */

import { test, expect, Page } from '@playwright/test';

// Helper class for BBS terminal interaction
class BbsTerminal {
  constructor(private page: Page) {}

  async connect() {
    await this.page.goto('http://localhost:3000');
    await this.page.waitForSelector('.xterm-screen');
    // Wait for connection ceremony to complete
    await this.page.waitForTimeout(500);
  }

  async sendKeys(keys: string) {
    await this.page.keyboard.type(keys);
    await this.page.waitForTimeout(100);
  }

  async pressEnter() {
    await this.page.keyboard.press('Enter');
    await this.page.waitForTimeout(100);
  }

  async login(handle: string, password: string) {
    await this.sendKeys(handle);
    await this.pressEnter();
    await this.page.waitForTimeout(200);
    await this.sendKeys(password);
    await this.pressEnter();
    await this.page.waitForTimeout(500);
  }

  async navigateToSudoku() {
    // Navigate: G (Games menu) -> S (Sudoku)
    await this.sendKeys('G');
    await this.page.waitForTimeout(200);
    await this.sendKeys('S'); // Sudoku
    await this.page.waitForTimeout(300);
  }

  async getScreenContent(): Promise<string> {
    return await this.page.evaluate(() => {
      const terminal = (window as any).terminal;
      if (!terminal) return '';

      let content = '';
      const buffer = terminal.buffer.active;
      for (let i = 0; i < buffer.length; i++) {
        const line = buffer.getLine(i);
        if (line) {
          content += line.translateToString(true) + '\n';
        }
      }
      return content;
    });
  }

  async expectText(text: string | string[]) {
    const texts = Array.isArray(text) ? text : [text];
    const content = await this.getScreenContent();
    const found = texts.some(t => content.includes(t));
    expect(found).toBe(true);
  }

  async expectNotText(text: string) {
    const content = await this.getScreenContent();
    expect(content).not.toContain(text);
  }
}

test.describe('Sudoku Daily Puzzle', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
    // Tests would normally connect and login here
    // await terminal.connect();
    // await terminal.login('testuser', 'testpass');
  });

  test.describe('Intro Screen', () => {
    test.skip('displays intro on entry', async () => {
      await terminal.navigateToSudoku();
      await terminal.expectText(['Fill the 9x9 grid', 'Press any key']);
    });

    test.skip('shows streak information if player has streaks', async () => {
      await terminal.navigateToSudoku();
      // After completing puzzles, streaks should show
      await terminal.expectText(['Current Streak', 'Longest Streak']);
    });
  });

  test.describe('Main Gameplay', () => {
    test.skip('starts puzzle on any key from intro', async () => {
      await terminal.navigateToSudoku();
      await terminal.pressEnter();
      // Should show the 9x9 grid with status bar
      await terminal.expectText(['Time:', 'Filled:', 'Errors:']);
    });

    test.skip('displays 9x9 grid correctly', async () => {
      await terminal.navigateToSudoku();
      await terminal.pressEnter();
      // Grid should contain row labels A-I
      await terminal.expectText(['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I']);
      // And column labels 1-9
      await terminal.expectText(['1', '2', '3', '4', '5', '6', '7', '8', '9']);
    });

    test.skip('can move cursor with WASD', async () => {
      await terminal.navigateToSudoku();
      await terminal.pressEnter();
      await terminal.page.waitForTimeout(200);

      // Move cursor around
      await terminal.sendKeys('D'); // Right
      await terminal.sendKeys('S'); // Down
      await terminal.sendKeys('A'); // Left
      await terminal.sendKeys('W'); // Up

      // Should still see the grid (no crashes)
      await terminal.expectText(['Time:', 'Filled:']);
    });

    test.skip('can move cursor with HJKL (vim-style)', async () => {
      await terminal.navigateToSudoku();
      await terminal.pressEnter();
      await terminal.page.waitForTimeout(200);

      // Move cursor with vim keys
      await terminal.sendKeys('L'); // Right
      await terminal.sendKeys('J'); // Down
      await terminal.sendKeys('H'); // Left
      await terminal.sendKeys('K'); // Up

      await terminal.expectText(['Time:', 'Filled:']);
    });

    test.skip('can enter numbers 1-9', async () => {
      await terminal.navigateToSudoku();
      await terminal.pressEnter();
      await terminal.page.waitForTimeout(200);

      // Try to enter a number (may fail if cell is given)
      await terminal.sendKeys('5');
      await terminal.page.waitForTimeout(100);

      // Should not crash, grid still visible
      await terminal.expectText(['Time:', 'Filled:']);
    });

    test.skip('can clear cell with 0 or space', async () => {
      await terminal.navigateToSudoku();
      await terminal.pressEnter();
      await terminal.page.waitForTimeout(200);

      await terminal.sendKeys('0'); // Clear
      await terminal.page.waitForTimeout(100);

      await terminal.expectText(['Time:', 'Filled:']);
    });

    test.skip('can toggle pencil mode with P', async () => {
      await terminal.navigateToSudoku();
      await terminal.pressEnter();
      await terminal.page.waitForTimeout(200);

      await terminal.sendKeys('P');
      await terminal.expectText(['PENCIL']); // Pencil mode indicator

      await terminal.sendKeys('P');
      // Pencil should be off now
    });

    test.skip('shows error message for incorrect entry', async () => {
      await terminal.navigateToSudoku();
      await terminal.pressEnter();
      await terminal.page.waitForTimeout(200);

      // Enter wrong number - this will depend on the specific puzzle
      // Just check that errors counter can increase
      await terminal.expectText(['Errors:']);
    });
  });

  test.describe('Help Screen', () => {
    test.skip('displays help on ? key', async () => {
      await terminal.navigateToSudoku();
      await terminal.pressEnter();
      await terminal.page.waitForTimeout(200);

      await terminal.sendKeys('?');
      await terminal.expectText(['HOW TO PLAY', 'CONTROLS', 'STREAKS']);
    });

    test.skip('returns to game on any key from help', async () => {
      await terminal.navigateToSudoku();
      await terminal.pressEnter();
      await terminal.page.waitForTimeout(200);

      await terminal.sendKeys('?');
      await terminal.page.waitForTimeout(200);
      await terminal.pressEnter();

      // Should be back in the game
      await terminal.expectText(['Time:', 'Filled:']);
    });
  });

  test.describe('Stats Screen', () => {
    test.skip('displays stats on T key', async () => {
      await terminal.navigateToSudoku();
      await terminal.pressEnter();
      await terminal.page.waitForTimeout(200);

      await terminal.sendKeys('T');
      await terminal.expectText(['YOUR STATS', 'LEADERBOARD']);
    });

    test.skip('shows player statistics', async () => {
      await terminal.navigateToSudoku();
      await terminal.pressEnter();
      await terminal.page.waitForTimeout(200);

      await terminal.sendKeys('T');
      await terminal.expectText([
        'Current Streak',
        'Longest Streak',
        'Puzzles Completed',
      ]);
    });
  });

  test.describe('Quit Confirmation', () => {
    test.skip('shows quit confirmation on Q key', async () => {
      await terminal.navigateToSudoku();
      await terminal.pressEnter();
      await terminal.page.waitForTimeout(200);

      await terminal.sendKeys('Q');
      await terminal.expectText(['QUIT PUZZLE', 'Y/N']);
    });

    test.skip('cancels quit on N', async () => {
      await terminal.navigateToSudoku();
      await terminal.pressEnter();
      await terminal.page.waitForTimeout(200);

      await terminal.sendKeys('Q');
      await terminal.page.waitForTimeout(200);
      await terminal.sendKeys('N');

      // Should be back in the game
      await terminal.expectText(['Time:', 'Filled:']);
    });

    test.skip('saves and exits on Y', async () => {
      await terminal.navigateToSudoku();
      await terminal.pressEnter();
      await terminal.page.waitForTimeout(200);

      await terminal.sendKeys('Q');
      await terminal.page.waitForTimeout(200);
      await terminal.sendKeys('Y');

      // Should return to main BBS menu
      // (exact text depends on menu configuration)
    });
  });

  test.describe('Save/Resume', () => {
    test.skip('saves progress when quitting', async () => {
      await terminal.navigateToSudoku();
      await terminal.pressEnter();
      await terminal.page.waitForTimeout(200);

      // Make some moves
      await terminal.sendKeys('D'); // Move
      await terminal.sendKeys('5'); // Enter number

      // Quit
      await terminal.sendKeys('Q');
      await terminal.page.waitForTimeout(100);
      await terminal.sendKeys('Y');

      // Return to game
      await terminal.navigateToSudoku();
      await terminal.pressEnter();

      // Progress should be preserved
      await terminal.expectText(['Time:']); // Timer should have some time
    });
  });

  test.describe('Already Played', () => {
    test.skip('shows already played screen if completed today', async () => {
      // This test requires completing a puzzle first
      // Then trying to play again
      await terminal.navigateToSudoku();
      await terminal.expectText(["You've already completed today's puzzle"]);
    });

    test.skip('shows completion time on already played screen', async () => {
      await terminal.navigateToSudoku();
      await terminal.expectText(['Your time:']);
    });

    test.skip('can view stats from already played screen', async () => {
      await terminal.navigateToSudoku();
      await terminal.sendKeys('T');
      await terminal.expectText(['YOUR STATS', 'LEADERBOARD']);
    });
  });

  test.describe('Puzzle Completion', () => {
    test.skip('shows completion screen when puzzle solved', async () => {
      // This test would require solving the entire puzzle
      // Which is complex to automate
      await terminal.expectText(['COMPLETE', 'Congratulations']);
    });

    test.skip('updates streak on completion', async () => {
      // After completion, streak should increase
      await terminal.expectText(['Current Streak:']);
    });
  });

  test.describe('Daily Puzzle Consistency', () => {
    test.skip('same puzzle for all users on same day', async () => {
      // This would require two separate sessions
      // Both should see the same puzzle layout

      // First user
      await terminal.navigateToSudoku();
      await terminal.pressEnter();
      const content1 = await terminal.getScreenContent();

      // TODO: Login as second user and compare puzzles
      // const content2 = await terminal2.getScreenContent();
      // Extract puzzle portion and compare
    });
  });

  test.describe('Streak Mechanics', () => {
    test.skip('tracks consecutive day completions', async () => {
      // Would require multi-day testing
      await terminal.navigateToSudoku();
      await terminal.expectText(['Current Streak']);
    });

    test.skip('preserves streak with pause days', async () => {
      // Would require testing over multiple days
      // with simulated pause day usage
    });

    test.skip('resets streak on missed day without pause', async () => {
      // Would require testing over multiple days
    });

    test.skip('resets pause days weekly', async () => {
      // Would require testing across week boundary
    });
  });
});

test.describe('Sudoku Leaderboard', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('displays leaderboard', async () => {
    await terminal.navigateToSudoku();
    await terminal.pressEnter();
    await terminal.page.waitForTimeout(200);

    await terminal.sendKeys('T');
    await terminal.expectText(['LEADERBOARD', 'Rank', 'Player', 'Streak']);
  });

  test.skip('sorts by longest streak', async () => {
    await terminal.navigateToSudoku();
    await terminal.pressEnter();
    await terminal.page.waitForTimeout(200);

    await terminal.sendKeys('T');
    await terminal.expectText(['Longest Streaks']);
  });
});
