# Phase 8: First Door Game (Drug Wars) - Research

**Researched:** 2026-01-28
**Domain:** Door game architecture, game state persistence, economic simulation
**Confidence:** MEDIUM-HIGH

## Summary

Grand Theft Meth is a commodity trading door game requiring game state persistence, economic simulation (price volatility, compound interest), random event systems, quest progression tracking, and leaderboard rankings. This phase validates the door game architecture pattern that will be reused for future games (LORD, Usurper, etc.).

The research identifies that the existing Service trait architecture is ideal for door games, with game state persisted as JSON in SQLite and state machines managing multi-screen flows. The game will use Rust's rand crate for weighted random events, serde_json for save state serialization, and SQLite window functions for leaderboard rankings.

Key technical challenges include: (1) modeling complex nested game state with proper serde derives, (2) implementing daily compound interest calculations without floating-point drift, (3) designing a quest progression system with partial completion tracking, and (4) balancing random event probabilities to maintain engagement without overwhelming players.

**Primary recommendation:** Use a GameState struct with serde derives persisted as JSON TEXT column in game_saves table, implement state machine pattern for game screens (similar to ComposeFlow/LoginFlow), use rand crate's weighted selection for events, and SQLite RANK() window function for leaderboards.

## Standard Stack

The established libraries/tools for this domain:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| serde + serde_json | 1.x | Game state serialization | De facto Rust serialization standard, handles complex nested structs |
| rand | 0.8 | Random number generation, weighted events | Official Rust RNG library, O(1) weighted sampling via Alias method |
| sqlx | 0.8 | Database operations | Already in use, async SQLite with compile-time query checking |
| chrono | 0.4 | Date/time arithmetic | Already in use, essential for daily resets and interest calculations |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| thiserror | 1.x | Error definitions | Already in use, simplifies game-specific error types |
| None needed | - | Quest tracking | Simple enum state machine, no additional crate needed |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| JSON in TEXT column | Separate columns per field | JSON easier to evolve schema, single atomic save/load |
| serde_json | bincode or rmp-serde | JSON human-readable for debugging, smaller size not critical |
| Built-in rand | weighted_rand crate | rand has built-in weighted sampling, no extra dependency needed |

**Installation:**
```bash
# All dependencies already in Cargo.toml
# No new crates required
```

## Architecture Patterns

### Recommended Project Structure
```
backend/src/
├── services/
│   └── grand_theft_meth.rs   # Main game service (Service trait impl)
├── game/
│   ├── mod.rs                 # Re-exports
│   ├── state.rs               # GameState struct, serde derives
│   ├── economy.rs             # Price generation, interest calculations
│   ├── events.rs              # Random event definitions and triggers
│   ├── quests.rs              # Quest definitions and progression
│   ├── combat.rs              # Combat resolution logic
│   └── render.rs              # ANSI rendering functions
└── db/
    └── game_saves.rs          # CRUD operations for game saves
```

### Pattern 1: Game State as JSON Column
**What:** Store entire game state as JSON TEXT in single database row per user
**When to use:** Complex nested state that evolves over development
**Example:**
```rust
// Source: Project architectural decisions + serde_json docs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub day: u32,
    pub actions_remaining: u32,
    pub location: String,
    pub cash: i64,
    pub bank_balance: i64,
    pub debt: i64,
    pub health: u32,
    pub max_health: u32,
    pub notoriety: u32,
    pub inventory: HashMap<String, u32>,  // drug -> quantity
    pub weapons: WeaponSlots,
    pub coat_tier: u32,
    pub quest_state: QuestProgress,
    pub gang_relations: HashMap<String, i32>,
    pub unlocked_locations: Vec<String>,
    pub addiction_levels: HashMap<String, u32>,
    pub game_start_time: String,
    pub last_save_time: String,
}

// Serialization
let json = serde_json::to_string(&game_state)?;
sqlx::query("INSERT INTO game_saves (user_id, state_json) VALUES (?, ?)")
    .bind(user_id)
    .bind(json)
    .execute(pool)
    .await?;

// Deserialization
let row: (String,) = sqlx::query_as("SELECT state_json FROM game_saves WHERE user_id = ?")
    .bind(user_id)
    .fetch_one(pool)
    .await?;
let game_state: GameState = serde_json::from_str(&row.0)?;
```

