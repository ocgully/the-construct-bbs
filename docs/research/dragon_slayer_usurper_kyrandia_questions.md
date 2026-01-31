# Research Questions: Dragon Slayer, The Usurper, Realm of Kyrandia

## Overview

This document identifies implementation questions, assumptions, and potential conflicts for three BBS door games before development begins. Each game has been analyzed against the existing architecture document (`GAME_ARCHITECTURE.md`) and the reference implementation (`grand_theft_meth/`).

---

# Game 1: Dragon Slayer (LORD Clone)

## Summary

Dragon Slayer is a medieval RPG inspired by Legend of the Red Dragon (LORD). Players fight monsters in the forest to gain experience, level up by defeating masters at Turgon's Training (12 levels), choose skills from three paths (Death Knight, Mystic, Thief), and ultimately slay the Red Dragon. Features PvP combat, romance mechanics, and daily turn limits. This is an asynchronous multiplayer game with shared player roster.

## Key Implementation Questions

### Daily Turn System

1. **How many forest fights per day?** The spec says "~15-20" - should this be exactly 15, exactly 20, or randomly generated per day?

2. **When do daily turns reset?** Options:
   - Midnight UTC (consistent for all players)?
   - Midnight in player's local timezone (requires storing timezone)?
   - 24 hours after player's last login?
   - When player explicitly "sleeps at the Inn"?

3. **What actions cost turns vs. are free?**
   - Forest fights: Cost turns (confirmed)
   - Player attacks: 1-3 per day (confirmed) - is this separate from forest fights?
   - Training attempts: "Unlimited if qualified" - free?
   - Shopping/Bank: Free?
   - Romance actions: Cost turns or free?
   - Viewing leaderboard: Free (per architecture doc)

### PvP Combat

