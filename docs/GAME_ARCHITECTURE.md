# BBS Door Games - Shared Architecture

This document defines the required **technical patterns** and **infrastructure** for ALL door games in The Construct BBS.

**Important**: This covers architecture, not aesthetics. Each game should have its own unique visual identity, theme, color palette, and personality. The shared patterns here ensure games integrate properly with the BBS - how each game looks and feels is up to the game's design.

---

## Directory Structure

Every game follows the same folder structure:

```
backend/src/
├── games/
│   ├── mod.rs                      # Game registry (add your game here)
│   └── {game_name}/                # Game logic (state machine, data, rendering)
│       ├── mod.rs                  # Public exports
│       ├── data.rs                 # Static game data (items, locations, enemies, etc.)
│       ├── state.rs                # GameState struct - player's persistent state
│       ├── screen.rs               # GameScreen enum + Flow state machine
│       ├── render.rs               # ANSI rendering functions
│       └── [optional modules]      # economy.rs, events.rs, combat.rs, etc.
│
└── services/
    └── {game_name}/                # Session routing & persistence
        ├── mod.rs                  # Public exports (SENTINEL, start_game, etc.)
        ├── service.rs              # Session entry points, save/load coordination
        └── db.rs                   # Game's SQLite database (separate from bbs.db)
```

---

## Required Components

### 1. Session Sentinel

Every game needs a unique sentinel string for session routing:

```rust
// services/{game_name}/service.rs
pub const SENTINEL: &str = "__game_{short_name}__";
```

Pattern: `__game_{abbreviation}__` (e.g., `__game_gtm__`, `__game_lord__`, `__game_chess__`)

### 2. GameScreen Enum

Defines all possible screens/states the player can be in:

```rust
// games/{game_name}/screen.rs
#[derive(Debug, Clone, PartialEq)]
pub enum GameScreen {
    Intro,              // Always have an intro screen
    MainMenu,           // Main hub
    // ... game-specific screens
    GameOver,           // End state
    ConfirmQuit,        // Exit confirmation
}
```

### 3. GameState Struct

The serializable player state. MUST be `Serialize`/`Deserialize`:

```rust
// games/{game_name}/state.rs
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    // Player identity (optional, for display)
    pub handle: Option<String>,

    // Game-specific state...

    // Message to display on next render (cleared after showing)
    pub last_message: Option<String>,
}
```

### 4. GameFlow State Machine

Manages screen transitions and input handling:

```rust
// games/{game_name}/screen.rs
pub struct {Game}Flow {
    pub state: GameState,
    pub screen: GameScreen,
    // Additional transient state (not persisted)
    input_buffer: String,
}

impl {Game}Flow {
    pub fn new() -> Self;
    pub fn from_state(state: GameState) -> Self;
    pub fn current_screen(&self) -> &GameScreen;
    pub fn game_state(&self) -> &GameState;
    pub fn handle_char(&mut self, ch: char) -> GameAction;
}
```

### 5. GameAction Enum

Actions returned by input handler for session to process:

```rust
// games/{game_name}/screen.rs
#[derive(Debug, Clone)]
pub enum GameAction {
    Continue,                           // No output needed
    Render(String),                     // Show screen output
    Echo(String),                       // Echo characters back
    SaveGame,                           // Trigger DB save
    GameOver { final_score: i64, ... }, // Game ended
    Quit,                               // Return to BBS menu
}
```

---

## Rendering Standards

### Use AnsiWriter

All rendering MUST use `AnsiWriter` from `crate::terminal`:

```rust
use crate::terminal::{AnsiWriter, Color};

pub fn render_main_menu(state: &GameState) -> String {
    let mut w = AnsiWriter::new();

    w.clear_screen();
    w.set_fg(Color::LightCyan);
    w.writeln("Game Title");
    w.reset_color();

    w.flush()
}
```

### Screen Dimensions

- **Target**: 80 columns x 24 rows (standard BBS terminal)
- **Never exceed 80 columns** - users on real terminals will have wrapping issues
- Leave 1-2 rows for input prompts at bottom

### Color Palette

Use the 16-color CGA palette via `Color` enum:
- `Black`, `Red`, `Green`, `Brown`, `Blue`, `Magenta`, `Cyan`, `LightGray`
- `DarkGray`, `LightRed`, `LightGreen`, `Yellow`, `LightBlue`, `LightMagenta`, `LightCyan`, `White`

### Visual Identity

**Each game should have its own distinct visual theme.** Don't copy colors from other games.

