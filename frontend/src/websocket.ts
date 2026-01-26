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
      terminal.clear();
      terminal.writeln('Connected to The Construct BBS');
      terminal.writeln('');
      reconnectDelay = INITIAL_RECONNECT_DELAY;
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
  terminal.onData((data) => {
    if (ws.readyState === WebSocket.OPEN) {
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
