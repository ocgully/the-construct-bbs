// frontend/tests/e2e/helpers/terminal.ts
import { Page, expect } from '@playwright/test';

export class TerminalHelper {
  constructor(private page: Page) {}

  /** Click connect button and wait for terminal */
  async connect() {
    await this.page.click('text=Connect to The Construct');
    await this.page.waitForSelector('.xterm-screen', { timeout: 10000 });
    // Wait for modem sound to finish and login prompt
    await this.page.waitForTimeout(3000);
  }

  /** Type text into terminal */
  async type(text: string) {
    await this.page.keyboard.type(text);
  }

  /** Press a single key */
  async press(key: string) {
    await this.page.keyboard.press(key);
  }

  /** Send text followed by Enter */
  async send(text: string) {
    await this.type(text);
    await this.press('Enter');
  }

  /** Wait for text to appear in terminal */
  async waitForText(text: string, timeout = 10000) {
    await expect(async () => {
      const content = await this.getTerminalContent();
      expect(content).toContain(text);
    }).toPass({ timeout });
  }

  /** Get current terminal text content */
  async getTerminalContent(): Promise<string> {
    return await this.page.evaluate(() => {
      const rows = document.querySelectorAll('.xterm-rows > div');
      return Array.from(rows).map(row => row.textContent || '').join('\n');
    });
  }

  /** Check if terminal contains text */
  async containsText(text: string): Promise<boolean> {
    const content = await this.getTerminalContent();
    return content.includes(text);
  }

  /** Wait for and dismiss any prompt */
  async waitForPrompt(promptText: string) {
    await this.waitForText(promptText);
  }

  /** Navigate menu by pressing a key */
  async menuSelect(key: string) {
    await this.press(key);
    await this.page.waitForTimeout(300); // Allow screen update
  }
}

export async function createTerminal(page: Page): Promise<TerminalHelper> {
  const terminal = new TerminalHelper(page);
  return terminal;
}
