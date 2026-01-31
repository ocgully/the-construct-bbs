# Consolidated Questions - All Games

**Generated**: 2026-01-30
**Purpose**: Answer these questions before implementation begins. Your answers will be saved and distributed to all implementation agents.

---

## SECTION A: Global Decisions (Affects All Games)

### A1. Time & Timezone
**Daily reset timing for all games?**
- [ ] Midnight UTC (consistent globally)
- [X] Midnight Eastern (match existing BBS timestamps)
- [ ] Other: ___

### A2. Notification System
**How should games notify users of async events?** (opponent moved, kingdom attacked, turn due)
- [ ] BBS login notification only (see alerts when you log in)
- [ ] In-game indicator (like mail icon)
- [ ] Email notifications (requires SMTP setup)
- [X] All of the above (user configurable), instead of e-mail though, use the BBS email system. No need for SMTP. Though we should setup SMTP for password key verification, and if they get in-bbs mail, they should be notified in their e-mail. With unscubrcribe/opt out options (Should default to notifying). For email notifications though, let's be careful to only do this for active players and to not over do it. I don't want to fill up our DB with mail that nobody is ever going to see.

### A3. Background Jobs
**Infrastructure for ticks/turns/timeouts?**
- [ ] Single scheduler for all games
- [X] Per-game background processes
- [ ] On-demand calculation (compute on login, no background jobs)

### A4. Content Rating
**Romance/drugs content level across games?**
- [ ] PG-13 (suggestive text, fade-to-black)
- [ ] Mature (explicit descriptions, drug mechanics)
- [X] Per-game setting (some darker than others)
- [ ] Sysop configurable

### A5. Griefing Prevention
**High-level player attacking newcomers?**
- [ ] Level-based restrictions (can only attack within X levels)
- [ ] Protection period for new players (48 hours? 1 week?)
- [ ] Opt-in PvP only
- [X] Game-specific rules - for some games, this is a "feature not a bug"

### A6. Multiplayer Infrastructure
**Build shared infrastructure or per-game?**
- [X] Shared lobby/matchmaking system for all real-time games
- [ ] Each game has its own implementation
- [ ] Shared core, game-specific extensions

### A7. Spectator Mode
**Allow watching ongoing games?**
- [X] Yes, for all multiplayer games, but only for Sysop.
- [ ] Only specific games (which?)
- [ ] No spectators
- [ ] Per-game decision

---

## SECTION B: Daily Puzzles (Sudoku, Queens)

