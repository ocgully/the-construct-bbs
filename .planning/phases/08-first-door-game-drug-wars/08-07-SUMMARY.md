---
phase: 08-first-door-game-drug-wars
plan: 07
subsystem: game-quests
tags: [rust, quest-system, gang-relations, story-progression, deliveries]

requires:
  - 08-01-SUMMARY.md  # Database layer for game persistence
  - 08-02-SUMMARY.md  # GameState structs (QuestProgress, DeliveryQuest)
  - 08-04-SUMMARY.md  # render.rs pattern (format_money export)

provides:
  - backend/src/game/quest.rs (quest logic module)
  - Gang tribute system for improving relations
  - Delivery quest generation and completion
  - 15-step story quest with narrative arc

affects:
  - 08-08-PLAN.md  # Screen integration will wire render_quest to session handler
  - 08-09-PLAN.md  # Service integration will add quest menu item

tech-stack:
  added:
    - None (uses existing rand crate)
  patterns:
    - Static story quest data with &'static lifetimes
    - Result<T, String> for quest operations
    - Gang territory lookup via borough data

key-files:
  created:
    - backend/src/game/quest.rs  # Quest system implementation
  modified:
    - backend/src/game/mod.rs    # Added quest module export
    - backend/src/game/render.rs # Added render_quest function
    - backend/src/game/screen.rs # Added handle_quest handler

decisions:
  - Gang tribute costs +30 relation (capped at 100): Meaningful but not instant alliance
  - Delivery quests max 3 active: Prevents quest spam, forces prioritization
  - Delivery expiry hurts gang relations -10: Consequences for failed commitments
  - Story step 12 requires $500k net worth: Mid-game wealth gate ensures progression isn't rushed
  - Story completion boosts all gang relations +5: Rewards for advancing narrative
  - 15-step story across 5 cities: Epic scope traversing full game world

metrics:
  duration: 511s
  commits: 2
  files_changed: 4
  lines_added: 587
  completed: 2026-01-29
---

# Phase 8 Plan 7: Quest System Summary

**One-liner:** Gang tribute, delivery quests with expiry consequences, 15-step globe-spanning story quest with Marcus betrayal narrative

## What Was Built

### Gang Relations System
- **pay_tribute**: Pay gang tribute cost for +30 relation (capped at 100)
- **gang_status**: Returns "Allied" (>=50), "Neutral" (>=0), "Hostile" (>=-50), "Enemy" (<-50) text
- Tribute costs: Triads $5k, Cartel $7.5k, Mafia $6k

### Delivery Quest System
- **generate_delivery_quest**: Random commodity, destination (city/borough), quantity 5-20 units
  - Reward: base_price × quantity + 50-200% variance
  - Expires: current_day + 3-7 days
  - Only generates if <3 active deliveries
- **accept_delivery**: Validates goods owned, reserves from inventory, adds to active_deliveries
- **check_deliveries**: Auto-completes when player arrives at destination, awards reward
- **expire_deliveries**: Removes expired quests, applies -10 gang relation penalty to territory owner
- Territory gang lookup via borough.gang_territory field

### Story Quest (15 Steps)
**Narrative Arc:** Marcus betrayal → international manhunt → final showdown

**Step Progression:**
1. **The Old Contact** (NYC/Bronx) - $500 reward
2. **A Simple Job** (NYC/Brooklyn) - Requires 5 weed, $250 reward
3. **Something's Off** (NYC/Manhattan) - Setup revelation
4. **Earning Trust** (Miami/Little Havana) - Requires 10 cocaine, $1,000 reward
5. **The Bigger Picture** (Miami/South Beach) - $500 reward
6. **Crossing the Pond** (London/East End) - $750 reward
7. **The Triads** (Tokyo/Shinjuku) - Requires 15 meth, $1,500 reward
8. **The Source** (Bogota/Chapinero) - $1,000 reward
9. **The Betrayal** (Bogota/La Candelaria) - Marcus reveal
10. **Old Friends** (NYC/Queens) - $750 reward
11. **The Plan** (London/Soho) - $1,000 reward
12. **War Chest** (no location) - Requires $500k net worth (wealth gate)
13. **The Hunt** (Miami/Downtown Miami) - $2,000 reward
14. **Showdown** (NYC/Manhattan) - $5,000 reward (full circle)
15. **Kingpin** (no location) - $10,000 reward, completion flag