4. **Level restrictions for attacking players:**
   - "Can only attack lower or same level" - can a Level 5 attack a Level 1? Is there a minimum level gap protection?
   - Should there be a grace period for new characters (e.g., can't be attacked until Level 3)?

5. **Sleeping players:** The spec says "Sleeping players can be attacked" - what defines "sleeping"?
   - Any player not currently online?
   - Players who explicitly chose "Return to Inn"?
   - Should there be Inn protection (pay gold to be immune)?

6. **Revenge mechanics:** If Player A kills Player B, can Player B immediately retaliate on their next turn, or does the "1-3 attacks per day" limit apply?

7. **What happens on player death?**
   - Lose 10% experience (confirmed)
   - Lose gold (unless banked) - what percentage?
   - "Respawn at Inn next day" - lose remaining turns for the day?

### Romance System

8. **How explicit should romance content be?**
   - PG-13 flavor text only?
   - Fade-to-black suggestive scenes?
   - The original LORD had some risque content - match that tone?

9. **Romance with other players:**
   - Both players must be online? Or async proposal system?
   - What happens if player A proposes to player B who is offline?
   - Can players divorce? Cost/cooldown?

10. **Mechanical benefits of marriage:**
    - Stat bonuses? HP regen? Shared bank?
    - Children: "stat bonuses" - what bonuses? Permanent or temporary?

### Skill System

11. **Skill point allocation:**
    - "Max 40 per skill" - how many skill points per level up?
    - Can players respec? Cost?
    - Are skills use-limited per day (like in original LORD)?

12. **Skill balance:**
    - Death Knight vs Mystic vs Thief - should they be balanced equally or have intentional asymmetry?
    - "Can learn all three" - is this optimal or should specialization be rewarded?

### The Red Dragon

13. **Dragon encounter rate at Level 12:**
    - Pure random chance per forest fight?
    - Guaranteed after X searches?
    - Can only attempt once per day?

14. **Reset on victory:**
    - "Character is reset (keeps some stats)" - which stats carry over?
    - Does dragon kill count persist?
    - New game+ style progression?

### IGM Support

15. **Is IGM support required for v1?**
    - The architecture doc mentions it as "Future Consideration"
    - Should we design hook points now even if not implementing?

## Assumptions (If Not Clarified)

1. Daily reset at midnight UTC
2. 15 forest fights per day (fixed)
3. 3 player attacks per day
4. Level 5+ can attack anyone Level 3+
5. "Sleeping" = any player not online in last 5 minutes
6. Romance is PG-13 flavor text only
7. Marriage proposals queue for offline players
8. 2 skill points per level up
9. Dragon has 1% encounter rate per forest fight at Level 12
10. IGM hooks designed but not implemented in v1

## Architecture Conflicts

1. **Multiplayer state**: The architecture shows `saves` table per user, but Dragon Slayer needs shared state (player roster visible to all). Need additional tables:
   - `ds_characters` - all active characters (not just saves)
   - `ds_daily_state` - per-character-per-day tracking
   - `ds_marriages` - relationship state

2. **Real-time vs async**: Spec says "Asynchronous" but features like "attack sleeping players" imply shared persistent world. Need to clarify if players can see each other online.

## Complexity Estimate

**Moderate-High: 2-3 weeks**
- State machine with 10+ screens
- Shared player database with concurrent access
- Daily turn tracking
- Combat system with skills
- Romance system with player-to-player interaction
- Master progression system (12 levels)

---

# Game 2: The Usurper

## Summary

The Usurper is a multiplayer hack-n-slash RPG with 100+ dungeon levels, solo or team play, PvP raids, a political system (rise to King or Godhood), romance with children, and a controversial drugs/steroids system that risks mental stability. The ultimate goal is reaching the bottom of Durunghins Mountain and defeating The Supreme Being. This is the most complex game in the set.

## Key Implementation Questions

### Dungeon System

1. **100+ dungeons - how are they generated?**
   - Hand-crafted per level?
   - Procedural generation with level-appropriate monsters?
   - Hybrid (key levels hand-crafted, others procedural)?

2. **Dungeon persistence:**
   - Do dungeons reset daily?
   - Per-player instance or shared state?
   - If shared, how are monsters/treasure handled when multiple players explore?

3. **Exploration mechanics:**
   - Turn-based room-by-room?
   - Text parser ("GO NORTH")?
   - Menu-driven ("1. Go deeper, 2. Search room")?

### Drugs/Steroids System

4. **How dark can this content get?**
   - The spec mentions "psychosis," "addiction," and "berserk" states
   - Should we show visual deterioration effects?
   - Rehabilitation mechanics - how long, how expensive?

5. **Mental Stability specifics:**
   - Starting value? Max value?
   - Recovery rate without rehab?
   - Can it go negative? What happens at -10 vs 0?

6. **Psychosis effects:**
   - "May attack allies" - automated or player choice?
   - "May lose items" - random or targeted?
   - Duration of psychosis episode?

7. **Steroid stacking:**
   - Can players use multiple steroids simultaneously?
   - Diminishing returns? Multiplied risk?

### Team/Clan System

8. **Team size limits?** Max players per team?

9. **Team base storage:**
   - Shared inventory? Limit?
   - Can team members steal from base?

10. **Territory control:**
    - What does owning territory provide?
    - How is territory captured/contested?

### Political System

11. **How does one become King?**
    - Pure combat (kill current king)?
    - Wealth threshold + challenge?
    - Election system among high-rank players?

12. **King powers:**
    - "Set taxes" - what gets taxed? All player income?
    - "Declare wars" - against other teams?
    - "Grant titles" - permanent or revocable?

13. **Rise to Godhood:**
    - Beyond King? What are the requirements?
    - God powers in-game?

### Romance & Family

14. **Children mechanics:**
    - How long from marriage to child?
    - "Stat inheritance" - from both parents? Average or random?
    - "Children can become playable" - separate characters or continuation?

15. **Dynasty tracking:**
    - Family tree visible?
    - Inheritance on death - what transfers?

### The Supreme Being

16. **How is The Supreme Being fight triggered?**
    - Simply reach dungeon level 100+?
    - Collect key items first?
    - Single attempt per character lifetime?

17. **"Can alter the destiny of the world":**
    - What does victory actually change for the game world?
    - Server-wide effect?
    - Seasonal reset trigger?

### Equipment System

18. **10+ equipment slots - exact list:**
    - Spec lists 10 (weapon, shield, helmet, armor, gloves, boots, 2 rings, amulet, cloak)
    - Any additional slots? (Belt, bracers, etc.)

19. **Equipment degradation:**
    - Do items wear out?
    - Repair mechanics?

## Assumptions (If Not Clarified)

1. Dungeons procedurally generated with hand-crafted boss levels (10, 25, 50, 75, 100)
2. Mental Stability starts at 100, max 100
3. Psychosis at 0 lasts until rehabilitation (costs gold + time)
4. Max team size: 5 players
5. One King per server, challenged via combat
6. Children appear after 30 in-game days of marriage
7. Drugs content kept at "dark but not gratuitous" level
8. Equipment does not degrade
9. Godhood is post-game content (beat The Supreme Being)
10. World state persists - Supreme Being victory recorded, not reset

## Architecture Conflicts

1. **Database complexity**: This game needs far more tables than the standard pattern:
   - Characters, equipment, inventory
   - Teams, team_membership, team_bases
   - Political_ranks, king_log
   - Families, marriages, children
   - Dungeon_state (if persistent)
   - Drug_effects, addiction_state

2. **Real-time elements**: Team raids suggest some real-time coordination. Architecture doc mentions WebSocket support for real-time multiplayer but this game is listed as "Asynchronous."

3. **Session length**: Deep dungeon runs might exceed typical BBS session length. Need save-mid-dungeon capability.

## Complexity Estimate

**Very High: 4-6 weeks**
- Most complex game in the entire specification
- 100+ dungeon levels
- Full equipment system (10+ slots, quality tiers)
- Team system with shared resources
- Political system with server-wide effects
- Romance with generational mechanics
- Controversial drug system requiring careful balance
- The Supreme Being as endgame boss

---

# Game 3: Realm of Kyrandia

## Summary

Realm of Kyrandia is a multi-player text adventure RPG where players progress from Apprentice to Arch-Mage of Legends by exploring four regions, learning spells, and solving mystical puzzles. Notable for its puzzle-heavy progression (specific commands must be discovered to advance levels) and the Fountain of Scrolls mechanic (throw pine cones to generate random spell scrolls). First player to complete becomes the Arch-Mage of Legends.

## Key Implementation Questions

### Puzzle System

1. **How obscure should puzzles be?**
   - Original Kyrandia was notoriously cryptic ("chant glory be to tashanna")
   - Modern players may have less patience - provide hint system?
   - Risk of solutions being posted externally immediately

2. **Hint system:**
   - Built-in hints that cost resources?
   - Progressive hints (vague -> specific)?
   - NPC hints only?
   - No hints (preserve challenge)?

3. **Puzzle design:**
   - Word/phrase puzzles: How do we prevent brute-force?
   - Item combination: Inventory limit forces puzzle attempts?
   - Timed sequences: Real-time or turn-based timer?

4. **Multi-player cooperation puzzles:**
   - Require multiple players in same location?
   - How do we handle low-population servers?
   - Can puzzles be soloed eventually?

### The Fountain of Scrolls

5. **Pine cone acquisition rate:**
   - How many pine cones per forest visit?
   - Are they consumed on pickup attempt?
   - Daily limit on pine cone gathering?

6. **Scroll randomization:**
   - True random or weighted toward needed scrolls?
   - Can players get duplicate/useless scrolls?
   - Is there a "bad luck protection" system?

7. **Scroll requirements for progression:**
   - Which scrolls are required vs optional?
   - Can progress be blocked by bad RNG?

### Multiplayer Interaction

8. **Real-time chat in-game:**
   - SAY command: radius-based (same room)?
   - WHISPER: Cross-map private message?
   - Is chat real-time or async (message board)?

9. **Player visibility:**
   - Can players see others in same location?
   - Player list visible globally?
   - Indication of how many players online?

10. **Trading:**
    - Direct trade UI or drop-and-pickup?
    - Trade scam protection?

11. **Dueling:**
    - Must both players consent?
    - Level restrictions?
    - Stakes (items, gold, XP)?

### Becoming Arch-Mage

12. **"First to complete = Arch-Mage of Legends":**
    - Only one ever per server?
    - Resets seasonally?
    - Multiple Arch-Mages but first has special status?

13. **Post-Arch-Mage gameplay:**
    - Can Arch-Mage continue playing?
    - Special powers that affect other players?
    - "Can reset for new game" - is this required or optional?

### Command Parser

14. **Parser complexity:**
    - Simple verb-noun ("TAKE SCROLL")?
    - Full sentences ("chant glory be to tashanna")?
    - Aliases and typo tolerance?

15. **Movement commands:**
    - Cardinal directions (N/S/E/W)?
    - Contextual ("ENTER FOREST")?
    - Numbered exits ("1. North, 2. East")?

### World Persistence

16. **Item respawn:**
    - Do forest items regenerate?
    - Daily reset or player-action triggered?
    - Competition for limited resources?

17. **NPC state:**
    - Do NPCs remember individual players?
    - Reputation system?

## Assumptions (If Not Clarified)

1. Puzzles include built-in progressive hint system (3 hints per puzzle, increasingly specific)
2. Chat is real-time for players in same location, async otherwise
3. Pine cones: 0-3 per forest area visit, random
4. Scrolls are weighted toward progression-required ones
5. Arch-Mage is seasonal (resets monthly), first each month gets special status
6. Parser supports aliases and common typos
7. Movement via menu selections, not typed directions
8. Items respawn daily
9. Multi-player puzzles can be soloed after 7 days without completion
10. Trading requires both players in same location

## Architecture Conflicts

1. **Text parser**: The architecture assumes menu-driven single-key input. Kyrandia needs a text parser for spell chanting and puzzle commands. This is a significant deviation.

2. **Real-time multiplayer**: Listed as "Multiplayer (Persistent)" but has real-time chat features. Architecture mentions WebSocket for real-time but this game needs:
   - Player presence tracking
   - Real-time message passing
   - Potentially shared screen updates

3. **Location-based state**: Unlike other games with per-player state, Kyrandia has world state:
   - Items in locations
   - NPCs with state
   - Other players' positions

4. **Screen rendering**: Text adventure format differs from menu-driven games. Need:
   - Room description rendering
   - Inventory display
   - Multi-line scrolling output

## Complexity Estimate

**High: 3-4 weeks**
- Text parser implementation (significant effort)
- World/location database with persistent items
- Spell system with components
- Puzzle system with hint progression
- Multi-player presence and chat
- Four distinct regions with unique content
- Fountain of Scrolls random mechanic
- Arch-Mage victory condition

---

# Cross-Cutting Questions (All Three Games)

## Daily Limits

1. **Reset timing:** Should all games use the same reset time (midnight UTC) for consistency?

2. **Offline progress:** Do daily limits accrue if players don't log in? (e.g., can a player who missed 3 days have 45+ forest fights?)

3. **Display of limits:** Should there be a universal "daily actions remaining" UI pattern?

## PvP Mechanics

4. **Cross-game PvP philosophy:**
   - Dragon Slayer: Attack sleeping players for XP
   - Usurper: Raid sleeping players, steal items
   - Kyrandia: Duel by consent
   - Should these be consistent or intentionally different?

5. **Griefing prevention:** How do we prevent high-level players from making the game unplayable for newcomers in all three games?

## Romance Systems

6. **Content rating:** Dragon Slayer and Usurper both have romance. Should there be a global BBS setting for content level? Or per-game?

7. **Same-sex relationships:** Support in all games? Player character gender selection?

## Inter-Player Interaction

8. **Real-time vs async:**
   - Dragon Slayer: Async (attack sleeping players)
   - Usurper: Async with team coordination
   - Kyrandia: Real-time chat implied
   - Can we standardize or must each be different?

9. **Cross-game communication:** Can players see who's playing what game from a central lobby?

## IGM/Extensibility

10. **V1 requirement:** The architecture doc lists IGM as "Future Consideration." Should any of these games have IGM hooks designed in v1?

11. **Hook points:** If designing for future IGM, what hook points:
    - New locations?
    - New NPCs?
    - New items/equipment?
    - New quests?
    - Custom events?

## Database Architecture

12. **Shared vs separate:** Each game gets its own SQLite database (per architecture). For multiplayer features, is this still appropriate or do we need a shared BBS-wide database?

13. **Character naming:** Can two players have the same character name in the same game? Across games?

## Progression Balance

14. **XP curves:** Should XP requirements follow a consistent formula across games, or each game its own curve?

15. **Session length:** Typical BBS session is 30-60 minutes. Are the daily limits calibrated for this?

16. **Catch-up mechanics:** If a player falls behind, can they ever catch up to someone who plays daily?

## Technical Concerns

17. **Concurrent access:** All three games have shared state (player rosters, world state). How do we handle:
    - Two players attacking same monster?
    - Two players trading same item?
    - Race conditions in PvP?

18. **State machine complexity:** Dragon Slayer has ~11 locations, Usurper has 100+ dungeon levels, Kyrandia has room-based navigation. How do we handle screen state?

19. **Save points:** Can players save mid-action (mid-combat, mid-puzzle)?

---

# Priority Recommendations

## Must Clarify Before Implementation

1. **Daily reset timing** (affects all games)
2. **Content rating for romance/drugs** (Dragon Slayer, Usurper)
3. **Puzzle hint system** (Kyrandia)
4. **Text parser scope** (Kyrandia - architectural impact)
5. **PvP griefing prevention** (Dragon Slayer, Usurper)

## Can Proceed with Assumptions

1. Forest fight counts
2. Equipment slot lists
3. Dungeon procedural generation approach
4. Most mechanical details

## Recommend Deferring

1. IGM support (all games) - design hooks but don't implement
2. Godhood system (Usurper) - post-v1
3. Multi-player cooperation puzzles (Kyrandia) - ensure solo path first
4. Children becoming playable (Usurper) - post-v1

---

*Document generated: 2026-01-30*
*Reference implementation: `backend/src/games/grand_theft_meth/`*
*Architecture doc: `docs/GAME_ARCHITECTURE.md`*
