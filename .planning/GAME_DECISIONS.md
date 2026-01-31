# Game Implementation Decisions

**Finalized**: 2026-01-30
**Purpose**: Authoritative reference for all implementation agents. These decisions are final and should not be re-asked.

---

## Global Decisions (All Games)

### Time & Timezone
- **Daily reset**: Midnight Eastern (matches existing BBS timestamps)

### Notification System
- All notification methods available, user configurable:
  - BBS login notifications
  - In-game indicators
  - BBS internal mail system (not SMTP email)
- External email for password verification and BBS mail notifications only
- Unsubscribe/opt-out options required (default: notify ON)
- Only send to active players; avoid DB bloat from unread mail

### Background Jobs
- **Per-game background processes** (not a single scheduler)
- Games can run daily updates even when no players online

### Content Rating
- **Per-game setting** - some games darker than others
- Not globally enforced

### Griefing Prevention
- **Game-specific rules** - for some games, griefing is "feature not a bug"
- No global protection system

### Multiplayer Infrastructure
- **Shared lobby/matchmaking system** for all real-time games
- Build once, use across games

### Spectator Mode
- **Sysop only** can spectate all multiplayer games
- No public spectating

---

## Daily Puzzles (Sudoku, Queens)

### Streak Policy
- Reset to 0 on miss, but:
  - Track "longest streak" separately
  - 3 automatic "pause days" per week (streak saved)

### Difficulty
- **No difficulty selection** - puzzles are what they are
- Default to approximately Medium difficulty

### Attempts
- **One attempt per day** - no retries

---

## Memory Garden

### Location
- **Main menu** (alongside Games, Mail, Chat)

### Content Permanence
- Users can always delete their own memories
- Edit allowed within 1 hour
- Flag system for reporting (max 3 flags/day per user)
- Flagged memories hidden until Sysop review

---

## Chess

### Concurrent Games
- **Sysop configurable**, default 5, max 20

### Matchmaking
- All methods available:
  - Open games list
  - ELO-based matching (calculate from win ratios)
  - Direct user challenges

---

## Classic RPGs (Dragon Slayer, Usurper, Kyrandia)

### Daily Turns (Dragon Slayer)
- **Sysop configurable**, default 15-20 (match LoRD conventions)

### PvP Rules
- **Attacker loses XP** for killing much lower level players
- No hard restrictions on who can attack whom

### Usurper Content (Drugs/Steroids)
- **Moderate** - descriptions of effects, not gratuitous
- Mechanical consequences included

### Kyrandia Text Parser
- **Decision deferred** - needs further design discussion
- Options were: full parser, menu-driven, or hybrid

### Romance Mechanics
- Flavor text + stat bonuses + divorce system
- **No dynasties/children inheritance**
- Same-sex romance: **Yes**

### IGM Support
- **Full IGM support required** for v1
- Not deferred

---

## Strategy Games (Dystopia, Master of Cygnus, Star Trader)

### Tick Implementation
- **Hybrid approach**:
  - Background jobs for active games
  - Catchup calculation for inactive
  - Daily updates run even with no players online

### Dystopia Age Duration
- **Sysop configurable**, default 4 weeks

### Star Trader Galaxy
- **Perpetual** - never resets

### Corporation/Kingdom Size
- **Max 10 players** per corp/kingdom

### Turn Timeout (Master of Cygnus)
- After 72-hour timeout: **AI takes over**
- Other players decide to continue or forfeit
- All forfeits become AI-controlled
- Game ends when no human players remain

---

## Real-Time Games (Summit, Tanks, Depths of Diablo, Acromania)

### Matchmaking Policy
- **Both options**: Friends-only (invite codes) AND public matchmaking

### Depths of Diablo Combat
- **Real-time** (Diablo-style, continuous action)
- Not turn-based

### Tanks Viewport
- **One tank at a time** with switching
- Base view as Picture-in-Picture in corner

### Daily Seed
- **Worldwide same seed** - everyone gets same mountain/dungeon

### Disconnection Policy
- Player removed from game (others continue)
- **Can rejoin at any time**

### Acromania Bots
- Game continues with fewer players
- **Game ends if below 2 players**
- No bot fill-ins

---

## Epic Games (Last Dream, Mineteria, Fortress, Ultimo, Cradle, Xodia)

### MVP Scope
- **No MVP** - do the full game for all epic games

### Prioritization
- **Do all games** - no prioritization, maximize parallelization

### Last Dream Breadcrumbs
- **Very rare** (1-2 per playthrough, easily missed)
- Simulation twist hints should be subtle

### Cradle Build Mistakes
- Players hit real walls requiring build changes/respec
- Respec available but **at significant cost**
- Not soft ceilings, not permanent dead-ends

### Xodia LLM Budget
- **Whatever it takes**
- Must support **Ollama for local LLMs**

### Xodia Fallback
- **Game goes offline** when LLM unavailable
- All games should support Sysop maintenance mode

---

## Quick Reference

| Setting | Value |
|---------|-------|
| Same-sex romance | Yes |
| Cross-game achievements | No |
| User handle max length | 12 characters |
| Leaderboard periods | All-time only |
| Max concurrent real-time games | 1 active (can switch between) |
| Profanity filter (Acromania) | Optional, default ON |
| Sound effects | Yes - period appropriate (beeps, buzzes, notes) |

---

## Sysop-Configurable Settings Summary

These settings must have Sysop configuration UI:

1. Chess concurrent game limit (default 5, max 20)
2. Dragon Slayer daily turns (default 15-20)
3. Dystopia age duration (default 4 weeks)
4. Game maintenance/offline mode (all games)
5. Notification preferences defaults

---

*Decisions compiled from user answers on 2026-01-30. All implementation agents should reference this document.*
