// frontend/tests/e2e/helpers/terminal.ts
import { Page, expect } from '@playwright/test';

export class TerminalHelper {
  constructor(private page: Page) {}

  /** Wait for terminal and press any key to connect */
  async connect() {
    // Wait for xterm.js to be ready
    await this.page.waitForSelector('.xterm-screen', { timeout: 15000 });
    // Wait for the "Press any key" prompt
    await this.waitForText('Press any key', 10000);
    // Press Enter to trigger connection
    await this.press('Enter');
    // Wait for modem sound to finish and login prompt
    await this.page.waitForTimeout(5000);
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
      // Try multiple methods to get terminal content

      // Method 1: Access xterm.js terminal object directly
      const win = window as any;
      if (win._terminal && win._terminal.buffer) {
        const buffer = win._terminal.buffer.active;
        const lines: string[] = [];
        for (let i = 0; i < buffer.length; i++) {
          const line = buffer.getLine(i);
          if (line) {
            lines.push(line.translateToString(true));
          }
        }
        return lines.join('\n');
      }

      // Method 2: Query xterm-rows div content
      const rows = document.querySelectorAll('.xterm-rows > div');
      if (rows.length > 0) {
        return Array.from(rows).map(row => row.textContent || '').join('\n');
      }

      // Method 3: Look for any text in xterm container
      const container = document.querySelector('.xterm');
      if (container) {
        return container.textContent || '';
      }

      return '';
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
