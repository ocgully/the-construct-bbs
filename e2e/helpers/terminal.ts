/**
 * BBS Terminal Helper for E2E Tests
 *
 * Provides a clean interface for interacting with the BBS
 * through the xterm.js terminal in Playwright tests.
 */

import { Page, expect } from '@playwright/test';

export class BbsTerminal {
  private page: Page;
  private baseUrl: string;

  constructor(page: Page, baseUrl: string = 'http://localhost:3000') {
    this.page = page;
    this.baseUrl = baseUrl;
  }

  /**
   * Connect to the BBS
   */
  async connect(): Promise<void> {
    await this.page.goto(this.baseUrl);
    await this.page.waitForSelector('.xterm-screen');
    // Wait for WebSocket connection and initial render
    await this.page.waitForTimeout(500);
  }

  /**
   * Send keys to the terminal
   */
  async sendKeys(keys: string): Promise<void> {
    await this.page.keyboard.type(keys);
    await this.page.waitForTimeout(100); // Allow render
  }

  /**
   * Press Enter key
   */
  async pressEnter(): Promise<void> {
    await this.page.keyboard.press('Enter');
    await this.page.waitForTimeout(100);
  }

  /**
   * Press a special key
   */
  async pressKey(key: 'Escape' | 'Tab' | 'Backspace' | 'ArrowUp' | 'ArrowDown' | 'ArrowLeft' | 'ArrowRight'): Promise<void> {
    await this.page.keyboard.press(key);
    await this.page.waitForTimeout(50);
  }

  /**
   * Login with credentials
   */
  async login(handle: string, password: string): Promise<void> {
    // Wait for login prompt
    await this.page.waitForTimeout(1000);

    // Enter handle
    await this.sendKeys(handle);
    await this.pressEnter();
    await this.page.waitForTimeout(300);

    // Enter password
    await this.sendKeys(password);
    await this.pressEnter();
    await this.page.waitForTimeout(500);
  }

  /**
   * Navigate to a game from the main menu
   */
  async navigateToGame(gameKey: string): Promise<void> {
    // Press G for Games menu
    await this.sendKeys('G');
    await this.page.waitForTimeout(200);

    // Select the specific game
    await this.sendKeys(gameKey);
    await this.page.waitForTimeout(300);
  }

  /**
   * Navigate directly to Chess
   */
  async navigateToChess(): Promise<void> {
    await this.navigateToGame('C'); // Assuming C for Chess
  }

  /**
   * Get the current terminal screen content
   */
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

  /**
   * Get visible lines (non-empty)
   */
  async getVisibleLines(): Promise<string[]> {
    const content = await this.getScreenContent();
    return content.split('\n').filter(line => line.trim() !== '');
  }

  /**
   * Assert that text appears on screen
   */
  async expectText(text: string | string[]): Promise<void> {
    const texts = Array.isArray(text) ? text : [text];
    const content = await this.getScreenContent();

    for (const t of texts) {
      expect(content, `Expected to find "${t}" on screen`).toContain(t);
    }
  }

  /**
   * Assert that any of the texts appear on screen
   */
  async expectAnyText(texts: string[]): Promise<void> {
    const content = await this.getScreenContent();
    const found = texts.some(t => content.includes(t));
    expect(found, `Expected to find one of: ${texts.join(', ')}`).toBe(true);
  }

  /**
   * Assert that text does NOT appear on screen
   */
  async expectNotText(text: string): Promise<void> {
    const content = await this.getScreenContent();
    expect(content, `Did not expect to find "${text}" on screen`).not.toContain(text);
  }

  /**
   * Wait for specific text to appear
   */
  async waitForText(text: string, timeout: number = 5000): Promise<void> {
    const startTime = Date.now();
    while (Date.now() - startTime < timeout) {
      const content = await this.getScreenContent();
      if (content.includes(text)) {
        return;
      }
      await this.page.waitForTimeout(100);
    }
    throw new Error(`Timeout waiting for text: "${text}"`);
  }

  /**
   * Extract a value from screen using regex
   */
  async extractValue(pattern: RegExp): Promise<string | null> {
    const content = await this.getScreenContent();
    const match = content.match(pattern);
    return match ? match[1] : null;
  }