### Pattern 2: State Machine for Game Screens
**What:** Each game screen (travel, trade, combat) is a state with transitions
**When to use:** Multi-screen flows with distinct input handling per screen
**Example:**
```rust
// Source: Existing ComposeFlow pattern in mail.rs
#[derive(Debug, Clone, PartialEq)]
pub enum GameScreen {
    MainMenu,
    Travel,
    Trade { viewing_drug: Option<String> },
    Combat { enemy: Enemy },
    Quest { quest_id: String },
    Casino { game_type: CasinoGame },
}

pub enum GameAction {
    Continue,
    StateChanged(GameState),
    ShowScreen(String),  // ANSI rendered output
    SaveGame,
    GameOver { final_score: i64 },
}

impl GameFlow {
    pub fn handle_input(&mut self, input: &str) -> GameAction {
        match &self.current_screen {
            GameScreen::MainMenu => self.handle_menu_input(input),
            GameScreen::Trade { viewing_drug } => self.handle_trade_input(input, viewing_drug),
            // ... other screens
        }
    }
}
```

### Pattern 3: Weighted Random Event Selection
**What:** Use rand crate's weighted distribution for event probabilities
**When to use:** Random events with different likelihood (police 5%, deal 10%, etc.)
**Example:**
```rust
// Source: rand crate docs + Rust Rand Book
use rand::prelude::*;
use rand::distributions::WeightedIndex;

pub enum RandomEvent {
    None,
    PoliceEncounter,
    PremiumBuyer,
    Mugging,
    TrenchcoatGuy,
    LoanSharkEnforcer,
}

pub fn trigger_random_event(rng: &mut impl Rng, debt: i64, day: u32) -> RandomEvent {
    let mut events = vec![
        (RandomEvent::None, 85),           // 85% nothing happens
        (RandomEvent::PoliceEncounter, 5),
        (RandomEvent::PremiumBuyer, 4),
        (RandomEvent::Mugging, 3),
        (RandomEvent::TrenchcoatGuy, 2),
        (RandomEvent::LoanSharkEnforcer, 1),
    ];

    // Adjust weights based on game state
    if debt > 10000 {
        events.push((RandomEvent::LoanSharkEnforcer, 5)); // Higher chance
    }

    let choices: Vec<_> = events.iter().map(|(e, _)| e).collect();
    let weights: Vec<_> = events.iter().map(|(_, w)| *w).collect();
    let dist = WeightedIndex::new(&weights).unwrap();

    choices[dist.sample(rng)].clone()
}
```

### Pattern 4: Daily Compound Interest
**What:** Calculate compound interest without floating-point drift
**When to use:** Interest on loans and bank deposits
**Example:**
```rust
// Source: Financial formulas + Rust integer arithmetic
/// Calculate daily compound interest using integer math (basis points)
/// rate_bp: rate in basis points (1000 = 10%)
/// Returns new amount after one day
pub fn apply_daily_interest(principal: i64, rate_bp: u32) -> i64 {
    // A = P(1 + r/n)^(nt) where n=365, t=1/365 (one day)
    // Simplified: A = P(1 + daily_rate)
    // Using integer math: multiply by (10000 + rate_bp) / 10000

    let multiplier = 10000 + (rate_bp as i64);
    (principal * multiplier) / 10000
}

// Usage
let debt = 5500;
let debt_after_day = apply_daily_interest(debt, 1000); // 10% annual = ~0.027% daily
// For true daily compound: rate_bp = ((1.10^(1/365) - 1) * 10000) ≈ 26 bp
```

