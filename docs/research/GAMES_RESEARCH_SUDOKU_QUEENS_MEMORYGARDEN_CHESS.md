# Game Research: Sudoku, Queens, Memory Garden, Chess

**Research Date**: 2026-01-30
**Reference Implementation**: `backend/src/games/grand_theft_meth/`
**Architecture Document**: `docs/GAME_ARCHITECTURE.md`
**Specifications Document**: `docs/bbs_game_specifications.md`

---

## Executive Summary

This document identifies implementation questions and assumptions for four games/features:

1. **Sudoku** - Daily 9x9 puzzle with streak tracking
2. **Queens** - Daily N-Queens variant with colored regions
3. **Memory Garden** - Social journaling feature (main menu)
4. **Chess** - Async multiplayer with ELO ratings

All games except Chess follow the "Daily Puzzle" pattern described in the architecture doc. Chess follows the "Async Multiplayer" pattern. Memory Garden is a BBS feature, not a door game.

---

## Game 1: Sudoku

### Summary

Classic 9x9 Sudoku puzzle refreshing daily at midnight UTC. All players receive the same date-seeded puzzle. Features difficulty selection (Easy/Medium/Hard), pencil marks for candidates, timer tracking, and streak counting for consecutive days completed. Similar to LinkedIn's daily puzzle games.

### Key Implementation Questions

#### 1. Puzzle Seeding Strategy
**Question**: How should the daily puzzle seed be constructed?
- Option A: Date only (e.g., `2026-01-30`) - same puzzle for all difficulties
- Option B: Date + difficulty (e.g., `2026-01-30-hard`) - different puzzle per difficulty
- Option C: Date + user-selected difficulty at start of day (locked for 24h)

**Impact**: Option A means fastest-time leaderboards must be segmented by difficulty. Option B/C affects whether users can try all difficulties.

#### 2. Difficulty Selection Timing
**Question**: When does the user choose difficulty?
- Option A: Once per day (first attempt locks it)
- Option B: Each session (can play easy, then try hard)
- Option C: Rotating system (Mon=Easy, Tue=Medium, Wed=Hard, repeat)

**Impact**: Affects fairness of streak tracking across difficulties.

#### 3. Streak Break Policy
**Question**: What happens when a user misses a day?
- Option A: Reset to 0 (strict - encourages daily play)
- Option B: Reset to 0, but track "longest streak" separately
- Option C: Allow 1 "grace day" before reset
- Option D: Streak pauses on miss, resumes when they return

**Impact**: Strict resets may frustrate casual users; lenient rules reduce engagement pressure.

#### 4. Completion Requirements
**Question**: What counts as "completing" a puzzle for streak purposes?
- Option A: Solve correctly (no errors at submission)
- Option B: Solve correctly OR attempt and fail
- Option C: Simply open the puzzle that day

**Impact**: If only correct solutions count, users could break streaks due to difficulty spikes.

#### 5. Pencil Marks Storage
**Question**: Should pencil marks persist across sessions?
- Option A: Yes - save partial state with candidates
- Option B: No - only save placed numbers, candidates are transient

**Impact**: Persistence requires additional state storage; transient is simpler.

#### 6. Puzzle Generation Library
**Question**: Should we use an existing Sudoku library or build from scratch?
- Option A: Use existing crate (e.g., `sudoku` crate for Rust)
- Option B: Build custom generator for full control

**Impact**: Existing libraries speed development but may not support deterministic seeding.

### Assumptions (If Not Clarified)

1. Seed = Date + Difficulty (Option B) - allows playing all three each day
2. Difficulty selection per session (Option B) - user can attempt any difficulty
3. Streak resets on miss, but tracks longest separately (Option B)
4. Only correct solutions count for streaks (Option A)
5. Pencil marks are transient/session-only (Option B)
6. Use existing Sudoku crate if it supports deterministic seeds; build custom otherwise

### Conflicts with Architecture Doc

