import { Terminal } from '@xterm/xterm';
import { playModemSuccess, playModemFail } from './audio';
import { SessionTimer } from './timer';

export function connectWebSocket(terminal: Terminal, opts?: { timer?: SessionTimer }): WebSocket {
  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
  const wsUrl = `${protocol}//${window.location.hostname}:3000/ws`;

  let ws: WebSocket;

  function connect() {
    ws = new WebSocket(wsUrl);

    ws.onopen = () => {
      console.log('WebSocket connected');
      // Clear the "Press any key to connect..." prompt
      terminal.clear();

      // Send auth token if available (enables session resumption in Plan 05)
      const storedToken = localStorage.getItem('bbs_session_token');
      ws.send(JSON.stringify({ type: 'auth', token: storedToken }));
    };

    ws.onmessage = (event) => {
      const data = event.data as string;

      // Try to intercept JSON control messages from the server
      try {
        const parsed = JSON.parse(data);
        if (parsed && typeof parsed === 'object' && parsed.type) {
          if (parsed.type === 'session' && parsed.token) {
            // Store session token for reconnect persistence
            localStorage.setItem('bbs_session_token', parsed.token);
            console.log('Session token stored');
            return; // Don't write JSON to terminal
          }
          if (parsed.type === 'logout') {
            // Clear stored session token
            localStorage.removeItem('bbs_session_token');
            if (opts?.timer) {
              opts.timer.stop();
            }
            console.log('Session token cleared');
            return; // Don't write JSON to terminal
          }
          if (parsed.type === 'modem') {
            // Play appropriate modem sound based on connection result
            if (parsed.status === 'success') {
              playModemSuccess();
            } else if (parsed.status === 'fail') {
              playModemFail();
            }
            return; // Don't write JSON to terminal
          }
          if (parsed.type === 'timer') {
            // Update status bar timer
            if (opts?.timer) {
              opts.timer.updateFromServer(parsed);
            }
            return; // Don't write JSON to terminal
          }
          if (parsed.type === 'timer_warning') {
            // Timer warning -- no action needed, status bar handles visuals
            // Could play a bell sound here if desired
            return;
          }
          if (parsed.type === 'timeout') {
            // Session timed out -- server will send goodbye screen as terminal text
            // Stop client timer
            if (opts?.timer) {
              opts.timer.stop();
            }
            return;
          }
        }
      } catch {
        // Not JSON -- this is normal terminal output, fall through
      }

      // Write received data to terminal
      terminal.write(data);
    };

    ws.onerror = (error) => {
      console.error('WebSocket error:', error);
    };

    ws.onclose = () => {
      console.log('WebSocket disconnected');
      if (opts?.timer) {
        opts.timer.stop();
      }
      terminal.writeln('\r\n\r\nDisconnected. Press any key to reconnect...');

      // Wait for a keypress before reconnecting
      function handleReconnectKey(e: KeyboardEvent) {
        if (['Shift', 'Control', 'Alt', 'Meta', 'F12'].includes(e.key)) return;
        document.removeEventListener('keydown', handleReconnectKey, true);
        terminal.clear();
        connect();
      }
      document.addEventListener('keydown', handleReconnectKey, true);
    };

    return ws;
  }

  ws = connect();

  // Wire terminal input to WebSocket
  // Filter out mouse/scroll escape sequences -- only send printable input and basic control chars
  terminal.onData((data) => {
    if (ws.readyState === WebSocket.OPEN) {
      // Ignore mouse reporting sequences (ESC [ M ..., ESC [ < ...)
      if (data.startsWith('\x1b[M') || data.startsWith('\x1b[<')) {
        return;
      }
      ws.send(data);
    }
  });

  // Cleanup on page unload
  window.addEventListener('beforeunload', () => {
    if (ws.readyState === WebSocket.OPEN) {
      ws.close();
    }
  });

  return ws;
}