### B1. Streak Policy
**What happens when user misses a day?**
- [ ] Reset to 0 (strict)
- [X] Reset to 0, but track "longest streak" separately, but have 3 days a week where the streak is automatically saved (like it's paused for the day)
- [ ] One grace day before reset
- [ ] Streak pauses, resumes when they return

### B2. Difficulty Selection
**How do users choose difficulty?**
- [ ] Pick per-session (can play Easy then Hard same day)
- [ ] Locked for the day (first choice is final)
- [ ] Rotating (Mon=Easy, Tue=Medium, Wed=Hard)
- [ ] Same puzzle all difficulties (different scoring)
- [X] No difficulty setting for games, they're just what they are. Should default to about Medium difficulty.

### B3. Multiple Attempts
**Can users retry after solving?**
- [X] One attempt per day
- [ ] Unlimited, first completion time counts
- [ ] Unlimited, best time counts

---

## SECTION C: Memory Garden (BBS Feature)

### C1. Location
**Where in BBS menu?**
- [X] Main menu (alongside Games, Mail, Chat)
- [ ] Under a "Community" submenu
- [ ] Accessible from user profile
- [ ] Easter egg (hidden)

### C2. Content Permanence
**Can users edit/delete memories?**
- [ ] No - permanent record
- [ ] Delete within 1 hour
- [X] Users can always delete their own memories, not other peoples. But they can flag memories for review for the SysOp. Doing so immediately hides the memory from public view, until the SysOp reviews. They can only flag a maximum of 3 memories per day.
- [X] Users can Edit within 1 hour

---

## SECTION D: Chess

### D1. Concurrent Games
**How many simultaneous games per user?**
- [ ] Unlimited
- [ ] Limit to 5
- [ ] Limit to 10
- [X] Sysop configurable, default to 5. Upper limit should be 20.

### D2. Matchmaking
**How are opponents found?**
- [ ] Open games list (first-come)
- [ ] ELO-based matching - make sure to calculate based on win ratios or other metrics available.
- [ ] Challenge specific users
- [X] All of the above

---

## SECTION E: Classic RPGs (Dragon Slayer, Usurper, Kyrandia)

### E1. Daily Turns (Dragon Slayer)
**Forest fights per day?**
- [ ] Fixed 15
- [ ] Fixed 20
- [ ] Random 15-20
- [ ] User level based
- [X] I think this should be configurable. Default to Whatever LoRD tends to do, but 15-20 sounds about right.

### E2. PvP Rules
**Can players attack anyone lower level?**
- [ ] Yes, any lower/same level
- [ ] Minimum gap protection (within 3 levels only)
- [ ] Can't attack below level 3
- [X] Attacker loses XP for killing much lower

### E3. Usurper Content
**How dark can drugs/steroids get?**
- [ ] Mechanical only (stats + debuffs, no narrative)
- [X] Moderate (descriptions of effects, not gratuitous)
- [ ] Full (psychosis scenes, dark spiral narrative)

### E4. Kyrandia Text Parser
**Input style for Kyrandia?**
- [ ] Full text parser ("chant glory be to tashanna")
- [ ] Menu-driven with text for puzzles only
- [ ] Hybrid (menus + typed spells)

### E5. Romance Mechanics
**Player-to-player romance?**
- [ ] Flavor text only (no mechanical benefits)
- [ ] Stat bonuses for marriage
- [ ] Children with inherited stats
- [ ] Full system with divorce, dynasties
- [X] Other - Text, bonus', and full system for divorce. No dynasties.

### E6. IGM Support
**In-Game Modules for v1?**
- [ ] Design hooks now, implement later
- [ ] No IGM consideration for v1
- [X] Full IGM support required

---

## SECTION F: Strategy Games (Dystopia, Master of Cygnus, Star Trader)

### F1. Tick Implementation
**How do resource ticks work?**
- [ ] Real-time background job (runs even when no one online)
- [ ] On-demand catchup calculation (compute on login)
- [X] Hybrid (background for active, catchup for inactive, games can run once a day updates even when people are not online.)

### F2. Dystopia Age Duration
**How long is one "age"?**
- [ ] 4 weeks
- [ ] 6 weeks
- [ ] 8 weeks
- [X] Sysop configurable - default to 4 weeks

### F3. Star Trader Galaxy
**Galaxy persistence?**
- [X] Perpetual (never resets)
- [ ] Monthly seasons (optional reset)
- [ ] Quarterly resets
- [ ] Multiple concurrent galaxies

### F4. Corporation Size
**Max players per corporation/kingdom?**
- [ ] 5 players
- [X] 10 players
- [ ] 25 players
- [ ] Unlimited

### F5. Turn Timeout (Master of Cygnus)
**72-hour timeout consequences?**
- [ ] 3 consecutive timeouts = forfeit
- [ ] 3 total timeouts = forfeit
- [X] AI takes over after timeout, other player(s) get to decide if they want to keep playing or not, if not they can forfeit. All Forfeits should be taken over by AI. If no players remain, then the game is done.
- [ ] Timeout extends deadline (no forfeit)

---

## SECTION G: Real-Time Games (Summit, Tanks, Depths of Diablo, Acromania)

### G1. Matchmaking Policy
**Who can join games?**
- [ ] Friends only (invite codes)
- [ ] Public matchmaking (strangers)
- [X] Both options available

### G2. Depths of Diablo Combat
**Combat model?** (CRITICAL - affects everything)
- [X] Real-time (Diablo-style, continuous action)
- [ ] Turn-based (traditional roguelike)
- [ ] Hybrid (real-time movement, turn-based combat)

### G3. Tanks Viewport
**80x24 terminal can't fit original layout. Solution?**
- [ ] Redesign with smaller viewports
- [X] One tank at a time (switch between using tanks), have your base view be Picture In picture in a corner. 
- [ ] Tab between base/tanks views
- [ ] Accept 100+ column requirement

### G4. Daily Seed Scope
**Same mountain/dungeon for everyone?**
- [X] Yes, worldwide same seed
- [ ] Regional seeds (Americas/Europe/Asia)
- [ ] Per-timezone seeds

### G5. Disconnection Policy
**Player disconnects mid-game?**
- [X] Removed from game (others continue)
- [ ] AI takeover
- [X] Can rejoin at any time
- [ ] Game pauses (if small group)

### G6. Acromania Bots
**If players leave mid-game?**
- [X] Game continues with fewer players
- [ ] Bots fill in with random phrases
- [X] Game ends if below minimum (2 players)
- [ ] Allow new players to join between rounds

---

## SECTION H: Epic Games (Last Dream, Mineteria, Fortress, Ultimo, Cradle, Xodia)

### H1. MVP Scope Philosophy
**For these massive games, what's MVP?**
- [ ] Vertical slice (full depth, limited content)
- [ ] Horizontal slice (all features, shallow)
- [ ] 1-2 hour playable loop
- [ ] Whatever fits in 4-6 weeks
- [X] No MVP, Do the full game

### H2. Prioritization
**Which epic games first?** (Pick 1-2)
- [ ] Last Dream (JRPG - most achievable)
- [ ] Cradle (progression - unique hook)
- [ ] Mineteria (sandbox - broad appeal)
- [ ] Fortress (colony sim - niche but deep)
- [ ] Ultimo (MMO - most ambitious)
- [ ] Xodia (LLM MUD - most experimental)
- [X] Do them all, I dont want to lose time.

### H3. Last Dream Breadcrumbs
**How subtle should simulation twist hints be?**
- [X] Very rare (1-2 per playthrough, easily missed)
- [ ] Moderate (one per area, attentive players notice)
- [ ] Obvious (clear pattern for those looking)

### H4. Cradle Build Mistakes
**How punishing are wrong build choices?**
- [ ] Soft ceiling (50% slower progress)
- [ ] Hard wall (cannot advance without reset)
- [ ] Fixable via expensive in-game mechanic
- [X] Maybe punishing doesn't make sense, players should hit real walls, and have to change to different builds/respec to progress. This should come at a cost.

### H5. Xodia LLM Budget
**Monthly LLM API budget for Xodia?**
- [ ] $50/month (limited usage)
- [ ] $200/month (moderate)
- [ ] $500/month (generous)
- [X] Whatever it takes, but support the option for local LLMs through Ollama
- [ ] Xodia deferred until costs are lower

### H6. Xodia Fallback
**When LLM API is unavailable?**
- [ ] Scripted fallback responses
- [ ] "The mists prevent action" message
- [ ] Queue actions for later
- [X] Game is offline, additionally all games should be able to be taken offline or down for mainaiance by the SysOp. 

---

## SECTION I: Quick Answers (Yes/No/Number)

Answer briefly:

1. **Same-sex romance in games?** Y/N: Y
2. **Cross-game achievements?** Y/N: N
3. **User handle max length for display?** (12/16/20): 12
4. **Leaderboard time periods?** (daily/weekly/monthly/all-time): all-time
5. **Max concurrent real-time games per user?** Number: 1 - that is to say they can only play 1 game at a time, but they could swap between multiple real-time games, especially ones that allow for jumping in/out (which i think are most of them)
6. **Profanity filter for Acromania?** Y/N: Optional, personal preference. Defaults to on.
7. **Sound effects in terminal games?** Y/N: Yes, but keep it period approriate (beeps, buzzes), notes. Basically anything that basic could produce.

---

## Your Answers

Please fill in your choices above. For multiple choice, mark with [X].

After answering, I'll save these as `.planning/GAME_DECISIONS.md` and all implementation agents will reference it.

---

*Questions compiled from 5 parallel research agents analyzing 20 games against the architecture document.*