Examples of thematic approaches:
- **Dragon Slayer**: Warm medieval palette (browns, reds, gold)
- **Star Trader**: Cool sci-fi palette (cyans, blues, greens)
- **Xodia MUD**: Mystical fantasy (purples, magentas, deep blues)
- **Summit**: Natural outdoors (greens, browns, white for snow)
- **Dystopia**: Dark political (grays, muted colors, red accents)

Define your game's palette early and be consistent within the game.

### Status Display

Games with persistent stats should display key info consistently, but the format is up to the game's design:

- **Inline status bar**: Single line at top/bottom (good for action games)
- **Side panel**: Stats in a column (good for RPGs with many stats)
- **Header block**: Multi-line header area (good for trading/strategy games)
- **Contextual**: Only show relevant stats per screen

Choose what fits your game's pacing and information density.

### Box Drawing

Use Unicode box drawing (converted from CP437):
- Corners: `┌ ┐ └ ┘`
- Lines: `─ │`
- Junctions: `├ ┤ ┬ ┴ ┼`
- Double: `╔ ╗ ╚ ╝ ═ ║` (for emphasis)

---

## Database Patterns

### Separate Database per Game

Each game has its own SQLite database file in `data/`:

```rust
// services/{game_name}/db.rs
pub struct {Game}Db {
    pool: SqlitePool,
}

impl {Game}Db {
    pub async fn new(db_path: &Path) -> Result<Self, sqlx::Error> {
        // Create parent directory
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await?;

        // Enable WAL mode
        sqlx::query("PRAGMA journal_mode=WAL")
            .execute(&pool)
            .await?;

        let db = Self { pool };
        db.init_schema().await?;
        Ok(db)
    }
}
```

Database path pattern: `data/{game_name}.db`

### Required Tables

Every game database should have at minimum:

```sql
-- Active saves (one per user)
CREATE TABLE IF NOT EXISTS saves (
    user_id INTEGER PRIMARY KEY,
    handle TEXT NOT NULL,
    state_json TEXT NOT NULL,
    last_saved TEXT NOT NULL
);

-- Completed games (for leaderboards)
CREATE TABLE IF NOT EXISTS completions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    handle TEXT NOT NULL,
    final_score INTEGER NOT NULL,
    completed_at TEXT NOT NULL
    -- Add game-specific columns as needed
);
```

### Required DB Methods

```rust
impl {Game}Db {
    pub async fn save_game(&self, user_id: i64, handle: &str, state_json: &str) -> Result<(), sqlx::Error>;
    pub async fn load_game(&self, user_id: i64) -> Result<Option<String>, sqlx::Error>;
    pub async fn delete_save(&self, user_id: i64) -> Result<(), sqlx::Error>;
    pub async fn record_completion(...) -> Result<(), sqlx::Error>;
    pub async fn get_leaderboard(&self, limit: i64) -> Result<Vec<LeaderboardEntry>, sqlx::Error>;
}
```

---

## Service Entry Points

### Required Functions

```rust
// services/{game_name}/service.rs

/// Initialize or resume a game session
pub async fn start_game(
    db: &{Game}Db,
    user_id: i64,
    handle: &str
) -> Result<({Game}Flow, String), String>;

/// Save current game state
pub async fn save_game_state(
    db: &{Game}Db,
    user_id: i64,
    handle: &str,
    flow: &{Game}Flow
) -> Result<(), String>;

/// Render current screen
pub fn render_screen(flow: &{Game}Flow) -> String;

/// Record game completion and delete save
pub async fn record_game_completion(
    db: &{Game}Db,
    user_id: i64,
    handle: &str,
    flow: &{Game}Flow,
) -> Result<(), String>;

/// Get leaderboard entries
pub async fn get_game_leaderboard(db: &{Game}Db) -> Vec<LeaderboardEntry>;
```

---

## Input Handling

### Character-by-Character

Games receive input character-by-character via `handle_char()`:

```rust
pub fn handle_char(&mut self, ch: char) -> GameAction {
    // Backspace
    if ch == '\x7f' || ch == '\x08' {
        if self.input_buffer.pop().is_some() {
            return GameAction::Echo("\x08 \x08".to_string());
        }
        return GameAction::Continue;
    }

    // Enter
    if ch == '\r' || ch == '\n' {
        return self.process_input();
    }

    // Regular character - add to buffer or process immediately
    // ...
}
```

### Immediate vs Buffered Input

- **Immediate**: Single-key menus (press `1` to buy, `Q` to quit)
- **Buffered**: Text input (names, quantities, messages)

### Common Hotkeys

Reserve these across all games:
- `Q` - Quit/back to previous screen
- `?` or `H` - Help
- `L` - Leaderboard (if applicable)
- Numbers `1-9`, `0` - Menu selections
- `Y`/`N` - Confirmations

---

## Game Types & Patterns

### Single-Player Turn-Based

