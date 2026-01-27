import '../styles/terminal.css';
import { initTerminal, fitAddon } from './terminal';
import { connectWebSocket } from './websocket';
import { CRTController, CRTLevel } from './crt-effects';
import { setupMobile, isMobile } from './mobile';
import { loadModemSound, playModemSound } from './audio';

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

    // Preload modem sound early (before user gesture)
    loadModemSound();

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

    // Show connect prompt -- user must press a key to "pick up the phone"
    // This serves double duty: user gesture for AudioContext AND atmospheric moment
    terminal.writeln('\x1b[96m\x1b[1m');
    terminal.writeln('  The Construct BBS');
    terminal.writeln('\x1b[0m');
    terminal.writeln('\x1b[93m  Press any key to dial in...\x1b[0m');
    terminal.writeln('');

    // Wait for a single keypress, then play modem sound and connect
    const disposable = terminal.onData(() => {
      disposable.dispose();

      // Play modem sound (user gesture satisfies autoplay policy)
      playModemSound();

      // Connect to the BBS
      connectWebSocket(terminal);
    });

    console.log('Terminal initialized, awaiting user input to connect');
  } catch (error) {
    console.error('Failed to initialize terminal:', error);
  }
});