- **None identified**. The architecture explicitly lists "Daily Puzzle" as a game type with date-seeded puzzles and streak tracking.
- Data model aligns: `sudoku_daily`, `sudoku_players`, `sudoku_completions` tables match the separate-database-per-game pattern.

### Complexity Estimate

**Simple** (as specified) - 2-3 days implementation
- Core puzzle validation: 0.5 day
- ASCII rendering with pencil marks: 0.5 day
- Streak/completion tracking: 0.5 day
- Database schema + service layer: 0.5 day
- Testing: 0.5-1 day

---

## Game 2: Queens

### Summary

N-Queens puzzle with colored regions, daily refresh. Players place N queens on NxN board where: (1) no two queens share row/column/diagonal, and (2) each colored region contains exactly one queen. Similar to LinkedIn's Queens game. Features streak tracking and timer.

### Key Implementation Questions

#### 1. Board Size Variation
**Question**: How does board size vary?
- Option A: Fixed 8x8 daily
- Option B: Difficulty-based (5x5=Easy, 6x6=Medium, 8x8=Hard)
- Option C: Random 5-8 daily (same for all users)
- Option D: Gradually increases over consecutive days

**Impact**: Larger boards are exponentially harder; affects casual accessibility.

#### 2. Region Generation Algorithm
**Question**: How to generate valid contiguous regions with unique solutions?
- Option A: Pre-generate and store a library of valid puzzles
- Option B: Procedural generation with backtracking validation
- Option C: Simple geometric patterns (rows, columns, diagonals split into regions)

**Impact**: Option B is complex but ensures variety; Option A requires upfront work; Option C may be too easy.

#### 3. Hint System Details
**Question**: How should hints work?
- Option A: Reveal one correct queen placement (deduct points)
- Option B: Highlight cells that are definitely wrong
- Option C: Show which regions have no valid placements for current state
- Option D: No hints - pure puzzle

**Impact**: Hints affect leaderboard fairness (hints_used tracking exists in spec).

#### 4. Streak vs Score Priority
**Question**: For leaderboard, what matters more?
- Option A: Streak length primary, time secondary
- Option B: Fastest time primary, streak as separate metric
- Option C: Combined score (streak * speed_bonus)

**Impact**: Determines what players optimize for.

#### 5. Color Display in ASCII
**Question**: How to show colored regions in 16-color terminal?
- Option A: Single-character color codes (R=Red, B=Blue, etc.)
- Option B: Background colors using ANSI
- Option C: Different ASCII patterns/symbols per region

**Impact**: Background colors may not render well on all terminals; symbols are safer.

#### 6. Multiple Daily Attempts
**Question**: Can users retry after solving?
- Option A: One attempt per day (first solve is final)
- Option B: Unlimited attempts, best time counts
- Option C: Unlimited attempts, first completion time counts

**Impact**: Affects strategy (careful vs. speed) and server load.

### Assumptions (If Not Clarified)

1. Difficulty-based size (5x5/6x6/8x8) - matches Sudoku pattern (Option B)
2. Procedural generation with validation, cached for the day (Option B)
3. Hints reveal one placement, tracked in completions (Option A)
4. Fastest time is primary metric; streak tracked separately (Option B)
5. Use letter codes (R/B/G/Y/P) with ANSI foreground colors (Option A)
6. Unlimited attempts, first completion counts for streak, best time for leaderboard

### Conflicts with Architecture Doc

- **None identified**. Follows Daily Puzzle pattern exactly.
- Spec includes `hints_used` in completions table, confirming hint feature is expected.

### Complexity Estimate

**Simple** (as specified) - 2-3 days implementation
- Region generation algorithm: 1 day (most complex part)
- Placement validation: 0.5 day
- ASCII rendering: 0.5 day
- Streak/timer tracking: 0.5 day
- Testing: 0.5 day

---

## Game 3: Memory Garden

### Summary

