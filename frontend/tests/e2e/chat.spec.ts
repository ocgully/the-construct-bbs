// frontend/tests/e2e/chat.spec.ts
// NOTE: These tests require a logged-in user, but email verification is required.
// Tests are skipped until verified user fixtures are available.

import { test, expect } from '@playwright/test';
import { createTerminal } from './helpers/terminal';

test.describe('Chat System', () => {
  test.skip('can enter chat room', async ({ page }) => {
    // Requires logged-in user
  });

  test.skip('can send chat message', async ({ page }) => {
    // Requires logged-in user
  });

  test.skip('can use /quit to exit chat', async ({ page }) => {
    // Requires logged-in user
  });
});