### Pattern 5: Quest Progression Tracking
**What:** Track multi-step quests with partial completion state
**When to use:** Story quest and delivery quests
**Example:**
```rust
// Source: Game design patterns + Rust enums
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestProgress {
    pub story_step: u32,              // 0 = not started, 1-15 = steps
    pub active_deliveries: Vec<DeliveryQuest>,
    pub completed_deliveries: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryQuest {
    pub id: String,
    pub drug: String,
    pub quantity: u32,
    pub from_location: String,
    pub to_location: String,
    pub reward: i64,
    pub expires_day: u32,
}

impl GameState {
    pub fn can_start_delivery(&self) -> bool {
        self.quest_state.active_deliveries.len() < 3
    }

    pub fn complete_delivery(&mut self, quest_id: &str) -> Option<i64> {
        if let Some(idx) = self.quest_state.active_deliveries
            .iter()
            .position(|q| q.id == quest_id && q.to_location == self.location)
        {
            let quest = self.quest_state.active_deliveries.remove(idx);
            self.quest_state.completed_deliveries += 1;
            Some(quest.reward)
        } else {
            None
        }
    }
}
```

### Pattern 6: Leaderboard with SQLite Window Functions
**What:** Use RANK() window function for efficient leaderboard queries
**When to use:** Displaying top 10 players by final score
**Example:**
```rust
// Source: SQLite window function docs
#[derive(Debug, FromRow)]
pub struct LeaderboardEntry {
    pub rank: i64,
    pub handle: String,
    pub final_score: i64,
    pub completion_date: String,
}

pub async fn get_leaderboard(pool: &SqlitePool) -> Result<Vec<LeaderboardEntry>, sqlx::Error> {
    sqlx::query_as::<_, LeaderboardEntry>(
        "SELECT
            RANK() OVER (ORDER BY final_score DESC) as rank,
            handle,
            final_score,
            completion_date
         FROM game_completions
         WHERE final_score > 0
         ORDER BY final_score DESC
         LIMIT 10"
    )
    .fetch_all(pool)
    .await
}
```

### Anti-Patterns to Avoid
- **Floating-point currency:** Use i64 for all money values to avoid rounding errors
- **Global mutable RNG:** Pass `&mut impl Rng` to functions for testability
- **Separate table per quest:** Store quest state as nested struct in GameState JSON
- **Real-time event polling:** Events trigger on discrete actions (travel, trade), not time-based
- **Saving after every action:** Auto-save on screen transitions only to reduce DB writes

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| JSON serialization | Custom save/load format | serde + serde_json | Handles nested structs, enums, Options automatically |
| Weighted random | Manual cumulative ranges | rand::distributions::WeightedIndex | O(1) sampling, well-tested algorithm |
| Leaderboard ranking | Application-level sorting | SQLite RANK() window function | Database does sorting/ranking efficiently |
| Random number generation | Linear congruential generator | rand::thread_rng() | Cryptographically secure, platform-optimized |
| Date arithmetic | Manual day counting | chrono::Duration | Handles edge cases, leap years, time zones |

**Key insight:** Door games have complex state that changes frequently during development. JSON serialization allows schema evolution without migrations. Window functions handle ranking more efficiently than application code sorting thousands of rows.

## Common Pitfalls

### Pitfall 1: Floating-Point Currency Drift
**What goes wrong:** Using f64 for money leads to rounding errors (e.g., $0.01 appearing as $0.009999)
**Why it happens:** Binary floating-point cannot represent decimal fractions exactly
**How to avoid:** Use i64 for all currency values (cents/pennies as base unit)
**Warning signs:** Test failures with assertions like `assert_eq!(balance, 1000)` failing with 999 or 1001

### Pitfall 2: Saving Partial Game State
**What goes wrong:** Saving some fields but not others leads to inconsistent state (e.g., inventory updated but cash not deducted)
**Why it happens:** Treating game state as multiple independent fields instead of atomic unit
**How to avoid:** Always save entire GameState as single JSON blob, use transactions for save + completion
**Warning signs:** Players report "lost money but didn't get item" bugs

### Pitfall 3: Not Validating Loaded State
**What goes wrong:** Old save files cause panics when schema changes (e.g., new field added to struct)
**Why it happens:** serde fails to deserialize if required field missing
**How to avoid:** Use `#[serde(default)]` on new fields, implement schema version migration
**Warning signs:** Players report "game won't load" after updates

### Pitfall 4: Random Event Overwhelm
**What goes wrong:** Too many events per travel makes game feel chaotic and unfair
**Why it happens:** Setting event probability too high (e.g., 30% instead of 15%)
**How to avoid:** Playtest extensively, consider event cooldowns, cap events per day
**Warning signs:** Playtesters say "I can't get anywhere without being robbed"

