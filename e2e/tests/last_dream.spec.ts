/**
 * Last Dream E2E Tests
 *
 * Tests for the FF1-2 style JRPG door game.
 * Uses Playwright to simulate BBS terminal sessions.
 *
 * Key features tested:
 * - Party creation (4 members, 6 classes)
 * - Overworld map navigation
 * - Turn-based combat with ATB system
 * - Town services (inn, shop, temple)
 * - Dungeon exploration
 * - Crystal quests progression
 * - Equipment and inventory management
 * - Magic and ability system
 * - Transportation (walking, ship, airship)
 * - Save/load functionality
 * - Very rare simulation hints
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

  async navigateToLastDream() {
    // Navigate: G (Games menu) -> L (Last Dream)
    await this.sendKeys('G');
    await this.page.waitForTimeout(200);
    await this.sendKeys('L'); // Last Dream
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

test.describe('Last Dream - Party Creation', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
    // Tests would normally connect and login here
    // await terminal.connect();
    // await terminal.login('testuser', 'testpass');
  });

  test.skip('displays party creation on first entry', async () => {
    await terminal.navigateToLastDream();
    await terminal.expectText(['PARTY CREATION', 'Create your party']);
  });

  test.skip('shows class selection options', async () => {
    await terminal.navigateToLastDream();
    await terminal.expectText([
      'Warrior',
      'Thief',
      'Mage',
      'Cleric',
      'Monk',
      'Knight',
    ]);
  });

  test.skip('accepts character name input', async () => {
    await terminal.navigateToLastDream();
    await terminal.sendKeys('1'); // Select Warrior
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('Cecil');
    await terminal.pressEnter();
    await terminal.expectText(['Character 2', 'Select class']);
  });

  test.skip('allows creating 4 party members', async () => {
    await terminal.navigateToLastDream();
    // Create 4 characters
    for (let i = 0; i < 4; i++) {
      await terminal.sendKeys('1'); // Warrior
      await terminal.page.waitForTimeout(100);
      await terminal.sendKeys(`Hero${i + 1}`);
      await terminal.pressEnter();
      await terminal.page.waitForTimeout(200);
    }
    // Should proceed to intro
    await terminal.expectText(['Light Warriors', 'crystals']);
  });

  test.skip('validates name length', async () => {
    await terminal.navigateToLastDream();
    await terminal.sendKeys('1');
    await terminal.page.waitForTimeout(100);
    await terminal.sendKeys('AB'); // Too short
    await terminal.pressEnter();
    await terminal.expectText(['at least 3 characters']);
  });

  test.skip('shows class stats before selection', async () => {
    await terminal.navigateToLastDream();
    await terminal.expectText(['HP:', 'STR:', 'DEF:', 'MAG:']);
  });
});

test.describe('Last Dream - Overworld Navigation', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('displays overworld map', async () => {
    await terminal.navigateToLastDream();
    await terminal.expectText(['OVERWORLD', 'Party Position']);
  });

  test.skip('shows party position on map', async () => {
    await terminal.navigateToLastDream();
    // Party marker visible on map
    await terminal.expectText(['@', 'Location:']);
  });

  test.skip('can move with arrow keys or WASD', async () => {
    await terminal.navigateToLastDream();
    await terminal.sendKeys('w'); // Move north
    await terminal.page.waitForTimeout(100);
    await terminal.expectText(['OVERWORLD']);
  });

  test.skip('shows current location info', async () => {
    await terminal.navigateToLastDream();
    await terminal.expectText(['Location:', 'Cornelia']);
  });

  test.skip('can enter town with E key', async () => {
    await terminal.navigateToLastDream();
    await terminal.sendKeys('e'); // Enter location
    await terminal.expectText(['Welcome to', 'Inn', 'Shop']);
  });

  test.skip('displays terrain types correctly', async () => {
    await terminal.navigateToLastDream();
    // Map shows various terrain
    await terminal.expectText(['~', '^', '.']); // Water, mountain, grass
  });
});

test.describe('Last Dream - Town Services', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('displays town menu with all services', async () => {
    await terminal.navigateToLastDream();
    await terminal.sendKeys('e'); // Enter town
    await terminal.expectText([
      'Inn',
      'Weapon Shop',
      'Armor Shop',
      'Item Shop',
      'Temple',
    ]);
  });

  test.skip('can rest at inn', async () => {
    await terminal.navigateToLastDream();
    await terminal.sendKeys('e');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('I'); // Inn
    await terminal.expectText(['Rest for the night', 'gold']);
  });

  test.skip('can buy weapons at shop', async () => {
    await terminal.navigateToLastDream();
    await terminal.sendKeys('e');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('W'); // Weapon shop
    await terminal.expectText(['WEAPON SHOP', 'ATK', 'Price']);
  });

  test.skip('can buy armor at shop', async () => {
    await terminal.navigateToLastDream();
    await terminal.sendKeys('e');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('A'); // Armor shop
    await terminal.expectText(['ARMOR SHOP', 'DEF', 'Price']);
  });

  test.skip('can buy items at shop', async () => {
    await terminal.navigateToLastDream();
    await terminal.sendKeys('e');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('S'); // Item shop
    await terminal.expectText(['ITEM SHOP', 'Potion', 'Phoenix Down']);
  });

  test.skip('can revive at temple', async () => {
    await terminal.navigateToLastDream();
    await terminal.sendKeys('e');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('T'); // Temple
    await terminal.expectText(['TEMPLE', 'Revive', 'Remove status']);
  });
});

test.describe('Last Dream - Combat System', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('random encounters occur on overworld', async () => {
    await terminal.navigateToLastDream();
    // Move around to trigger encounter
    for (let i = 0; i < 20; i++) {
      await terminal.sendKeys('w');
      await terminal.page.waitForTimeout(50);
    }
    // Should eventually get a battle
    await terminal.expectText(['BATTLE', 'Enemy HP:', 'appears!']);
  });

  test.skip('displays ATB gauges for party', async () => {
    await terminal.navigateToLastDream();
    await terminal.expectText(['ATB:', '|']);
  });

  test.skip('shows combat actions when ATB full', async () => {
    await terminal.navigateToLastDream();
    await terminal.expectText(['Attack', 'Magic', 'Item', 'Defend', 'Run']);
  });

  test.skip('can attack with A key', async () => {
    await terminal.navigateToLastDream();
    await terminal.sendKeys('a'); // Attack
    await terminal.expectText(['attacks', 'damage']);
  });

  test.skip('can use magic with M key', async () => {
    await terminal.navigateToLastDream();
    await terminal.sendKeys('m'); // Magic menu
    await terminal.expectText(['MAGIC', 'MP:']);
  });

  test.skip('can use items with I key', async () => {
    await terminal.navigateToLastDream();
    await terminal.sendKeys('i'); // Item menu
    await terminal.expectText(['ITEMS', 'Potion']);
  });

  test.skip('can defend with D key', async () => {
    await terminal.navigateToLastDream();
    await terminal.sendKeys('d'); // Defend
    await terminal.expectText(['defends', 'defense']);
  });

  test.skip('can attempt to run with R key', async () => {
    await terminal.navigateToLastDream();
    await terminal.sendKeys('r'); // Run
    await terminal.expectText(['escape', 'fled', 'run']);
  });

  test.skip('shows combat log with messages', async () => {
    await terminal.navigateToLastDream();
    await terminal.expectText(['attacks', 'damage', 'HP:']);
  });

  test.skip('awards experience after victory', async () => {
    await terminal.navigateToLastDream();
    await terminal.expectText(['VICTORY', 'EXP gained:', 'Gold:']);
  });

  test.skip('party members can level up', async () => {
    await terminal.navigateToLastDream();
    await terminal.expectText(['LEVEL UP!', 'learned']);
  });
});

test.describe('Last Dream - Magic System', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('mages can cast black magic', async () => {
    await terminal.navigateToLastDream();
    await terminal.sendKeys('m');
    await terminal.expectText(['Fire', 'Ice', 'Bolt']);
  });

  test.skip('clerics can cast white magic', async () => {
    await terminal.navigateToLastDream();
    await terminal.sendKeys('m');
    await terminal.expectText(['Cure', 'Heal', 'Life']);
  });

  test.skip('spells consume MP', async () => {
    await terminal.navigateToLastDream();
    await terminal.sendKeys('m');
    await terminal.expectText(['MP:', 'Cost:']);
  });

  test.skip('cannot cast without enough MP', async () => {
    await terminal.navigateToLastDream();
    await terminal.sendKeys('m');
    await terminal.page.waitForTimeout(100);
    await terminal.sendKeys('1'); // Try to cast
    await terminal.expectText(['Not enough MP', 'MP:']);
  });

  test.skip('magic targets can be selected', async () => {
    await terminal.navigateToLastDream();
    await terminal.sendKeys('m');
    await terminal.page.waitForTimeout(100);
    await terminal.sendKeys('1');
    await terminal.expectText(['Select target', 'Enemy', 'Ally']);
  });
});

test.describe('Last Dream - Equipment Management', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('displays equipment screen', async () => {
    await terminal.navigateToLastDream();
    await terminal.sendKeys('E'); // Equipment
    await terminal.expectText(['EQUIPMENT', 'Weapon:', 'Armor:', 'Shield:']);
  });

  test.skip('shows equipment slots', async () => {
    await terminal.navigateToLastDream();
    await terminal.sendKeys('E');
    await terminal.expectText(['Weapon', 'Armor', 'Shield', 'Helmet', 'Accessory']);
  });

  test.skip('can equip items', async () => {
    await terminal.navigateToLastDream();
    await terminal.sendKeys('E');
    await terminal.page.waitForTimeout(100);
    await terminal.sendKeys('1'); // Select slot
    await terminal.expectText(['Equip', 'ATK:', 'DEF:']);
  });

  test.skip('class restrictions apply', async () => {
    await terminal.navigateToLastDream();
    await terminal.sendKeys('E');
    // Some equipment shows "Cannot equip"
    await terminal.expectText(['Cannot equip', 'restricted']);
  });

  test.skip('shows stat changes on equip', async () => {
    await terminal.navigateToLastDream();
    await terminal.sendKeys('E');
    await terminal.expectText(['ATK:', '+', 'DEF:']);
  });
});

test.describe('Last Dream - Inventory System', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('displays inventory screen', async () => {
    await terminal.navigateToLastDream();
    await terminal.sendKeys('I'); // Inventory
    await terminal.expectText(['INVENTORY', 'Items:', 'Gold:']);
  });

  test.skip('shows consumable items', async () => {
    await terminal.navigateToLastDream();
    await terminal.sendKeys('I');
    await terminal.expectText(['Potion', 'Ether', 'Phoenix Down']);
  });

  test.skip('can use items outside combat', async () => {
    await terminal.navigateToLastDream();
    await terminal.sendKeys('I');
    await terminal.page.waitForTimeout(100);
    await terminal.sendKeys('1'); // Use first item
    await terminal.expectText(['Use on', 'Select character']);
  });

  test.skip('shows key items', async () => {
    await terminal.navigateToLastDream();
    await terminal.sendKeys('I');
    await terminal.sendKeys('K'); // Key items tab
    await terminal.expectText(['KEY ITEMS']);
  });
});

test.describe('Last Dream - Dungeon Exploration', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('can enter dungeon from overworld', async () => {
    await terminal.navigateToLastDream();
    await terminal.sendKeys('e'); // Enter dungeon
    await terminal.expectText(['DUNGEON', 'Floor:']);
  });

  test.skip('displays dungeon floor layout', async () => {
    await terminal.navigateToLastDream();
    await terminal.expectText(['#', '.', '@']); // Walls, floor, party
  });

  test.skip('higher encounter rate in dungeons', async () => {
    await terminal.navigateToLastDream();
    // Encounters more frequent
    for (let i = 0; i < 10; i++) {
      await terminal.sendKeys('w');
      await terminal.page.waitForTimeout(50);
    }
    await terminal.expectText(['BATTLE']);
  });

  test.skip('can find treasure chests', async () => {
    await terminal.navigateToLastDream();
    await terminal.expectText(['!', 'Treasure']);
  });

  test.skip('can open chests', async () => {
    await terminal.navigateToLastDream();
    await terminal.sendKeys('e'); // Open chest
    await terminal.expectText(['Found:', 'gold', 'item']);
  });

  test.skip('displays stairs to next floor', async () => {
    await terminal.navigateToLastDream();
    await terminal.expectText(['>', 'stairs']);
  });

  test.skip('boss awaits on final floor', async () => {
    await terminal.navigateToLastDream();
    await terminal.expectText(['BOSS', 'Guardian']);
  });
});

test.describe('Last Dream - Crystal Quests', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('displays crystal status', async () => {
    await terminal.navigateToLastDream();
    await terminal.sendKeys('C'); // Crystal status
    await terminal.expectText(['CRYSTALS', 'Earth:', 'Fire:', 'Water:', 'Wind:']);
  });

  test.skip('earth crystal is first quest', async () => {
    await terminal.navigateToLastDream();
    await terminal.sendKeys('C');
    await terminal.expectText(['Earth Crystal', 'Darkened']);
  });

  test.skip('crystals light up after boss defeat', async () => {
    await terminal.navigateToLastDream();
    await terminal.sendKeys('C');
    await terminal.expectText(['Lit', 'Restored']);
  });

  test.skip('new areas unlock with crystals', async () => {
    await terminal.navigateToLastDream();
    // After lighting a crystal, new locations accessible
    await terminal.expectText(['unlocked', 'available']);
  });
});

test.describe('Last Dream - Transportation', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('starts with walking only', async () => {
    await terminal.navigateToLastDream();
    await terminal.expectText(['Walking', 'On foot']);
  });

  test.skip('can acquire ship', async () => {
    await terminal.navigateToLastDream();
    // After quest, ship available
    await terminal.expectText(['Ship', 'Board ship']);
  });

  test.skip('ship allows water travel', async () => {
    await terminal.navigateToLastDream();
    await terminal.sendKeys('B'); // Board ship
    await terminal.expectText(['Sailing', '~']); // Water tiles
  });

  test.skip('can acquire airship', async () => {
    await terminal.navigateToLastDream();
    await terminal.expectText(['Airship', 'Board airship']);
  });

  test.skip('airship allows flying over terrain', async () => {
    await terminal.navigateToLastDream();
    await terminal.sendKeys('B'); // Board airship
    await terminal.expectText(['Flying', 'Airship']);
  });
});

test.describe('Last Dream - Party Management', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('displays party status screen', async () => {
    await terminal.navigateToLastDream();
    await terminal.sendKeys('P'); // Party
    await terminal.expectText(['PARTY STATUS', 'HP:', 'MP:', 'Level:']);
  });

  test.skip('shows all 4 party members', async () => {
    await terminal.navigateToLastDream();
    await terminal.sendKeys('P');
    await terminal.expectText(['Character 1', 'Character 2', 'Character 3', 'Character 4']);
  });

  test.skip('displays individual stats', async () => {
    await terminal.navigateToLastDream();
    await terminal.sendKeys('P');
    await terminal.expectText(['STR:', 'DEF:', 'AGI:', 'MAG:', 'LCK:']);
  });

  test.skip('shows experience to next level', async () => {
    await terminal.navigateToLastDream();
    await terminal.sendKeys('P');
    await terminal.expectText(['EXP:', 'Next:']);
  });

  test.skip('can reorder party', async () => {
    await terminal.navigateToLastDream();
    await terminal.sendKeys('P');
    await terminal.page.waitForTimeout(100);
    await terminal.sendKeys('R'); // Reorder
    await terminal.expectText(['Reorder party', 'Select member']);
  });
});

test.describe('Last Dream - Save and Load', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('can save game with S key', async () => {
    await terminal.navigateToLastDream();
    await terminal.sendKeys('S'); // Save
    await terminal.expectText(['SAVE GAME', 'Progress saved']);
  });

  test.skip('saves progress when quitting', async () => {
    await terminal.navigateToLastDream();
    await terminal.sendKeys('Q'); // Quit
    await terminal.expectText(['SAVE & QUIT', 'Are you sure?']);
  });

  test.skip('resumes from saved state', async () => {
    await terminal.navigateToLastDream();
    // Should resume existing game
    await terminal.expectText(['Welcome back', 'Resuming']);
  });

  test.skip('displays play time', async () => {
    await terminal.navigateToLastDream();
    await terminal.expectText(['Play Time:', ':']);
  });
});

test.describe('Last Dream - Boss Battles', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('boss battles have unique intro', async () => {
    await terminal.navigateToLastDream();
    await terminal.expectText(['BOSS BATTLE', 'Guardian']);
  });

  test.skip('bosses have high stats', async () => {
    await terminal.navigateToLastDream();
    await terminal.expectText(['HP:', 'Boss']);
  });

  test.skip('cannot run from boss battles', async () => {
    await terminal.navigateToLastDream();
    await terminal.sendKeys('r'); // Try to run
    await terminal.expectText(["Can't escape", 'Cannot run']);
  });

  test.skip('victory lights crystal', async () => {
    await terminal.navigateToLastDream();
    await terminal.expectText(['Crystal restored', 'Light returns']);
  });
});

test.describe('Last Dream - Simulation Hints', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('simulation hints are very rare', async () => {
    // Hints appear with ~1/500 probability
    // This test documents the feature but cannot reliably test it
    await terminal.navigateToLastDream();
    // Hints would appear as subtle messages
  });

  test.skip('hints are easy to miss', async () => {
    // Hints blend into normal gameplay
    await terminal.navigateToLastDream();
    // Example hints: "static flicker", "patterns repeat"
  });

  test.skip('max 2 hints per playthrough', async () => {
    // Only 1-2 hints can be seen in a single game
    await terminal.navigateToLastDream();
  });
});

test.describe('Last Dream - Endgame', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('final dungeon unlocks after 4 crystals', async () => {
    await terminal.navigateToLastDream();
    await terminal.expectText(['Chaos Shrine', 'Final dungeon']);
  });

  test.skip('final boss battle', async () => {
    await terminal.navigateToLastDream();
    await terminal.expectText(['FINAL BATTLE', 'Chaos']);
  });

  test.skip('victory shows ending sequence', async () => {
    await terminal.navigateToLastDream();
    await terminal.expectText(['VICTORY', 'Light restored', 'Peace returns']);
  });

  test.skip('ending records completion', async () => {
    await terminal.navigateToLastDream();
    await terminal.expectText(['Game Complete', 'Play Time:', 'Hall of Fame']);
  });
});

test.describe('Last Dream - Leaderboard', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('displays leaderboard', async () => {
    await terminal.navigateToLastDream();
    await terminal.sendKeys('L'); // Leaderboard
    await terminal.expectText(['HALL OF FAME', 'Rank', 'Player', 'Time']);
  });

  test.skip('shows completion times', async () => {
    await terminal.navigateToLastDream();
    await terminal.sendKeys('L');
    await terminal.expectText(['Time:', ':']);
  });

  test.skip('shows party composition', async () => {
    await terminal.navigateToLastDream();
    await terminal.sendKeys('L');
    await terminal.expectText(['Party:', 'Level']);
  });

  test.skip('ranks by fastest completion', async () => {
    await terminal.navigateToLastDream();
    await terminal.sendKeys('L');
    await terminal.expectText(['#1', '#2', '#3']);
  });
});
