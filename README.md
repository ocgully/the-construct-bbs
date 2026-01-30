# The Construct BBS

A web-based Bulletin Board System built in Rust, delivering an authentic retro BBS experience through a modern browser terminal.

## Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) (v18+)

## Quick Start (Windows)

```powershell
cd C:\Git\bbs\frontend; npm run build; cd ..\backend; cargo run
```

Then open http://localhost:3000.

## Development

From the project root (`C:\Git\bbs`):

```powershell
cd C:\Git\bbs
.\dev.ps1
```

This builds the frontend, starts the backend, and launches the Vite dev server. Press `Ctrl+C` to stop both.

- **http://localhost:3000** -- Backend serving the production build
- **http://localhost:5173** -- Vite dev server with hot reload

### Manual startup

**Build frontend:**

```powershell
cd C:\Git\bbs\frontend
npm install
npm run build
```

**Run backend:**

```powershell
cd C:\Git\bbs\backend
cargo run
```

Then open http://localhost:3000.

## Configuration

Edit `config.toml` in the project root. Key sections:

- `[server]` -- Host, port
- `[terminal]` -- Terminal dimensions, CRT effects
- `[auth]` -- Login attempts, lockout, session duration, sysop handles
- `[connection]` -- Max nodes, baud simulation, idle timeout, ceremony settings
- `[email]` -- Optional SMTP config (omit for console-logged verification codes)

## Testing

```powershell
cd C:\Git\bbs\backend
cargo test
```
