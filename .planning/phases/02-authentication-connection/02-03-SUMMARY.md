---
phase: 02-authentication-connection
plan: 03
subsystem: connection, ui
tags: [websocket, ansi, cp437, web-audio-api, ceremony, modem, baud-simulation]

# Dependency graph
requires:
  - phase: 01-terminal-foundation
    provides: "AnsiWriter, CP437, WebSocket session layer, xterm.js frontend"
  - phase: 02-authentication-connection
    provides: "NodeManager (02-02), ConnectionConfig (02-01)"
provides:
  - "Connection ceremony with typewriter-paced text over WebSocket"
  - "ANSI splash screen with baud-rate line-by-line reveal"
  - "Line-busy rejection with CP437 box-drawing art"
  - "Modem handshake audio playback via Web Audio API"
  - "Node assignment during ceremony, release on disconnect"
  - "Frontend auth token send on WebSocket connect"
affects: [02-05-login-flow, 02-07-integration, 03-main-menu]

# Tech tracking
tech-stack:
  added: []
  patterns: ["ceremony.rs direct tx writes with tokio::time::sleep for timed output", "Web Audio API with user gesture for autoplay policy"]

key-files:
  created:
    - "frontend/public/audio/modem.mp3"
    - "frontend/src/audio.ts"
    - "backend/src/connection/ceremony.rs"
  modified:
    - "frontend/src/main.ts"
    - "frontend/src/websocket.ts"
    - "backend/src/connection/mod.rs"
    - "backend/src/websocket/session.rs"
    - "backend/src/websocket/mod.rs"

key-decisions:
  - "Ceremony writes directly to tx channel (bypasses output_buffer) for timing control"
  - "Eager node assignment during ceremony with placeholder user info"
  - "on_connect returns bool for line-busy disconnect flow"
  - "Frontend connect prompt doubles as user gesture for AudioContext autoplay"

patterns-established:
  - "Direct tx channel writes with sleep delays for timed ceremony output"
  - "Session field tracking (node_id, disconnecting) for lifecycle management"
  - "Auth token sent as first WebSocket message for session resumption"

# Metrics
duration: 5min
completed: 2026-01-27
---

# Phase 2 Plan 3: Connection Ceremony and Modem Audio Summary

**Typewriter-paced connection ceremony with modem audio, ANSI splash screen, node assignment display, and CP437 line-busy rejection art**

## Performance

- **Duration:** 5 min
- **Started:** 2026-01-27T01:36:20Z
- **Completed:** 2026-01-27T01:41:48Z
- **Tasks:** 2
- **Files modified:** 8

## Accomplishments
- Modem handshake audio plays in browser after user keypress (Web Audio API with autoplay policy compliance)
- Connection ceremony delivers typewriter-paced text: modem dial simulation, protocol negotiation, node assignment
- ANSI art splash screen renders line-by-line with configurable baud-rate delay
- Line-busy users see CP437 box-drawing rejection message and connection closes cleanly
- Frontend sends auth token on WebSocket open (enables session resumption in Plan 05)
- Node assignment happens eagerly during ceremony; node released on disconnect

## Task Commits

Each task was committed atomically:

1. **Task 1: Frontend modem audio and connection ceremony trigger** - `e96102d` (feat)
2. **Task 2: Backend connection ceremony and line-busy logic** - `936f256` (feat)

## Files Created/Modified
- `frontend/public/audio/modem.mp3` - Placeholder modem handshake sound (replace with authentic audio)
- `frontend/src/audio.ts` - Web Audio API modem sound loader/player with user gesture handling
- `frontend/src/main.ts` - Connect prompt ("Press any key to dial in"), preloads audio, plays on keypress
- `frontend/src/websocket.ts` - Sends auth token on connect, removed hardcoded welcome text
- `backend/src/connection/ceremony.rs` - Connection ceremony, splash screen, line-busy logic with 5 tests
- `backend/src/connection/mod.rs` - Added ceremony module export
- `backend/src/websocket/session.rs` - Ceremony integration: node_id/disconnecting fields, on_connect returns bool, node release on disconnect
- `backend/src/websocket/mod.rs` - Handle ceremony result, clean disconnect on line-busy

## Decisions Made
- Ceremony writes directly to tx channel with tokio::time::sleep delays, bypassing the output_buffer -- needed for real-time typewriter pacing
- Eager node assignment during ceremony (with placeholder user_id=0, handle="connecting") ensures accurate node counts; updated with real user info after login
- on_connect() changed to return bool (true=continue, false=disconnect) for clean line-busy handling
- Frontend connect prompt serves double duty: atmospheric "pick up the phone" moment AND user gesture for AudioContext autoplay policy
- Modem MP3 is a minimal placeholder -- should be replaced with authentic modem handshake sound

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Connection ceremony complete, ready for login flow (Plan 05) to wire auth state machine
- Node assignment placeholder (user_id=0) will be updated when login identifies the user
- Auth token send on WebSocket connect is ready for session resumption in Plan 05
- Plan 02-04 (registration) runs in parallel and does not conflict with files modified here

---
*Phase: 02-authentication-connection*
*Completed: 2026-01-27*
