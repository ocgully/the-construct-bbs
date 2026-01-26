import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';

export function isMobile(): boolean {
  return /Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(
    navigator.userAgent
  );
}

export function setupMobile(terminal: Terminal, fitAddon: FitAddon): void {
  if (!isMobile()) {
    return;
  }

  console.log('Mobile device detected, applying mobile optimizations');

  // Get terminal wrapper
  const terminalWrapper = document.querySelector('.terminal-wrapper') as HTMLElement;
  if (!terminalWrapper) {
    console.error('Terminal wrapper not found');
    return;
  }

  // Fit terminal on initial load
  setTimeout(() => {
    fitAddon.fit();
  }, 100);

  // Handle visual viewport resize (keyboard show/hide)
  if (window.visualViewport) {
    window.visualViewport.addEventListener('resize', () => {
      // Adjust layout when keyboard appears/disappears
      requestAnimationFrame(() => {
        fitAddon.fit();
      });
    });
  }

  // Handle orientation change
  window.addEventListener('orientationchange', () => {
    setTimeout(() => {
      fitAddon.fit();
    }, 300);
  });

  // Handle screen resize
  window.addEventListener('resize', () => {
    setTimeout(() => {
      fitAddon.fit();
    }, 100);
  });

  // Tap to focus terminal (helps with mobile keyboard)
  terminalWrapper.addEventListener('click', () => {
    terminal.focus();
  });

  // Prevent default touch behaviors that might interfere
  terminalWrapper.addEventListener('touchstart', (e) => {
    e.preventDefault();
    terminal.focus();
  }, { passive: false });

  // Focus terminal initially
  terminal.focus();

  // Adjust font size for better mobile readability
  const updateFontSize = () => {
    const viewportWidth = window.innerWidth;
    if (viewportWidth < 768) {
      // Small screens
      terminal.options.fontSize = 10;
    } else {
      // Tablets in portrait
      terminal.options.fontSize = 12;
    }
    fitAddon.fit();
  };

  updateFontSize();
  window.addEventListener('resize', updateFontSize);
}
