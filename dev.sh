#!/usr/bin/env bash
# dev.sh - Start backend and frontend dev servers together.
# Press Ctrl+C to shut both down.

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
export PATH="$HOME/.cargo/bin:$PATH"

BACKEND_PID=""
FRONTEND_PID=""

cleanup() {
    echo ""
    echo "Shutting down..."
    [ -n "$FRONTEND_PID" ] && kill "$FRONTEND_PID" 2>/dev/null && echo "  Frontend stopped."
    [ -n "$BACKEND_PID" ]  && kill "$BACKEND_PID"  2>/dev/null && echo "  Backend stopped."
    wait 2>/dev/null
    echo "Done."
    exit 0
}

trap cleanup SIGINT SIGTERM

# Build frontend first so backend can serve static files
echo "=== Building frontend ==="
cd "$SCRIPT_DIR/frontend"
npm run build

# Start backend
echo ""
echo "=== Starting backend (http://localhost:3000) ==="
cd "$SCRIPT_DIR/backend"
cargo run &
BACKEND_PID=$!

# Give backend a moment to start
sleep 2

# Start frontend dev server (hot reload on http://localhost:5173)
echo ""
echo "=== Starting frontend dev server (http://localhost:5173) ==="
cd "$SCRIPT_DIR/frontend"
npm run dev &
FRONTEND_PID=$!

echo ""
echo "================================================"
echo "  Backend:  http://localhost:3000  (static build)"
echo "  Frontend: http://localhost:5173  (dev + hot reload)"
echo ""
echo "  Press Ctrl+C to stop both servers."
echo "================================================"
echo ""

# Wait for either process to exit
wait