Most games (Dragon Slayer, Grand Theft Meth, Last Dream):
- One active save per user
- Autosave after significant actions
- Daily/turn limits stored in GameState
- Check limits on action, not on login

### Daily Puzzle

Games like Sudoku, Queens:
- Same puzzle for all users (date-seeded)
- Track streaks and completion times
- No save needed - state is date + completion status

### Async Multiplayer

Games like Chess, Dystopia, Star Trader:
- Game state stored in shared tables
- Player states linked to game_id
- Timeout handling for inactive players
- Notification of opponent moves

### Real-Time Multiplayer

Games like Summit, Acromania, Tanks:
- WebSocket-based synchronization
- Lobby/matchmaking system
- Session-based (not persistent saves)
- Spectator mode consideration

---

## Daily Limits & Turn Systems

### Turn-Based Games

```rust
#[derive(Serialize, Deserialize)]
pub struct GameState {
    pub turns_remaining: u32,
    pub last_turn_date: String,  // "2026-01-30"
}

impl GameState {
    pub fn check_new_day(&mut self) -> bool {
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        if self.last_turn_date != today {
            self.last_turn_date = today;
            self.turns_remaining = DAILY_TURNS;
            return true;
        }
        false
    }
}
```

### Action Costs

Be consistent within each game about what costs turns:
- Combat: Usually costs turns
- Travel: Usually costs turns
- Shopping/inventory: Usually free
- Viewing stats/leaderboard: Always free

---

## Multiplayer Considerations

### Shared World State

For games where players interact:
- Use database transactions for atomic updates
- Handle race conditions (two players attacking same target)
- Consider optimistic locking

### Async Turn Resolution

```rust
pub struct GameTurn {
    pub game_id: i64,
    pub turn_number: i64,
    pub deadline: DateTime<Utc>,
    pub submitted: HashMap<i64, bool>,  // user_id -> submitted
}
```

### Real-Time Sync

For real-time games, define message types:
```rust
pub enum GameMessage {
    PlayerJoined { user_id: i64, handle: String },
    PlayerMoved { user_id: i64, x: i32, y: i32 },
    PlayerAction { user_id: i64, action: String },
    GameState { ... },
}
```

---

## Leaderboard Standards

Every game should have a leaderboard. Required data structure:

```rust
pub struct LeaderboardEntry {
    pub rank: i64,
    pub handle: String,
    pub score: i64,
    pub date: String,
    // Game-specific fields (time, level reached, etc.)
}
```

Display format is up to each game's visual design - match the game's aesthetic.

---

## Registration Checklist

When adding a new game:

**Code Structure**
1. [ ] Create `games/{game_name}/` folder with required modules
2. [ ] Add `pub mod {game_name};` to `games/mod.rs`
3. [ ] Create `services/{game_name}/` folder with service and db
4. [ ] Add `pub mod {game_name};` to `services/mod.rs`
5. [ ] Add `{game_name}_db: Arc<{Game}Db>` to `AppState` in `main.rs`
6. [ ] Initialize database in `main.rs`
7. [ ] Add menu entry in `config.toml`
8. [ ] Add session routing case in `websocket/session.rs`

**Testing (Required before merge)**
9. [ ] Unit tests for state serialization
10. [ ] Unit tests for screen transitions
11. [ ] Unit tests for input handling
12. [ ] Unit tests for core game logic
13. [ ] Playwright E2E test file: `e2e/tests/{game_name}.spec.ts`
14. [ ] E2E tests for new game flow
15. [ ] E2E tests for core gameplay loop
16. [ ] E2E tests for save/resume
17. [ ] E2E tests for game completion
18. [ ] All tests passing: `cargo test && npx playwright test`

---

## Testing Requirements

### Unit Tests (Required)

Every game MUST have comprehensive unit tests in Rust. Place tests in the same file or in a `tests` submodule.

**Required test coverage:**

