// frontend/tests/e2e/gtm.spec.ts
// NOTE: These tests require a logged-in user, but email verification is required.
// Tests are skipped until verified user fixtures are available.

import { test, expect } from '@playwright/test';
import { createTerminal } from './helpers/terminal';

test.describe('Grand Theft Meth', () => {
  test.skip('can launch game from menu', async ({ page }) => {
    // Requires logged-in user
  });

  test.skip('can skip intro and see main menu', async ({ page }) => {
    // Requires logged-in user
  });

  test.skip('shows status bar with game stats', async ({ page }) => {
    // Requires logged-in user
  });

  test.skip('can access travel screen', async ({ page }) => {
    // Requires logged-in user
  });

  test.skip('can access trade screen', async ({ page }) => {
    // Requires logged-in user
  });

  test.skip('can access loan shark', async ({ page }) => {
    // Requires logged-in user
  });

  test.skip('can quit game and return to BBS menu', async ({ page }) => {
    // Requires logged-in user
  });

  test.skip('game saves on quit and resumes', async ({ page }) => {
    // Requires logged-in user
  });

  test.skip('can use drugs if in inventory', async ({ page }) => {
    // Requires logged-in user
  });

  test.skip('can access quests screen', async ({ page }) => {
    // Requires logged-in user
  });
});
