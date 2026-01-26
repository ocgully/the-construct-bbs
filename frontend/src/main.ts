import '../styles/terminal.css';
import { initTerminal, fitAddon } from './terminal';
import { connectWebSocket } from './websocket';
import { CRTController, CRTLevel } from './crt-effects';
import { setupMobile, isMobile } from './mobile';

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

    // Show connecting message
    terminal.writeln('Connecting to The Construct BBS...');
    terminal.writeln('');

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

    // Connect WebSocket
    connectWebSocket(terminal);

    console.log('Terminal initialized and connected');
  } catch (error) {
    console.error('Failed to initialize terminal:', error);
  }
});