A communal digital garden on the BBS main menu (NOT a door game) where users leave daily memories (280-character limit). Features random sampling of recent memories on entry, paginated exploration, and system-generated milestone memories for BBS statistics (user counts, session counts, total usage time).

### Key Implementation Questions

#### 1. Menu Placement
**Question**: Where does Memory Garden appear in the BBS?
- Option A: Main menu item (alongside Games, Mail, etc.)
- Option B: Sub-menu under a "Community" section
- Option C: Accessible from user profile
- Option D: Easter egg / hidden feature

**Impact**: Main menu visibility drives engagement; hidden reduces discovery.

#### 2. Entry Screen Display
**Question**: What shows when entering the garden?
- Option A: Random sample of 5-10 recent memories
- Option B: Most recent memories (newest first)
- Option C: Random sample weighted toward newer memories
- Option D: Themed selection (e.g., memories from same day in previous years)

**Impact**: Random sampling creates discovery; chronological is predictable.

#### 3. User Daily Memory Timing
**Question**: What defines "1 per user per day"?
- Option A: UTC midnight reset
- Option B: Server local time (Eastern per db timestamps)
- Option C: User's local timezone
- Option D: Rolling 24-hour window

**Impact**: UTC is simplest; user timezone is fairest but requires user timezone storage.

#### 4. Memory Editing/Deletion
**Question**: Can users edit or delete their memories?
- Option A: No editing or deletion (permanent record)
- Option B: Can delete within 1 hour of posting
- Option C: Can always delete own memories
- Option D: Can edit within 1 hour, never delete

**Impact**: Permanent adds weight to submissions; editing allows fixing typos.

#### 5. Content Moderation
**Question**: How to handle inappropriate content?
- Option A: Sysop manual review queue
- Option B: Word filter/blocklist
- Option C: Community flagging system
- Option D: Trust new users less (approval required)

**Impact**: BBS context suggests manual moderation is acceptable for this scale.

#### 6. Starter Memory
**Question**: The spec mentions "Starter memory: 'I was born' dated 1/25/2026". Is this literal?
- Option A: Yes, exact seed memory
- Option B: Should be configurable per BBS instance
- Option C: Should include BBS name

**Impact**: Minor, but affects initial impression.

