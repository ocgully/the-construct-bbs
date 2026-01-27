import { Terminal } from '@xterm/xterm';

const INITIAL_RECONNECT_DELAY = 1000;
const MAX_RECONNECT_DELAY = 30000;
const RECONNECT_MULTIPLIER = 1.5;

export function connectWebSocket(terminal: Terminal): WebSocket {
  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
  const wsUrl = `${protocol}//${window.location.hostname}:3000/ws`;

  let reconnectDelay = INITIAL_RECONNECT_DELAY;
  let reconnectTimeout: number;
  let ws: WebSocket;

  function connect() {
    ws = new WebSocket(wsUrl);

    ws.onopen = () => {
      console.log('WebSocket connected');
      // Clear the "Press any key to dial in..." prompt
      terminal.clear();
      reconnectDelay = INITIAL_RECONNECT_DELAY;

      // Send auth token if available (enables session resumption in Plan 05)
      const storedToken = localStorage.getItem('bbs_session_token');
      ws.send(JSON.stringify({ type: 'auth', token: storedToken }));
    };

    ws.onmessage = (event) => {
      // Write received data to terminal
      terminal.write(event.data);
    };

    ws.onerror = (error) => {
      console.error('WebSocket error:', error);
    };

    ws.onclose = () => {
      console.log('WebSocket disconnected');
      terminal.writeln('\r\n\r\nDisconnected from server.');
      terminal.writeln(`Reconnecting in ${Math.round(reconnectDelay / 1000)}s...`);

      // Exponential backoff reconnection
      reconnectTimeout = window.setTimeout(() => {
        console.log('Attempting to reconnect...');
        connect();
      }, reconnectDelay);

      reconnectDelay = Math.min(
        reconnectDelay * RECONNECT_MULTIPLIER,
        MAX_RECONNECT_DELAY
      );
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
    clearTimeout(reconnectTimeout);
    if (ws.readyState === WebSocket.OPEN) {
      ws.close();
    }
  });

  return ws;
}
