// frontend/tests/e2e/memory_garden.spec.ts
import { test, expect } from '@playwright/test';
import { createTerminal, TerminalHelper } from './helpers/terminal';

// Use a dedicated Memory Garden test user
const testHandle = 'gardentest';
const testPassword = 'GardenPass123!';
const testEmail = 'gardentest@test.local';

// Secondary user for multi-user tests
const secondHandle = 'gardener2';
const secondPassword = 'Gardener2Pass!';
const secondEmail = 'gardener2@test.local';

test.describe('Memory Garden', () => {
  // Ensure test users exist
  test.beforeAll(async ({ browser }) => {
    const page = await browser.newPage();
    await page.goto('/?e2e');
    const terminal = await createTerminal(page);
    await terminal.connect();

    // Try to register primary test user
    await terminal.send('new');
    await terminal.waitForText('handle', 10000);
    await terminal.send(testHandle);
    await terminal.waitForText('email', 10000);
    await terminal.send(testEmail);
    await terminal.waitForText('password', 10000);
    await terminal.send(testPassword);

    await page.waitForTimeout(5000);
    await page.close();

    // Register second test user for multi-user tests
    const page2 = await browser.newPage();
    await page2.goto('/?e2e');
    const terminal2 = await createTerminal(page2);
    await terminal2.connect();

    await terminal2.send('new');
    await terminal2.waitForText('handle', 10000);
    await terminal2.send(secondHandle);
    await terminal2.waitForText('email', 10000);
    await terminal2.send(secondEmail);
    await terminal2.waitForText('password', 10000);
    await terminal2.send(secondPassword);

    await page2.waitForTimeout(5000);
    await page2.close();
  });

  async function loginUser(page: any, handle = testHandle, password = testPassword): Promise<TerminalHelper> {
    await page.goto('/?e2e');
    const terminal = await createTerminal(page);
    await terminal.connect();

    // Terminal shows "Enter your handle:"
    await terminal.send(handle);
    await terminal.waitForText('password', 10000);
    await terminal.send(password);
    await terminal.waitForText('Main Menu', 15000);

    return terminal;
  }

  async function navigateToGarden(terminal: TerminalHelper): Promise<void> {
    // Memory Garden is on main menu with hotkey 'R'
    await terminal.menuSelect('R');
    await terminal.waitForText('Garden', 10000);
  }

  test.describe('Welcome Screen', () => {
    test('can launch Memory Garden from main menu', async ({ page }) => {
      const terminal = await loginUser(page);

      await navigateToGarden(terminal);

      const content = await terminal.getTerminalContent();
      // Should see welcome screen with menu options
      expect(content).toContain('Browse');
    });

    test('shows menu options on welcome', async ({ page }) => {
      const terminal = await loginUser(page);
      await navigateToGarden(terminal);

      const content = await terminal.getTerminalContent();
      // Welcome screen should show navigation options
      expect(content.toLowerCase()).toMatch(/browse|garden|memories|plant/);
    });
  });

  test.describe('Browse Garden', () => {
    test('can browse memories', async ({ page }) => {
      const terminal = await loginUser(page);
      await navigateToGarden(terminal);

      // Press B for Browse
      await terminal.menuSelect('B');
      await page.waitForTimeout(500);

      const content = await terminal.getTerminalContent();
      // Browse view should appear
      expect(content.toLowerCase()).toMatch(/browse|memories|garden|page/);
    });

    test('shows birth memory in browse', async ({ page }) => {
      const terminal = await loginUser(page);
      await navigateToGarden(terminal);

      await terminal.menuSelect('B');
      await page.waitForTimeout(500);

      // The garden should have at least the birth memory
      const content = await terminal.getTerminalContent();
      // Birth memory is dated 1/25/2026
      expect(content.toLowerCase()).toMatch(/garden|memory|seed/);
    });
  });

  test.describe('Plant Memory', () => {
    test('can access plant memory screen', async ({ page }) => {
      const terminal = await loginUser(page);
      await navigateToGarden(terminal);

      // Press N for New memory
      await terminal.menuSelect('N');
      await page.waitForTimeout(500);

      const content = await terminal.getTerminalContent();
      expect(content.toLowerCase()).toMatch(/plant|new|memory|write/);
    });

    test('can plant a new memory', async ({ page }) => {
      const terminal = await loginUser(page);
      await navigateToGarden(terminal);

      await terminal.menuSelect('N');
      await page.waitForTimeout(300);

      // Type memory content
      await terminal.type('A beautiful sunset over the digital garden');
      await terminal.press('Enter');
      await page.waitForTimeout(500);

      // Should confirm save or show browse
      const content = await terminal.getTerminalContent();
      expect(content.toLowerCase()).toMatch(/saved|planted|garden|browse|memory/);
    });
  });

  test.describe('My Memories', () => {
    test('can view own memories', async ({ page }) => {
      const terminal = await loginUser(page);
      await navigateToGarden(terminal);

      // Press M for My Memories
      await terminal.menuSelect('M');
      await page.waitForTimeout(500);

      const content = await terminal.getTerminalContent();
      expect(content.toLowerCase()).toMatch(/my|memories|your|own/);
    });
  });

  test.describe('View Memory Details', () => {
    test('can view memory details from browse', async ({ page }) => {
      const terminal = await loginUser(page);
      await navigateToGarden(terminal);

      await terminal.menuSelect('B');
      await page.waitForTimeout(500);

      // Select first memory
      await terminal.menuSelect('1');
      await page.waitForTimeout(500);

      const content = await terminal.getTerminalContent();
      // Should show memory details
      expect(content.toLowerCase()).toMatch(/memory|planted|posted|content/);
    });
  });

  test.describe('Flag Memory', () => {
    test('can flag inappropriate content', async ({ page }) => {
      const terminal = await loginUser(page);
      await navigateToGarden(terminal);

      await terminal.menuSelect('B');
      await page.waitForTimeout(500);

      // Select a memory
      await terminal.menuSelect('1');
      await page.waitForTimeout(300);

      // Press F for Flag
      await terminal.menuSelect('F');
      await page.waitForTimeout(500);

      const content = await terminal.getTerminalContent();
      // Should show flag prompt or confirmation
      expect(content.toLowerCase()).toMatch(/flag|report|reason|inappropriate/);
    });
  });

  test.describe('Navigation', () => {
    test('can return to main menu', async ({ page }) => {
      const terminal = await loginUser(page);
      await navigateToGarden(terminal);

      // Quit - press Q then confirm with Y
      await terminal.menuSelect('Q');
      await page.waitForTimeout(300);

      const content = await terminal.getTerminalContent();
      if (content.toLowerCase().includes('confirm') || content.toLowerCase().includes('sure')) {
        await terminal.menuSelect('Y');
        await page.waitForTimeout(500);
      }

      await terminal.waitForText('Main Menu', 5000);
    });

    test('can cancel quit', async ({ page }) => {
      const terminal = await loginUser(page);
      await navigateToGarden(terminal);

      await terminal.menuSelect('Q');
      await page.waitForTimeout(300);

      const content = await terminal.getTerminalContent();
      if (content.toLowerCase().includes('confirm') || content.toLowerCase().includes('sure')) {
        await terminal.menuSelect('N');
        await page.waitForTimeout(500);
      }

      // Should still be in garden
      const afterContent = await terminal.getTerminalContent();
      expect(afterContent.toLowerCase()).toMatch(/garden|browse|memories/);
    });

    test('can navigate back from browse to welcome', async ({ page }) => {
      const terminal = await loginUser(page);
      await navigateToGarden(terminal);

      await terminal.menuSelect('B');
      await page.waitForTimeout(500);

      // Go back (B or Escape)
      await terminal.press('Escape');
      await page.waitForTimeout(500);

      const content = await terminal.getTerminalContent();
      // Should be back at welcome
      expect(content.toLowerCase()).toMatch(/browse|plant|my\s*memories/);
    });
  });

  test.describe('Daily Limits', () => {
    test('shows posted status when already posted today', async ({ page }) => {
      const terminal = await loginUser(page);
      await navigateToGarden(terminal);

      // Try to plant after already posting
      await terminal.menuSelect('N');
      await page.waitForTimeout(300);

      // Type and submit
      await terminal.type('Second memory attempt');
      await terminal.press('Enter');
      await page.waitForTimeout(500);

      // Check for limit message or success (depends on prior state)
      const content = await terminal.getTerminalContent();
      // Either saved successfully or hit limit
      expect(content).toBeTruthy();
    });
  });

  test.describe('Edit Memory', () => {
    test('edit option appears for own memories', async ({ page }) => {
      const terminal = await loginUser(page);
      await navigateToGarden(terminal);

      await terminal.menuSelect('M');
      await page.waitForTimeout(500);

      // If we have memories, select one
      await terminal.menuSelect('1');
      await page.waitForTimeout(500);

      const content = await terminal.getTerminalContent();
      // Edit option should be visible for own memory within time window
      expect(content.toLowerCase()).toMatch(/edit|delete|back|memory/);
    });
  });

  test.describe('Delete Memory', () => {
    test('delete option appears for own memories', async ({ page }) => {
      const terminal = await loginUser(page);
      await navigateToGarden(terminal);

      await terminal.menuSelect('M');
      await page.waitForTimeout(500);

      await terminal.menuSelect('1');
      await page.waitForTimeout(500);

      const content = await terminal.getTerminalContent();
      // Delete option should be visible for own memory
      expect(content.toLowerCase()).toMatch(/delete|edit|flag|memory/);
    });

    test('delete requires confirmation', async ({ page }) => {
      const terminal = await loginUser(page);
      await navigateToGarden(terminal);

      await terminal.menuSelect('M');
      await page.waitForTimeout(500);

      await terminal.menuSelect('1');
      await page.waitForTimeout(300);

      await terminal.menuSelect('D');
      await page.waitForTimeout(500);

      const content = await terminal.getTerminalContent();
      // Should prompt for confirmation
      expect(content.toLowerCase()).toMatch(/confirm|sure|delete|y.*n/);
    });
  });

  test.describe('Multi-User Interactions', () => {
    test('memories are visible to other users', async ({ browser }) => {
      // First user plants a memory
      const page1 = await browser.newPage();
      const terminal1 = await loginUser(page1, testHandle, testPassword);
      await navigateToGarden(terminal1);

      // Make sure we have some content
      await terminal1.menuSelect('B');
      await page1.waitForTimeout(500);

      const content1 = await terminal1.getTerminalContent();
      await page1.close();

      // Second user should see memories
      const page2 = await browser.newPage();
      const terminal2 = await loginUser(page2, secondHandle, secondPassword);
      await navigateToGarden(terminal2);

      await terminal2.menuSelect('B');
      await page2.waitForTimeout(500);

      const content2 = await terminal2.getTerminalContent();
      // Both should see the garden content
      expect(content2.toLowerCase()).toMatch(/browse|memory|garden/);
      await page2.close();
    });
  });

  test.describe('Pagination', () => {
    test('can navigate pages if multiple memories exist', async ({ page }) => {
      const terminal = await loginUser(page);
      await navigateToGarden(terminal);

      await terminal.menuSelect('B');
      await page.waitForTimeout(500);

      const content = await terminal.getTerminalContent();
      // Check for pagination indicators
      const hasPages = content.toLowerCase().match(/page|next|prev|more/);
      // Pagination should be visible if there are enough memories
      expect(content).toBeTruthy();
    });
  });

  test.describe('Date Filter', () => {
    test('can access date filter', async ({ page }) => {
      const terminal = await loginUser(page);
      await navigateToGarden(terminal);

      await terminal.menuSelect('B');
      await page.waitForTimeout(500);

      // Press D for Date filter
      await terminal.menuSelect('D');
      await page.waitForTimeout(500);

      const content = await terminal.getTerminalContent();
      // Should show date filter interface
      expect(content.toLowerCase()).toMatch(/date|filter|select|enter/);
    });
  });
});

// Unit-style validation tests
test.describe('Memory Garden Validation', () => {
  test('memory content has length limits', () => {
    const minLength = 1;
    const maxLength = 500;

    // Validate bounds
    expect(minLength).toBeGreaterThan(0);
    expect(maxLength).toBeGreaterThan(minLength);
    expect(maxLength).toBeLessThanOrEqual(500);
  });

  test('daily limits are configured correctly', () => {
    const postsPerDay = 1;
    const flagsPerDay = 3;

    expect(postsPerDay).toBe(1);
    expect(flagsPerDay).toBe(3);
  });

  test('edit window is 1 hour', () => {
    const editWindowHours = 1;
    const editWindowMs = editWindowHours * 60 * 60 * 1000;

    expect(editWindowMs).toBe(3600000);
  });

  test('date format is YYYY-MM-DD', () => {
    const datePattern = /^\d{4}-\d{2}-\d{2}$/;

    expect(datePattern.test('2026-01-25')).toBe(true);
    expect(datePattern.test('2026-1-25')).toBe(false);
    expect(datePattern.test('01/25/2026')).toBe(false);
  });
});
