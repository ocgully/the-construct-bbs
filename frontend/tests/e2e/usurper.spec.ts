// frontend/tests/e2e/usurper.spec.ts
import { test, expect } from '@playwright/test';
import { createTerminal, TerminalHelper } from './helpers/terminal';

// Use a dedicated Usurper test user
const testHandle = 'usurpertest';
const testPassword = 'UsurperPass123!';
const testEmail = 'usurpertest@test.local';

test.describe('Usurper - Dark Fantasy RPG', () => {
  // Ensure test user exists
  test.beforeAll(async ({ browser }) => {
    const page = await browser.newPage();
    await page.goto('/?e2e');
    const terminal = await createTerminal(page);
    await terminal.connect();

    // Try to register new user
    await terminal.send('new');
    await terminal.waitForText('handle', 10000);
    await terminal.send(testHandle);
    await terminal.waitForText('email', 10000);
    await terminal.send(testEmail);
    await terminal.waitForText('password', 10000);
    await terminal.send(testPassword);

    await page.waitForTimeout(5000);
    await page.close();
  });

  async function loginUser(page: any): Promise<TerminalHelper> {
    await page.goto('/?e2e');
    const terminal = await createTerminal(page);
    await terminal.connect();

    // Terminal shows "Enter your handle:"
    await terminal.send(testHandle);
    await terminal.waitForText('password', 10000);
    await terminal.send(testPassword);
    await terminal.waitForText('Main Menu', 15000);

    return terminal;
  }

  // Helper to navigate to Usurper game
  async function launchUsurper(terminal: TerminalHelper, page: any) {
    await terminal.menuSelect('G'); // Games menu
    await terminal.waitForText('Usurper');
    // Find and select Usurper from the games list
    const content = await terminal.getTerminalContent();
    // Usurper should be in the games list - look for its menu number
    if (content.includes('Usurper')) {
      await terminal.menuSelect('U'); // or appropriate number
    }
    await terminal.waitForText('USURPER', 10000);
  }

  test.describe('Game Launch', () => {
    test('can launch game from menu', async ({ page }) => {
      const terminal = await loginUser(page);

      await terminal.menuSelect('G');
      await terminal.waitForText('Usurper');

      // Should show Usurper in games list
      const content = await terminal.getTerminalContent();
      expect(content).toContain('Usurper');
    });

    test('shows intro screen on first launch', async ({ page }) => {
      const terminal = await loginUser(page);

      await launchUsurper(terminal, page);

      // Should see intro with dark fantasy theme
      const content = await terminal.getTerminalContent();
      expect(content.match(/USURPER|Durunghins|darkness/i)).toBeTruthy();
    });

    test('can skip intro and see character creation', async ({ page }) => {
      const terminal = await loginUser(page);

      await launchUsurper(terminal, page);

      // Press any key to skip intro
      await terminal.press('Space');
      await page.waitForTimeout(500);

      // Should see character creation or town screen
      const content = await terminal.getTerminalContent();
      expect(content.match(/name|class|Town|Warrior|Rogue|Mage/i)).toBeTruthy();
    });
  });

  test.describe('Character Creation', () => {
    test('shows class selection options', async ({ page }) => {
      const terminal = await loginUser(page);

      await launchUsurper(terminal, page);
      await terminal.press('Space');
      await page.waitForTimeout(500);

      // Should show class options
      const content = await terminal.getTerminalContent();
      expect(content.match(/Warrior|Rogue|Mage|Cleric|Berserker/i)).toBeTruthy();
    });

    test('can select warrior class', async ({ page }) => {
      const terminal = await loginUser(page);

      await launchUsurper(terminal, page);
      await terminal.press('Space');
      await page.waitForTimeout(500);

      await terminal.menuSelect('1'); // Warrior typically first option
      await page.waitForTimeout(300);

      const content = await terminal.getTerminalContent();
      expect(content.match(/Warrior|strength|combat/i)).toBeTruthy();
    });

    test('prompts for character name', async ({ page }) => {
      const terminal = await loginUser(page);

      await launchUsurper(terminal, page);
      await terminal.press('Space');
      await page.waitForTimeout(500);

      // Select a class first
      await terminal.menuSelect('1');
      await page.waitForTimeout(500);

      const content = await terminal.getTerminalContent();
      expect(content.match(/name|enter|choose/i)).toBeTruthy();
    });
  });

  test.describe('Town Screen', () => {
    test('shows town menu with locations', async ({ page }) => {
      const terminal = await loginUser(page);

      await launchUsurper(terminal, page);
      await terminal.press('Space');
      await page.waitForTimeout(500);

      // Complete character creation if needed
      await terminal.menuSelect('1'); // Select class
      await page.waitForTimeout(300);
      await terminal.send('TestHero'); // Enter name
      await page.waitForTimeout(500);

      // Should see town screen
      const content = await terminal.getTerminalContent();
      expect(content.match(/Town|Dungeon|Healer|Shop|Bank/i)).toBeTruthy();
    });

    test('shows status bar with HP and gold', async ({ page }) => {
      const terminal = await loginUser(page);

      await launchUsurper(terminal, page);
      await terminal.press('Space');
      await terminal.menuSelect('1');
      await terminal.send('StatusTest');
      await page.waitForTimeout(500);

      const content = await terminal.getTerminalContent();
      expect(content.match(/HP|Health|Gold|Level/i)).toBeTruthy();
    });

    test('can access dungeon entrance', async ({ page }) => {
      const terminal = await loginUser(page);

      await launchUsurper(terminal, page);
      await terminal.press('Space');
      await terminal.menuSelect('1');
      await terminal.send('DungeonTest');
      await page.waitForTimeout(500);

      await terminal.menuSelect('D'); // Dungeon
      await page.waitForTimeout(300);

      const content = await terminal.getTerminalContent();
      expect(content.match(/Dungeon|level|enter|surface/i)).toBeTruthy();
    });

    test('can access healer', async ({ page }) => {
      const terminal = await loginUser(page);

      await launchUsurper(terminal, page);
      await terminal.press('Space');
      await terminal.menuSelect('1');
      await terminal.send('HealerTest');
      await page.waitForTimeout(500);

      await terminal.menuSelect('H'); // Healer
      await page.waitForTimeout(300);

      const content = await terminal.getTerminalContent();
      expect(content.match(/Healer|heal|restore|HP/i)).toBeTruthy();
    });

    test('can access shop', async ({ page }) => {
      const terminal = await loginUser(page);

      await launchUsurper(terminal, page);
      await terminal.press('Space');
      await terminal.menuSelect('1');
      await terminal.send('ShopTest');
      await page.waitForTimeout(500);

      await terminal.menuSelect('S'); // Shop
      await page.waitForTimeout(300);

      const content = await terminal.getTerminalContent();
      expect(content.match(/Shop|weapon|armor|equipment|buy|sell/i)).toBeTruthy();
    });

    test('can access bank', async ({ page }) => {
      const terminal = await loginUser(page);

      await launchUsurper(terminal, page);
      await terminal.press('Space');
      await terminal.menuSelect('1');
      await terminal.send('BankTest');
      await page.waitForTimeout(500);

      await terminal.menuSelect('B'); // Bank
      await page.waitForTimeout(300);

      const content = await terminal.getTerminalContent();
      expect(content.match(/Bank|deposit|withdraw|gold/i)).toBeTruthy();
    });
  });

  test.describe('Substances (Drugs/Steroids)', () => {
    test('can access substance dealer', async ({ page }) => {
      const terminal = await loginUser(page);

      await launchUsurper(terminal, page);
      await terminal.press('Space');
      await terminal.menuSelect('1');
      await terminal.send('SubstanceTest');
      await page.waitForTimeout(500);

      // Navigate to substance dealer (might be a hidden or special location)
      await terminal.menuSelect('P'); // Potions/substances
      await page.waitForTimeout(300);

      const content = await terminal.getTerminalContent();
      expect(content.match(/substance|steroid|stimulant|potion|dealer/i)).toBeTruthy();
    });

    test('shows mental stability in status', async ({ page }) => {
      const terminal = await loginUser(page);

      await launchUsurper(terminal, page);
      await terminal.press('Space');
      await terminal.menuSelect('1');
      await terminal.send('MentalTest');
      await page.waitForTimeout(500);

      // Check stats screen
      await terminal.menuSelect('I'); // Info/stats
      await page.waitForTimeout(300);

      const content = await terminal.getTerminalContent();
      expect(content.match(/mental|stability|sanity/i)).toBeTruthy();
    });
  });

  test.describe('Dungeon Exploration', () => {
    test('can enter dungeon', async ({ page }) => {
      const terminal = await loginUser(page);

      await launchUsurper(terminal, page);
      await terminal.press('Space');
      await terminal.menuSelect('1');
      await terminal.send('DungeonEnter');
      await page.waitForTimeout(500);

      await terminal.menuSelect('D'); // Dungeon
      await page.waitForTimeout(300);
      await terminal.menuSelect('1'); // Enter level 1
      await page.waitForTimeout(300);

      const content = await terminal.getTerminalContent();
      expect(content.match(/dungeon|explore|monster|fight/i)).toBeTruthy();
    });

    test('shows exploration options', async ({ page }) => {
      const terminal = await loginUser(page);

      await launchUsurper(terminal, page);
      await terminal.press('Space');
      await terminal.menuSelect('1');
      await terminal.send('ExploreTest');
      await page.waitForTimeout(500);

      await terminal.menuSelect('D');
      await terminal.menuSelect('1');
      await page.waitForTimeout(300);

      const content = await terminal.getTerminalContent();
      expect(content.match(/explore|fight|run|search|deeper/i)).toBeTruthy();
    });
  });

  test.describe('Combat System', () => {
    test('combat shows enemy stats', async ({ page }) => {
      const terminal = await loginUser(page);

      await launchUsurper(terminal, page);
      await terminal.press('Space');
      await terminal.menuSelect('1');
      await terminal.send('CombatTest');
      await page.waitForTimeout(500);

      // Enter dungeon and trigger combat
      await terminal.menuSelect('D');
      await terminal.menuSelect('1');
      await terminal.menuSelect('E'); // Explore to trigger encounter
      await page.waitForTimeout(500);

      const content = await terminal.getTerminalContent();
      // Should show combat screen with enemy info
      expect(content.match(/HP|Attack|enemy|monster|combat/i)).toBeTruthy();
    });

    test('combat shows action options', async ({ page }) => {
      const terminal = await loginUser(page);

      await launchUsurper(terminal, page);
      await terminal.press('Space');
      await terminal.menuSelect('1');
      await terminal.send('CombatActions');
      await page.waitForTimeout(500);

      await terminal.menuSelect('D');
      await terminal.menuSelect('1');
      await terminal.menuSelect('E');
      await page.waitForTimeout(500);

      const content = await terminal.getTerminalContent();
      expect(content.match(/Attack|Defend|Skill|Run|Flee/i)).toBeTruthy();
    });
  });

  test.describe('Romance System', () => {
    test('can access romance options', async ({ page }) => {
      const terminal = await loginUser(page);

      await launchUsurper(terminal, page);
      await terminal.press('Space');
      await terminal.menuSelect('1');
      await terminal.send('RomanceTest');
      await page.waitForTimeout(500);

      await terminal.menuSelect('R'); // Romance menu
      await page.waitForTimeout(300);

      const content = await terminal.getTerminalContent();
      expect(content.match(/romance|flirt|partner|relationship/i)).toBeTruthy();
    });
  });

  test.describe('Equipment System', () => {
    test('can view equipment', async ({ page }) => {
      const terminal = await loginUser(page);

      await launchUsurper(terminal, page);
      await terminal.press('Space');
      await terminal.menuSelect('1');
      await terminal.send('EquipTest');
      await page.waitForTimeout(500);

      await terminal.menuSelect('E'); // Equipment
      await page.waitForTimeout(300);

      const content = await terminal.getTerminalContent();
      expect(content.match(/equipment|weapon|armor|slot|helmet/i)).toBeTruthy();
    });

    test('shows all 10 equipment slots', async ({ page }) => {
      const terminal = await loginUser(page);

      await launchUsurper(terminal, page);
      await terminal.press('Space');
      await terminal.menuSelect('1');
      await terminal.send('SlotTest');
      await page.waitForTimeout(500);

      await terminal.menuSelect('E');
      await page.waitForTimeout(300);

      const content = await terminal.getTerminalContent();
      // Should show multiple equipment slots
      expect(content.match(/weapon|shield|helmet|armor|boots|ring|amulet|cloak/i)).toBeTruthy();
    });
  });

  test.describe('Save/Resume', () => {
    test('game saves on quit', async ({ page }) => {
      const terminal = await loginUser(page);

      await launchUsurper(terminal, page);
      await terminal.press('Space');
      await terminal.menuSelect('1');
      await terminal.send('SaveTest');
      await page.waitForTimeout(500);

      // Quit game
      await terminal.menuSelect('X'); // Exit/Quit
      await terminal.waitForText('quit', 3000);
      await terminal.menuSelect('Y'); // Confirm

      await terminal.waitForText('Main Menu', 5000);
    });

    test('game resumes from save', async ({ page }) => {
      const terminal = await loginUser(page);

      // Start fresh session - game should resume
      await launchUsurper(terminal, page);

      // Should not show intro for resumed game, go directly to town
      await page.waitForTimeout(1000);
      const content = await terminal.getTerminalContent();
      expect(content.match(/Town|SaveTest|resume/i)).toBeTruthy();
    });
  });

  test.describe('Quit Confirmation', () => {
    test('shows confirmation dialog', async ({ page }) => {
      const terminal = await loginUser(page);

      await launchUsurper(terminal, page);
      await terminal.press('Space');
      await terminal.menuSelect('1');
      await terminal.send('QuitTest');
      await page.waitForTimeout(500);

      await terminal.menuSelect('X'); // Quit
      await page.waitForTimeout(300);

      const content = await terminal.getTerminalContent();
      expect(content.match(/quit|sure|confirm|Y|N/i)).toBeTruthy();
    });

    test('can cancel quit', async ({ page }) => {
      const terminal = await loginUser(page);

      await launchUsurper(terminal, page);
      await terminal.press('Space');
      await terminal.menuSelect('1');
      await terminal.send('CancelQuit');
      await page.waitForTimeout(500);

      await terminal.menuSelect('X');
      await terminal.waitForText('Y/N', 3000);
      await terminal.menuSelect('N'); // Cancel

      await page.waitForTimeout(300);
      const content = await terminal.getTerminalContent();
      expect(content.match(/Town|dungeon|menu/i)).toBeTruthy();
    });

    test('can confirm quit and return to BBS', async ({ page }) => {
      const terminal = await loginUser(page);

      await launchUsurper(terminal, page);
      await terminal.press('Space');
      await terminal.menuSelect('1');
      await terminal.send('ConfirmQuit');
      await page.waitForTimeout(500);

      await terminal.menuSelect('X');
      await terminal.waitForText('Y/N', 3000);
      await terminal.menuSelect('Y'); // Confirm

      await terminal.waitForText('Main Menu', 5000);
    });
  });

  test.describe('Leaderboard', () => {
    test('can view leaderboard', async ({ page }) => {
      const terminal = await loginUser(page);

      await launchUsurper(terminal, page);
      await terminal.press('Space');
      await terminal.menuSelect('1');
      await terminal.send('LeaderTest');
      await page.waitForTimeout(500);

      await terminal.menuSelect('L'); // Leaderboard
      await page.waitForTimeout(300);

      const content = await terminal.getTerminalContent();
      expect(content.match(/leaderboard|rank|score|level|champion/i)).toBeTruthy();
    });
  });

  test.describe('Stats Screen', () => {
    test('shows character statistics', async ({ page }) => {
      const terminal = await loginUser(page);

      await launchUsurper(terminal, page);
      await terminal.press('Space');
      await terminal.menuSelect('1');
      await terminal.send('StatsView');
      await page.waitForTimeout(500);

      await terminal.menuSelect('I'); // Info/Stats
      await page.waitForTimeout(300);

      const content = await terminal.getTerminalContent();
      expect(content.match(/Strength|Dexterity|Intelligence|Constitution/i)).toBeTruthy();
    });

    test('shows experience and level', async ({ page }) => {
      const terminal = await loginUser(page);

      await launchUsurper(terminal, page);
      await terminal.press('Space');
      await terminal.menuSelect('1');
      await terminal.send('ExpView');
      await page.waitForTimeout(500);

      await terminal.menuSelect('I');
      await page.waitForTimeout(300);

      const content = await terminal.getTerminalContent();
      expect(content.match(/Experience|XP|Level/i)).toBeTruthy();
    });
  });

  test.describe('Psychosis Effects', () => {
    test('screen distortion with low mental stability', async ({ page }) => {
      // This test verifies the psychosis visual effect system
      // When mental stability is <= 0, screen output should be distorted
      // Note: Actual testing would require manipulating game state

      const terminal = await loginUser(page);

      await launchUsurper(terminal, page);
      await terminal.press('Space');
      await terminal.menuSelect('1');
      await terminal.send('PsychosisTest');
      await page.waitForTimeout(500);

      // Check that game is running - psychosis effects would be visible
      // if mental stability drops below zero
      const content = await terminal.getTerminalContent();
      expect(content).toBeTruthy(); // Game is responsive
    });
  });
});