### Pitfall 5: Quest State Bloat
**What goes wrong:** GameState JSON grows to megabytes with thousands of completed quests
**Why it happens:** Storing full quest history instead of just active/counts
**How to avoid:** Only store active quests in GameState, move completed quests to separate completions table
**Warning signs:** Database queries slow down, JSON serialization takes >100ms

### Pitfall 6: Integer Overflow on Compound Interest
**What goes wrong:** Debt/bank balance overflows i64 after many days
**Why it happens:** Exponential growth without bounds checking
**How to avoid:** Cap debt/bank at i64::MAX / 2, or use saturating arithmetic
**Warning signs:** Negative balances appear when debt should be positive

### Pitfall 7: State Machine Input Buffer Confusion
**What goes wrong:** Input from previous screen leaks into next screen
**Why it happens:** Not clearing input buffer on state transitions
**How to avoid:** Clear buffer explicitly when transitioning between screens
**Warning signs:** Pressing Enter on one screen causes action on next screen

## Code Examples

Verified patterns from official sources:

### Database Schema for Game Saves
```sql
-- Store one save per user, JSON column for flexibility
CREATE TABLE IF NOT EXISTS game_saves (
    user_id INTEGER PRIMARY KEY,
    state_json TEXT NOT NULL,
    last_saved TEXT NOT NULL DEFAULT (datetime('now', '-5 hours')),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Completions for leaderboard (only written when game ends)
CREATE TABLE IF NOT EXISTS game_completions (
    id INTEGER PRIMARY KEY,
    user_id INTEGER NOT NULL,
    handle TEXT NOT NULL,
    final_score INTEGER NOT NULL,
    days_played INTEGER NOT NULL,
    completion_date TEXT NOT NULL DEFAULT (datetime('now', '-5 hours')),
    story_completed INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_game_completions_score
    ON game_completions(final_score DESC);
```

### Service Registration
```rust
// In services/registry.rs, add to service factory:
use crate::services::grand_theft_meth::GrandTheftMethService;

let service: Arc<dyn Service> = match service_config.name.as_str() {
    "grand_theft_meth" => Arc::new(GrandTheftMethService::new(state.clone())),
    // ... other services
};
```

### Price Volatility Algorithm
```rust
// Generate market prices with regional variation and volatility
pub fn generate_market_prices(
    rng: &mut impl Rng,
    location: &str,
    day: u32,
) -> HashMap<String, i64> {
    let mut prices = HashMap::new();

    // Base prices (cents)
    let base = [
        ("Cocaine", 15000, 30000),
        ("Heroin", 5000, 14000),
        ("Acid", 1500, 4500),
        ("Weed", 300, 900),
        ("Meth", 2000, 5500),
        ("Speed", 90, 250),
        ("Ludes", 11, 60),
    ];

    for (drug, min, max) in base {
        // Base price in range
        let base_price = rng.gen_range(min..=max);

        // Regional modifier (some cities don't have certain drugs)
        let regional_mult = match (location, drug) {
            ("Tokyo", "Meth") => 1.5,      // High demand
            ("London", "Cocaine") => 0.7,  // Oversupply
            _ => 1.0,
        };

        // Random volatility ±20%
        let volatility = rng.gen_range(0.8..=1.2);

        let final_price = ((base_price as f64) * regional_mult * volatility) as i64;
        prices.insert(drug.to_string(), final_price);
    }

    prices
}
```

