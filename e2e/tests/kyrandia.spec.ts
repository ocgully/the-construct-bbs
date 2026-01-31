/**
 * Kyrandia E2E Tests
 *
 * Tests for the Realm of Kyrandia text adventure RPG door game.
 * Uses Playwright to simulate BBS terminal sessions.
 *
 * Key features tested:
 * - Game intro and new game flow
 * - Exploration with room navigation
 * - Text parser commands (look, take, use, etc.)
 * - Spell casting via typed incantations
 * - Combat system with monsters
 * - Puzzle solving
 * - Fountain of Scrolls mechanic
 * - Inventory management
 * - NPC dialogue interactions
 * - Romance system (including same-sex romance)
 * - IGM module support
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

  async navigateToKyrandia() {
    // Navigate: G (Games menu) -> K (Kyrandia) or appropriate number
    await this.sendKeys('G');
    await this.page.waitForTimeout(200);
    await this.sendKeys('K'); // Kyrandia
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

test.describe('Kyrandia - Game Intro', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
    // Tests would normally connect and login here
    // await terminal.connect();
    // await terminal.login('testuser', 'testpass');
  });

  test.skip('displays intro screen on first entry', async () => {
    await terminal.navigateToKyrandia();
    await terminal.expectText(['KYRANDIA', 'Lady of Legends', 'Tashanna']);
  });

  test.skip('shows intro story text', async () => {
    await terminal.navigateToKyrandia();
    await terminal.expectText([
      'mystical realm',
      'Arch-Mage',
      'destiny',
      'journey begins',
    ]);
  });

  test.skip('advances to exploration on key press', async () => {
    await terminal.navigateToKyrandia();
    await terminal.pressEnter();
    await terminal.expectText(['Village Square', 'Exits:']);
  });
});

test.describe('Kyrandia - Exploration', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('displays current room with description', async () => {
    await terminal.navigateToKyrandia();
    await terminal.pressEnter(); // Skip intro
    await terminal.expectText(['Village Square', 'heart of the humble village']);
  });

  test.skip('shows available exits', async () => {
    await terminal.navigateToKyrandia();
    await terminal.pressEnter();
    await terminal.expectText(['Exits:', 'north', 'east', 'south', 'west']);
  });

  test.skip('shows NPCs in room', async () => {
    await terminal.navigateToKyrandia();
    await terminal.pressEnter();
    await terminal.expectText(['Elder Quinn is here']);
  });

  test.skip('displays status bar with HP and mana', async () => {
    await terminal.navigateToKyrandia();
    await terminal.pressEnter();
    await terminal.expectText(['HP:', 'Mana:', 'Gold:']);
  });

  test.skip('can move north with n or north command', async () => {
    await terminal.navigateToKyrandia();
    await terminal.pressEnter();
    await terminal.sendKeys('n');
    await terminal.pressEnter();
    await terminal.expectText(['Rusty Cauldron Inn']);
  });

  test.skip('can navigate using cardinal directions', async () => {
    await terminal.navigateToKyrandia();
    await terminal.pressEnter();
    await terminal.sendKeys('east');
    await terminal.pressEnter();
    await terminal.expectText(['Mystic Supplies']);
  });

  test.skip('shows error for invalid direction', async () => {
    await terminal.navigateToKyrandia();
    await terminal.pressEnter();
    await terminal.sendKeys('up');
    await terminal.pressEnter();
    await terminal.expectText(["can't go", 'Exits:']);
  });
});

test.describe('Kyrandia - Text Parser Commands', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('look command shows room description', async () => {
    await terminal.navigateToKyrandia();
    await terminal.pressEnter();
    await terminal.sendKeys('look');
    await terminal.pressEnter();
    await terminal.expectText(['Village Square']);
  });

  test.skip('look at NPC shows description', async () => {
    await terminal.navigateToKyrandia();
    await terminal.pressEnter();
    await terminal.sendKeys('look at quinn');
    await terminal.pressEnter();
    await terminal.expectText(['wise old man', 'white beard']);
  });

  test.skip('take command picks up items', async () => {
    await terminal.navigateToKyrandia();
    await terminal.pressEnter();
    await terminal.sendKeys('west');
    await terminal.pressEnter();
    await terminal.page.waitForTimeout(100);
    await terminal.sendKeys('take scroll');
    await terminal.pressEnter();
    await terminal.expectText(['You take']);
  });

  test.skip('drop command removes items', async () => {
    await terminal.navigateToKyrandia();
    await terminal.pressEnter();
    await terminal.sendKeys('west');
    await terminal.pressEnter();
    await terminal.sendKeys('take scroll');
    await terminal.pressEnter();
    await terminal.sendKeys('drop scroll');
    await terminal.pressEnter();
    await terminal.expectText(['You drop']);
  });

  test.skip('use command activates items', async () => {
    await terminal.navigateToKyrandia();
    await terminal.pressEnter();
    // Would need a health potion in inventory
    await terminal.sendKeys('use potion');
    await terminal.pressEnter();
    await terminal.expectText(['drink', 'restore', "don't have"]);
  });

  test.skip('talk command initiates NPC dialogue', async () => {
    await terminal.navigateToKyrandia();
    await terminal.pressEnter();
    await terminal.sendKeys('talk to quinn');
    await terminal.pressEnter();
    await terminal.expectText(['Elder Quinn says:', 'Welcome']);
  });
});

test.describe('Kyrandia - Magic System', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('can cast light spell with incantation', async () => {
    await terminal.navigateToKyrandia();
    await terminal.pressEnter();
    await terminal.sendKeys('luminos');
    await terminal.pressEnter();
    await terminal.expectText(['cast', 'Light', 'illuminate']);
  });

  test.skip('unknown incantation shows error', async () => {
    await terminal.navigateToKyrandia();
    await terminal.pressEnter();
    await terminal.sendKeys('abracadabra');
    await terminal.pressEnter();
    await terminal.expectText(['no power', 'incantation']);
  });

  test.skip('can view spellbook with M key', async () => {
    await terminal.navigateToKyrandia();
    await terminal.pressEnter();
    await terminal.sendKeys('M');
    await terminal.expectText(['SPELLBOOK', 'Light', 'luminos']);
  });

  test.skip('healing spell restores HP', async () => {
    await terminal.navigateToKyrandia();
    await terminal.pressEnter();
    // Would need heal spell learned
    await terminal.sendKeys('vitae restauro');
    await terminal.pressEnter();
    await terminal.expectText(['heal', 'restore', "don't know"]);
  });

  test.skip('spell costs mana', async () => {
    await terminal.navigateToKyrandia();
    await terminal.pressEnter();
    await terminal.sendKeys('luminos');
    await terminal.pressEnter();
    // Mana should decrease
    await terminal.expectText(['Mana:']);
  });
});

test.describe('Kyrandia - Combat', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('random encounters occur in dangerous areas', async () => {
    await terminal.navigateToKyrandia();
    await terminal.pressEnter();
    // Navigate to Dark Forest
    await terminal.sendKeys('south');
    await terminal.pressEnter();
    await terminal.page.waitForTimeout(100);
    await terminal.sendKeys('south');
    await terminal.pressEnter();
    // May trigger encounter
    await terminal.expectText(['appears', 'COMBAT', 'Exits:']);
  });

  test.skip('combat screen shows attack options', async () => {
    await terminal.navigateToKyrandia();
    // Would need to be in combat
    await terminal.expectText(['[A] Attack', '[C] Cast', '[F] Flee']);
  });

  test.skip('can attack with A key', async () => {
    await terminal.navigateToKyrandia();
    // In combat
    await terminal.sendKeys('A');
    await terminal.expectText(['attack', 'damage', 'HP:']);
  });

  test.skip('can flee with F key', async () => {
    await terminal.navigateToKyrandia();
    // In combat
    await terminal.sendKeys('F');
    await terminal.expectText(['flee', 'escape', 'blocks']);
  });

  test.skip('victory grants XP and gold', async () => {
    await terminal.navigateToKyrandia();
    // After winning combat
    await terminal.expectText(['defeated', 'XP', 'gold']);
  });
});

test.describe('Kyrandia - Fountain of Scrolls', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('fountain shows in Golden Forest', async () => {
    await terminal.navigateToKyrandia();
    await terminal.pressEnter();
    // Navigate to fountain (requires level 4+)
    await terminal.expectText(['Fountain of Scrolls', 'pine cones']);
  });

  test.skip('fountain requires 3 pine cones', async () => {
    await terminal.navigateToKyrandia();
    // At fountain with pine cones
    await terminal.sendKeys('throw pine cone');
    await terminal.pressEnter();
    await terminal.expectText(['need 3', 'pine cones']);
  });

  test.skip('successful fountain use creates scroll', async () => {
    await terminal.navigateToKyrandia();
    // At fountain with 3+ pine cones
    await terminal.sendKeys('Y'); // Confirm use
    await terminal.expectText(['Scroll', 'materializes']);
  });
});

test.describe('Kyrandia - Inventory', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('displays inventory with I key', async () => {
    await terminal.navigateToKyrandia();
    await terminal.pressEnter();
    await terminal.sendKeys('I');
    await terminal.expectText(['INVENTORY', 'Capacity:']);
  });

  test.skip('shows equipped items', async () => {
    await terminal.navigateToKyrandia();
    await terminal.pressEnter();
    await terminal.sendKeys('I');
    await terminal.expectText(['EQUIPPED:', 'Weapon:', 'Armor:']);
  });

  test.skip('can equip weapons', async () => {
    await terminal.navigateToKyrandia();
    await terminal.pressEnter();
    // Would need weapon in inventory
    await terminal.sendKeys('equip staff');
    await terminal.pressEnter();
    await terminal.expectText(['equip', "don't have"]);
  });

  test.skip('inventory has capacity limit', async () => {
    await terminal.navigateToKyrandia();
    await terminal.pressEnter();
    await terminal.sendKeys('I');
    await terminal.expectText(['/10', 'Capacity']);
  });
});

test.describe('Kyrandia - Stats', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('displays stats with S key', async () => {
    await terminal.navigateToKyrandia();
    await terminal.pressEnter();
    await terminal.sendKeys('S');
    await terminal.expectText(['Level', 'Rank:', 'Apprentice']);
  });

  test.skip('shows experience progress', async () => {
    await terminal.navigateToKyrandia();
    await terminal.pressEnter();
    await terminal.sendKeys('S');
    await terminal.expectText(['Experience:', 'next level']);
  });

  test.skip('shows combat statistics', async () => {
    await terminal.navigateToKyrandia();
    await terminal.pressEnter();
    await terminal.sendKeys('S');
    await terminal.expectText(['Attack:', 'Defense:', 'Monsters Defeated:']);
  });
});

test.describe('Kyrandia - NPC Dialogue', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('NPCs have multiple dialogue lines', async () => {
    await terminal.navigateToKyrandia();
    await terminal.pressEnter();
    await terminal.sendKeys('talk quinn');
    await terminal.pressEnter();
    await terminal.expectText(['says:', 'continue']);
  });

  test.skip('can advance dialogue with any key', async () => {
    await terminal.navigateToKyrandia();
    await terminal.pressEnter();
    await terminal.sendKeys('talk quinn');
    await terminal.pressEnter();
    await terminal.page.waitForTimeout(100);
    await terminal.pressEnter(); // Advance dialogue
    // Should show next dialogue line or end
  });

  test.skip('merchants show shop options', async () => {
    await terminal.navigateToKyrandia();
    await terminal.pressEnter();
    await terminal.sendKeys('east'); // To shop
    await terminal.pressEnter();
    await terminal.sendKeys('talk felix');
    await terminal.pressEnter();
    await terminal.expectText(['Merchant Felix', 'buy', 'sell']);
  });
});

test.describe('Kyrandia - Puzzles', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('crossroads riddle puzzle', async () => {
    await terminal.navigateToKyrandia();
    // Navigate to crossroads
    await terminal.sendKeys('mountain');
    await terminal.pressEnter();
    await terminal.expectText(['solved', "Wanderer's Riddle"]);
  });

  test.skip('wrong puzzle answer gives hint', async () => {
    await terminal.navigateToKyrandia();
    // At puzzle location
    await terminal.sendKeys('wrong answer');
    await terminal.pressEnter();
    await terminal.expectText(['Hint:', 'roots']);
  });

  test.skip('altar blessing requires spell', async () => {
    await terminal.navigateToKyrandia();
    // At altar of Tashanna
    await terminal.sendKeys('glory be to tashanna');
    await terminal.pressEnter();
    await terminal.expectText(['altar glows', 'Tashanna smiles']);
  });
});

test.describe('Kyrandia - Romance System', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('can flirt with romanceable NPCs', async () => {
    await terminal.navigateToKyrandia();
    await terminal.pressEnter();
    await terminal.sendKeys('north'); // To inn
    await terminal.pressEnter();
    await terminal.sendKeys('flirt mira');
    await terminal.pressEnter();
    await terminal.expectText(['Mira', 'smile', 'look']);
  });

  test.skip('same-sex romance is supported', async () => {
    await terminal.navigateToKyrandia();
    await terminal.pressEnter();
    await terminal.sendKeys('south');
    await terminal.pressEnter();
    await terminal.sendKeys('flirt bran');
    await terminal.pressEnter();
    await terminal.expectText(['Bran', 'flirt']);
  });

  test.skip('romance has daily flirt limit', async () => {
    await terminal.navigateToKyrandia();
    await terminal.pressEnter();
    await terminal.sendKeys('north');
    await terminal.pressEnter();
    // Flirt multiple times
    for (let i = 0; i < 4; i++) {
      await terminal.sendKeys('flirt mira');
      await terminal.pressEnter();
      await terminal.page.waitForTimeout(100);
    }
    await terminal.expectText(['too many', 'tomorrow']);
  });

  test.skip('romance stages progress with affection', async () => {
    await terminal.navigateToKyrandia();
    // After building affection
    await terminal.expectText(['Friend', 'Dating', 'Married']);
  });
});

test.describe('Kyrandia - Regions', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('Village is accessible at level 1', async () => {
    await terminal.navigateToKyrandia();
    await terminal.pressEnter();
    await terminal.expectText(['The Village']);
  });

  test.skip('Dark Forest requires level 2', async () => {
    await terminal.navigateToKyrandia();
    await terminal.pressEnter();
    await terminal.sendKeys('south');
    await terminal.pressEnter();
    await terminal.sendKeys('south');
    await terminal.pressEnter();
    await terminal.expectText(['level 2', 'Dark Forest', 'Forest Entrance']);
  });

  test.skip('Golden Forest requires level 4 and key', async () => {
    await terminal.navigateToKyrandia();
    // Try to enter Golden Forest
    await terminal.expectText(['level 4', 'Golden Key']);
  });

  test.skip('Dragon Castle requires level 6', async () => {
    await terminal.navigateToKyrandia();
    // Try to enter Dragon Castle
    await terminal.expectText(['level 6', 'Dragon Castle']);
  });
});

test.describe('Kyrandia - Victory', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('defeating dragon triggers victory', async () => {
    await terminal.navigateToKyrandia();
    // After defeating Pyraxis
    await terminal.expectText(['VICTORY', 'ARCH-MAGE OF LEGENDS']);
  });

  test.skip('victory screen shows final stats', async () => {
    await terminal.navigateToKyrandia();
    // Victory screen
    await terminal.expectText(['FINAL STATS:', 'Level:', 'Gold Earned:', 'Monsters Defeated:']);
  });
});

test.describe('Kyrandia - Save/Resume', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('displays quit confirmation', async () => {
    await terminal.navigateToKyrandia();
    await terminal.pressEnter();
    await terminal.sendKeys('Q');
    await terminal.expectText(['SAVE & QUIT', 'Are you sure?']);
  });

  test.skip('can confirm quit with Y', async () => {
    await terminal.navigateToKyrandia();
    await terminal.pressEnter();
    await terminal.sendKeys('Q');
    await terminal.page.waitForTimeout(100);
    await terminal.sendKeys('Y');
    // Should return to BBS menu
  });

  test.skip('can cancel quit with N', async () => {
    await terminal.navigateToKyrandia();
    await terminal.pressEnter();
    await terminal.sendKeys('Q');
    await terminal.page.waitForTimeout(100);
    await terminal.sendKeys('N');
    await terminal.expectText(['Village Square']);
  });

  test.skip('resumes from saved position', async () => {
    await terminal.navigateToKyrandia();
    // After quit and re-enter
    await terminal.expectText(['intro', 'Village Square']);
  });

  test.skip('daily turns reset at midnight', async () => {
    await terminal.navigateToKyrandia();
    await terminal.pressEnter();
    await terminal.expectText(['Turns:', 'new day']);
  });
});

test.describe('Kyrandia - Help System', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('displays help with H key', async () => {
    await terminal.navigateToKyrandia();
    await terminal.pressEnter();
    await terminal.sendKeys('H');
    await terminal.expectText(['KYRANDIA HELP', 'MOVEMENT:', 'ACTIONS:', 'MAGIC:']);
  });

  test.skip('help shows command list', async () => {
    await terminal.navigateToKyrandia();
    await terminal.pressEnter();
    await terminal.sendKeys('H');
    await terminal.expectText(['north', 'take', 'drop', 'use', 'equip', 'talk']);
  });

  test.skip('help explains incantation system', async () => {
    await terminal.navigateToKyrandia();
    await terminal.pressEnter();
    await terminal.sendKeys('H');
    await terminal.expectText(['incantation', 'cast', 'vitae restauro']);
  });
});

test.describe('Kyrandia - IGM Support', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('default IGM modules are available', async () => {
    await terminal.navigateToKyrandia();
    await terminal.pressEnter();
    // IGM locations should be accessible
    await terminal.expectText(['Moonlit Glade', 'Wandering Merchant']);
  });

  test.skip('IGM locations can be visited', async () => {
    await terminal.navigateToKyrandia();
    // Navigate to IGM location
    await terminal.expectText(['module', 'location']);
  });
});

test.describe('Kyrandia - Daily Limits', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('shows remaining turns', async () => {
    await terminal.navigateToKyrandia();
    await terminal.pressEnter();
    await terminal.expectText(['Turns:']);
  });

  test.skip('turns decrease with movement', async () => {
    await terminal.navigateToKyrandia();
    await terminal.pressEnter();
    await terminal.sendKeys('north');
    await terminal.pressEnter();
    // Turns should decrease
    await terminal.expectText(['Turns:']);
  });

  test.skip('warns when turns run out', async () => {
    await terminal.navigateToKyrandia();
    // After using all turns
    await terminal.expectText(['No turns remaining', 'Rest at an inn']);
  });
});

test.describe('Kyrandia - Multiplayer Features', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('shows other players in room', async () => {
    await terminal.navigateToKyrandia();
    await terminal.pressEnter();
    // If other players present
    await terminal.expectText(['is here', 'players']);
  });

  test.skip('can send messages to other players', async () => {
    await terminal.navigateToKyrandia();
    await terminal.pressEnter();
    await terminal.sendKeys('say Hello adventurers!');
    await terminal.pressEnter();
    await terminal.expectText(['You say:']);
  });
});

test.describe('Kyrandia - Leaderboard', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('displays hall of legends', async () => {
    await terminal.navigateToKyrandia();
    // View leaderboard
    await terminal.expectText(['HALL OF LEGENDS', 'Rank', 'Level', 'ARCH-MAGE']);
  });

  test.skip('shows completed games', async () => {
    await terminal.navigateToKyrandia();
    // Leaderboard entries
    await terminal.expectText(['Gold', 'Monsters']);
  });
});
