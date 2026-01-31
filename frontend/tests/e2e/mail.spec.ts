// frontend/tests/e2e/mail.spec.ts
// NOTE: These tests require a logged-in user, but email verification is required.
// Tests are skipped until verified user fixtures are available.

import { test, expect } from '@playwright/test';
import { createTerminal } from './helpers/terminal';

test.describe('Mail System', () => {
  test.skip('can access mail inbox', async ({ page }) => {
    // Requires logged-in user
  });

  test.skip('can compose new message', async ({ page }) => {
    // Requires logged-in user
  });

  test.skip('can quit mail and return to menu', async ({ page }) => {
    // Requires logged-in user
  });
});