### Combat Resolution
```rust
pub struct CombatResult {
    pub player_won: bool,
    pub player_damage: u32,
    pub loot_cash: i64,
    pub loot_items: HashMap<String, u32>,
    pub notoriety_change: i32,
}

pub fn resolve_combat(
    player_health: u32,
    player_weapon: &Weapon,
    enemy: &Enemy,
    rng: &mut impl Rng,
) -> CombatResult {
    // Simple combat formula: weapon damage ± 20% randomness
    let player_attack = (player_weapon.damage as f64 * rng.gen_range(0.8..=1.2)) as u32;
    let enemy_attack = (enemy.damage as f64 * rng.gen_range(0.8..=1.2)) as u32;

    // Determine winner (simplified: higher attack wins)
    let player_won = player_attack > enemy_attack;

    CombatResult {
        player_won,
        player_damage: if player_won { enemy_attack / 2 } else { enemy_attack },
        loot_cash: if player_won { enemy.loot_cash } else { 0 },
        loot_items: if player_won { enemy.loot_items.clone() } else { HashMap::new() },
        notoriety_change: if player_won { 5 } else { -2 },
    }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Separate SQL columns per field | JSON state column | ~2020 | Easier schema evolution, atomic saves |
| Global rand() function | Thread-local RNG | Rust 1.0 (2015) | Better performance, thread-safe |
| Manual cumulative ranges | WeightedIndex | rand 0.7 (2019) | O(1) sampling, cleaner code |
| LIMIT + OFFSET pagination | Window functions | SQLite 3.25 (2018) | Efficient ranking without full table scan |

**Deprecated/outdated:**
- **pickle/bincode for saves:** Human-readable JSON preferred for debugging and manual editing
- **f64 currency:** Integer arithmetic standard in financial software
- **Custom state machine macros:** Type-driven state machines cleaner than macro-generated code

## Open Questions

Things that couldn't be fully resolved:

1. **Precise daily compound interest formula**
   - What we know: Standard formula is A = P(1 + r/n)^(nt), daily compounding means n=365
   - What's unclear: Whether to use 365 or 366 for leap years, and exact basis point conversion
   - Recommendation: Use simplified (10000 + rate_bp) / 10000 per day, close enough for game purposes

2. **Quest completion vs. net worth scoring**
   - What we know: Game ends at day 90, leaderboard shows top players
   - What's unclear: Should story completion boost score, or is it purely net worth?
   - Recommendation: Story completion unlocks "Kingpin" title but leaderboard ranks by net worth only

3. **Save corruption recovery**
   - What we know: JSON deserialization can fail if schema changes
   - What's unclear: Should we version saves and migrate, or just let player restart?
   - Recommendation: Version 1.0 MVP can fail gracefully with "corrupt save" message, add migration in 1.1

4. **Multiple save slots vs. single slot**
   - What we know: Context says "one save slot per user"
   - What's unclear: Should user be able to clear and restart mid-game?
   - Recommendation: Single slot with "Clear Save" option in main menu

## Sources

### Primary (HIGH confidence)
- [rand crate official docs](https://docs.rs/rand/latest/rand/) - RNG patterns and weighted sampling
- [serde_json official docs](https://docs.rs/serde_json/latest/serde_json/) - Serialization methods
- [SQLite window functions tutorial](https://www.sqlitetutorial.net/sqlite-window-functions/sqlite-rank/) - RANK() usage
- Project codebase: ComposeFlow pattern (services/mail.rs), GameState examples

### Secondary (MEDIUM confidence)
- [Rust state machine patterns (Hoverbear)](https://hoverbear.org/blog/rust-state-machine-pattern/) - Type-driven state machines
- [BBS door game architecture (Break Into Chat)](https://breakintochat.com/wiki/BBS_door_game) - Historical door game patterns
- [Rust Rand Book](https://rust-random.github.io/book/guide-dist.html) - Distribution usage patterns
- [High Performance SQLite - Ranking](https://highperformancesqlite.com/watch/ranking-results) - Leaderboard optimization

### Tertiary (LOW confidence)
- [CICAL compound interest crate](https://crates.io/crates/cical) - Example compound interest implementation (not using, but validates approach)
- [Rust quest system patterns](https://umod.org/plugins/quests) - From Rust game plugin (different domain but similar patterns)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - All libraries already in use or standard Rust ecosystem
- Architecture: HIGH - Patterns verified in existing codebase (mail.rs, news.rs)
- Pitfalls: MEDIUM - Based on general Rust/SQLite knowledge, not game-specific testing
- Code examples: MEDIUM-HIGH - Synthesized from docs and existing patterns, not tested in-game

**Research date:** 2026-01-28
**Valid until:** 2026-02-28 (30 days - stable ecosystem, no major changes expected)
