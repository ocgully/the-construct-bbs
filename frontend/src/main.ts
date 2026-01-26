import '../styles/terminal.css';
import { initTerminal } from './terminal';

// Main initialization
document.addEventListener('DOMContentLoaded', async () => {
  const wrapper = document.getElementById('terminal-wrapper');
  if (!wrapper) {
    console.error('Terminal wrapper not found');
    return;
  }

  try {
    // Initialize terminal
    const terminal = await initTerminal(wrapper);

    // Show connecting message
    terminal.writeln('Connecting to The Construct BBS...');
    terminal.writeln('');

    // WebSocket, CRT, and Mobile support will be added in Task 2
    console.log('Terminal initialized');
  } catch (error) {
    console.error('Failed to initialize terminal:', error);
  }
});