**Quest Features:**
- **get_current_story**: Returns next incomplete step based on quest_state.story_step
- **can_complete_story_step**: Validates location, commodity requirements, net worth
- **complete_story_step**: Consumes commodities, awards reward, advances step, boosts all gang relations +5
- **is_story_complete**: Returns true when story_step >= 15

### Quest Screen Rendering
- **render_quest**: Full quest overview screen
  - Story Quest section: Chapter number, title, narrative quote, location requirement, commodity requirement, net worth requirement, reward, completion indicator ([S] when ready)
  - Active Deliveries section: Quantity, commodity, destination, expiry day, reward
  - Gang Relations section: Name, status (color-coded), numeric relation value
- **handle_quest**: S to complete story step (if can_complete), Q to return to main menu
- Added Quest to is_single_key_screen for immediate input processing

## Technical Implementation

**Quest Module Structure:**
- Static STORY_STEPS array with StoryStep struct (15 entries)
- StoryStep fields: step, title, location, narrative, requirement, min_net_worth, reward
- Gang territory lookup: split location "city/borough", iterate boroughs, return gang_territory
- Delivery quest ID: "del_{day}_{random_1000-9999}" for uniqueness

**Integration Points:**
- quest.rs exports: pay_tribute, gang_status, generate_delivery_quest, accept_delivery, check_deliveries, expire_deliveries, get_current_story, can_complete_story_step, complete_story_step, is_story_complete
- render.rs: render_quest function with color-coded gang relations (green/yellow/red)
- screen.rs: handle_quest with story completion logic, SaveGame action
- mod.rs: pub mod quest; pub use quest::*;

**Rendering Pattern:**
- render_quest uses AnsiWriter, returns String (matches render.rs pattern)
- Color coding: LightGreen (>=50), Yellow (>=0), LightRed (<0)
- Uses existing format_money helper (exported from render.rs)

## Commits

1. **465c4ee** - feat(08-07): create quest module with delivery quests and story
   - backend/src/game/quest.rs (434 lines added)
   - Gang tribute, delivery quests, 15-step story
   - Static data with &'static lifetimes

2. **6a082f7** - feat(08-07): add quest screen rendering and handlers
   - backend/src/game/render.rs (+136 lines)
   - backend/src/game/screen.rs (+17 lines, -3 lines)
   - render_quest with story/deliveries/gang relations
   - handle_quest with S to complete story step

## Verification

**Compilation:** `cargo check` passed (1.38s) - 130 warnings (unused code, expected pre-integration)
**Quest module:** Compiles cleanly with no errors
**Render function:** Added to render.rs export list
**Handler integration:** handle_quest called from screen.rs process_input dispatcher

## Deviations from Plan

None - plan executed exactly as written.

## Next Phase Readiness

**08-08 (Screen Integration):**
- Quest screen rendering ready (render_quest implemented)
- Handler ready (handle_quest with story completion)
- Awaiting: Session.rs integration to call render_quest for GameScreen::Quest

**08-09 (Service Integration):**
- Quest module exports ready for service wrapper
- Awaiting: Door service to launch GtmFlow with quest functionality

**Future Enhancement Opportunities:**
- Delivery quest board UI (browse available quests)
- Gang tribute UI (select gang, see current relation)
- Story quest completion messages (celebrate each chapter)
- Quest completion stats tracking (total deliveries, story steps completed)

## Lessons Learned

**Static Quest Data Pattern Works Well:**
- STORY_STEPS as &'static [StoryStep] provides zero-allocation quest data
- Location strings as Option<&'static str> avoid runtime allocations
- Narrative text embedded directly in binary (no file I/O)

**Gang Relations Integrate Cleanly:**
- Existing gang_relations HashMap in GameState ready for tribute system
- Territory lookup via existing borough.gang_territory field
- Delivery expiry consequences create meaningful risk/reward

**Story Quest Provides Structure:**
- 15 steps give clear progression path across 90-day game
- Net worth gate (step 12) ensures mid-game economic milestone
- Global travel requirement encourages exploring all cities
- Commodity requirements drive trading economy

**Quest Screen Rendering:**
- Color-coded gang relations provide instant visual feedback
- Story narrative quotes create atmospheric context
- Active deliveries list shows clear time pressure (expiry days)

**Integration Readiness:**
- Quest module compiles independently (good separation of concerns)
- Rendering functions follow established pattern (return String)
- Handler uses GtmAction::SaveGame for state persistence