1. **State serialization**: `GameState` round-trips through JSON without data loss
2. **Screen transitions**: All `GameScreen` variants are reachable and transitions are valid
3. **Input handling**: All input paths produce expected `GameAction` results
4. **Game logic**: Core mechanics (combat, economy, scoring, etc.) behave correctly
5. **Edge cases**: Empty inventory, zero health, max values, boundary conditions
6. **Render output**: No panics, output contains expected elements

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_serialization() {
        let state = GameState::new();
        let json = serde_json::to_string(&state).unwrap();
        let restored: GameState = serde_json::from_str(&json).unwrap();
        assert_eq!(state.cash, restored.cash);
        assert_eq!(state.inventory, restored.inventory);
    }

    #[test]
    fn test_screen_transitions() {
        let mut flow = GameFlow::new();
        assert_eq!(*flow.current_screen(), GameScreen::Intro);

        // Press enter to advance from intro
        flow.handle_char('\r');
        assert_eq!(*flow.current_screen(), GameScreen::MainMenu);
    }

    #[test]
    fn test_combat_damage_calculation() {
        let mut state = GameState::new();
        state.health = 100;
        state.apply_damage(25);
        assert_eq!(state.health, 75);
    }
}
```

Run with: `cargo test -p bbs-backend {game_name}::`

### End-to-End Tests (Required)

Every game MUST have Playwright automation tests that verify the complete user experience through the terminal UI.

**Location**: `e2e/tests/{game_name}.spec.ts`

**Required E2E coverage:**

1. **Game launch**: Menu navigation to start the game
2. **New game flow**: Character creation, intro screens, initial state
3. **Core gameplay loop**: Primary game mechanics work end-to-end
4. **Save/resume**: Exit game, return, verify state persisted
5. **Game completion**: Win condition, game over, leaderboard entry
6. **Edge cases**: Invalid input handling, quit confirmation

```typescript
// e2e/tests/dragon_slayer.spec.ts
import { test, expect } from '@playwright/test';
import { BbsTerminal } from '../helpers/terminal';

test.describe('Dragon Slayer', () => {
  let terminal: BbsTerminal;

  test.beforeEach(async ({ page }) => {
    terminal = new BbsTerminal(page);
    await terminal.connect();
    await terminal.login('testuser', 'testpass');
  });

  test('can start new game', async () => {
    await terminal.sendKeys('G'); // Games menu
    await terminal.sendKeys('2'); // Dragon Slayer
    await terminal.expectText('Welcome to Dragon Slayer');
    await terminal.sendKeys('\r'); // Start
    await terminal.expectText('The Red Dragon');
  });

  test('forest combat works', async () => {
    await terminal.navigateTo('dragon_slayer');
    await terminal.sendKeys('F'); // Forest
    await terminal.expectText(['Attack', 'Run', 'Skill']);
    await terminal.sendKeys('A'); // Attack
    await terminal.expectText(['damage', 'hit', 'miss']);
  });

  test('game state persists after quit', async () => {
    await terminal.navigateTo('dragon_slayer');
    // Get some gold
    await terminal.sendKeys('F'); // Forest
    await terminal.completeEncounter();
    const gold = await terminal.extractValue(/Gold: (\d+)/);

    // Quit and return
    await terminal.sendKeys('Q');
    await terminal.sendKeys('Y'); // Confirm
    await terminal.navigateTo('dragon_slayer');

    // Verify state restored
    await terminal.expectText(`Gold: ${gold}`);
  });

  test('leaderboard shows after game over', async () => {
    // ... complete game or simulate game over
    await terminal.expectText('HIGH SCORES');
  });
});
```

**Playwright helpers** (`e2e/helpers/terminal.ts`):

```typescript
export class BbsTerminal {
  constructor(private page: Page) {}

  async connect() {
    await this.page.goto('http://localhost:3000');
    await this.page.waitForSelector('.xterm-screen');
  }

  async sendKeys(keys: string) {
    await this.page.keyboard.type(keys);
    await this.page.waitForTimeout(100); // Allow render
  }

  async expectText(text: string | string[]) {
    const texts = Array.isArray(text) ? text : [text];
    const content = await this.getScreenContent();
    const found = texts.some(t => content.includes(t));
    expect(found).toBe(true);
  }

  async getScreenContent(): Promise<string> {
    return await this.page.evaluate(() => {
      // Extract text from xterm.js terminal
      const terminal = (window as any).terminal;
      return terminal.buffer.active.getLine(0)?.translateToString() || '';
    });
  }

  async navigateTo(game: string) {
    // Navigate from main menu to specific game
    await this.sendKeys('G'); // Games
    // ... game-specific navigation
  }
}
```

Run with: `npx playwright test {game_name}`

### Test Data & Fixtures

- Use deterministic seeds for random elements in tests
- Create fixture states for common scenarios (mid-game, near-death, wealthy, etc.)
- Mock time-based features (daily limits) for testability

### CI Integration

All tests must pass before merge:
- `cargo test` - All unit tests
- `npx playwright test` - All E2E tests

---

## Future Considerations

### IGM (In-Game Modules) Support

Plan for extensibility - some games (Dragon Slayer, Star Trader) historically supported plug-in modules. Consider:
- Hook points for custom content
- Data-driven event/location systems
- Scriptable NPCs

### Cross-Game Features

- Unified achievement system
- Inter-game economy (transfer credits?)
- Player profiles visible across games
- Shared friends/social layer

---

*Last updated: 2026-01-30*
