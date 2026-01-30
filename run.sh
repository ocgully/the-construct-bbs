#!/usr/bin/env bash
# run.sh - Build frontend then start the backend server.
# Usage: ./run.sh
# The backend serves the built frontend at http://localhost:3000

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
export PATH="$HOME/.cargo/bin:$PATH"

# Build frontend
echo "=== Building frontend ==="
cd "$SCRIPT_DIR/frontend"
npm run build

# Run backend (serves frontend static files from frontend/dist)
echo ""
echo "=== Starting backend ==="
echo "  http://localhost:3000"
echo ""
cd "$SCRIPT_DIR/backend"
cargo run
