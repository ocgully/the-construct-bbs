// frontend/tests/e2e/news.spec.ts
// NOTE: These tests require a logged-in user, but email verification is required.
// Tests are skipped until verified user fixtures are available.

import { test, expect } from '@playwright/test';
import { createTerminal } from './helpers/terminal';

test.describe('News System', () => {
  test.skip('can access news', async ({ page }) => {
    // Requires logged-in user
  });

  test.skip('can navigate news with arrow keys', async ({ page }) => {
    // Requires logged-in user
  });

  test.skip('can exit news with Q', async ({ page }) => {
    // Requires logged-in user
  });
});
