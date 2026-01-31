import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './tests',
  fullyParallel: false, // Sequential for BBS state
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: 1, // Single worker - BBS has limited nodes
  reporter: 'html',
  use: {
    baseURL: 'http://localhost:3000/?e2e',
    trace: 'on-first-retry',
    video: 'on-first-retry',
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
  webServer: {
    command: process.platform === 'win32'
      ? 'cd /d C:\\Git\\bbs\\backend && C:\\Users\\chris\\.cargo\\bin\\cargo.exe run --release'
      : 'cd ../backend && cargo run --release',
    url: 'http://localhost:3000',
    reuseExistingServer: true, // Always reuse for e2e tests
    timeout: 180000, // 3 min for backend to compile/start
  },
});