  /**
   * Complete a simple encounter (for testing game flows)
   */
  async completeEncounter(): Promise<void> {
    // Generic encounter completion - games may override
    await this.pressEnter();
    await this.page.waitForTimeout(300);
  }

  /**
   * Take a screenshot for debugging
   */
  async screenshot(name: string): Promise<void> {
    await this.page.screenshot({ path: `screenshots/${name}.png` });
  }

  /**
   * Wait for screen to stabilize (no changes for given duration)
   */
  async waitForStableScreen(duration: number = 500): Promise<void> {
    let lastContent = '';
    let stableTime = 0;

    while (stableTime < duration) {
      const content = await this.getScreenContent();
      if (content === lastContent) {
        stableTime += 50;
      } else {
        stableTime = 0;
        lastContent = content;
      }
      await this.page.waitForTimeout(50);
    }
  }
}

/**
 * Chess-specific terminal helper
 */
export class ChessTerminal extends BbsTerminal {
  /**
   * Make a chess move
   */
  async makeMove(move: string): Promise<void> {
    await this.sendKeys(move);
    await this.pressEnter();
    await this.page.waitForTimeout(300);
  }

  /**
   * Create a new open game
   */
  async createOpenGame(): Promise<void> {
    await this.sendKeys('N'); // New game
    await this.page.waitForTimeout(200);
    await this.sendKeys('1'); // Open game
    await this.page.waitForTimeout(300);
  }

  /**
   * Create an ELO matched game
   */
  async createEloMatchedGame(minElo?: number, maxElo?: number): Promise<void> {
    await this.sendKeys('N'); // New game
    await this.page.waitForTimeout(200);
    await this.sendKeys('2'); // ELO matched
    await this.page.waitForTimeout(200);

    if (minElo !== undefined) {
      await this.sendKeys(minElo.toString());
      await this.pressEnter();
    } else {
      await this.pressEnter(); // Accept default
    }

    if (maxElo !== undefined) {
      await this.sendKeys(maxElo.toString());
      await this.pressEnter();
    } else {
      await this.pressEnter(); // Accept default
    }
  }

  /**
   * Challenge a specific player
   */
  async challengePlayer(handle: string): Promise<void> {
    await this.sendKeys('N'); // New game
    await this.page.waitForTimeout(200);
    await this.sendKeys('3'); // Challenge player
    await this.page.waitForTimeout(200);
    await this.sendKeys(handle);
    await this.pressEnter();
    await this.page.waitForTimeout(300);
  }

  /**
   * Join an open game by index
   */
  async joinGame(index: number): Promise<void> {
    await this.sendKeys(index.toString());
    await this.page.waitForTimeout(300);
  }

  /**
   * Continue an active game
   */
  async continueGame(index: number): Promise<void> {
    await this.sendKeys(`G${index}`);
    await this.page.waitForTimeout(300);
  }

  /**
   * Accept a challenge
   */
  async acceptChallenge(letter: string = 'A'): Promise<void> {
    await this.sendKeys(letter);
    await this.page.waitForTimeout(300);
  }

  /**
   * Offer or accept draw
   */
  async offerDraw(): Promise<void> {
    await this.sendKeys('D');
    await this.page.waitForTimeout(200);
  }

  /**
   * Resign the game
   */
  async resign(): Promise<void> {
    await this.sendKeys('R');
    await this.page.waitForTimeout(200);
    await this.sendKeys('Y'); // Confirm
    await this.page.waitForTimeout(300);
  }

  /**
   * Return to lobby
   */
  async backToLobby(): Promise<void> {
    await this.sendKeys('Q');
    await this.page.waitForTimeout(300);
  }

  /**
   * View leaderboard
   */
  async viewLeaderboard(): Promise<void> {
    await this.sendKeys('L');
    await this.page.waitForTimeout(300);
  }

  /**
   * Check if it's our turn
   */
  async isOurTurn(): Promise<boolean> {
    const content = await this.getScreenContent();
    return content.includes('YOUR TURN');
  }

  /**
   * Get current move number
   */
  async getMoveNumber(): Promise<number | null> {
    const value = await this.extractValue(/Move:\s*(\d+)/);
    return value ? parseInt(value) : null;
  }

  /**
   * Get player ELO from screen
   */
  async getPlayerElo(): Promise<number | null> {
    const value = await this.extractValue(/ELO:\s*(\d+)/);
    return value ? parseInt(value) : null;
  }
}
