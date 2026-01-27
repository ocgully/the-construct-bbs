---
phase: 02-authentication-connection
plan: 06
subsystem: user-profile-goodbye
tags: [profile, goodbye, session-time, ANSI-art, box-drawing, stats, NO-CARRIER]
depends_on:
  requires: ["02-05"]
  provides: ["profile-card", "goodbye-sequence", "session-time-tracking", "format-duration-minutes"]
  affects: ["03-*", "04-*"]
tech_stack:
  added: []
  patterns: ["ANSI art card rendering", "session elapsed time via Instant", "clean vs unclean disconnect"]
key_files:
  created:
    - backend/src/services/profile.rs
    - backend/src/services/goodbye.rs
  modified:
    - backend/src/services/mod.rs
    - backend/src/websocket/session.rs
decisions:
  - id: "02-06-01"
    decision: "Profile card and goodbye screen use CP437 double-line box-drawing with CGA colors"
    rationale: "Matches BBS aesthetic established in earlier plans; consistent visual language"
  - id: "02-06-02"
    decision: "Session time calculated from Instant::now() at login, stored as minutes"
    rationale: "Instant is monotonic and immune to system clock changes; minute granularity matches DB schema"
  - id: "02-06-03"
    decision: "Clean quit sends logout JSON before goodbye screen, sets disconnecting flag after 3s delay"
    rationale: "Frontend clears localStorage token immediately; user reads goodbye screen before WebSocket closes"
  - id: "02-06-04"
    decision: "Unclean disconnect saves session time without goodbye screen"
    rationale: "User already gone; still need accurate stats and node release"
metrics:
  duration: "4 min"
  completed: "2026-01-27"
---

# Phase 2 Plan 6: User Profile Card and Goodbye Sequence Summary

ANSI art profile card with CP437 box-drawing displaying user identity, stats, and signature; goodbye sequence with session time tracking, NO CARRIER modem disconnect, and clean/unclean disconnect handling.

## What Was Built

### Task 1: User Profile ANSI Art Card
- **profile.rs**: `render_profile_card()` renders 80-char wide ANSI card with double-line borders
  - USER PROFILE section: handle, real name, location, member since, last on
  - STATISTICS section: total calls, messages sent, time online, games played
  - SIGNATURE section: up to 3 lines of user signature (may contain ANSI codes)
  - Colors: LightCyan borders, Yellow section headers, White values, LightMagenta/LightGreen badge
  - Optional fields show "(not set)" when None; empty signature shows "(none)"
- **format_duration_minutes()**: Converts minutes to human-readable "Xh Ym" format (reused by goodbye)
- **format_date()/format_datetime()**: Parse ISO datetime strings to "Month Day, Year at H:MM AM/PM"
- **render_profile_edit_menu()**: Prep for future profile editing (not wired up yet)
- 15 unit tests covering formatting and rendering

### Task 2: Goodbye Sequence with Session Stats
- **goodbye.rs**: `render_goodbye()` renders farewell ANSI card with session stats
  - Centered "Thanks for calling, {handle}!" in Yellow
  - Session Time, Total Calls, Total Time stats
  - "Call again soon! The Construct awaits..." in LightGreen
  - "NO CARRIER" in LightRed bold (classic modem disconnect)
- **session.rs quit flow**: Full goodbye sequence on clean quit:
  1. Send `{ type: "logout" }` JSON to frontend (clears localStorage token)
  2. Calculate session_minutes from login_time Instant
  3. Call update_user_time() to add session minutes to DB
  4. Fetch updated user stats for goodbye screen
  5. Delete session token from DB
  6. Send rendered goodbye screen
  7. Sleep 3 seconds (user reads goodbye)
  8. Set disconnecting = true
- **session.rs on_disconnect**: Updated for unclean disconnects:
  - Calculates and saves elapsed session time to DB
  - Deletes session token
  - Releases node
  - No goodbye screen (user already gone)
- 8 unit tests for goodbye rendering

## Deviations from Plan

None -- plan executed exactly as written.

## Verification Results

1. `cargo check` passes (no errors, only pre-existing warnings)
2. `cargo test` passes: 144 tests (136 existing + 8 goodbye tests)
3. Profile card renders with all fields, handles optional fields gracefully
4. Goodbye screen shows session time, total calls, total time, NO CARRIER
5. Session time tracking works for both clean and unclean disconnects
6. Frontend already handles logout JSON (implemented in 02-05)

## Test Coverage

| Area | Tests Added | Total |
|------|-------------|-------|
| profile.rs | 15 (formatting + rendering) | 15 |
| goodbye.rs | 8 (rendering + content) | 8 |
| **Total new** | **23** | **144 total** |

## Files Changed

| File | Action | Lines | Purpose |
|------|--------|-------|---------|
| backend/src/services/profile.rs | Created | +472 | Profile card ANSI art, formatting helpers |
| backend/src/services/goodbye.rs | Created | +213 | Goodbye screen ANSI art |
| backend/src/services/mod.rs | Modified | +2 | Added profile and goodbye modules |
| backend/src/websocket/session.rs | Modified | +60/-14 | Goodbye sequence on quit, time tracking on disconnect |

## Next Phase Readiness

Phase 2 authentication is now fully complete with all 6 plans:
- 02-01 through 02-05: DB, auth core, ceremony, registration, login
- 02-06: Profile display and goodbye sequence

The profile card and goodbye sequence provide closure to the auth lifecycle. Phase 3 (Message Boards) can proceed with full user identity and session management in place.
