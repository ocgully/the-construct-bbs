import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { WebglAddon } from '@xterm/addon-webgl';
import { WebFontsAddon } from '@xterm/addon-web-fonts';
import '@xterm/xterm/css/xterm.css';

// CGA 16-color palette (authentic DOS colors with brown instead of dark yellow)
const CGA_PALETTE = {
  black: '#000000',
  red: '#aa0000',
  green: '#00aa00',
  yellow: '#aa5500',      // CRITICAL: Brown, not bright yellow
  blue: '#0000aa',
  magenta: '#aa00aa',
  cyan: '#00aaaa',
  white: '#aaaaaa',
  brightBlack: '#555555',
  brightRed: '#ff5555',
  brightGreen: '#55ff55',
  brightYellow: '#ffff55',
  brightBlue: '#5555ff',
  brightMagenta: '#ff55ff',
  brightCyan: '#55ffff',
  brightWhite: '#ffffff',
};

export let fitAddon: FitAddon;

export async function initTerminal(container: HTMLElement): Promise<Terminal> {
  // Create terminal with CP437 configuration
  const terminal = new Terminal({
    cols: 80,
    rows: 24,
    fontFamily: "'PerfectDOSVGA437', 'Courier New', monospace",
    fontSize: 16,
    theme: {
      background: CGA_PALETTE.black,
      foreground: CGA_PALETTE.white,
      cursor: CGA_PALETTE.white,
      cursorAccent: CGA_PALETTE.black,
      selectionBackground: CGA_PALETTE.blue,
      selectionForeground: CGA_PALETTE.white,
      black: CGA_PALETTE.black,
      red: CGA_PALETTE.red,
      green: CGA_PALETTE.green,
      yellow: CGA_PALETTE.yellow,
      blue: CGA_PALETTE.blue,
      magenta: CGA_PALETTE.magenta,
      cyan: CGA_PALETTE.cyan,
      white: CGA_PALETTE.white,
      brightBlack: CGA_PALETTE.brightBlack,
      brightRed: CGA_PALETTE.brightRed,
      brightGreen: CGA_PALETTE.brightGreen,
      brightYellow: CGA_PALETTE.brightYellow,
      brightBlue: CGA_PALETTE.brightBlue,
      brightMagenta: CGA_PALETTE.brightMagenta,
      brightCyan: CGA_PALETTE.brightCyan,
      brightWhite: CGA_PALETTE.brightWhite,
    },
    scrollback: 0,           // No scrollback buffer
    cursorBlink: true,
    cursorStyle: 'block',
    allowProposedApi: true,  // Required for some addons
  });

  // Load WebFonts addon first to ensure font loads
  const webFontsAddon = new WebFontsAddon();
  terminal.loadAddon(webFontsAddon);

  // Load FitAddon for responsive sizing
  fitAddon = new FitAddon();
  terminal.loadAddon(fitAddon);

  // Open terminal in container
  terminal.open(container);

  // Load WebGL addon for performance (with fallback)
  try {
    const webglAddon = new WebglAddon();
    terminal.loadAddon(webglAddon);
    console.log('WebGL renderer enabled');
  } catch (e) {
    console.warn('WebGL addon failed to load, using canvas renderer:', e);
  }

  // Initial fit
  fitAddon.fit();

  // Debounced resize handler
  let resizeTimeout: number;
  window.addEventListener('resize', () => {
    clearTimeout(resizeTimeout);
    resizeTimeout = window.setTimeout(() => {
      fitAddon.fit();
    }, 100);
  });

  return terminal;
}