#### 7. Database Location
**Question**: Where should Memory Garden data live?
- Option A: Main bbs.db (it's a BBS feature, not a door game)
- Option B: Separate memory_garden.db (follows game pattern)

**Impact**: Spec data model suggests separate, but architecture doc says games get separate DBs; this isn't a game.

### Assumptions (If Not Clarified)

1. Main menu item for visibility (Option A)
2. Random sample of 5-10 recent memories on entry (Option A)
3. UTC midnight reset for daily limits (Option A)
4. No editing/deletion - memories are permanent (Option A)
5. Sysop manual moderation (appropriate for BBS scale) (Option A)
6. Configurable starter memory (Option B)
7. Lives in main bbs.db since it's a BBS feature, not a game (Option A)

### Conflicts with Architecture Doc

- **Potential Conflict**: Architecture doc is for door games. Memory Garden is explicitly a "BBS FEATURE" not a game.
- Does NOT need: GameFlow, GameScreen, GameAction, SENTINEL
- DOES need: Integration with main BBS menu system, shared bbs.db access
- Recommend: Treat as a BBS module, not a game module

### Complexity Estimate

**Simple** (as specified) - 1-2 days implementation
- Database schema (memories, bbs_stats): 0.25 day
- Memory submission/viewing: 0.5 day
- Pagination/exploration: 0.5 day
- Milestone logic: 0.25 day
- ASCII garden aesthetic: 0.25 day
- Testing: 0.25 day

---

## Game 4: Chess

### Summary

Full chess implementation with async multiplayer. Players can start games (white moves first, waits for opponent), join open games, or play against matched opponents. Features ELO rating system, 3-day move timeout, game history/review, and all standard chess rules including castling, en passant, and pawn promotion.

### Key Implementation Questions

#### 1. Opponent Move Notification
**Question**: How do we notify users when opponents move?
- Option A: Notification appears when user logs into BBS
- Option B: In-game notification when in chess lobby
- Option C: BBS-wide notification system (like new mail indicator)
- Option D: Email notification (requires email integration)
- Option E: Polling - user must check

**Impact**: Real-time notification is ideal for engagement; polling frustrates users.

#### 2. Matchmaking System
**Question**: How are opponents matched?
- Option A: Open games list (first-come-first-served)
- Option B: ELO-based matchmaking (similar ratings)
- Option C: Challenge specific users by handle
- Option D: All of the above

**Impact**: ELO matching is fairest but may have queue issues with small user base.

#### 3. Concurrent Game Limit
**Question**: How many simultaneous games per user?
- Option A: Unlimited
- Option B: Fixed limit (e.g., 5 games)
- Option C: Configurable by sysop

**Impact**: Unlimited may cause neglected games and timeouts; limits frustrate active players.

#### 4. Timeout Handling
**Question**: Details of 3-day timeout?
- Option A: Background job checks all games periodically
- Option B: Check on next move attempt
- Option C: Check when either player enters chess lobby

**Impact**: Background job is cleanest but requires cron/scheduler. On-access is simpler.

#### 5. Rating System Bootstrap
**Question**: Starting ELO and adjustment formula?
- Option A: Standard 1200 starting, K-factor 32 for new players
- Option B: Provisional period (first 10 games have higher K-factor)
- Option C: Simple +/- 10-30 points based on opponent rating

**Impact**: Standard ELO is well-understood; provisional helps new players calibrate faster.

#### 6. Move Validation Library
**Question**: Use existing chess library or build from scratch?
- Option A: Use `chess` or `shakmaty` crate
- Option B: Build custom for full control

**Impact**: Chess rules are complex; existing libraries are well-tested.

#### 7. Game Abandonment
**Question**: What happens to unfinished games if a user is deleted/banned?
- Option A: Opponent wins by forfeit
- Option B: Game is deleted (no rating change)
- Option C: Game frozen indefinitely

**Impact**: Affects rating integrity and cleanup.

#### 8. Draw Offers
**Question**: How to handle draw offers?
- Option A: Draw button, opponent sees offer next turn
- Option B: Mutual agreement (both must offer in same turn)
- Option C: Only automatic draws (stalemate, 50-move, repetition)

**Impact**: Draw offers add UI complexity but are standard in chess.

#### 9. Board Orientation
**Question**: Which way does the board display?
- Option A: Always white at bottom
- Option B: Player's color at bottom
- Option C: User preference

**Impact**: Player's color at bottom is standard for playability.

### Assumptions (If Not Clarified)

1. BBS login notification for opponent moves (Option C - like mail)
2. All three matchmaking options (open games, ELO match, direct challenge) (Option D)
3. Limit to 10 concurrent games per user (reasonable middle ground)
4. Background job checks timeouts hourly (Option A)
5. Standard ELO 1200 start with K=32, provisional first 10 games (Option B)
6. Use `shakmaty` crate for move validation (well-maintained Rust library)
7. Opponent wins by forfeit if user deleted/banned (Option A)
8. Async draw offers (offer persists until opponent responds) (Option A)
9. Player's color at bottom (Option B)

### Conflicts with Architecture Doc

- **Async Multiplayer Pattern**: Architecture doc mentions this pattern but provides less detail than single-player.
- **Notification System**: Architecture doc doesn't specify how async games notify users. This is a gap.
- **Multiple Concurrent Games**: Architecture shows one save per user (`user_id INTEGER PRIMARY KEY`). Chess needs multiple simultaneous games per user.
- **Database Schema Difference**: Chess needs `chess_games`, `chess_moves`, `chess_ratings`, `open_games` - more complex than single-player pattern.

### Complexity Estimate

**Moderate** (as specified) - 5-7 days implementation
- Chess rules/validation (using library): 1 day
- Game state management: 1 day
- Lobby/matchmaking: 1 day
- Move input/notation parsing: 0.5 day
- ASCII board rendering: 0.5 day
- ELO system: 0.5 day
- Timeout background job: 0.5 day
- Database schema: 0.5 day
- Testing: 1 day

---

## Cross-Cutting Questions (Applies to Multiple Games)

### 1. Daily Puzzle Seeding Consistency

**Applies to**: Sudoku, Queens

**Question**: Should date-based seeds use UTC or server-local time?
- Option A: UTC (consistent globally)
- Option B: Server local time (matches existing GTM db timestamps which use Eastern)

**Recommendation**: UTC for puzzle seeding to ensure all users get same puzzle regardless of timezone.

### 2. Streak Tracking Cross-Game

**Applies to**: Sudoku, Queens

**Question**: Should there be a combined "daily puzzle" streak across both games?
- Option A: Separate streaks per game
- Option B: Combined streak (complete any daily puzzle = streak continues)
- Option C: Both tracked separately

**Recommendation**: Separate per game (Option A) - matches individual game identity.

### 3. Shared Database for Daily Games

**Applies to**: Sudoku, Queens

**Question**: Should daily puzzle games share a database or have separate databases?
- Option A: Shared `daily_puzzles.db` with tables for each game
- Option B: Separate `sudoku.db`, `queens.db` per architecture pattern

**Recommendation**: Separate databases (Option B) - follows architecture pattern, allows independent deployment.

### 4. Leaderboard Time Period

**Applies to**: All games

**Question**: What time periods for leaderboards?
- Option A: All-time only
- Option B: Daily + Weekly + Monthly + All-time
- Option C: Per-puzzle (today's puzzle) + All-time streak

**Recommendation**: Daily (today's fastest time) + All-time (longest streak, best times) for puzzle games; All-time for Chess.

### 5. User Handle Display

**Applies to**: All games

**Question**: How to display user handles in leaderboards and games?
- Option A: Always full handle
- Option B: Truncated to 12 chars
- Option C: Full in game, truncated on leaderboards

**Recommendation**: Truncated to 12-16 chars for 80-column terminal fit, with padding.

### 6. Timezone for Display

**Applies to**: All games

**Question**: Timestamps in UI - which timezone?
- Option A: UTC
- Option B: Server local (Eastern based on existing code)
- Option C: User's timezone (requires user timezone storage)

**Recommendation**: Server local (Eastern) for consistency with existing GTM implementation.

### 7. Test Data and Deterministic Seeds

**Applies to**: Sudoku, Queens, Chess

**Question**: How to test puzzle generation and game logic?
- Use fixed seeds in tests for reproducibility
- Chess can use known positions (FEN notation)

**Recommendation**: All tests should use deterministic seeds; document seed -> expected result mapping.

---

## Implementation Priority Recommendation

Based on complexity and dependencies:

1. **Memory Garden** (1-2 days) - Standalone feature, no dependencies
2. **Sudoku** (2-3 days) - Daily puzzle pattern, simpler validation
3. **Queens** (2-3 days) - Daily puzzle pattern, harder generation
4. **Chess** (5-7 days) - Most complex, async multiplayer, needs notification system

**Total estimate**: 10-15 days for all four

---

## Open Questions for Product Owner

1. **Streak forgiveness**: Should any daily game have a "streak freeze" feature (like Duolingo)?
2. **Cross-game rewards**: Should completing daily puzzles unlock anything in other games?
3. **Memory Garden visibility**: Is this a core BBS feature or optional module?
4. **Chess tournaments**: Future feature consideration - Swiss or round-robin tournaments?
5. **Notification infrastructure**: Does the BBS have a notification system, or does Chess need to build it?

---

*Document prepared for implementation planning. Answers to questions above will unblock development.*
