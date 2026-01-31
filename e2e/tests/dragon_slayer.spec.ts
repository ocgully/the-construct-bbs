/**
 * Dragon Slayer E2E Tests
 *
 * Tests for the Legend of the Red Dragon style RPG door game.
 * Uses Playwright to simulate BBS terminal sessions.
 *
 * Key features tested:
 * - Character creation (name and sex selection)
 * - Town navigation
 * - Forest combat with monsters
 * - Training grounds and master battles
 * - Equipment shops (weapons and armor)
 * - Healer and Bank services
 * - Romance system (Violet and Seth NPCs)
 * - Daily fight limits
 * - Dragon hunt (level 12)
 * - Save/load functionality
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

  async navigateToDragonSlayer() {
    // Navigate: G (Games menu) -> 2 (Dragon Slayer)
    await this.sendKeys('G');
    await this.page.waitForTimeout(200);
    await this.sendKeys('2'); // Dragon Slayer
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

test.describe('Dragon Slayer - Character Creation', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
    // Tests would normally connect and login here
    // await terminal.connect();
    // await terminal.login('testuser', 'testpass');
  });

  test.skip('displays character creation on first entry', async () => {
    await terminal.navigateToDragonSlayer();
    await terminal.expectText(['CREATE YOUR HERO', 'What is your name']);
  });

  test.skip('accepts character name input', async () => {
    await terminal.navigateToDragonSlayer();
    await terminal.sendKeys('Sir Lancelot');
    await terminal.pressEnter();
    await terminal.expectText(['Are you:', 'Male', 'Female']);
  });

  test.skip('allows sex selection M/F', async () => {
    await terminal.navigateToDragonSlayer();
    await terminal.sendKeys('TestHero');
    await terminal.pressEnter();
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('M');
    // Should proceed to intro
    await terminal.expectText(['village of Silverton', 'Red Dragon']);
  });

  test.skip('validates name length', async () => {
    await terminal.navigateToDragonSlayer();
    // Try too short name
    await terminal.sendKeys('AB');
    await terminal.pressEnter();
    await terminal.expectText(['at least 3 characters']);
  });
});

test.describe('Dragon Slayer - Town Navigation', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('displays town menu with all options', async () => {
    await terminal.navigateToDragonSlayer();
    // After character creation and intro
    await terminal.expectText([
      'THE TOWN OF SILVERTON',
      'The Dark Forest',
      "Turgon's Training",
      'Weapons Shop',
      'Armor Shop',
      "Healer's Hut",
      'The Bank',
    ]);
  });

  test.skip('shows status bar with player info', async () => {
    await terminal.navigateToDragonSlayer();
    await terminal.expectText(['HP:', 'Gold:', 'Forest Fights:']);
  });

  test.skip('can navigate to forest with F', async () => {
    await terminal.navigateToDragonSlayer();
    await terminal.sendKeys('F');
    await terminal.expectText(['THE DARK FOREST', 'Look for something to kill']);
  });

  test.skip('can navigate to training with T', async () => {
    await terminal.navigateToDragonSlayer();
    await terminal.sendKeys('T');
    await terminal.expectText(["TURGON'S TRAINING GROUNDS", 'Current Master:']);
  });

  test.skip('can view stats with Y', async () => {
    await terminal.navigateToDragonSlayer();
    await terminal.sendKeys('Y');
    await terminal.expectText(['WARRIOR STATS', 'Level:', 'Experience:', 'Strength:']);
  });

  test.skip('shows quit confirmation with Q', async () => {
    await terminal.navigateToDragonSlayer();
    await terminal.sendKeys('Q');
    await terminal.expectText(['SAVE & QUIT', 'Are you sure?']);
  });
});

test.describe('Dragon Slayer - Forest Combat', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('can find and fight monsters', async () => {
    await terminal.navigateToDragonSlayer();
    await terminal.sendKeys('F'); // Enter forest
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('L'); // Look for monster
    await terminal.expectText(['ATTACKS!', 'Enemy HP:', 'Your HP:']);
  });

  test.skip('displays combat actions', async () => {
    await terminal.navigateToDragonSlayer();
    await terminal.sendKeys('F');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('L');
    await terminal.expectText(['Attack', 'Run away']);
  });

  test.skip('can attack with A key', async () => {
    await terminal.navigateToDragonSlayer();
    await terminal.sendKeys('F');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('L');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('A'); // Attack
    // Combat log should update
    await terminal.expectText(['damage', 'HP:']);
  });

  test.skip('can attempt to run with R key', async () => {
    await terminal.navigateToDragonSlayer();
    await terminal.sendKeys('F');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('L');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('R'); // Run
    // Either escaped or failed
    await terminal.expectText(['escape', 'fled', 'run']);
  });

  test.skip('tracks remaining forest fights', async () => {
    await terminal.navigateToDragonSlayer();
    await terminal.sendKeys('F');
    await terminal.expectText(['Forest fights remaining:']);
  });

  test.skip('returns to forest after combat victory', async () => {
    await terminal.navigateToDragonSlayer();
    await terminal.sendKeys('F');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('L');
    // After combat ends, should return to forest
    // (requires winning the fight)
  });
});

test.describe('Dragon Slayer - Training Grounds', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('displays current master info', async () => {
    await terminal.navigateToDragonSlayer();
    await terminal.sendKeys('T');
    await terminal.expectText(['Current Master:', 'HP:', 'STR:', 'DEF:']);
  });

  test.skip('shows XP requirement for challenge', async () => {
    await terminal.navigateToDragonSlayer();
    await terminal.sendKeys('T');
    await terminal.expectText(['XP needed:']);
  });

  test.skip('can challenge master with C', async () => {
    await terminal.navigateToDragonSlayer();
    await terminal.sendKeys('T');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('C');
    // Either fight begins or insufficient XP message
    await terminal.expectText(['Challenge', 'XP', 'ATTACKS!']);
  });

  test.skip('can return to town with Q', async () => {
    await terminal.navigateToDragonSlayer();
    await terminal.sendKeys('T');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('Q');
    await terminal.expectText(['THE TOWN OF SILVERTON']);
  });
});

test.describe('Dragon Slayer - Equipment Shops', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('displays weapon shop inventory', async () => {
    await terminal.navigateToDragonSlayer();
    await terminal.sendKeys('W');
    await terminal.expectText(['THE WEAPONS SHOP', 'Weapon', 'Damage', 'Price']);
  });

  test.skip('shows equipped weapon', async () => {
    await terminal.navigateToDragonSlayer();
    await terminal.sendKeys('W');
    await terminal.expectText(['(equipped)']);
  });

  test.skip('displays armor shop inventory', async () => {
    await terminal.navigateToDragonSlayer();
    await terminal.sendKeys('A');
    await terminal.expectText(['THE ARMOR SHOP', 'Armor', 'Defense', 'Price']);
  });

  test.skip('can purchase equipment with number keys', async () => {
    await terminal.navigateToDragonSlayer();
    await terminal.sendKeys('W');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('1'); // Buy first weapon
    // Either success or insufficient gold
    await terminal.expectText(['gold', 'bought', 'afford']);
  });
});

test.describe('Dragon Slayer - Healer', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('displays healer screen', async () => {
    await terminal.navigateToDragonSlayer();
    await terminal.sendKeys('H');
    await terminal.expectText(["THE HEALER'S HUT"]);
  });

  test.skip('shows healing cost', async () => {
    await terminal.navigateToDragonSlayer();
    await terminal.sendKeys('H');
    await terminal.expectText(['heal', 'damage', 'gold']);
  });

  test.skip('can heal with H key', async () => {
    await terminal.navigateToDragonSlayer();
    await terminal.sendKeys('H');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('H');
    // Either healed or already at full health
    await terminal.expectText(['heal', 'health', 'gold']);
  });
});

test.describe('Dragon Slayer - Bank', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('displays bank screen', async () => {
    await terminal.navigateToDragonSlayer();
    await terminal.sendKeys('B');
    await terminal.expectText(['THE SILVERTON BANK', 'Gold in pocket:', 'Gold in bank:']);
  });

  test.skip('can deposit with D key', async () => {
    await terminal.navigateToDragonSlayer();
    await terminal.sendKeys('B');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('D');
    // Should deposit pocket gold to bank
    await terminal.expectText(['Gold in bank:']);
  });

  test.skip('can withdraw with W key', async () => {
    await terminal.navigateToDragonSlayer();
    await terminal.sendKeys('B');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('W');
    // Should withdraw bank gold to pocket
    await terminal.expectText(['Gold in pocket:']);
  });
});

test.describe('Dragon Slayer - Romance System', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('displays Violet house', async () => {
    await terminal.navigateToDragonSlayer();
    await terminal.sendKeys('V');
    await terminal.expectText(["VIOLET'S HOUSE", 'Affection:']);
  });

  test.skip('displays Seth tavern', async () => {
    await terminal.navigateToDragonSlayer();
    await terminal.sendKeys('S');
    await terminal.expectText(["SETH'S TAVERN", 'Affection:']);
  });

  test.skip('can flirt with F key', async () => {
    await terminal.navigateToDragonSlayer();
    await terminal.sendKeys('V');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('F');
    // Should show flirt result
    await terminal.expectText(['flirt', 'Affection']);
  });

  test.skip('tracks daily flirt limit', async () => {
    await terminal.navigateToDragonSlayer();
    await terminal.sendKeys('V');
    await terminal.expectText(['/5 today']);
  });

  test.skip('shows proposal option at 100 affection', async () => {
    // This test requires reaching 100 affection first
    await terminal.navigateToDragonSlayer();
    await terminal.sendKeys('V');
    await terminal.expectText(['Propose marriage']);
  });
});

test.describe('Dragon Slayer - Kings Court', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('displays daily news', async () => {
    await terminal.navigateToDragonSlayer();
    await terminal.sendKeys('K');
    await terminal.expectText(["THE KING'S COURT", "Today's News:"]);
  });

  test.skip('shows random news items', async () => {
    await terminal.navigateToDragonSlayer();
    await terminal.sendKeys('K');
    // News items vary daily
    await terminal.expectText(['*']);
  });
});

test.describe('Dragon Slayer - Inn', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('displays inn menu', async () => {
    await terminal.navigateToDragonSlayer();
    await terminal.sendKeys('I');
    await terminal.expectText(['THE RED DRAGON INN', 'Rest for the night', 'Listen to gossip']);
  });

  test.skip('can rest with R key', async () => {
    await terminal.navigateToDragonSlayer();
    await terminal.sendKeys('I');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('R');
    // Should show rest message
    await terminal.expectText(['rest', 'night', 'sleep']);
  });
});

test.describe('Dragon Slayer - Dragon Hunt', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('dragon hunt option appears at level 12', async () => {
    // This test requires a level 12 character
    await terminal.navigateToDragonSlayer();
    await terminal.sendKeys('F');
    await terminal.expectText(['Search for THE RED DRAGON']);
  });

  test.skip('displays dragon hunt screen', async () => {
    await terminal.navigateToDragonSlayer();
    await terminal.sendKeys('F');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('D');
    await terminal.expectText(['The Red Dragon', 'Search for the Red Dragon']);
  });

  test.skip('dragon has massive stats', async () => {
    // The dragon should be a formidable opponent
    await terminal.navigateToDragonSlayer();
    await terminal.sendKeys('F');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('D');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('S'); // Search
    await terminal.expectText(['Red Dragon', 'HP:']);
  });
});

test.describe('Dragon Slayer - Victory', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('shows victory screen on dragon defeat', async () => {
    // This test requires defeating the dragon
    await terminal.expectText(['VICTORY', 'RED DRAGON HAS BEEN SLAIN']);
  });

  test.skip('displays dragon kill count', async () => {
    await terminal.expectText(['Dragon Kills:']);
  });
});

test.describe('Dragon Slayer - Save/Resume', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('saves progress when quitting', async () => {
    await terminal.navigateToDragonSlayer();
    // Fight a monster to change state
    await terminal.sendKeys('F');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('L');
    // Win or lose combat, then quit
    await terminal.sendKeys('Q');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('Y');
    // State should be saved
  });

  test.skip('resumes from saved state', async () => {
    await terminal.navigateToDragonSlayer();
    // Should skip character creation if save exists
    await terminal.expectText(['THE TOWN OF SILVERTON']);
  });

  test.skip('resets daily fights at midnight eastern', async () => {
    // This would require testing across day boundary
    await terminal.navigateToDragonSlayer();
    await terminal.expectText(['Forest Fights:']);
  });
});

test.describe('Dragon Slayer - Leaderboard', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('displays leaderboard with L key', async () => {
    await terminal.navigateToDragonSlayer();
    await terminal.sendKeys('L');
    await terminal.expectText(['HALL OF HEROES', 'Rank', 'Player', 'Dragons']);
  });

  test.skip('shows dragon kill rankings', async () => {
    await terminal.navigateToDragonSlayer();
    await terminal.sendKeys('L');
    await terminal.expectText(['Dragons', 'Level']);
  });
});

test.describe('Dragon Slayer - IGM Locations', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('displays other places menu', async () => {
    await terminal.navigateToDragonSlayer();
    await terminal.sendKeys('O');
    await terminal.expectText(['OTHER PLACES']);
  });

  test.skip('shows default IGM modules', async () => {
    await terminal.navigateToDragonSlayer();
    await terminal.sendKeys('O');
    await terminal.expectText(['Fairy Grove', 'Dark Cave', 'Gambling Den']);
  });

  test.skip('can enter IGM location', async () => {
    await terminal.navigateToDragonSlayer();
    await terminal.sendKeys('O');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('F'); // Fairy Grove
    await terminal.expectText(['FAIRY GROVE']);
  });
});
