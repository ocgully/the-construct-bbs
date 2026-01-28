import '../styles/terminal.css';
import { initTerminal, fitAddon } from './terminal';
import { connectWebSocket } from './websocket';
import { CRTController, CRTLevel } from './crt-effects';
import { setupMobile, isMobile } from './mobile';
import { loadModemSounds, ensureAudioReady } from './audio';
import { StatusBar } from './status-bar';
import { SessionTimer } from './timer';

// Main initialization
document.addEventListener('DOMContentLoaded', async () => {
  const wrapper = document.getElementById('terminal-wrapper');
  const container = document.getElementById('terminal-container');

  if (!wrapper || !container) {
    console.error('Terminal elements not found');
    return;
  }

  try {
    // Initialize terminal
    const terminal = await initTerminal(wrapper);

    // Preload modem sounds early (decoded when AudioContext is ready)
    loadModemSounds();

    // Initialize CRT controller
    const crtController = new CRTController(container);

    // Set up CRT toggle (F12 key)
    document.addEventListener('keydown', (e) => {
      if (e.key === 'F12') {
        e.preventDefault();
        crtController.cycle();

        // Show notification of current level
        const level = crtController.getLevel();
        const levelName = level.replace('crt-', '').toUpperCase();
        terminal.writeln(`\r\nCRT Effect: ${levelName}`);
      }
    });

    // Set up mobile support
    setupMobile(terminal, fitAddon);

    if (isMobile()) {
      console.log('Mobile mode active');
    }

    // Create status bar and timer
    const statusBar = new StatusBar(terminal);
    const timer = new SessionTimer(statusBar);

    // Show "press any key" prompt and wait for user gesture before connecting.
    // This satisfies browser autoplay policy so modem sounds can play.
    terminal.write('\r\n  Press any key to connect to \x1b[32mThe Construct BBS\x1b[0m...\r\n');

    function handleFirstKey(e: KeyboardEvent) {
      // Ignore modifier keys and F12 (CRT toggle)
      if (['Shift', 'Control', 'Alt', 'Meta', 'F12'].includes(e.key)) return;

      document.removeEventListener('keydown', handleFirstKey, true);

      // Resume AudioContext during this user gesture (browser autoplay policy)
      ensureAudioReady();

      // Connect to BBS
      connectWebSocket(terminal, { timer });
    }

    // Use capture phase so Enter/Space reach us before xterm.js absorbs them
    document.addEventListener('keydown', handleFirstKey, true);

    console.log('Terminal initialized, press any key to connect');
  } catch (error) {
    console.error('Failed to initialize terminal:', error);
  }
});
