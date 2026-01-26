# Domain Pitfalls: Web-Based BBS

**Domain:** Web-based retro BBS (xterm.js + Rust backend)
**Project:** The Construct
**Researched:** 2026-01-26
**Confidence:** MEDIUM (based on training knowledge, WebSearch unavailable)

## Critical Pitfalls

Mistakes that cause rewrites or major issues.

### Pitfall 1: ANSI Art Character Set Mismatch (CP437 vs Unicode)

**What goes wrong:**
ANSI art created for DOS BBSes uses CP437 (Code Page 437) encoding with box-drawing characters and extended ASCII. Modern browsers/terminals use Unicode (UTF-8). Direct rendering causes broken art—lines don't connect, characters appear as question marks or replacement glyphs.

**Why it happens:**
- Developers assume ANSI files are "just text" and serve them as UTF-8
- xterm.js defaults to Unicode without CP437 font mapping
- Box-drawing characters (0xB0-0xDF in CP437) map to different Unicode codepoints
- Font files don't include proper CP437 glyphs or use wrong Unicode mappings

**Consequences:**
- All classic ANSI art appears broken
- User-created art in editors like TheDraw won't display correctly
- Community loses trust ("they don't understand BBS culture")

**Prevention:**
1. Use a CP437-to-Unicode mapping table for ANSI parsing
2. Load a proper CP437-compatible font (like "Perfect DOS VGA 437")
3. Convert CP437 byte values to correct Unicode codepoints before rendering
4. Test with actual BBS ANSI files (not modern ASCII art)
5. Consider a client-side ANSI parser that handles CP437 explicitly

**Detection:**
- Warning signs: Box-drawing characters render as broken/misaligned
- Test case: Render a classic ANSI logo with ═══ and ║ characters
- Validator: Load ANSI files from 16colo.rs archive

**Phase impact:** Must be addressed in Phase 1 (terminal rendering foundation)

---

### Pitfall 2: WebSocket Escape Sequence Splitting

**What goes wrong:**
Terminal escape sequences (ANSI codes, cursor movements) get split across WebSocket message boundaries. A sequence like `\x1b[31m` (set red color) arrives as two messages: `\x1b[31` and `m`. xterm.js processes the incomplete sequence, resulting in garbled output, wrong colors, or cursor in wrong position.

