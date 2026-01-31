/**
 * Ultimo E2E Tests
 *
 * Tests for the Ultima Online inspired MMO door game.
 * Uses Playwright to simulate BBS terminal sessions.
 *
 * Key features tested:
 * - Character creation (name and stat allocation)
 * - World navigation with WASD movement
 * - Skill-based progression (skills improve through use)
 * - Combat system (PvE with monsters)
 * - NPC interaction (shops, trainers, healers, bankers)
 * - Crafting system (blacksmithing, tailoring, alchemy, etc.)
 * - Housing system (purchase, storage, friends)
 * - Player trading marketplace
 * - Party system
 * - Quests and quest completion
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

  async navigateToUltimo() {
    // Navigate: G (Games menu) -> U (Ultimo)
    await this.sendKeys('G');
    await this.page.waitForTimeout(200);
    await this.sendKeys('U'); // Ultimo
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

test.describe('Ultimo - Intro and Character Creation', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('displays intro screen on first entry', async () => {
    await terminal.navigateToUltimo();
    await terminal.expectText(['ULTIMO', 'A World of Adventure']);
  });

  test.skip('advances from intro to character creation', async () => {
    await terminal.navigateToUltimo();
    await terminal.pressEnter();
    await terminal.expectText(['CREATE YOUR ADVENTURER', 'Enter your name']);
  });

  test.skip('validates character name length', async () => {
    await terminal.navigateToUltimo();
    await terminal.pressEnter();
    await terminal.sendKeys('AB');
    await terminal.pressEnter();
    await terminal.expectText(['at least 3 characters']);
  });

  test.skip('accepts valid character name', async () => {
    await terminal.navigateToUltimo();
    await terminal.pressEnter();
    await terminal.sendKeys('SirGalahad');
    await terminal.pressEnter();
    await terminal.expectText(['Allocate your starting attributes']);
  });

  test.skip('displays stat allocation with 15 points', async () => {
    await terminal.navigateToUltimo();
    await terminal.pressEnter();
    await terminal.sendKeys('TestHero');
    await terminal.pressEnter();
    await terminal.expectText(['Points remaining: 15', 'Strength', 'Dexterity', 'Intelligence']);
  });

  test.skip('allocates stats with number keys', async () => {
    await terminal.navigateToUltimo();
    await terminal.pressEnter();
    await terminal.sendKeys('TestHero');
    await terminal.pressEnter();
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('1'); // Add strength
    await terminal.expectText(['Points remaining: 14']);
  });

  test.skip('finishes creation with F when points used', async () => {
    await terminal.navigateToUltimo();
    await terminal.pressEnter();
    await terminal.sendKeys('TestHero');
    await terminal.pressEnter();
    // Allocate all 15 points
    for (let i = 0; i < 15; i++) {
      await terminal.sendKeys('1');
      await terminal.page.waitForTimeout(50);
    }
    await terminal.sendKeys('F');
    await terminal.expectText(['Welcome to Britannia']);
  });
});

test.describe('Ultimo - World Navigation', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('displays world view with status bar', async () => {
    await terminal.navigateToUltimo();
    await terminal.expectText(['HP:', 'MP:', 'Gold:']);
  });

  test.skip('shows terrain map', async () => {
    await terminal.navigateToUltimo();
    await terminal.expectText(['@']); // Player character
  });

  test.skip('moves with WASD keys', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('D'); // Move right
    // Position should update
    await terminal.expectText(['Britain']);
  });

  test.skip('moves with numpad keys', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('6'); // Move right (numpad)
    await terminal.expectText(['Britain']);
  });

  test.skip('supports diagonal movement', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('9'); // Move up-right
    await terminal.expectText(['Britain']);
  });

  test.skip('shows help with ? key', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('?');
    await terminal.expectText(['WASD=Move', 'C=Stats', 'I=Inv']);
  });

  test.skip('shows quit confirmation with Q', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('Q');
    await terminal.expectText(['SAVE & QUIT', 'Are you sure?']);
  });
});

test.describe('Ultimo - Character Stats', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('displays character stats with C key', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('C');
    await terminal.expectText(['CHARACTER STATS', 'Strength:', 'Dexterity:', 'Intelligence:']);
  });

  test.skip('shows combat stats', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('C');
    await terminal.expectText(['Attack Power:', 'Defense:']);
  });

  test.skip('shows equipment slots', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('C');
    await terminal.expectText(['Weapon:', 'Armor:', 'Shield:']);
  });

  test.skip('shows kill statistics', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('C');
    await terminal.expectText(['Monsters Slain:', 'Deaths:']);
  });

  test.skip('returns to world with Q', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('C');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('Q');
    await terminal.expectText(['@']); // Back to world view
  });
});

test.describe('Ultimo - Inventory', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('displays inventory with I key', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('I');
    await terminal.expectText(['INVENTORY']);
  });

  test.skip('shows starting items', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('I');
    await terminal.expectText(['Bread', 'Bandage']);
  });

  test.skip('can equip weapons with number keys', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('I');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('1');
    await terminal.expectText(['equipped', 'equip']);
  });

  test.skip('shows equipped marker', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('I');
    await terminal.expectText(['(equipped)']);
  });
});

test.describe('Ultimo - Skills', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('displays skills with K key', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('K');
    await terminal.expectText(['SKILLS']);
  });

  test.skip('shows starting skills', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('K');
    await terminal.expectText(['Wrestling', 'Healing']);
  });

  test.skip('groups skills by category', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('K');
    await terminal.expectText(['Combat:', 'Misc:']);
  });
});

test.describe('Ultimo - Combat', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('starts combat with F key', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('F');
    await terminal.expectText(['COMBAT:', 'Enemy HP:']);
  });

  test.skip('displays combat actions', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('F');
    await terminal.expectText(['Attack', 'Cast Spell', 'Run Away']);
  });

  test.skip('can attack with A key', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('F');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('A');
    await terminal.expectText(['damage', 'HP:']);
  });

  test.skip('can cast spells with C key', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('F');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('C');
    // Mana should be checked
    await terminal.expectText(['mana', 'cast', 'damage']);
  });

  test.skip('can run with R key', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('F');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('R');
    await terminal.expectText(['escaped', 'fled', 'run']);
  });

  test.skip('awards XP on victory', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('F');
    // Win combat
    await terminal.expectText(['XP', 'gold', 'Victory']);
  });
});

test.describe('Ultimo - NPC Interaction', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('talks to NPCs with T key', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('T');
    await terminal.expectText(['No one nearby', 'talk']);
  });

  test.skip('shows NPC dialogue', async () => {
    await terminal.navigateToUltimo();
    // Move to NPC location first
    await terminal.sendKeys('T');
    await terminal.expectText(['dialogue', 'Buy', 'Sell']);
  });

  test.skip('can buy from shops', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('T');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('1'); // Buy option
    await terminal.expectText(['BUY', 'gold']);
  });

  test.skip('can sell to shops', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('T');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('2'); // Sell option
    await terminal.expectText(['SELL', 'gold']);
  });
});

test.describe('Ultimo - Bank', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('displays bank screen', async () => {
    // Navigate to banker NPC
    await terminal.navigateToUltimo();
    await terminal.expectText(['BANK OF BRITANNIA', 'Gold on hand:', 'Gold in bank:']);
  });

  test.skip('can deposit with D key', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('D');
    await terminal.expectText(['Deposited', 'gold']);
  });

  test.skip('can withdraw with W key', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('W');
    await terminal.expectText(['Withdrew', 'gold']);
  });
});

test.describe('Ultimo - Healer', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('displays healer screen', async () => {
    await terminal.navigateToUltimo();
    await terminal.expectText(["HEALER'S SANCTUARY", 'HP:']);
  });

  test.skip('shows healing cost', async () => {
    await terminal.navigateToUltimo();
    await terminal.expectText(['Healing cost:', 'gold']);
  });

  test.skip('can heal with H key', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('H');
    await terminal.expectText(['Healed', 'health', 'gold']);
  });

  test.skip('can resurrect when dead', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('R');
    await terminal.expectText(['Resurrect', '500 gold']);
  });
});

test.describe('Ultimo - Training', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('displays training screen', async () => {
    await terminal.navigateToUltimo();
    await terminal.expectText(['TRAINING', 'gold']);
  });

  test.skip('shows trainable skills', async () => {
    await terminal.navigateToUltimo();
    await terminal.expectText(['Swordsmanship', 'Tactics']);
  });

  test.skip('shows training cost', async () => {
    await terminal.navigateToUltimo();
    await terminal.expectText(['gold']);
  });

  test.skip('can train skills with number keys', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('1');
    await terminal.expectText(['Trained', 'Cost:']);
  });
});

test.describe('Ultimo - Crafting', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('displays crafting menu with R key', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('R');
    await terminal.expectText(['CRAFTING', 'Blacksmithing', 'Tailoring', 'Carpentry', 'Alchemy', 'Cooking']);
  });

  test.skip('shows skill levels for each profession', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('R');
    await terminal.expectText(['Lv']);
  });

  test.skip('can select blacksmithing with 1', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('R');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('1');
    await terminal.expectText(['BLACKSMITHING RECIPES', 'Smelt Iron', 'Forge']);
  });

  test.skip('shows recipe requirements', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('R');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('1');
    await terminal.expectText(['Req:', '[READY]', '[need mats]']);
  });

  test.skip('shows materials needed', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('R');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('1');
    await terminal.expectText(['Iron Ore', 'Iron Ingot']);
  });

  test.skip('can craft items with number keys', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('R');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('1');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('1'); // Smelt iron
    await terminal.expectText(['Created', 'Crafting failed', 'materials']);
  });

  test.skip('returns to menu with B key', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('R');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('1');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('B');
    await terminal.expectText(['CRAFTING']);
  });
});

test.describe('Ultimo - Housing', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('displays housing menu with O key', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('O');
    await terminal.expectText(['HOUSING', 'Buy a House']);
  });

  test.skip('shows house ownership status', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('O');
    await terminal.expectText(['do not own a house', 'own a house']);
  });

  test.skip('can access buy screen with 1', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('O');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('1');
    await terminal.expectText(['BUY A HOUSE', 'Small Cottage', 'Castle']);
  });

  test.skip('shows house types with prices', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('O');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('1');
    await terminal.expectText(['10000 gold', '1000000 gold', 'slots']);
  });

  test.skip('can purchase house with number keys', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('O');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('1');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('1'); // Buy small cottage
    await terminal.expectText(['purchased', 'Not enough gold']);
  });

  test.skip('can access storage with 2', async () => {
    // Requires owning a house first
    await terminal.navigateToUltimo();
    await terminal.sendKeys('O');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('2');
    await terminal.expectText(['HOUSE STORAGE', 'Deposit', 'Withdraw']);
  });
});

test.describe('Ultimo - Player Trading', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('displays trade marketplace with M key', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('M');
    await terminal.expectText(['PLAYER MARKETPLACE', 'Your gold:']);
  });

  test.skip('shows available listings', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('M');
    await terminal.expectText(['Available Listings', 'No listings']);
  });

  test.skip('can create listing with C key', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('M');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('C');
    await terminal.expectText(['CREATE TRADE LISTING', 'Select an item']);
  });

  test.skip('shows suggested prices', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('M');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('C');
    await terminal.expectText(['~', 'gold']);
  });

  test.skip('can create listing with number keys', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('M');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('C');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('1');
    await terminal.expectText(['Trade listing created']);
  });
});

test.describe('Ultimo - Party System', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('displays party menu with P key', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('P');
    await terminal.expectText(['PARTY MANAGEMENT']);
  });

  test.skip('shows party status', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('P');
    await terminal.expectText(['not in a party', 'party members']);
  });

  test.skip('shows nearby players', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('P');
    await terminal.expectText(['Nearby players', 'No players']);
  });

  test.skip('can create party with C key', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('P');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('C');
    await terminal.expectText(['Party created']);
  });

  test.skip('can invite with I key', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('P');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('I');
    await terminal.expectText(['Invited', 'No players nearby']);
  });
});

test.describe('Ultimo - Quests', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('displays quest log with J key', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('J');
    await terminal.expectText(['QUEST LOG']);
  });

  test.skip('shows empty quest message', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('J');
    await terminal.expectText(['No active quests', 'Talk to NPCs']);
  });

  test.skip('shows quest progress', async () => {
    // Requires having a quest first
    await terminal.navigateToUltimo();
    await terminal.sendKeys('J');
    await terminal.expectText(['Kill', 'Collect']);
  });

  test.skip('shows ready to turn in marker', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('J');
    await terminal.expectText(['Ready to turn in']);
  });

  test.skip('can complete quests with number keys', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('J');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('1');
    await terminal.expectText(['Quest completed', 'requirements not met']);
  });
});

test.describe('Ultimo - Leaderboard', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('displays leaderboard with L key', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('L');
    await terminal.expectText(['HALL OF LEGENDS', 'Rank', 'Name', 'Level']);
  });

  test.skip('shows net worth', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('L');
    await terminal.expectText(['Net Worth']);
  });

  test.skip('returns to world on any key', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('L');
    await terminal.page.waitForTimeout(200);
    await terminal.pressEnter();
    await terminal.expectText(['@']); // Back to world
  });
});

test.describe('Ultimo - Death and Resurrection', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('displays death screen on defeat', async () => {
    await terminal.navigateToUltimo();
    await terminal.expectText(['YOU DIED', 'spirit wanders']);
  });

  test.skip('shows resurrection option', async () => {
    await terminal.navigateToUltimo();
    await terminal.expectText(['Resurrect at healer', '500 gold']);
  });

  test.skip('can resurrect with R key', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('R');
    await terminal.expectText(['resurrected', 'Not enough gold']);
  });

  test.skip('can quit from death screen', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('Q');
    await terminal.expectText(['SAVE & QUIT']);
  });
});

test.describe('Ultimo - Save/Resume', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('saves progress when quitting', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('Q');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('Y');
    await terminal.expectText(['saved', 'progress']);
  });

  test.skip('resumes from saved state', async () => {
    await terminal.navigateToUltimo();
    // Should skip character creation if save exists
    await terminal.expectText(['@']); // World view, not intro
  });

  test.skip('preserves inventory on resume', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('I');
    await terminal.expectText(['INVENTORY']);
  });

  test.skip('preserves skill progress on resume', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('K');
    await terminal.expectText(['SKILLS']);
  });
});

test.describe('Ultimo - Resource Gathering', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('can use environment with E key', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('E');
    await terminal.expectText(['Nothing to use', 'gather', 'You need a tool']);
  });

  test.skip('requires tools for gathering', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('E');
    await terminal.expectText(['need a tool', 'pickaxe', 'hatchet']);
  });

  test.skip('gathers resources with correct tool', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('E');
    await terminal.expectText(['You gather:', 'find nothing']);
  });
});

test.describe('Ultimo - Multiplayer Features', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('shows other players on map', async () => {
    await terminal.navigateToUltimo();
    await terminal.expectText(['P']); // Other player marker
  });

  test.skip('displays visible player names', async () => {
    await terminal.navigateToUltimo();
    await terminal.sendKeys('P'); // Party menu shows nearby
    await terminal.expectText(['Nearby players']);
  });
});
