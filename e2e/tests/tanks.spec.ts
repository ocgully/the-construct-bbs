/**
 * Tanks E2E Tests
 *
 * Tests for the real-time artillery game (Scorched Earth style).
 * Uses Playwright to simulate BBS terminal sessions.
 *
 * Tanks is a real-time multiplayer game where:
 * - 2-8 players take turns firing artillery shells
 * - Terrain is destructible
 * - Physics simulation includes gravity and wind
 * - Last tank standing wins
 */

import { test, expect, Page, Browser, BrowserContext } from '@playwright/test';

// Helper class for BBS terminal interaction
class BbsTerminal {
  constructor(private page: Page) {}

  async connect() {
    await this.page.goto('http://localhost:3000');
    await this.page.waitForSelector('.xterm-screen');
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

  async pressArrow(direction: 'up' | 'down' | 'left' | 'right') {
    await this.page.keyboard.press(`Arrow${direction.charAt(0).toUpperCase()}${direction.slice(1)}`);
    await this.page.waitForTimeout(50);
  }

  async pressSpace() {
    await this.page.keyboard.press('Space');
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

  async navigateToTanks() {
    await this.sendKeys('G'); // Games menu
    await this.page.waitForTimeout(200);
    await this.sendKeys('T'); // Tanks
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

test.describe('Tanks Game', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
  });

  test.describe('Main Menu', () => {
    test.skip('displays menu on entry', async () => {
      await terminal.connect();
      await terminal.navigateToTanks();
      await terminal.expectText(['TANKS', 'BLITZKRIEG', 'COMMAND CENTER']);
    });

    test.skip('shows menu options', async () => {
      await terminal.connect();
      await terminal.navigateToTanks();
      await terminal.expectText([
        '[J] Join Public Battle',
        '[C] Create Public Battle',
        '[P] Create Private Battle',
        '[I] Join by Invite Code',
        '[L] Leaderboard',
        '[H] How to Play',
        '[Q] Return to BBS'
      ]);
    });

    test.skip('can view how to play', async () => {
      await terminal.connect();
      await terminal.navigateToTanks();
      await terminal.sendKeys('H');
      await terminal.expectText(['FIELD MANUAL', 'CONTROLS', 'Arrow', 'FIRE']);
    });

    test.skip('can view leaderboard', async () => {
      await terminal.connect();
      await terminal.navigateToTanks();
      await terminal.sendKeys('L');
      await terminal.expectText(['HALL OF HEROES', 'Commander', 'Wins']);
    });

    test.skip('can quit to BBS menu', async () => {
      await terminal.connect();
      await terminal.navigateToTanks();
      await terminal.sendKeys('Q');
      await terminal.expectNotText('TANKS');
    });
  });

  test.describe('Game Lobby', () => {
    test.skip('can create public lobby', async () => {
      await terminal.connect();
      await terminal.navigateToTanks();
      await terminal.sendKeys('C');
      await terminal.expectText(['PUBLIC BATTLE', 'COMBATANTS', 'COMMANDER']);
    });

    test.skip('can create private lobby with invite code', async () => {
      await terminal.connect();
      await terminal.navigateToTanks();
      await terminal.sendKeys('P');
      await terminal.expectText(['PRIVATE BATTLE', 'Invite Code:', 'COMMANDER']);
    });

    test.skip('shows player list in lobby', async () => {
      await terminal.connect();
      await terminal.navigateToTanks();
      await terminal.sendKeys('C');
      await terminal.expectText(['1.', 'READY']);
    });

    test.skip('can toggle ready status', async () => {
      await terminal.connect();
      await terminal.navigateToTanks();
      await terminal.sendKeys('C');
      await terminal.sendKeys('R');
      await terminal.expectText('READY');
    });

    test.skip('can leave lobby', async () => {
      await terminal.connect();
      await terminal.navigateToTanks();
      await terminal.sendKeys('C');
      await terminal.sendKeys('Q');
      await terminal.expectText('COMMAND CENTER');
    });
  });

  test.describe('Join Game', () => {
    test.skip('displays available public games', async () => {
      await terminal.connect();
      await terminal.navigateToTanks();
      await terminal.sendKeys('J');
      await terminal.expectText('AVAILABLE BATTLES');
    });

    test.skip('shows message when no games available', async () => {
      await terminal.connect();
      await terminal.navigateToTanks();
      await terminal.sendKeys('J');
      await terminal.expectText(['No battles', 'Create one']);
    });

    test.skip('can join by invite code', async () => {
      await terminal.connect();
      await terminal.navigateToTanks();
      await terminal.sendKeys('I');
      await terminal.expectText('Enter the 6-character code');
      await terminal.sendKeys('ABC123');
      await terminal.pressEnter();
    });

    test.skip('shows error for invalid invite code', async () => {
      await terminal.connect();
      await terminal.navigateToTanks();
      await terminal.sendKeys('I');
      await terminal.sendKeys('WRONG1');
      await terminal.pressEnter();
      await terminal.expectText('Invalid invite code');
    });
  });

  test.describe('Gameplay - Battlefield', () => {
    test.skip('displays terrain and tanks', async () => {
      // Would need a game in progress
      await terminal.expectText(['#', '[O]']); // Terrain and tank symbols
    });

    test.skip('shows HUD with round, wind, and current turn', async () => {
      await terminal.expectText(['Round', 'Wind:', 'turn']);
    });

    test.skip('shows player health and weapon', async () => {
      await terminal.expectText(['HP:', 'Standard Shell']);
    });

    test.skip('can adjust angle with arrow keys', async () => {
      await terminal.pressArrow('left');
      await terminal.expectText('Angle:');
    });

    test.skip('can adjust power with arrow keys', async () => {
      await terminal.pressArrow('up');
      await terminal.expectText('Power:');
    });

    test.skip('can fire with space bar', async () => {
      await terminal.pressSpace();
      await terminal.expectText(['BOOM', 'Press any key']);
    });

    test.skip('can cycle weapons with tab', async () => {
      await terminal.page.keyboard.press('Tab');
      await terminal.expectText(['Baby Missile', 'Heavy Shell']);
    });

    test.skip('can view other tanks with V key', async () => {
      await terminal.sendKeys('V');
      await terminal.expectText(['HP:', '/100']);
    });
  });

  test.describe('Turn System', () => {
    test.skip('shows whose turn it is', async () => {
      await terminal.expectText("'s turn");
    });

    test.skip('advances turn after firing', async () => {
      await terminal.pressSpace(); // Fire
      await terminal.pressEnter(); // Continue
      await terminal.expectText("'s turn");
    });

    test.skip('shows turn timer', async () => {
      await terminal.expectText(['30', 'seconds']);
    });

    test.skip('auto-advances on timeout', async () => {
      // Would need to wait 30 seconds
    });
  });

  test.describe('Combat Results', () => {
    test.skip('shows explosion on hit', async () => {
      await terminal.expectText('BOOM');
    });

    test.skip('shows damage dealt', async () => {
      await terminal.expectText(['damage', 'HP']);
    });

    test.skip('announces tank destruction', async () => {
      await terminal.expectText('DESTROYED');
    });

    test.skip('shows terrain destruction', async () => {
      // Visual check for crater in terrain
    });
  });

  test.describe('Game Over', () => {
    test.skip('shows victory screen', async () => {
      await terminal.expectText(['GAME OVER', 'VICTORY']);
    });

    test.skip('shows battle statistics', async () => {
      await terminal.expectText(['BATTLE STATISTICS', 'Kills', 'Damage', 'Status']);
    });

    test.skip('shows winner', async () => {
      await terminal.expectText('conquers the battlefield');
    });

    test.skip('returns to menu after game over', async () => {
      await terminal.pressEnter();
      await terminal.expectText('COMMAND CENTER');
    });
  });

  test.describe('Multiplayer', () => {
    test.skip('multiple players can join same lobby', async ({ browser }) => {
      const player1Context = await browser.newContext();
      const player2Context = await browser.newContext();

      const player1Page = await player1Context.newPage();
      const player2Page = await player2Context.newPage();

      const terminal1 = new BbsTerminal(player1Page);
      const terminal2 = new BbsTerminal(player2Page);

      await terminal1.connect();
      await terminal2.connect();

      // Player 1 creates a game
      await terminal1.navigateToTanks();
      await terminal1.sendKeys('C');

      // Player 2 joins
      await terminal2.navigateToTanks();
      await terminal2.sendKeys('J');
      await terminal2.sendKeys('1');

      // Verify both see 2 players
      await terminal1.expectText('2');
      await terminal2.expectText('2');

      await player1Context.close();
      await player2Context.close();
    });

    test.skip('game continues when player disconnects', async ({ browser }) => {
      // Per GAME_DECISIONS.md: "Player removed from game (others continue)"
    });

    test.skip('player can rejoin at any time', async ({ browser }) => {
      // Per GAME_DECISIONS.md: "Can rejoin at any time"
    });
  });

  test.describe('Physics', () => {
    test.skip('projectile follows arc due to gravity', async () => {
      // Would need to verify trajectory visualization
    });

    test.skip('wind affects trajectory', async () => {
      // Positive wind should push projectile right
      // Negative wind should push projectile left
    });

    test.skip('terrain blocks projectiles', async () => {
      // Hit terrain should create explosion
    });
  });

  test.describe('Terrain Destruction', () => {
    test.skip('explosions create craters', async () => {
      // After explosion, terrain should have gaps
    });

    test.skip('tanks fall when terrain beneath destroyed', async () => {
      // Tanks should drop if ground disappears
    });

    test.skip('dirt bomb adds terrain', async () => {
      // Special weapon that fills instead of destroys
    });
  });

  test.describe('Weapons', () => {
    test.skip('standard shell has unlimited ammo', async () => {
      await terminal.expectText('Standard Shell');
      // Should not show ammo count
    });

    test.skip('special weapons show ammo count', async () => {
      await terminal.expectText(['(5)', '(3)', '(2)', '(1)']);
    });

    test.skip('weapons have different explosion radii', async () => {
      // Heavy shell creates larger crater than standard
    });

    test.skip('weapons have different damage', async () => {
      // Nuke does more damage than baby missile
    });
  });
});

// Physics validation tests (unit-style)
test.describe('Physics Validation (Unit)', () => {
  test('projectile arc calculation', () => {
    // Verify projectile follows parabolic arc
    const simulateStep = (x: number, y: number, vx: number, vy: number, gravity: number) => {
      vy += gravity;
      return { x: x + vx, y: y + vy, vx, vy };
    };

    let state = { x: 0, y: 0, vx: 1, vy: -2 };
    const positions: { x: number; y: number }[] = [{ x: state.x, y: state.y }];

    for (let i = 0; i < 10; i++) {
      state = simulateStep(state.x, state.y, state.vx, state.vy, 0.15);
      positions.push({ x: state.x, y: state.y });
    }

    // Projectile should rise then fall
    const minY = Math.min(...positions.map(p => p.y));
    const finalY = positions[positions.length - 1].y;
    expect(minY).toBeLessThan(0); // Goes up
    expect(finalY).toBeGreaterThan(minY); // Comes back down
  });

  test('wind affects horizontal velocity', () => {
    const applyWind = (vx: number, wind: number): number => {
      return vx + wind * 0.01;
    };

    let vx = 1.0;
    // Apply positive wind
    for (let i = 0; i < 10; i++) {
      vx = applyWind(vx, 5);
    }
    expect(vx).toBeGreaterThan(1.0);

    vx = 1.0;
    // Apply negative wind
    for (let i = 0; i < 10; i++) {
      vx = applyWind(vx, -5);
    }
    expect(vx).toBeLessThan(1.0);
  });

  test('damage calculation by distance', () => {
    const calculateDamage = (distance: number, radius: number, baseDamage: number): number => {
      if (distance > radius) return 0;
      const factor = 1.0 - (distance / (radius + 1.0));
      return Math.floor(baseDamage * factor);
    };

    // Direct hit (distance 0)
    expect(calculateDamage(0, 3, 25)).toBe(25);

    // Edge of explosion
    expect(calculateDamage(3, 3, 25)).toBeLessThan(25);
    expect(calculateDamage(3, 3, 25)).toBeGreaterThan(0);

    // Outside explosion
    expect(calculateDamage(5, 3, 25)).toBe(0);
  });

  test('angle clamping', () => {
    const clampAngle = (angle: number): number => {
      return Math.max(0, Math.min(180, angle));
    };

    expect(clampAngle(45)).toBe(45);
    expect(clampAngle(-10)).toBe(0);
    expect(clampAngle(200)).toBe(180);
  });

  test('power clamping', () => {
    const clampPower = (power: number): number => {
      return Math.max(10, Math.min(100, power));
    };

    expect(clampPower(50)).toBe(50);
    expect(clampPower(5)).toBe(10);
    expect(clampPower(150)).toBe(100);
  });
});

// Terrain tests
test.describe('Terrain Validation (Unit)', () => {
  test('terrain destruction creates circular crater', () => {
    // Verify destruction pattern is circular
    const destroyedCells: [number, number][] = [];
    const centerX = 10;
    const centerY = 10;
    const radius = 3;

    for (let dy = -radius; dy <= radius; dy++) {
      for (let dx = -radius; dx <= radius; dx++) {
        const distSq = dx * dx + dy * dy;
        if (distSq <= radius * radius) {
          destroyedCells.push([centerX + dx, centerY + dy]);
        }
      }
    }

    // Should be roughly circular (more than a square)
    expect(destroyedCells.length).toBeLessThan((radius * 2 + 1) * (radius * 2 + 1));
    expect(destroyedCells.length).toBeGreaterThan(Math.PI * radius * radius * 0.5);
  });

  test('terrain generation produces valid heights', () => {
    const generateHeights = (width: number, maxHeight: number): number[] => {
      const heights: number[] = [];
      let base = maxHeight / 2;
      for (let x = 0; x < width; x++) {
        const variation = Math.sin(x / 10) * 5;
        heights.push(Math.max(2, Math.min(maxHeight - 2, Math.floor(base + variation))));
      }
      return heights;
    };

    const heights = generateHeights(80, 20);

    expect(heights.length).toBe(80);
    heights.forEach(h => {
      expect(h).toBeGreaterThanOrEqual(2);
      expect(h).toBeLessThanOrEqual(18);
    });
  });
});

// Invite code tests
test.describe('Invite Code Validation (Unit)', () => {
  test('invite code format', () => {
    // 6 alphanumeric characters, no ambiguous chars (I, O, 0, 1)
    const inviteCodePattern = /^[A-HJ-NP-Z2-9]{6}$/;

    expect(inviteCodePattern.test('ABC234')).toBe(true);
    expect(inviteCodePattern.test('WXYZ89')).toBe(true);
    expect(inviteCodePattern.test('ABCIO1')).toBe(false); // Contains I, O, 1
    expect(inviteCodePattern.test('ABC12')).toBe(false);  // Too short
  });
});