**Why it happens:**
- WebSocket frame boundaries are network-dependent (TCP packet splits, Nagle's algorithm)
- Developers send terminal output directly without buffering/framing
- Rust async code writes to WebSocket as soon as data is available
- High-speed output (like scrolling file listings) maximizes split probability

**Consequences:**
- Random color changes mid-screen
- Cursor jumps to wrong positions
- ANSI art renders partially or with artifacts
- Intermittent bugs that are hard to reproduce

**Prevention:**
1. **Server-side buffering:** Accumulate escape sequences in complete units before sending
2. **Framing protocol:** Wrap terminal data in length-prefixed messages
3. **Client-side parser state:** Use xterm.js parser that handles incomplete sequences
4. **Flush boundaries:** Only send on complete lines or timeout (16ms for 60fps)
5. Use a state machine to detect incomplete escape sequences and hold them

**Detection:**
- Warning signs: Colors/formatting occasionally wrong, especially on slow connections
- Test case: Send rapid terminal output over throttled WebSocket (simulate 3G)
- Reproducer: cat a large ANSI file and watch for artifacts

**Phase impact:** Must be addressed in Phase 1 (WebSocket terminal I/O layer)

---

### Pitfall 3: Mobile Virtual Keyboard Blocking Terminal

**What goes wrong:**
On mobile, the virtual keyboard covers 50%+ of the screen when focused. User can't see what they're typing or the terminal output above. Scrolling is broken because keyboard resize doesn't trigger proper viewport recalculation. Copy/paste doesn't work because mobile browsers have different clipboard APIs.

**Why it happens:**
- xterm.js wasn't originally designed for mobile
- Mobile browsers resize viewport when keyboard opens, but xterm.js doesn't reflow
- Touch events don't map cleanly to terminal focus/selection
- iOS Safari and Android Chrome have different keyboard behaviors
- Developers test on desktop and assume "it's responsive"

**Consequences:**
- BBS is unusable on phones (50%+ of potential users)
- Can't play door games on mobile
- Community complains about "mobile support" being broken
- High bounce rate from mobile users

**Prevention:**
1. **Keyboard detection:** Listen for `visualViewport` resize, reposition terminal
2. **Input overlay:** Use a floating input field above keyboard, mirror to terminal
3. **Hybrid mode:** Detect mobile and switch to a different UI (chat-style input box)
4. **Touch handlers:** Implement custom touch selection for xterm.js
5. **Test on real devices:** iOS Safari and Android Chrome, not just emulators
6. Consider a "mobile mode" that trades terminal authenticity for usability

**Detection:**
- Warning signs: No mobile testing in CI/dev workflow
- Test case: Open BBS on iPhone Safari, try to type in a door game
- Validator: Can a user complete a full chat session on mobile?

**Phase impact:** Must have strategy by Phase 2 (before public release)

---

### Pitfall 4: SQLite Write Contention in Multiplayer Games

**What goes wrong:**
Multiple users playing door games simultaneously cause SQLite write lock contention. Games freeze waiting for database, transactions fail with `SQLITE_BUSY`, or worse—data corruption if not handled properly. Turn-based games lose state when writes conflict.

**Why it happens:**
- SQLite only allows one writer at a time (WAL mode helps but doesn't eliminate)
- Developers treat SQLite like PostgreSQL (multi-writer safe)
- Game loops write state on every action (overkill for database)
- Long-running transactions hold write locks during user input
- Rust's async + SQLite blocking I/O creates task starvation

**Consequences:**
- Games timeout or crash under concurrent load
- User progress lost due to failed transactions
- Poor multiplayer experience (lag spikes when DB locked)
- Difficult to debug (timing-dependent, doesn't happen in dev)

**Prevention:**
1. **WAL mode:** Enable Write-Ahead Logging (`PRAGMA journal_mode=WAL`)
2. **Connection pooling:** Use r2d2 or deadpool, limit pool size to 1 writer + N readers
3. **Batch writes:** Accumulate game state in memory, flush periodically (every 5-10 seconds)
4. **Optimistic locking:** Use version numbers, retry on conflict
5. **Actor model:** Single task owns DB writes, other tasks send messages via channel
6. **Read replicas:** Read game state from in-memory cache, only write critical state
7. Consider message queue (like crossbeam channel) to serialize writes

**Detection:**
- Warning signs: `SQLITE_BUSY` errors in logs
- Test case: 10 users playing door games simultaneously
- Reproducer: Stress test with concurrent writes to same game state

**Phase impact:** Must be designed in Phase 1 (data layer architecture)

---

### Pitfall 5: Rust Async Runtime Blocking in Game Loops

**What goes wrong:**
Game loop logic blocks the async runtime (Tokio) with expensive computations or blocking I/O. This freezes all users' connections because Tokio's work-stealing scheduler is starved. Door games that do pathfinding, AI, or database queries inline cause entire BBS to lag.

**Why it happens:**
- Developers put synchronous/CPU-heavy code in async functions
- SQLite queries block the runtime (rusqlite is not async-native)
- Game AI or simulation runs for 10ms+ without yielding
- Didn't understand Tokio's cooperative multitasking model

**Consequences:**
- All users experience lag when one game does heavy computation
- WebSocket ping timeouts, disconnections
- Poor scalability (can't handle 50+ concurrent users)
- Intermittent freezes that are hard to diagnose

**Prevention:**
1. **spawn_blocking:** Use `tokio::task::spawn_blocking` for any blocking I/O (SQLite, file I/O)
2. **Compute budget:** Limit game loop iterations, yield with `tokio::task::yield_now()` every 1ms
3. **Separate runtime:** Run door games on a dedicated Tokio runtime or thread pool
4. **Background tasks:** Offload AI/simulation to background tasks, communicate via channels
5. **Profiling:** Use tokio-console to detect tasks blocking the runtime
6. **Never:** Call `.wait()`, `.join()`, or thread sleeps in async context

**Detection:**
- Warning signs: WebSocket latency spikes, ping timeouts
- Test case: Run CPU-intensive door game, measure impact on other connections
- Tool: tokio-console shows task poll times

**Phase impact:** Must be enforced by Phase 2 (game engine architecture)

---

### Pitfall 6: Over-Engineered Plugin System

**What goes wrong:**
Developers create a complex trait-based plugin system with dynamic loading, trait objects, async traits, and heavy abstraction. Results in:
- Confusing API that's hard to implement
- Runtime overhead from dynamic dispatch
- Compile-time explosion (trait bounds infect everything)
- Plugins can't be easily added without framework expertise

**Why it happens:**
- Desire to "do it right" with maximum flexibility
- Copy patterns from large frameworks (not appropriate for BBS scale)
- Premature generalization without concrete use cases
- Rust trait system encourages over-abstraction

**Consequences:**
- Plugin development is slow and frustrating
- Few community contributions (API too complex)
- Performance overhead from unnecessary abstraction
- Maintenance burden (change one trait, recompile everything)

**Prevention:**
1. **Start simple:** Hardcode first 2-3 door games, extract commonality after
2. **Concrete types:** Use enums for game types, not trait objects
3. **Static dispatch:** Compile plugins in, not dynamic loading (for v1)
4. **Small API surface:** Minimal trait with 3-5 methods max
5. **Examples first:** Write 2 example games before designing plugin API
6. **Defer:** Don't build plugin system until you have 3+ concrete games

**Detection:**
- Warning signs: Plugin trait has >8 methods or async trait objects
- Code smell: `Box<dyn GamePlugin + Send + Sync + 'static>`
- Test: Can a Rust beginner implement a simple plugin in <2 hours?

**Phase impact:** Don't tackle until Phase 3+, after core games proven

---

### Pitfall 7: Time Limit Enforcement Race Conditions

**What goes wrong:**
User's session time limit expires mid-game or mid-message. Poor handling causes:
- Lost game progress (transaction aborted)
- Half-written emails vanish
- WebSocket forcibly closed without cleanup
- User confusion ("I got kicked mid-sentence")

**Why it happens:**
- Time limit checked only on command boundaries, not during long operations
- No grace period for in-progress actions
- Async cancellation doesn't run cleanup code
- Database transaction not committed before disconnect

**Consequences:**
- User frustration and data loss
- Bug reports: "BBS ate my message"
- Unfair game outcomes (kicked during boss fight)

**Prevention:**
1. **Grace warnings:** Warn at 5min, 1min remaining
2. **Operation completion:** Let current operation finish before enforcing limit
3. **Graceful shutdown:** Use `tokio::select!` with timeout, run cleanup on both paths
4. **Save state:** Auto-save game/message state every 30 seconds
5. **Resume capability:** Allow user to resume interrupted game on next login

**Detection:**
- Warning signs: No timeout handling in game loop
- Test case: Set 5-minute limit, start door game, wait for expiration
- Validator: Verify game state was saved

**Phase impact:** Phase 2 (time limit system)

---

## Moderate Pitfalls

Mistakes that cause delays or technical debt.

### Pitfall 8: ANSI Color Palette Assumptions

**What goes wrong:**
BBS assumes standard 16-color ANSI palette, but xterm.js can be themed. User's terminal theme makes text unreadable (e.g., light text on light background). Or developers hardcode colors without testing against different palettes.

**Prevention:**
- Ship a default CP437-era color palette (matching DOS colors)
- Allow theme override but validate contrast ratios
- Test with light and dark terminal themes
- Use semantic colors ("warning", "success") not hardcoded escape codes

**Phase impact:** Phase 1 (terminal theming)

---

### Pitfall 9: Browser Audio Autoplay Restrictions

**What goes wrong:**
Modem sounds, door game music, notification beeps don't play because browser blocks autoplay without user gesture. Kills retro atmosphere.

**Prevention:**
- Require user interaction before loading BBS (e.g., "Click to Connect" splash)
- Use Web Audio API with user-initiated AudioContext
- Fallback to visual indicators if audio blocked
- Test in Safari (strictest autoplay policy)

**Phase impact:** Phase 2 (audio/ambiance)

---

### Pitfall 10: Copy/Paste Character Encoding

**What goes wrong:**
User copies text from terminal, pastes elsewhere, gets mojibake. Or pastes Unicode emoji into terminal, crashes ANSI parser.

**Prevention:**
- Implement custom clipboard handlers
- Strip/convert non-CP437 characters on paste
- Provide "copy as plain text" option
- Sanitize pasted input before sending to backend

**Phase impact:** Phase 2 (terminal UX polish)

---

### Pitfall 11: Door Game State Staleness

**What goes wrong:**
Multiplayer door game shows stale state because client caches data. Player A kills monster, Player B still sees it alive. Leads to race conditions and invalid actions.

**Prevention:**
- Event-driven updates via WebSocket (push, don't poll)
- Optimistic UI with server reconciliation
- Version numbers on all game state
- Clear cache on game start

**Phase impact:** Phase 3 (multiplayer games)

---

### Pitfall 12: Mobile Text Size

**What goes wrong:**
Terminal font too small on mobile (authentic 80x25 DOS font = unreadable on phone). Users can't read text or tap precise locations.

**Prevention:**
- Responsive font sizing (larger on mobile)
- Reduce columns to 40-60 on small screens
- Touch target size for menu options (minimum 44x44 CSS pixels)
- Test on actual phones, not just browser DevTools

**Phase impact:** Phase 2 (mobile UX)

---

## Minor Pitfalls

Mistakes that cause annoyance but are fixable.

### Pitfall 13: Escape Sequence Injection

**What goes wrong:**
User enters malicious ANSI codes in chat/email, causing recipient's terminal to change colors, move cursor, or potentially execute terminal commands (rare in xterm.js but possible).

**Prevention:**
- Sanitize user input: strip all escape sequences except safe ones (colors)
- Whitelist allowed escape codes
- Use xterm.js security options
- Never echo raw user input to other users' terminals

**Phase impact:** Phase 1 (input sanitization)

---

### Pitfall 14: Session State Leak

**What goes wrong:**
User disconnects abruptly (close browser tab), session state not cleaned up. Memory leak, stale locks in database, user appears online forever.

**Prevention:**
- WebSocket close handlers with cleanup
- Heartbeat/ping to detect dead connections
- Timeout-based session cleanup (30 seconds without ping)
- Use RAII patterns in Rust (Drop trait)

**Phase impact:** Phase 1 (session management)

---

### Pitfall 15: Nostalgia-Driven Scope Creep

**What goes wrong:**
"Let's add door game X, Y, Z!" "We need FidoNet!" "Multi-node support!" Feature list explodes because of nostalgia, never ships.

**Prevention:**
- Define MVP strictly: 3 door games, basic email/chat, single node
- Defer FidoNet, multi-node, advanced features to Phase 4+
- Resist urge to clone every door game from memory
- Ship something playable, iterate

**Phase impact:** Discipline required throughout

---

### Pitfall 16: Line Ending Confusion (CRLF vs LF)

**What goes wrong:**
BBS sends LF-only line endings, xterm.js cursor doesn't return to column 0. Or sends CRLF, get double spacing. Mix of DOS/Unix conventions.

**Prevention:**
- Standardize on CRLF (`\r\n`) for terminal output
- Configure xterm.js for CRLF mode
- Test on Windows, Linux, macOS clients

**Phase impact:** Phase 1 (terminal output)

---

### Pitfall 17: Database Migration Strategy Missing

**What goes wrong:**
Schema changes during development break existing databases. No migration path from v1 to v2.

**Prevention:**
- Use migration framework (diesel, sqlx, or refinery)
- Version your schema from day 1
- Test migrations with production-like data

**Phase impact:** Phase 1 (before first users)

---

### Pitfall 18: WebSocket Backpressure Ignored

**What goes wrong:**
Server sends terminal data faster than client can render. WebSocket buffer fills up, causing memory bloat or dropped messages.

**Prevention:**
- Monitor WebSocket buffer size
- Implement backpressure: pause sending if buffer > threshold
- Use xterm.js write callbacks to know when rendering complete

**Phase impact:** Phase 2 (performance tuning)

---

### Pitfall 19: Game Balance Ignoring Time Limits

**What goes wrong:**
Door game designed for unlimited playtime, but BBS has 60-minute sessions. Players can't complete quests, progress feels grindy.

**Prevention:**
- Design games for 10-20 minute sessions
- Save progress between sessions
- Balance XP/rewards for short bursts
- Playtest with actual time limits enforced

**Phase impact:** Phase 3 (game design)

---

### Pitfall 20: Error Messages Break ANSI Layout

**What goes wrong:**
Error message printed to terminal mid-screen ruins ANSI layout. Message appears in wrong color or position.

**Prevention:**
- Reserve status line at top/bottom for messages
- Clear line before writing error
- Use consistent message format (color, position)
- Restore cursor position after message

**Phase impact:** Phase 1 (error handling UX)

---

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|-------------|---------------|------------|
| Phase 1: Terminal Foundation | CP437 encoding, escape sequence splitting | Address immediately, breaks everything else |
| Phase 1: Data Layer | SQLite concurrency model | Design actor pattern or write queue upfront |
| Phase 2: Mobile Support | Virtual keyboard blocking | Plan hybrid UI, don't assume desktop-only |
| Phase 2: Time Limits | Ungraceful timeout handling | Implement save/resume early |
| Phase 3: Door Games | Rust async blocking in game loops | Use spawn_blocking, dedicated runtime |
| Phase 3: Multiplayer | State staleness, race conditions | Event-driven architecture from start |
| Phase 4: Plugins | Over-engineering | Resist until you have 3+ concrete games |
| All Phases | Nostalgia scope creep | Strict MVP, defer retro features |

---

## Domain-Specific Anti-Patterns

### Anti-Pattern 1: "Pixel-Perfect DOS Emulation"

**What it is:** Trying to emulate DOS BBS behavior exactly, including bugs, quirks, and limitations.

**Why bad:**
- Wastes time on irrelevant details (e.g., FOSSIL driver emulation)
- Limits what you can improve (e.g., better mobile support breaks "authenticity")
- Nostalgia doesn't excuse bad UX

**Instead:** Capture the feel (ANSI art, door games, time limits) but improve UX where it doesn't hurt authenticity (mobile keyboard handling, better error messages).

---

### Anti-Pattern 2: "Everything In Traits"

**What it is:** Abstracting every system behind traits for "flexibility."

**Why bad:**
- Trait bounds infect entire codebase
- Compile times explode
- No concrete use cases to drive design

**Instead:** Use concrete types, extract traits only when you have 2+ implementations that share behavior.

---

### Anti-Pattern 3: "Polling Game State"

**What it is:** Clients poll server every N seconds for game updates.

**Why bad:**
- Latency (N seconds stale)
- Server load (all clients polling)
- Doesn't fit terminal model (server pushes output)

**Instead:** WebSocket push model—server sends updates immediately.

---

### Anti-Pattern 4: "Shared Mutable State"

**What it is:** Multiple async tasks accessing game state via `Arc<Mutex<GameState>>`.

**Why bad:**
- Lock contention under load
- Easy to deadlock
- Doesn't compose with async (blocking mutex in async)

**Instead:** Actor model (task owns state, others send messages) or message-passing via channels.

---

## Security Considerations

### Terminal Injection

**Risk:** User sends ANSI escape codes that affect other users' terminals.

**Mitigation:**
- Strip all escape codes from user input
- Whitelist safe codes (basic colors only)
- Never echo raw input to other sessions

**Phase:** Phase 1 (input handling)

---

### SQL Injection (Less Obvious in SQLite)

**Risk:** User input in door game commands gets interpolated into SQL.

**Mitigation:**
- Always use parameterized queries
- Never format strings into SQL
- Use query builder or ORM

**Phase:** Phase 1 (data layer)

---

### Resource Exhaustion

**Risk:** User floods WebSocket with messages, exhausts server resources.

**Mitigation:**
- Rate limiting per connection
- Message size limits
- Disconnect on abuse

**Phase:** Phase 2 (production hardening)

---

### Session Fixation

**Risk:** User can hijack another user's session.

**Mitigation:**
- Cryptographically random session IDs
- Regenerate session ID on login
- HTTP-only cookies (if using cookies)

**Phase:** Phase 1 (authentication)

---

## Testing Recommendations

| Pitfall Category | Test Strategy |
|------------------|---------------|
| ANSI rendering | Load real BBS ANSI files from archives (16colo.rs) |
| Mobile UX | Test on real iOS/Android devices, not emulators |
| SQLite concurrency | Stress test with 50+ concurrent door game sessions |
| WebSocket splitting | Throttle network in DevTools, send rapid output |
| Async blocking | Use tokio-console, measure task poll times |
| Time limits | Set 5-minute limit, verify graceful handling |

---

## Sources

**Confidence:** MEDIUM

This research draws from:
- xterm.js documentation and known mobile limitations (training knowledge)
- SQLite concurrency model and WAL mode (training knowledge)
- Rust async runtime patterns (Tokio documentation, training knowledge)
- BBS domain knowledge (CP437 encoding, ANSI art standards, door game mechanics)
- Web platform constraints (autoplay policies, mobile keyboard behavior)

**Limitation:** WebSearch unavailable—could not verify:
- Latest xterm.js mobile keyboard improvements (2025-2026)
- Recent Rust async ecosystem best practices
- Current BBS revival project post-mortems

**Verification needed:**
- xterm.js mobile handling (check official docs/issues)
- Tokio game loop patterns (check Tokio docs/examples)
- CP437 font recommendations (check modern terminal emulator choices)

**Next steps:**
- Validate with Context7 when available for xterm.js, Tokio specifics
- Check official xterm.js GitHub issues for mobile keyboard solutions
- Review rusqlite/sqlx documentation for concurrency recommendations
