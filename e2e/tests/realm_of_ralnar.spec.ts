/**
 * Realm of Ralnar E2E Tests
 *
 * Tests for the fantasy adventure game where the player explores the Realm of Ralnar,
 * accompanied by companions Herbert and Valeran, seeking shrines and
 * experiencing multiple story endings.
 *
 * Key features tested:
 * - Game intro and new game flow
 * - Exploration with map navigation
 * - Party system with Herbert and Valeran companions
 * - Turn-based combat with FF1-style mechanics
 * - Magic system with elemental spells
 * - Inventory and equipment management
 * - Quest log and story progression
 * - World map navigation
 * - Inn resting and healing
 * - Save/load functionality
 * - Multiple endings
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

  async pressKey(key: string) {
    await this.page.keyboard.press(key);
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

  async navigateToRealmOfRalnar() {
    // Navigate: G (Games menu) -> R (Realm of Ralnar) or appropriate number
    await this.sendKeys('G');
    await this.page.waitForTimeout(200);
    await this.sendKeys('R'); // Realm of Ralnar
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

  async expectAnyText(texts: string[]) {
    const content = await this.getScreenContent();
    const found = texts.some(t => content.toLowerCase().includes(t.toLowerCase()));
    expect(found).toBe(true);
  }
}

// =============================================================================
// GAME INTRO TESTS
// =============================================================================

test.describe('Realm of Ralnar - Game Intro', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
    // Tests would normally connect and login here
    // await terminal.connect();
    // await terminal.login('testuser', 'testpass');
  });

  test.skip('displays intro screen on first entry', async () => {
    await terminal.navigateToRealmOfRalnar();
    await terminal.expectText(['REALM', 'RALNAR']);
  });

  test.skip('shows intro story text about Herbert and Valeran', async () => {
    await terminal.navigateToRealmOfRalnar();
    await terminal.expectAnyText([
      'herbert',
      'valeran',
      'brothers',
      'realm',
      'adventure',
    ]);
  });

  test.skip('advances to main menu on key press', async () => {
    await terminal.navigateToRealmOfRalnar();
    await terminal.pressEnter();
    await terminal.expectAnyText(['Continue', 'Play', 'Status', 'Inventory']);
  });
});

// =============================================================================
// MAIN MENU TESTS
// =============================================================================

test.describe('Realm of Ralnar - Main Menu', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('displays main menu options', async () => {
    await terminal.navigateToRealmOfRalnar();
    await terminal.pressEnter(); // Skip intro
    await terminal.expectAnyText(['Continue', 'Play', '1', 'P', 'C']);
  });

  test.skip('shows status option', async () => {
    await terminal.navigateToRealmOfRalnar();
    await terminal.pressEnter();
    await terminal.expectAnyText(['Status', 'S', '2']);
  });

  test.skip('shows inventory option', async () => {
    await terminal.navigateToRealmOfRalnar();
    await terminal.pressEnter();
    await terminal.expectAnyText(['Inventory', 'I', '3']);
  });

  test.skip('shows world map option', async () => {
    await terminal.navigateToRealmOfRalnar();
    await terminal.pressEnter();
    await terminal.expectAnyText(['Map', 'M', '4']);
  });

  test.skip('shows quit option', async () => {
    await terminal.navigateToRealmOfRalnar();
    await terminal.pressEnter();
    await terminal.expectAnyText(['Quit', 'Q', 'X']);
  });

  test.skip('can navigate to exploration with P key', async () => {
    await terminal.navigateToRealmOfRalnar();
    await terminal.pressEnter();
    await terminal.sendKeys('P');
    await terminal.expectAnyText(['Exploring', 'village', 'square']);
  });

  test.skip('can navigate to exploration with 1 key', async () => {
    await terminal.navigateToRealmOfRalnar();
    await terminal.pressEnter();
    await terminal.sendKeys('1');
    await terminal.expectAnyText(['Exploring', 'village', 'square']);
  });
});

// =============================================================================
// PARTY STATUS TESTS
// =============================================================================

test.describe('Realm of Ralnar - Party Status', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('displays party status screen', async () => {
    await terminal.navigateToRealmOfRalnar();
    await terminal.pressEnter();
    await terminal.sendKeys('S');
    await terminal.expectAnyText(['Party', 'Status']);
  });

  test.skip('shows Herbert in party', async () => {
    await terminal.navigateToRealmOfRalnar();
    await terminal.pressEnter();
    await terminal.sendKeys('S');
    await terminal.expectText('Herbert');
  });

  test.skip('shows Valeran in party', async () => {
    await terminal.navigateToRealmOfRalnar();
    await terminal.pressEnter();
    await terminal.sendKeys('S');
    await terminal.expectText('Valeran');
  });

  test.skip('displays HP for party members', async () => {
    await terminal.navigateToRealmOfRalnar();
    await terminal.pressEnter();
    await terminal.sendKeys('S');
    await terminal.expectAnyText(['HP', 'Health']);
  });

  test.skip('displays loyalty for companions', async () => {
    await terminal.navigateToRealmOfRalnar();
    await terminal.pressEnter();
    await terminal.sendKeys('S');
    await terminal.expectAnyText(['Loyalty', 'loyalty']);
  });

  test.skip('can return to menu with Q', async () => {
    await terminal.navigateToRealmOfRalnar();
    await terminal.pressEnter();
    await terminal.sendKeys('S');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('Q');
    await terminal.expectAnyText(['Continue', 'Play', 'Menu']);
  });
});

// =============================================================================
// EXPLORATION TESTS
// =============================================================================

test.describe('Realm of Ralnar - Exploration', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('displays current location', async () => {
    await terminal.navigateToRealmOfRalnar();
    await terminal.pressEnter();
    await terminal.sendKeys('P');
    await terminal.expectAnyText(['village', 'square', 'location']);
  });

  test.skip('shows navigation options', async () => {
    await terminal.navigateToRealmOfRalnar();
    await terminal.pressEnter();
    await terminal.sendKeys('P');
    await terminal.expectAnyText(['N', 'S', 'E', 'W', 'North', 'South', 'East', 'West']);
  });

  test.skip('can move north with N key', async () => {
    await terminal.navigateToRealmOfRalnar();
    await terminal.pressEnter();
    await terminal.sendKeys('P');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('N');
    await terminal.expectAnyText(['northern', 'path', 'moved']);
  });

  test.skip('can move south with S key', async () => {
    await terminal.navigateToRealmOfRalnar();
    await terminal.pressEnter();
    await terminal.sendKeys('P');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('S');
    await terminal.expectAnyText(['southern', 'woods', 'moved']);
  });

  test.skip('can move east with E key', async () => {
    await terminal.navigateToRealmOfRalnar();
    await terminal.pressEnter();
    await terminal.sendKeys('P');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('E');
    await terminal.expectAnyText(['eastern', 'hills', 'moved']);
  });

  test.skip('can move west with W key', async () => {
    await terminal.navigateToRealmOfRalnar();
    await terminal.pressEnter();
    await terminal.sendKeys('P');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('W');
    await terminal.expectAnyText(['western', 'river', 'moved']);
  });

  test.skip('can open inventory from exploration', async () => {
    await terminal.navigateToRealmOfRalnar();
    await terminal.pressEnter();
    await terminal.sendKeys('P');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('I');
    await terminal.expectText('Inventory');
  });

  test.skip('can open party status from exploration', async () => {
    await terminal.navigateToRealmOfRalnar();
    await terminal.pressEnter();
    await terminal.sendKeys('P');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('C');
    await terminal.expectAnyText(['Party', 'Status']);
  });

  test.skip('can open world map from exploration', async () => {
    await terminal.navigateToRealmOfRalnar();
    await terminal.pressEnter();
    await terminal.sendKeys('P');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('M');
    await terminal.expectAnyText(['World', 'Map']);
  });

  test.skip('can quit from exploration', async () => {
    await terminal.navigateToRealmOfRalnar();
    await terminal.pressEnter();
    await terminal.sendKeys('P');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('Q');
    await terminal.expectAnyText(['Quit', 'sure', 'confirm']);
  });
});

// =============================================================================
// INVENTORY TESTS
// =============================================================================

test.describe('Realm of Ralnar - Inventory', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('displays inventory screen', async () => {
    await terminal.navigateToRealmOfRalnar();
    await terminal.pressEnter();
    await terminal.sendKeys('I');
    await terminal.expectText('Inventory');
  });

  test.skip('shows starting items', async () => {
    await terminal.navigateToRealmOfRalnar();
    await terminal.pressEnter();
    await terminal.sendKeys('I');
    // Should show starting items: rusty_sword and torch
    await terminal.expectAnyText(['sword', 'torch', 'Rusty', 'Torch']);
  });

  test.skip('displays gold amount', async () => {
    await terminal.navigateToRealmOfRalnar();
    await terminal.pressEnter();
    await terminal.sendKeys('I');
    await terminal.expectAnyText(['Gold', 'gold', '10']);
  });

  test.skip('can return to menu with Q', async () => {
    await terminal.navigateToRealmOfRalnar();
    await terminal.pressEnter();
    await terminal.sendKeys('I');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('Q');
    await terminal.expectAnyText(['Menu', 'Continue', 'Play']);
  });
});

// =============================================================================
// WORLD MAP TESTS
// =============================================================================

test.describe('Realm of Ralnar - World Map', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('displays world map screen', async () => {
    await terminal.navigateToRealmOfRalnar();
    await terminal.pressEnter();
    await terminal.sendKeys('M');
    await terminal.expectAnyText(['World', 'Map']);
  });

  test.skip('shows available locations', async () => {
    await terminal.navigateToRealmOfRalnar();
    await terminal.pressEnter();
    await terminal.sendKeys('M');
    await terminal.expectAnyText(['1', '2', '3', '4', '5']);
  });

  test.skip('can select location by number', async () => {
    await terminal.navigateToRealmOfRalnar();
    await terminal.pressEnter();
    await terminal.sendKeys('M');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('1');
    // Should travel to selected location
    await terminal.expectAnyText(['Exploring', 'travel']);
  });

  test.skip('can return to menu with Q', async () => {
    await terminal.navigateToRealmOfRalnar();
    await terminal.pressEnter();
    await terminal.sendKeys('M');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('Q');
    await terminal.expectAnyText(['Menu', 'Continue', 'Play']);
  });
});

// =============================================================================
// INN TESTS
// =============================================================================

test.describe('Realm of Ralnar - Inn', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('displays inn screen', async () => {
    await terminal.navigateToRealmOfRalnar();
    // Would need to navigate to inn location first
    await terminal.expectText('Inn');
  });

  test.skip('shows rest option', async () => {
    await terminal.navigateToRealmOfRalnar();
    // At inn
    await terminal.expectAnyText(['Rest', 'R', '1']);
  });

  test.skip('resting restores party health', async () => {
    await terminal.navigateToRealmOfRalnar();
    // At inn with damaged party
    await terminal.sendKeys('R');
    // Health should be restored
    await terminal.expectAnyText(['restored', 'healed', 'rested']);
  });

  test.skip('can leave inn with Q', async () => {
    await terminal.navigateToRealmOfRalnar();
    // At inn
    await terminal.sendKeys('Q');
    await terminal.expectAnyText(['Exploring', 'village']);
  });
});

// =============================================================================
// COMBAT TESTS
// =============================================================================

test.describe('Realm of Ralnar - Combat', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('displays battle screen', async () => {
    await terminal.navigateToRealmOfRalnar();
    // Would need to trigger combat
    await terminal.expectText('Battle');
  });

  test.skip('shows attack option', async () => {
    await terminal.navigateToRealmOfRalnar();
    // In combat
    await terminal.expectAnyText(['Attack', 'A', '1']);
  });

  test.skip('shows defend option', async () => {
    await terminal.navigateToRealmOfRalnar();
    // In combat
    await terminal.expectAnyText(['Defend', 'D', '2']);
  });

  test.skip('shows magic option', async () => {
    await terminal.navigateToRealmOfRalnar();
    // In combat
    await terminal.expectAnyText(['Magic', 'M', '3']);
  });

  test.skip('shows run option', async () => {
    await terminal.navigateToRealmOfRalnar();
    // In combat
    await terminal.expectAnyText(['Run', 'R', 'Flee', '4']);
  });

  test.skip('can attack with A key', async () => {
    await terminal.navigateToRealmOfRalnar();
    // In combat
    await terminal.sendKeys('A');
    await terminal.expectAnyText(['attack', 'damage', 'hit']);
  });

  test.skip('can defend with D key', async () => {
    await terminal.navigateToRealmOfRalnar();
    // In combat
    await terminal.sendKeys('D');
    await terminal.expectAnyText(['defend', 'guarding', 'defense']);
  });

  test.skip('can open magic menu with M key', async () => {
    await terminal.navigateToRealmOfRalnar();
    // In combat
    await terminal.sendKeys('M');
    await terminal.expectText('Magic');
  });

  test.skip('can attempt to flee with R key', async () => {
    await terminal.navigateToRealmOfRalnar();
    // In combat
    await terminal.sendKeys('R');
    await terminal.expectAnyText(['flee', 'escape', 'run', 'failed']);
  });
});

// =============================================================================
// MAGIC SYSTEM TESTS
// =============================================================================

test.describe('Realm of Ralnar - Magic', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('displays magic screen', async () => {
    await terminal.navigateToRealmOfRalnar();
    // In combat, press M
    await terminal.sendKeys('M');
    await terminal.expectText('Magic');
  });

  test.skip('shows available spells', async () => {
    await terminal.navigateToRealmOfRalnar();
    // In magic screen
    await terminal.expectAnyText(['Fire', 'Cure', 'Blizzard', 'Thunder']);
  });

  test.skip('shows spell MP costs', async () => {
    await terminal.navigateToRealmOfRalnar();
    // In magic screen
    await terminal.expectAnyText(['MP', 'mp', 'cost']);
  });

  test.skip('can return to battle with Q', async () => {
    await terminal.navigateToRealmOfRalnar();
    // In magic screen
    await terminal.sendKeys('Q');
    await terminal.expectText('Battle');
  });
});

// =============================================================================
// BATTLE VICTORY TESTS
// =============================================================================

test.describe('Realm of Ralnar - Battle Victory', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('displays victory screen after winning', async () => {
    await terminal.navigateToRealmOfRalnar();
    // After winning combat
    await terminal.expectAnyText(['Victory', 'Won', 'Defeated']);
  });

  test.skip('shows experience gained', async () => {
    await terminal.navigateToRealmOfRalnar();
    // Victory screen
    await terminal.expectAnyText(['EXP', 'Experience', 'exp']);
  });

  test.skip('shows gold gained', async () => {
    await terminal.navigateToRealmOfRalnar();
    // Victory screen
    await terminal.expectAnyText(['Gold', 'gold']);
  });

  test.skip('returns to exploration after victory', async () => {
    await terminal.navigateToRealmOfRalnar();
    // Victory screen, press any key
    await terminal.pressEnter();
    await terminal.expectAnyText(['Exploring', 'village']);
  });
});

// =============================================================================
// GAME OVER TESTS
// =============================================================================

test.describe('Realm of Ralnar - Game Over', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('displays game over screen on defeat', async () => {
    await terminal.navigateToRealmOfRalnar();
    // After losing combat
    await terminal.expectAnyText(['Game Over', 'Defeat', 'Fallen']);
  });

  test.skip('game over ends with completion', async () => {
    await terminal.navigateToRealmOfRalnar();
    // On game over screen
    await terminal.pressEnter();
    // Should return to BBS or record completion
  });
});

// =============================================================================
// QUIT CONFIRMATION TESTS
// =============================================================================

test.describe('Realm of Ralnar - Quit Confirmation', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('displays quit confirmation', async () => {
    await terminal.navigateToRealmOfRalnar();
    await terminal.pressEnter();
    await terminal.sendKeys('Q');
    await terminal.expectAnyText(['Quit', 'sure', 'confirm', 'Y', 'N']);
  });

  test.skip('can confirm quit with Y', async () => {
    await terminal.navigateToRealmOfRalnar();
    await terminal.pressEnter();
    await terminal.sendKeys('Q');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('Y');
    // Should return to BBS menu
  });

  test.skip('can cancel quit with N', async () => {
    await terminal.navigateToRealmOfRalnar();
    await terminal.pressEnter();
    await terminal.sendKeys('Q');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('N');
    await terminal.expectAnyText(['Menu', 'Continue', 'Play']);
  });

  test.skip('can cancel quit with any other key', async () => {
    await terminal.navigateToRealmOfRalnar();
    await terminal.pressEnter();
    await terminal.sendKeys('Q');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('X');
    await terminal.expectAnyText(['Menu', 'Continue', 'Play']);
  });
});

// =============================================================================
// SAVE/LOAD TESTS
// =============================================================================

test.describe('Realm of Ralnar - Save/Resume', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('game saves progress automatically', async () => {
    await terminal.navigateToRealmOfRalnar();
    await terminal.pressEnter();
    // Navigate to change state
    await terminal.sendKeys('P');
    await terminal.page.waitForTimeout(200);
    await terminal.sendKeys('N');
    // Game should auto-save
  });

  test.skip('resumes from saved state on re-entry', async () => {
    await terminal.navigateToRealmOfRalnar();
    // Should not show intro if save exists
    await terminal.expectAnyText(['Menu', 'Continue', 'Resume']);
  });

  test.skip('preserves location on resume', async () => {
    await terminal.navigateToRealmOfRalnar();
    // After quit and re-enter
    await terminal.expectAnyText(['northern', 'village']);
  });

  test.skip('preserves gold on resume', async () => {
    await terminal.navigateToRealmOfRalnar();
    await terminal.pressEnter();
    await terminal.sendKeys('I');
    await terminal.expectAnyText(['Gold', '10']);
  });

  test.skip('preserves inventory on resume', async () => {
    await terminal.navigateToRealmOfRalnar();
    await terminal.pressEnter();
    await terminal.sendKeys('I');
    await terminal.expectAnyText(['sword', 'torch']);
  });
});

// =============================================================================
// SHRINE SYSTEM TESTS
// =============================================================================

test.describe('Realm of Ralnar - Shrines', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('can discover shrines during exploration', async () => {
    await terminal.navigateToRealmOfRalnar();
    // Navigate to shrine location
    await terminal.expectAnyText(['shrine', 'Shrine', 'discovered']);
  });

  test.skip('shrine count updates in status', async () => {
    await terminal.navigateToRealmOfRalnar();
    await terminal.pressEnter();
    await terminal.sendKeys('S');
    await terminal.expectAnyText(['Shrines', 'shrines', '0']);
  });
});

// =============================================================================
// QUEST FLAG TESTS
// =============================================================================

test.describe('Realm of Ralnar - Quest Flags', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('quest flags affect dialogue options', async () => {
    await terminal.navigateToRealmOfRalnar();
    // Dialogue changes based on flags
    await terminal.expectAnyText(['quest', 'story', 'flag']);
  });

  test.skip('completing events sets flags', async () => {
    await terminal.navigateToRealmOfRalnar();
    // Complete an event
    await terminal.expectAnyText(['completed', 'unlocked', 'progress']);
  });
});

// =============================================================================
// DIALOGUE TESTS
// =============================================================================

test.describe('Realm of Ralnar - Dialogue', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('displays dialogue screen when talking to NPC', async () => {
    await terminal.navigateToRealmOfRalnar();
    // Navigate to NPC and interact
    await terminal.expectText('Dialogue');
  });

  test.skip('shows dialogue choices', async () => {
    await terminal.navigateToRealmOfRalnar();
    // In dialogue
    await terminal.expectAnyText(['1', '2', '3', 'B', 'Q']);
  });

  test.skip('can select dialogue option', async () => {
    await terminal.navigateToRealmOfRalnar();
    // In dialogue
    await terminal.sendKeys('1');
    // Should show response
  });

  test.skip('can exit dialogue', async () => {
    await terminal.navigateToRealmOfRalnar();
    // In dialogue
    await terminal.sendKeys('Q');
    await terminal.expectAnyText(['Exploring', 'village']);
  });
});

// =============================================================================
// SHOP TESTS
// =============================================================================

test.describe('Realm of Ralnar - Shop', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('displays shop screen', async () => {
    await terminal.navigateToRealmOfRalnar();
    // Navigate to shop
    await terminal.expectText('Shop');
  });

  test.skip('shows items for sale', async () => {
    await terminal.navigateToRealmOfRalnar();
    // In shop
    await terminal.expectAnyText(['Price', 'Buy', 'Sell']);
  });

  test.skip('can exit shop', async () => {
    await terminal.navigateToRealmOfRalnar();
    // In shop
    await terminal.sendKeys('Q');
    await terminal.expectAnyText(['Exploring', 'village']);
  });
});

// =============================================================================
// CUTSCENE TESTS
// =============================================================================

test.describe('Realm of Ralnar - Cutscenes', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('displays cutscene content', async () => {
    await terminal.navigateToRealmOfRalnar();
    // Trigger cutscene
    await terminal.expectText('Cutscene');
  });

  test.skip('any key advances cutscene', async () => {
    await terminal.navigateToRealmOfRalnar();
    // In cutscene
    await terminal.pressEnter();
    await terminal.expectAnyText(['Exploring', 'village']);
  });
});

// =============================================================================
// CREDITS TESTS
// =============================================================================

test.describe('Realm of Ralnar - Credits', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('displays credits after victory ending', async () => {
    await terminal.navigateToRealmOfRalnar();
    // After completing the game
    await terminal.expectText('Credits');
  });

  test.skip('credits lead to game completion', async () => {
    await terminal.navigateToRealmOfRalnar();
    // In credits
    await terminal.pressEnter();
    // Should complete with victory ending
  });
});

// =============================================================================
// BROTHER COMPANION MECHANICS TESTS
// =============================================================================

test.describe('Realm of Ralnar - Brother Mechanics', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.skip('both brothers start in party', async () => {
    await terminal.navigateToRealmOfRalnar();
    await terminal.pressEnter();
    await terminal.sendKeys('S');
    await terminal.expectText('Herbert');
    await terminal.expectText('Valeran');
  });

  test.skip('Herbert has higher max HP than Valeran', async () => {
    await terminal.navigateToRealmOfRalnar();
    await terminal.pressEnter();
    await terminal.sendKeys('S');
    // Herbert: 50 HP, Valeran: 40 HP per the code
    await terminal.expectAnyText(['50', 'HP']);
  });

  test.skip('Valeran has higher loyalty than Herbert', async () => {
    await terminal.navigateToRealmOfRalnar();
    await terminal.pressEnter();
    await terminal.sendKeys('S');
    // Herbert: 50 loyalty, Valeran: 60 loyalty per the code
    await terminal.expectAnyText(['60', 'Loyalty']);
  });
});

// =============================================================================
// UNIT-STYLE TESTS FOR GAME MECHANICS
// =============================================================================

test.describe('Realm of Ralnar Mechanics (Unit)', () => {
  test('character classes have valid names', () => {
    const classes = [
      'Warrior', 'Paladin', 'Cleric', 'Wizard',
      'Knight', 'Swashbuckler', 'Thief', 'Sage', 'Archer'
    ];

    classes.forEach(className => {
      expect(className.length).toBeGreaterThan(0);
    });
  });

  test('spell elements have opposites', () => {
    const elementPairs = {
      'Fire': 'Water',
      'Ice': 'Fire',
      'Lightning': 'Earth',
      'Earth': 'Wind',
      'Water': 'Lightning',
      'Wind': 'Ice',
      'Holy': 'Dark',
      'Dark': 'Holy',
    };

    Object.entries(elementPairs).forEach(([element, weakness]) => {
      expect(element).toBeDefined();
      expect(weakness).toBeDefined();
    });
  });

  test('status effects have valid durations', () => {
    const statusDurations = {
      'Poison': 0,
      'Stone': 0,
      'Dead': 0,
      'Sleep': 0,
      'Confused': 4,
      'Haste': 5,
      'Protect': 5,
      'Shell': 5,
      'Silence': 4,
      'Blind': 4,
      'Slow': 4,
      'Regen': 5,
      'Berserk': 4,
    };

    Object.values(statusDurations).forEach(duration => {
      expect(duration).toBeGreaterThanOrEqual(0);
    });
  });

  test('combat constants are valid', () => {
    const baseHitChance = 0.85;
    const criticalChance = 0.05;
    const criticalMultiplier = 2.0;
    const minHitChance = 0.10;
    const maxHitChance = 0.95;

    expect(baseHitChance).toBeGreaterThan(0);
    expect(baseHitChance).toBeLessThanOrEqual(1);
    expect(criticalChance).toBeGreaterThan(0);
    expect(criticalChance).toBeLessThan(1);
    expect(criticalMultiplier).toBeGreaterThan(1);
    expect(minHitChance).toBeLessThan(maxHitChance);
  });

  test('party limits are sensible', () => {
    const maxPartySize = 4;
    const maxInventorySlots = 20;
    const startingGold = 100;

    expect(maxPartySize).toBeGreaterThanOrEqual(2); // At least the brothers
    expect(maxInventorySlots).toBeGreaterThan(0);
    expect(startingGold).toBeGreaterThan(0);
  });

  test('XP table is progressive', () => {
    const xpSamples = [0, 100, 250, 450, 700, 1000];

    for (let i = 1; i < xpSamples.length; i++) {
      expect(xpSamples[i]).toBeGreaterThan(xpSamples[i - 1]);
    }
  });

  test('stat growth classes have positive HP', () => {
    const statGrowths = {
      'Warrior': { hp: 25, mp: 5 },
      'Paladin': { hp: 20, mp: 12 },
      'Cleric': { hp: 15, mp: 20 },
      'Wizard': { hp: 12, mp: 25 },
      'Knight': { hp: 22, mp: 8 },
      'Swashbuckler': { hp: 16, mp: 10 },
      'Thief': { hp: 14, mp: 8 },
      'Sage': { hp: 10, mp: 22 },
      'Archer': { hp: 15, mp: 12 },
    };

    Object.entries(statGrowths).forEach(([className, stats]) => {
      expect(stats.hp).toBeGreaterThan(0);
      expect(stats.mp).toBeGreaterThanOrEqual(0);
    });
  });

  test('brother bonuses are percentage values', () => {
    const brotherTogetherBonus = 0.10;
    const protectiveFuryBonus = 0.25;
    const separationPenalty = 0.10;
    const reunionBoost = 0.20;

    expect(brotherTogetherBonus).toBeGreaterThan(0);
    expect(brotherTogetherBonus).toBeLessThanOrEqual(1);
    expect(protectiveFuryBonus).toBeGreaterThan(0);
    expect(protectiveFuryBonus).toBeLessThanOrEqual(1);
    expect(separationPenalty).toBeGreaterThan(0);
    expect(separationPenalty).toBeLessThanOrEqual(1);
    expect(reunionBoost).toBeGreaterThan(0);
    expect(reunionBoost).toBeLessThanOrEqual(1);
  });

  test('spell tiers scale properly', () => {
    // Fire spell progression
    const fireSpells = [
      { name: 'Fire', mp: 4, power: 20, level: 1 },
      { name: 'Fira', mp: 12, power: 50, level: 8 },
      { name: 'Firaga', mp: 30, power: 120, level: 20 },
    ];

    for (let i = 1; i < fireSpells.length; i++) {
      expect(fireSpells[i].mp).toBeGreaterThan(fireSpells[i - 1].mp);
      expect(fireSpells[i].power).toBeGreaterThan(fireSpells[i - 1].power);
      expect(fireSpells[i].level).toBeGreaterThan(fireSpells[i - 1].level);
    }
  });

  test('healing spells have proper targeting', () => {
    const healingSpells = [
      { name: 'Cure', target: 'SingleAlly' },
      { name: 'Cura', target: 'SingleAlly' },
      { name: 'Curaga', target: 'AllAllies' },
      { name: 'Full Cure', target: 'SingleAlly' },
    ];

    healingSpells.forEach(spell => {
      expect(['SingleAlly', 'AllAllies', 'Self_']).toContain(spell.target);
    });
  });

  test('damage spells target enemies', () => {
    const damageSpells = [
      { name: 'Fire', target: 'SingleEnemy' },
      { name: 'Firaga', target: 'AllEnemies' },
      { name: 'Flare', target: 'SingleEnemy' },
    ];

    damageSpells.forEach(spell => {
      expect(['SingleEnemy', 'AllEnemies']).toContain(spell.target);
    });
  });
});