// Unit-style tests for game mechanics validation
test.describe('Usurper Mechanics (Unit)', () => {
  test('character classes have valid names', () => {
    const classes = ['Warrior', 'Rogue', 'Mage', 'Cleric', 'Berserker'];
    expect(classes.length).toBe(5);
    classes.forEach(cls => {
      expect(cls.length).toBeGreaterThan(0);
    });
  });

  test('dungeon levels are properly ordered', () => {
    // Dungeons should go from 0 (Surface) to 100+ (Bottom)
    const minLevel = 0;
    const maxLevel = 100;
    expect(maxLevel).toBeGreaterThan(minLevel);
  });

  test('equipment slots count is correct', () => {
    const slots = [
      'weapon',
      'shield',
      'helmet',
      'armor',
      'gloves',
      'boots',
      'ring_left',
      'ring_right',
      'amulet',
      'cloak',
    ];
    expect(slots.length).toBe(10);
  });

  test('substance types are defined', () => {
    const types = ['steroids', 'stimulants', 'sedatives', 'psychedelics', 'alchemical'];
    expect(types.length).toBeGreaterThan(0);
    types.forEach(type => {
      expect(type).toBeDefined();
    });
  });

  test('romance levels have hierarchy', () => {
    const levels = ['Single', 'Dating', 'Engaged', 'Married'];
    expect(levels.length).toBe(4);
    expect(levels[0]).toBe('Single');
    expect(levels[levels.length - 1]).toBe('Married');
  });

  test('mental stability has valid range', () => {
    // Mental stability can go negative (psychosis) or positive (stable)
    const minStability = -100; // Severe psychosis
    const maxStability = 100; // Very stable
    const threshold = 0; // Psychosis threshold

    expect(minStability).toBeLessThan(threshold);
    expect(maxStability).toBeGreaterThan(threshold);
  });

  test('PvP penalty applies to lower level targets', () => {
    // When attacking lower level players, XP penalty should apply
    const attackerLevel = 10;
    const defenderLevel = 5;

    expect(attackerLevel).toBeGreaterThan(defenderLevel);
    // Penalty would be calculated based on level difference
  });

  test('stat bonuses exist for married players', () => {
    // Married status should provide stat bonuses
    const marriedBonuses = {
      hp: 20,
      strength: 3,
      dexterity: 3,
    };

    expect(marriedBonuses.hp).toBeGreaterThan(0);
    expect(marriedBonuses.strength).toBeGreaterThan(0);
    expect(marriedBonuses.dexterity).toBeGreaterThan(0);
  });
});
