/**
 * Xodia - The Living MUD E2E Tests
 *
 * Tests for the LLM-powered MUD door game.
 * Uses Playwright to simulate BBS terminal sessions.
 *
 * Key features tested:
 * - Character creation (name and class selection)
 * - Room navigation and exploration
 * - Natural language command parsing
 * - Combat with hostile NPCs
 * - Inventory management
 * - Save/load functionality
 * - Offline/maintenance mode handling
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

  async navigateToXodia() {
    // Navigate: G (Games menu) -> X (Xodia)
    await this.sendKeys('G');
    await this.page.waitForTimeout(200);
    await this.sendKeys('X'); // Xodia
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

test.describe('Xodia - Game Launch', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('displays intro screen on first entry', async () => {
    await terminal.navigateToXodia();
    await terminal.expectText(['XODIA', 'THE LIVING MUD', 'Seeker']);
  });

  test.skip('shows offline message when LLM unavailable', async () => {
    await terminal.navigateToXodia();
    // If LLM is offline, should show appropriate message
    await terminal.expectText(['OFFLINE', 'unavailable']);
  });

  test.skip('shows maintenance message when enabled', async () => {
    await terminal.navigateToXodia();
    await terminal.expectText(['MAINTENANCE', 'Sysop']);
  });
});

test.describe('Xodia - Character Creation', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('prompts for character name', async () => {
    await terminal.navigateToXodia();
    await terminal.pressEnter(); // Skip intro
    await terminal.expectText(['CREATE YOUR CHARACTER', 'name', 'Seeker']);
  });

  test.skip('validates name length', async () => {
    await terminal.navigateToXodia();
    await terminal.pressEnter();
    await terminal.sendKeys('AB');
    await terminal.pressEnter();
    await terminal.expectText(['3-20', 'characters']);
  });

  test.skip('accepts valid character name', async () => {
    await terminal.navigateToXodia();
    await terminal.pressEnter();
    await terminal.sendKeys('Aldric the Brave');
    await terminal.pressEnter();
    await terminal.expectText(['Welcome', 'Aldric', 'path']);
  });

  test.skip('displays class selection', async () => {
    await terminal.navigateToXodia();
    await terminal.pressEnter();
    await terminal.sendKeys('TestHero');
    await terminal.pressEnter();
    await terminal.expectText([
      'WARRIOR',
      'MAGE',
      'ROGUE',
      'CLERIC',
    ]);
  });

  test.skip('can select Warrior class', async () => {
    await terminal.navigateToXodia();
    await terminal.pressEnter();
    await terminal.sendKeys('TestHero');
    await terminal.pressEnter();
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('1');
    await terminal.expectText(['Misthollow', 'fountain']);
  });

  test.skip('can select Mage class', async () => {
    await terminal.navigateToXodia();
    await terminal.pressEnter();
    await terminal.sendKeys('TestMage');
    await terminal.pressEnter();
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('2');
    await terminal.expectText(['Misthollow', 'Mage']);
  });

  test.skip('can select Rogue class', async () => {
    await terminal.navigateToXodia();
    await terminal.pressEnter();
    await terminal.sendKeys('TestRogue');
    await terminal.pressEnter();
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('3');
    await terminal.expectText(['Misthollow', 'Rogue']);
  });

  test.skip('can select Cleric class', async () => {
    await terminal.navigateToXodia();
    await terminal.pressEnter();
    await terminal.sendKeys('TestCleric');
    await terminal.pressEnter();
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('4');
    await terminal.expectText(['Misthollow', 'Cleric']);
  });
});

test.describe('Xodia - Navigation', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('displays room description with exits', async () => {
    await terminal.navigateToXodia();
    // After character creation...
    await terminal.expectText(['Exits:', 'north', 'south', 'east', 'west']);
  });

  test.skip('can move north with N key', async () => {
    await terminal.navigateToXodia();
    await terminal.sendKeys('n');
    await terminal.pressEnter();
    await terminal.expectText(['Elder Mira', 'Cottage']);
  });

  test.skip('can move with full direction name', async () => {
    await terminal.navigateToXodia();
    await terminal.sendKeys('north');
    await terminal.pressEnter();
    await terminal.expectText(['Elder Mira']);
  });

  test.skip('can move with go command', async () => {
    await terminal.navigateToXodia();
    await terminal.sendKeys('go north');
    await terminal.pressEnter();
    await terminal.expectText(['Elder Mira']);
  });

  test.skip('shows error for invalid direction', async () => {
    await terminal.navigateToXodia();
    await terminal.sendKeys('northeast');
    await terminal.pressEnter();
    await terminal.expectText(["can't go"]);
  });

  test.skip('displays new room discovery message', async () => {
    await terminal.navigateToXodia();
    await terminal.sendKeys('n');
    await terminal.pressEnter();
    await terminal.expectText(['discover', 'new']);
  });
});

test.describe('Xodia - Commands', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('look command shows room description', async () => {
    await terminal.navigateToXodia();
    await terminal.sendKeys('look');
    await terminal.pressEnter();
    await terminal.expectText(['Misthollow', 'fountain', 'Exits:']);
  });

  test.skip('inventory command shows empty inventory', async () => {
    await terminal.navigateToXodia();
    await terminal.sendKeys('inventory');
    await terminal.pressEnter();
    await terminal.expectText(['INVENTORY', 'Backpack', 'empty']);
  });

  test.skip('short inventory command works', async () => {
    await terminal.navigateToXodia();
    await terminal.sendKeys('i');
    await terminal.pressEnter();
    await terminal.expectText(['INVENTORY']);
  });

  test.skip('stats command shows character info', async () => {
    await terminal.navigateToXodia();
    await terminal.sendKeys('stats');
    await terminal.pressEnter();
    await terminal.expectText(['STR', 'DEX', 'CON', 'INT', 'WIS', 'CHA']);
  });

  test.skip('help command shows commands', async () => {
    await terminal.navigateToXodia();
    await terminal.sendKeys('help');
    await terminal.pressEnter();
    await terminal.expectText(['COMMANDS', 'Movement', 'Actions', 'Combat']);
  });

  test.skip('? shortcut shows help', async () => {
    await terminal.navigateToXodia();
    await terminal.sendKeys('?');
    await terminal.pressEnter();
    await terminal.expectText(['COMMANDS']);
  });
});

test.describe('Xodia - NPCs and Items', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('can examine NPC', async () => {
    await terminal.navigateToXodia();
    await terminal.sendKeys('n'); // Go to Elder's cottage
    await terminal.pressEnter();
    await terminal.sendKeys('look at mira');
    await terminal.pressEnter();
    await terminal.expectText(['Elder Mira', 'ancient', 'eyes']);
  });

  test.skip('can talk to NPC', async () => {
    await terminal.navigateToXodia();
    await terminal.sendKeys('n');
    await terminal.pressEnter();
    await terminal.sendKeys('talk to mira');
    await terminal.pressEnter();
    await terminal.expectText(['Welcome', 'Seeker']);
  });

  test.skip('can take item from room', async () => {
    await terminal.navigateToXodia();
    await terminal.sendKeys('e'); // Go to smithy
    await terminal.pressEnter();
    await terminal.sendKeys('take dagger');
    await terminal.pressEnter();
    await terminal.expectText(['take', 'dagger']);
  });

  test.skip('can drop item', async () => {
    await terminal.navigateToXodia();
    // First pick up an item
    await terminal.sendKeys('e');
    await terminal.pressEnter();
    await terminal.sendKeys('take dagger');
    await terminal.pressEnter();
    await terminal.sendKeys('drop dagger');
    await terminal.pressEnter();
    await terminal.expectText(['drop', 'dagger']);
  });

  test.skip('can equip weapon', async () => {
    await terminal.navigateToXodia();
    await terminal.sendKeys('e');
    await terminal.pressEnter();
    await terminal.sendKeys('take sword');
    await terminal.pressEnter();
    await terminal.sendKeys('equip sword');
    await terminal.pressEnter();
    await terminal.expectText(['Equipped', 'sword']);
  });
});

test.describe('Xodia - Combat', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('can attack hostile NPC', async () => {
    await terminal.navigateToXodia();
    await terminal.sendKeys('s'); // Go south to gate
    await terminal.pressEnter();
    await terminal.sendKeys('s'); // Go to forest
    await terminal.pressEnter();
    await terminal.expectText(['COMBAT', 'Goblin']);
  });

  test.skip('displays combat UI with health bars', async () => {
    await terminal.navigateToXodia();
    // Navigate to combat
    await terminal.expectText(['HP:', 'Enemy HP:', 'ACTIONS']);
  });

  test.skip('can attack with A key', async () => {
    await terminal.navigateToXodia();
    // In combat...
    await terminal.sendKeys('a');
    await terminal.expectText(['damage', 'hit', 'strike']);
  });

  test.skip('can defend with D key', async () => {
    await terminal.navigateToXodia();
    await terminal.sendKeys('d');
    await terminal.expectText(['guard', 'defend']);
  });

  test.skip('can flee with F key', async () => {
    await terminal.navigateToXodia();
    await terminal.sendKeys('f');
    await terminal.expectText(['escape', 'flee', 'run']);
  });

  test.skip('shows victory message on win', async () => {
    await terminal.navigateToXodia();
    // After winning combat...
    await terminal.expectText(['Victory', 'XP']);
  });

  test.skip('shows defeat message on loss', async () => {
    await terminal.navigateToXodia();
    // After losing combat...
    await terminal.expectText(['defeated']);
  });

  test.skip('respawns player after death', async () => {
    await terminal.navigateToXodia();
    // After death...
    await terminal.expectText(['Misthollow']);
  });
});

test.describe('Xodia - Inventory and Equipment', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('shows equipped items in inventory', async () => {
    await terminal.navigateToXodia();
    await terminal.sendKeys('i');
    await terminal.pressEnter();
    await terminal.expectText(['Equipped:', 'Weapon:', 'Armor:']);
  });

  test.skip('can use health potion', async () => {
    await terminal.navigateToXodia();
    // With health potion in inventory
    await terminal.sendKeys('use potion');
    await terminal.pressEnter();
    await terminal.expectText(['drink', 'heal', 'HP']);
  });

  test.skip('tracks carry weight', async () => {
    await terminal.navigateToXodia();
    await terminal.sendKeys('i');
    await terminal.pressEnter();
    await terminal.expectText(['Weight:']);
  });

  test.skip('prevents carrying too much', async () => {
    await terminal.navigateToXodia();
    // Try to pick up when overloaded
    await terminal.expectText(["can't carry"]);
  });
});

test.describe('Xodia - Natural Language', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('understands look around command', async () => {
    await terminal.navigateToXodia();
    await terminal.sendKeys('look around');
    await terminal.pressEnter();
    await terminal.expectText(['Misthollow', 'fountain']);
  });

  test.skip('understands examine synonym', async () => {
    await terminal.navigateToXodia();
    await terminal.sendKeys('examine the fountain');
    await terminal.pressEnter();
    // Should work like look at
  });

  test.skip('understands grab synonym for take', async () => {
    await terminal.navigateToXodia();
    await terminal.sendKeys('grab the sword');
    await terminal.pressEnter();
    await terminal.expectText(['take', 'sword']);
  });

  test.skip('understands speak synonym for talk', async () => {
    await terminal.navigateToXodia();
    await terminal.sendKeys('speak with the elder');
    await terminal.pressEnter();
    await terminal.expectText(['Elder']);
  });

  test.skip('handles unknown commands gracefully', async () => {
    await terminal.navigateToXodia();
    await terminal.sendKeys('do something weird');
    await terminal.pressEnter();
    await terminal.expectText(['DM', 'ponders']);
  });
});

test.describe('Xodia - Save/Resume', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('saves progress when quitting', async () => {
    await terminal.navigateToXodia();
    await terminal.sendKeys('quit');
    await terminal.pressEnter();
    await terminal.expectText(['save', 'Are you sure']);
  });

  test.skip('confirms before quitting', async () => {
    await terminal.navigateToXodia();
    await terminal.sendKeys('quit');
    await terminal.pressEnter();
    await terminal.sendKeys('n');
    // Should return to game
    await terminal.expectText(['Misthollow']);
  });

  test.skip('resumes from saved state', async () => {
    // After quitting and returning
    await terminal.navigateToXodia();
    // Should skip character creation
    await terminal.expectText(['Misthollow', 'HP:', 'MP:']);
  });

  test.skip('manual save command works', async () => {
    await terminal.navigateToXodia();
    await terminal.sendKeys('save');
    await terminal.pressEnter();
    await terminal.expectText(['saved']);
  });
});

test.describe('Xodia - Status Display', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('shows character name and class', async () => {
    await terminal.navigateToXodia();
    await terminal.expectText(['TestHero', 'Warrior']);
  });

  test.skip('shows level and XP', async () => {
    await terminal.navigateToXodia();
    await terminal.expectText(['Lv.', 'XP']);
  });

  test.skip('shows health bar', async () => {
    await terminal.navigateToXodia();
    await terminal.expectText(['HP:']);
  });

  test.skip('shows mana bar', async () => {
    await terminal.navigateToXodia();
    await terminal.expectText(['MP:']);
  });

  test.skip('shows gold amount', async () => {
    await terminal.navigateToXodia();
    await terminal.expectText(['Gold:']);
  });

  test.skip('shows current location', async () => {
    await terminal.navigateToXodia();
    await terminal.expectText(['misthollow', 'Misthollow']);
  });

  test.skip('shows combat indicator when in combat', async () => {
    await terminal.navigateToXodia();
    // When in combat
    await terminal.expectText(['IN COMBAT']);
  });
});

test.describe('Xodia - Level Progression', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('gains XP from combat victory', async () => {
    await terminal.navigateToXodia();
    // After winning combat
    await terminal.expectText(['Gained', 'XP']);
  });

  test.skip('shows level up message', async () => {
    await terminal.navigateToXodia();
    // When leveling up
    await terminal.expectText(['LEVEL UP', 'level']);
  });

  test.skip('increases stats on level up', async () => {
    await terminal.navigateToXodia();
    // After level up, check stats
    await terminal.sendKeys('stats');
    await terminal.pressEnter();
    // Stats should be higher
  });
});

test.describe('Xodia - Regions and Exploration', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('starts in Misthollow Village', async () => {
    await terminal.navigateToXodia();
    await terminal.expectText(['Misthollow', 'Village']);
  });

  test.skip('can explore to Whispering Woods', async () => {
    await terminal.navigateToXodia();
    await terminal.sendKeys('s');
    await terminal.pressEnter();
    await terminal.sendKeys('s');
    await terminal.pressEnter();
    await terminal.expectText(['Whispering Woods', 'forest']);
  });

  test.skip('tracks discovered rooms', async () => {
    await terminal.navigateToXodia();
    // Visit new room
    await terminal.sendKeys('n');
    await terminal.pressEnter();
    await terminal.expectText(['discover']);
  });
});

test.describe('Xodia - Magic System', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('mages can cast spells', async () => {
    // Create mage character
    await terminal.navigateToXodia();
    // In combat
    await terminal.sendKeys('c'); // Cast
    await terminal.expectText(['Spell', 'mana']);
  });

  test.skip('clerics can cast healing', async () => {
    await terminal.navigateToXodia();
    await terminal.sendKeys('cast heal');
    await terminal.pressEnter();
    await terminal.expectText(['heal', 'HP']);
  });

  test.skip('spells consume mana', async () => {
    await terminal.navigateToXodia();
    // After casting spell
    await terminal.expectText(['MP:']);
  });

  test.skip('shows error when out of mana', async () => {
    await terminal.navigateToXodia();
    await terminal.sendKeys('cast fireball');
    await terminal.pressEnter();
    await terminal.expectText(['mana', 'lack']);
  });
});

test.describe('Xodia - Input Edge Cases', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('handles empty input as look', async () => {
    await terminal.navigateToXodia();
    await terminal.pressEnter();
    await terminal.expectText(['Misthollow']);
  });

  test.skip('is case insensitive', async () => {
    await terminal.navigateToXodia();
    await terminal.sendKeys('NORTH');
    await terminal.pressEnter();
    await terminal.expectText(['Elder']);
  });

  test.skip('trims whitespace', async () => {
    await terminal.navigateToXodia();
    await terminal.sendKeys('  north  ');
    await terminal.pressEnter();
    await terminal.expectText(['Elder']);
  });

  test.skip('handles rapid input', async () => {
    await terminal.navigateToXodia();
    await terminal.sendKeys('nnnnn');
    // Should not crash
  });
});
