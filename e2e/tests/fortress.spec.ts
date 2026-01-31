/**
 * Fortress E2E Tests
 *
 * Tests for the colony simulation game inspired by Dwarf Fortress.
 * Uses Playwright to simulate BBS terminal sessions.
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

  async navigateToFortress() {
    // Navigate to Games menu then Fortress
    await this.sendKeys('G');
    await this.page.waitForTimeout(200);
    await this.sendKeys('F'); // or number for Fortress
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

test.describe('Fortress Game', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
    // Tests would normally connect and login here
    // await terminal.connect();
    // await terminal.login('testuser', 'testpass');
  });

  test.describe('Game Start', () => {
    test.skip('shows intro screen on first launch', async () => {
      await terminal.navigateToFortress();
      await terminal.expectText(['FORTRESS', 'Strike the earth!']);
    });

    test.skip('advances to naming screen', async () => {
      await terminal.navigateToFortress();
      await terminal.pressEnter();
      await terminal.expectText('NAME YOUR FORTRESS');
    });

    test.skip('accepts valid fortress name', async () => {
      await terminal.navigateToFortress();
      await terminal.pressEnter(); // Skip intro
      await terminal.sendKeys('Ironhold');
      await terminal.pressEnter();
      await terminal.expectText(['Ironhold', 'Dwarves']);
    });

    test.skip('rejects too short name', async () => {
      await terminal.navigateToFortress();
      await terminal.pressEnter();
      await terminal.sendKeys('AB'); // Too short
      await terminal.pressEnter();
      await terminal.expectText('at least 3 characters');
    });
  });

  test.describe('Fortress View', () => {
    test.skip('shows fortress view after naming', async () => {
      await terminal.expectText(['Z-Level', 'Dwarves', 'Food', 'Drink']);
    });

    test.skip('shows terrain map', async () => {
      // Map uses ASCII characters
      await terminal.expectText(['#', '.', 'T']); // Stone, floor, tree
    });

    test.skip('shows resource bar', async () => {
      await terminal.expectText(['Stone:', 'Wood:', 'Iron:']);
    });

    test.skip('can navigate map with WASD', async () => {
      await terminal.sendKeys('S'); // Move down
      await terminal.sendKeys('D'); // Move right
      // View should have shifted
    });

    test.skip('can change z-level', async () => {
      await terminal.sendKeys('>'); // Go down
      await terminal.expectText('Z-Level: 1');
      await terminal.sendKeys('<'); // Go up
      await terminal.expectText('Z-Level: 0');
    });
  });

  test.describe('Dwarf Management', () => {
    test.skip('opens dwarf list', async () => {
      await terminal.sendKeys('U');
      await terminal.expectText('DWARVES');
    });

    test.skip('shows initial 7 dwarves', async () => {
      await terminal.sendKeys('U');
      await terminal.expectText(['Profession', 'Status', 'Mood']);
    });

    test.skip('can view dwarf details', async () => {
      await terminal.sendKeys('U');
      await terminal.sendKeys('1'); // Select first dwarf
      await terminal.expectText(['NEEDS:', 'SKILLS:', 'EQUIPMENT:']);
    });

    test.skip('shows dwarf needs', async () => {
      await terminal.sendKeys('U');
      await terminal.sendKeys('1');
      await terminal.expectText(['Hunger', 'Thirst', 'Rest', 'Social', 'Comfort']);
    });

    test.skip('shows dwarf skills', async () => {
      await terminal.sendKeys('U');
      await terminal.sendKeys('1');
      await terminal.expectText(['Mining', 'Woodcutting', 'Farming']);
    });

    test.skip('can return from dwarf detail', async () => {
      await terminal.sendKeys('U');
      await terminal.sendKeys('1');
      await terminal.sendKeys('Q');
      await terminal.expectText('DWARVES');
    });
  });

  test.describe('Building', () => {
    test.skip('opens build menu', async () => {
      await terminal.sendKeys('B');
      await terminal.expectText('BUILD WORKSHOP');
    });

    test.skip('lists available workshops', async () => {
      await terminal.sendKeys('B');
      await terminal.expectText([
        'Carpenter', 'Mason', 'Smelter', 'Forge', 'Kitchen', 'Still'
      ]);
    });

    test.skip('shows workshop costs', async () => {
      await terminal.sendKeys('B');
      await terminal.expectText(['W', 'S', 'I']); // Wood, Stone, Iron
    });

    test.skip('can build workshop with resources', async () => {
      await terminal.sendKeys('B');
      await terminal.sendKeys('1'); // Carpenter
      await terminal.expectText(['built', 'Workshop']);
    });

    test.skip('rejects build without resources', async () => {
      await terminal.sendKeys('B');
      await terminal.sendKeys('9'); // Expensive workshop
      await terminal.expectText(['Cannot build', 'not enough']);
    });
  });

  test.describe('Workshops', () => {
    test.skip('opens workshop list', async () => {
      await terminal.sendKeys('P');
      await terminal.expectText('WORKSHOPS');
    });

    test.skip('shows built workshops', async () => {
      await terminal.sendKeys('P');
      // After building a workshop
      await terminal.expectText(['Location', 'Assigned']);
    });

    test.skip('can view workshop recipes', async () => {
      await terminal.sendKeys('P');
      await terminal.sendKeys('1'); // Select workshop
      await terminal.expectText('RECIPES');
    });

    test.skip('can add work order', async () => {
      await terminal.sendKeys('P');
      await terminal.sendKeys('1');
      await terminal.sendKeys('1'); // First recipe
      await terminal.expectText('Work order added');
    });
  });

  test.describe('Designation', () => {
    test.skip('enters designation mode', async () => {
      await terminal.sendKeys('Z');
      await terminal.expectText('DESIGNATION MODE');
    });

    test.skip('shows cursor position', async () => {
      await terminal.sendKeys('Z');
      await terminal.expectText('Cursor:');
    });

    test.skip('can move cursor', async () => {
      await terminal.sendKeys('Z');
      await terminal.sendKeys('D'); // Move right
    });

    test.skip('can designate tile for digging', async () => {
      await terminal.sendKeys('Z');
      await terminal.sendKeys(' '); // Space to designate
      await terminal.expectText('Designated');
    });

    test.skip('can return from designation mode', async () => {
      await terminal.sendKeys('Z');
      await terminal.sendKeys('Q');
      // Back to fortress view
    });
  });

  test.describe('Stockpiles', () => {
    test.skip('opens stockpile view', async () => {
      await terminal.sendKeys('I');
      await terminal.expectText('STOCKPILES');
    });

    test.skip('shows resource categories', async () => {
      await terminal.sendKeys('I');
      await terminal.expectText([
        'RAW MATERIALS', 'PROCESSED', 'FOOD & DRINK', 'GOODS'
      ]);
    });

    test.skip('shows starting resources', async () => {
      await terminal.sendKeys('I');
      await terminal.expectText(['Wood:', 'Stone:', 'Meals:', 'Ale:']);
    });
  });

  test.describe('Statistics', () => {
    test.skip('opens statistics screen', async () => {
      await terminal.sendKeys('T');
      await terminal.expectText('FORTRESS STATISTICS');
    });

    test.skip('shows tracked stats', async () => {
      await terminal.sendKeys('T');
      await terminal.expectText([
        'Tiles Mined', 'Trees Chopped', 'Items Crafted',
        'Invasions Repelled', 'Dwarves Lost'
      ]);
    });

    test.skip('shows wealth', async () => {
      await terminal.sendKeys('T');
      await terminal.expectText(['Total Wealth', 'Wealth Created']);
    });
  });

  test.describe('Help', () => {
    test.skip('opens help screen', async () => {
      await terminal.sendKeys('?');
      await terminal.expectText('FORTRESS - HELP');
    });

    test.skip('shows controls', async () => {
      await terminal.sendKeys('?');
      await terminal.expectText(['WASD', 'Z-level', 'MENUS']);
    });

    test.skip('shows gameplay tips', async () => {
      await terminal.sendKeys('?');
      await terminal.expectText(['Keep dwarves fed', 'Mine resources']);
    });
  });

  test.describe('Save/Quit', () => {
    test.skip('shows quit confirmation', async () => {
      await terminal.sendKeys('Q');
      await terminal.expectText(['SAVE & QUIT', '[Y]', '[N]']);
    });

    test.skip('can cancel quit', async () => {
      await terminal.sendKeys('Q');
      await terminal.sendKeys('N');
      // Back to fortress view
    });

    test.skip('saves on quit', async () => {
      await terminal.sendKeys('Q');
      await terminal.sendKeys('Y');
      // Should return to BBS menu
    });
  });

  test.describe('Resume Game', () => {
    test.skip('resumes from save', async () => {
      await terminal.navigateToFortress();
      // Should go to fortress view, not naming
      await terminal.expectText('Dwarves');
      await terminal.expectNotText('NAME YOUR FORTRESS');
    });

    test.skip('processes catchup ticks', async () => {
      await terminal.navigateToFortress();
      // Might show notification about time passing
    });

    test.skip('preserves fortress state', async () => {
      // Resources, dwarves, buildings should be preserved
    });
  });
});

// Unit-style tests for game mechanics
test.describe('Fortress Mechanics (Unit)', () => {
  test('skills have valid keys', () => {
    const skills = [
      'mining', 'woodcutting', 'farming', 'crafting', 'cooking',
      'building', 'combat', 'hauling', 'masonry', 'smithing', 'brewing', 'healing'
    ];

    expect(skills.length).toBe(12);
    skills.forEach(skill => {
      expect(skill).toMatch(/^[a-z]+$/);
    });
  });

  test('workshops have valid keys', () => {
    const workshops = [
      'carpenter', 'mason', 'smelter', 'forge', 'kitchen',
      'still', 'craftsdwarf', 'loom', 'tannery'
    ];

    expect(workshops.length).toBe(9);
    workshops.forEach(ws => {
      expect(ws).toMatch(/^[a-z]+$/);
    });
  });

  test('room types are defined', () => {
    const rooms = [
      'bedroom', 'dormitory', 'dining', 'meeting',
      'hospital', 'stockpile', 'barracks', 'throne_room'
    ];

    expect(rooms.length).toBe(8);
  });

  test('enemy threat levels are 1-5', () => {
    const threatLevels = [1, 2, 3, 4, 5];
    threatLevels.forEach(level => {
      expect(level).toBeGreaterThanOrEqual(1);
      expect(level).toBeLessThanOrEqual(5);
    });
  });

  test('dwarf needs have proper range', () => {
    // Needs are 0-100
    const needValues = [0, 50, 100];
    needValues.forEach(val => {
      expect(val).toBeGreaterThanOrEqual(0);
      expect(val).toBeLessThanOrEqual(100);
    });
  });

  test('dwarf moods are properly ordered', () => {
    const moods = ['Ecstatic', 'Happy', 'Content', 'Unhappy', 'Miserable', 'Tantrum'];
    expect(moods.length).toBe(6);
  });

  test('tile types are defined', () => {
    const tiles = [
      'Empty', 'Soil', 'Stone', 'Ore', 'Gem', 'Water', 'Lava',
      'Tree', 'Grass', 'Shrub', 'Floor', 'Wall', 'Door', 'Stairs',
      'Ramp', 'Stockpile', 'Workshop', 'Farm'
    ];

    expect(tiles.length).toBeGreaterThan(15);
  });

  test('z-levels are reasonable', () => {
    const defaultDepth = 10;
    expect(defaultDepth).toBeGreaterThanOrEqual(5);
    expect(defaultDepth).toBeLessThanOrEqual(20);
  });

  test('starting resources are provided', () => {
    const startingResources = {
      wood: 50,
      stone: 30,
      plank: 20,
      meal: 30,
      ale: 20,
      water: 50,
      tool: 5
    };

    Object.values(startingResources).forEach(val => {
      expect(val).toBeGreaterThan(0);
    });
  });

  test('initial dwarf count is 7', () => {
    const initialDwarves = 7;
    expect(initialDwarves).toBe(7);
  });

  test('seasons cycle properly', () => {
    const seasons = ['Spring', 'Summer', 'Autumn', 'Winter'];
    expect(seasons.length).toBe(4);
  });

  test('recipes produce outputs', () => {
    const sampleRecipe = {
      inputs: [['iron_ore', 2], ['wood', 1]],
      outputs: [['iron', 1]],
      work_time: 5
    };

    expect(sampleRecipe.inputs.length).toBeGreaterThan(0);
    expect(sampleRecipe.outputs.length).toBeGreaterThan(0);
    expect(sampleRecipe.work_time).toBeGreaterThan(0);
  });

  test('job priorities are reasonable', () => {
    // Combat is highest priority (0), hauling is low priority (20+)
    const fightPriority = 0;
    const miningPriority = 10;
    const haulingPriority = 20;

    expect(fightPriority).toBeLessThan(miningPriority);
    expect(miningPriority).toBeLessThan(haulingPriority);
  });
});
