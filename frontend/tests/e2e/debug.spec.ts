// Debug test to understand xterm.js content extraction
import { test, expect } from '@playwright/test';

test('debug terminal content extraction', async ({ page }) => {
  await page.goto('/?e2e');

  // Wait for xterm.js to initialize
  await page.waitForSelector('.xterm-screen', { timeout: 15000 });

  // Debug: check what DOM elements exist
  const terminalInfo = await page.evaluate(() => {
    const result: any = {
      hasXterm: !!document.querySelector('.xterm'),
      hasXtermScreen: !!document.querySelector('.xterm-screen'),
      hasXtermRows: !!document.querySelector('.xterm-rows'),
      hasXtermViewport: !!document.querySelector('.xterm-viewport'),
      xtermRowsChildCount: document.querySelector('.xterm-rows')?.children.length || 0,
      bodyText: document.body.innerText?.substring(0, 500) || '',
      windowTerminal: !!(window as any).terminal,
    };

    // Check xterm-rows content
    const rows = document.querySelectorAll('.xterm-rows > div');
    result.rowsContent = Array.from(rows).map(r => r.textContent || '').filter(t => t.trim()).slice(0, 5);

    // Check for canvas (WebGL rendering)
    const canvases = document.querySelectorAll('canvas');
    result.canvasCount = canvases.length;

    return result;
  });

  console.log('Terminal Info:', JSON.stringify(terminalInfo, null, 2));

  // Screenshot for visual debugging
  await page.screenshot({ path: 'test-results/debug-terminal.png' });

  // The test passes but logs debug info
  expect(terminalInfo.hasXterm).toBe(true);
});
