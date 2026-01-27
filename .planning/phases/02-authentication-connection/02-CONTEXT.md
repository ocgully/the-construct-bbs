# Phase 2: Authentication & Connection - Context

**Gathered:** 2026-01-26
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can create accounts, log in securely, and experience an authentic BBS connection sequence. This phase delivers: modem handshake audio, connection ceremony with terminal simulation, ANSI splash screen, user registration/login, session management, node-based scarcity, user profiles with stats, and the goodbye sequence. Navigation menus and time limits are separate phases.

</domain>

<decisions>
## Implementation Decisions

### Connection Ceremony
- Shortened modem handshake audio (~3-5 seconds) plays through browser speakers on every visit
- Real audio file (WAV/MP3), not text-only simulation
- BBS-style text simulation follows audio: "Connecting to The Construct...", "Negotiating...", "ANSI Detected", etc. (not raw AT commands)
- Typewriter text pacing — each line appears one at a time with slight delays simulating serial output
- Full terminal simulation shown: CONNECT speed, ANSI detection, node assignment
- Real node assignment shown: "Connected to Node 3 of 16" with actual live counts
- ANSI art splash screen with animated reveal (line-by-line rendering simulating baud speed)
- Baud simulation speed is sysop-configurable, with per-service override capability (games may want different render speeds)
- Connection ceremony skippability is sysop-configurable (press any key to skip, or mandatory)
- Node count always shown on login screen: "Node X of Y — Z lines available"

### Account Flow
- Registration flow: Claude's discretion on exact prompts/UX
- Registration fields: handle, password, email (required) — minimal, everything else set in profile later
- Handle rules: BBS-style loose — alphanumeric + spaces, 3-20 chars (expressive handles like "Dark Angel")
- Moderate profanity filter on handles with leetspeak variant detection
- Handles are user-changeable (must remain unique, profanity filter applies)
- Sysop can also rename users via admin panel
- Post-registration behavior: sysop-configurable (straight to menu / welcome ANSI art / new user bulletin), all three options available
- Email required at registration for verification and password reset
- Email verification required before account access — uses 6-digit code entered in terminal (not email link)
- Password reset also uses 6-digit code flow (consistent with verification)
- Password reset option surfaces after 1+ failed login attempts (not shown by default)
- Password entry shows asterisks (not silent/no-echo)
- Failed login lockout threshold: sysop-configurable
- Email provider: Claude's discretion (pick simplest for the project)
- Verification code expiry: sysop-configurable
- Terms/rules screen during registration: sysop-configurable (enable/disable, custom text)
- No guest access — must register to see anything
- Open registration (no invite codes)
- Self-service account deletion with password confirmation
- SQLite for user storage

### Login Experience
- Quick and classic: "Enter your handle:" → "Password:" → "Welcome back, DarkAngel! Last login: Jan 25" → main menu
- After splash art, user lands at login prompt (must authenticate before seeing main menu)
- Last callers display: sysop-configurable (show on login vs menu-only, default: show on login)
- Login screen shows BBS name + static sysop-set tagline
- Returning user flow: login → last callers (if enabled) → main menu

### Scarcity & Limits
- Max concurrent users: sysop-configurable, default 16 nodes
- Line busy behavior: sysop-configurable (busy signal + disconnect OR queue with position), default: busy signal + disconnect
- Queue max size (when queue enabled): sysop-configurable
- Reconnect window (grace period after disconnect): sysop-configurable
- Session persistence: Claude picks token approach (cookie vs localStorage)
- Reconnect returns user to their active service/game (seamless resume)
- Duplicate session policy: sysop-configurable (kick old session OR block new session), default: block new session
- Idle timeout: sysop-configurable, default 15 minutes with warnings
- Idle warning style: sysop-configurable (terminal bell + text OR text only), default: bell + text

### Goodbye Sequence
- Full goodbye sequence: session stats (time online, messages, games played), goodbye ANSI art, then disconnect

### User Identity
- 3 user levels: Guest (pre-login only), User, Sysop
- Multiple users can be Sysop (no Co-Sysop tier)
- Sysop assignment: config-based (handles listed in config file, server restart to change)
- Extended profile: handle, real name (optional), location (free text), join date, last login, total calls, time online, messages sent, games played, signature, bio
- Location is free text — "Newark, DE", "The Matrix", "Cyberspace" all valid
- ANSI signatures: multi-line with ANSI color codes, max 3 lines, 80 chars wide
- Profile display: ANSI art card format (box-drawn with colors, like classic BBS user info screen)
- Comprehensive stats tracking: logins, total time online, messages sent, games played, achievements

### Claude's Discretion
- Registration flow UX (prompt sequence, wording)
- Email provider selection
- Session token implementation (cookie vs localStorage)
- Reconnect grace period default value
- Modem handshake audio source/file
- Splash art ANSI design
- Default baud simulation speed
- Profanity filter word list and implementation approach

</decisions>

<specifics>
## Specific Ideas

- Connection should feel like "dialing in" every time — the ceremony is core to the BBS experience
- Per-service baud rate override: games should be able to set their own rendering speed independent of the BBS default
- The 6-digit code flow (same for verification and password reset) keeps everything in-terminal — no switching to browser for link clicks
- Node display on login builds scarcity awareness: "You got in, but there are only X spots left"
- Sysop configuration is a major theme — many behaviors are sysop-toggleable to let operators customize their BBS personality

</specifics>

<deferred>
## Deferred Ideas

- Firebase analytics integration for user stats — future phase or infrastructure enhancement
- Invite code registration mode — could add later as sysop option
- OAuth/social login — explicitly out of scope per requirements (breaks immersion)

</deferred>

---

*Phase: 02-authentication-connection*
*Context gathered: 2026-01-26*
