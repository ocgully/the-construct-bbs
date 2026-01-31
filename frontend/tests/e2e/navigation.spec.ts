// frontend/tests/e2e/navigation.spec.ts
// NOTE: These tests require a logged-in user, but email verification is required.
// Tests are skipped until either:
// 1. A skip_verification config option is added
// 2. Test fixtures are created with pre-verified users
// 3. E2E test harness can read verification codes from backend logs

import { test, expect } from '@playwright/test';
import { createTerminal } from './helpers/terminal';

test.describe('Navigation', () => {
  test.skip('shows main menu with options', async ({ page }) => {
    // Requires logged-in user
  });

  test.skip('can enter Games submenu', async ({ page }) => {
    // Requires logged-in user
  });

  test.skip('can return from submenu with Q', async ({ page }) => {
    // Requires logged-in user
  });

  test.skip('can view profile with P', async ({ page }) => {
    // Requires logged-in user
  });

  test.skip("can access Who's Online with W", async ({ page }) => {
    // Requires logged-in user
  });
});
