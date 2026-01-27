# Phase 3: Navigation System - Research

**Researched:** 2026-01-27
**Domain:** Menu navigation system with state management, config-driven architecture, type-ahead buffering
**Confidence:** MEDIUM-HIGH

## Summary

Phase 3 implements a Wildcat-style hierarchical menu navigation system with hotkey controls, ANSI art headers, and type-ahead command stacking. The research focused on five key technical domains:

1. **State management patterns** - Enum-based state machines are the idiomatic Rust approach for representing menu states (main menu vs. submenu with parent context). Type-state patterns exist but add complexity without benefit for a 2-level hierarchy.

2. **Config-driven menus** - TOML with internally-tagged enums (`#[serde(tag = "type")]`) provides clean sysop-readable configuration. Menu items as enums (Service/Submenu/Command variants) with ordering and level-gating.

3. **Type-ahead buffering** - `VecDeque<char>` is the standard library solution for FIFO input buffering with O(1) push_back/pop_front operations. No external crate needed.

4. **ANSI rendering** - Existing `AnsiWriter` infrastructure already supports CP437 box-drawing characters used in Phase 2. Double-line borders for main menu, single-line for submenus creates visual hierarchy.

5. **Quote rotation** - `rand` crate (current: 0.9.2) with `choose()` method for random selection from embedded quote array. Session-scoped or time-based rotation avoids external dependencies.

**Primary recommendation:** Use enum-based menu state with `VecDeque<char>` for type-ahead buffer, internally-tagged serde enums for menu config, and `rand::choose()` for Stoic quote rotation.

## Standard Stack

The established libraries/tools for this domain:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `serde` | 1.x (in use) | Config deserialization | Already in use, industry standard for config |
| `toml` | 0.9.11+spec-1.1.0 | TOML parsing | Already in use (0.8), upgrade for latest spec |
| `std::collections::VecDeque` | stdlib | Type-ahead buffer | Built-in, O(1) FIFO operations, zero dependencies |
| `rand` | 0.9.2 | Random quote selection | Standard RNG crate, `choose()` method for slices |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `codepage-437` | 0.1 (in use) | CP437 box-drawing | Already rendering double-line borders in Phase 2 |
| `chrono` | 0.4 (in use) | Time-based rotation | Optional: if rotating quotes by time-of-day |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Internally-tagged enums | Externally-tagged (default) | Verbose TOML syntax: `[[menu]] Submenu = { name = "..." }` vs `[[menu]] type = "submenu"` |
| `VecDeque` | `ringbuffer` crate | Adds dependency for fixed-size, but type-ahead rarely exceeds ~10 chars |
| Enum state machine | Type-state pattern | Compile-time guarantees, but overkill for 2-level menu hierarchy |

**Installation:**
```bash
# Already in Cargo.toml: serde, toml (0.8), codepage-437, chrono
cargo add rand@0.9
cargo upgrade toml@0.9  # Optional: stay on 0.8 if working
```

## Architecture Patterns

### Recommended Project Structure
```
src/
├── services/
│   ├── mod.rs           # Service trait (existing)
│   ├── registry.rs      # ServiceRegistry (existing)
│   └── menu.rs          # NEW: MenuService implementing Service trait
├── menu/                # NEW: menu-specific logic
│   ├── mod.rs           # Public API
│   ├── state.rs         # MenuState enum (MainMenu, Submenu)
│   ├── config.rs        # MenuItem enum, config deserialization
│   ├── render.rs        # Menu rendering with AnsiWriter
│   └── quotes.rs        # Stoic quote list and selection
├── config.rs            # Add [menu] section
└── terminal/
    └── ansi.rs          # AnsiWriter (existing)
```

### Pattern 1: Enum-Based Menu State Machine
**What:** Represent menu location as enum with variants carrying state-specific data.
**When to use:** Navigational state with finite set of screens (main menu, submenus).
**Example:**
```rust
// Source: https://blog.yoshuawuyts.com/state-machines-2/ (enum-based design)
pub enum MenuState {
    MainMenu,
    Submenu {
        parent_name: String,
        submenu_key: String,
    },
}

impl MenuState {
    pub fn handle_input(&mut self, key: char, config: &MenuConfig) -> MenuAction {
        match self {
            MenuState::MainMenu => {
                // Match against main menu items
                if let Some(item) = config.main_menu_item(key) {
                    match item {
                        MenuItem::Submenu { key, name, .. } => {
                            *self = MenuState::Submenu {
                                parent_name: "Main Menu".to_string(),
                                submenu_key: key.clone(),
                            };
                            MenuAction::RedrawMenu
                        }
                        MenuItem::Service { service_name, .. } => {
                            MenuAction::LaunchService(service_name.clone())
                        }
                        MenuItem::Command { command, .. } => {
                            MenuAction::ExecuteCommand(command.clone())
                        }
                    }
                } else {
                    MenuAction::InvalidInput
                }
            }
            MenuState::Submenu { submenu_key, .. } => {
                if key.eq_ignore_ascii_case(&'q') {
                    *self = MenuState::MainMenu;
                    MenuAction::RedrawMenu
                } else {
                    // Match submenu items
                    MenuAction::Continue
                }
            }
        }
    }
}
```

### Pattern 2: Type-Ahead Buffer with VecDeque
**What:** FIFO queue for buffering keypresses during menu transitions.
**When to use:** Command stacking (e.g., "G1" = Games submenu + item 1).
**Example:**
```rust
// Source: https://doc.rust-lang.org/std/collections/struct.VecDeque.html
use std::collections::VecDeque;

pub struct TypeAheadBuffer {
    buffer: VecDeque<char>,
    max_size: usize,
}

impl TypeAheadBuffer {
    pub fn new(max_size: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    pub fn push(&mut self, ch: char) {
        if self.buffer.len() >= self.max_size {
            // Drop oldest if full (prevent memory exhaustion)
            self.buffer.pop_front();
        }
        self.buffer.push_back(ch);
    }

    pub fn pop(&mut self) -> Option<char> {
        self.buffer.pop_front()  // O(1) FIFO dequeue
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}
```

### Pattern 3: Config-Driven Menu Items with Internally-Tagged Enums
**What:** Serde enum with type discriminator field for menu item types.
**When to use:** Sysop-editable menu configuration in TOML.
**Example:**
```rust
// Source: https://serde.rs/enum-representations.html (internally tagged)
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum MenuItem {
    Service {
        hotkey: char,
        name: String,
        service_name: String,
        #[serde(default)]
        min_level: u8,
        #[serde(default)]
        order: i32,
    },
    Submenu {
        hotkey: char,
        name: String,
        submenu_key: String,
        #[serde(default)]
        min_level: u8,
        #[serde(default)]
        order: i32,
    },
    Command {
        hotkey: char,
        name: String,
        command: String,  // "quit", "profile", etc.
        #[serde(default)]
        min_level: u8,
        #[serde(default)]
        order: i32,
    },
}

// In config.toml:
// [[menu.main]]
// type = "submenu"
// hotkey = "G"
// name = "Games"
// submenu_key = "games"
// order = 1
```

### Pattern 4: Random Quote Selection
**What:** Use `rand::prelude::*` with `choose()` for random element from slice.
**When to use:** Rotating MOTD quotes without external API.
**Example:**
```rust
// Source: https://docs.rs/rand (version 0.9.2)
use rand::prelude::*;

const STOIC_QUOTES: &[&str] = &[
    "The obstacle is the way. — Marcus Aurelius",
    "It is not that we have a short time to live, but that we waste a lot of it. — Seneca",
    "He who fears death will never do anything worthy of a living man. — Seneca",
    // ... more quotes
];

pub fn random_stoic_quote() -> &'static str {
    let mut rng = rand::rng();
    STOIC_QUOTES.choose(&mut rng).unwrap_or(&STOIC_QUOTES[0])
}
```

### Pattern 5: Menu Rendering with AnsiWriter
**What:** Reuse existing AnsiWriter pattern from Phase 2 profile cards.
**When to use:** All menu screen rendering with box-drawing borders.
**Example:**
```rust
// Source: Existing backend/src/services/profile.rs pattern
use crate::terminal::{AnsiWriter, Color};

pub fn render_menu_header(title: &str, border_style: BorderStyle) -> String {
    let mut w = AnsiWriter::new();
    let inner = 78; // 80 - 2 border chars

    let (top_left, top_right, horizontal, vertical, bottom_left, bottom_right) = match border_style {
        BorderStyle::Double => ('\u{2554}', '\u{2557}', '\u{2550}', '\u{2551}', '\u{255A}', '\u{255D}'),
        BorderStyle::Single => ('\u{250C}', '\u{2510}', '\u{2500}', '\u{2502}', '\u{2514}', '\u{2518}'),
    };

    // Top border
    w.set_fg(Color::LightCyan);
    w.writeln(&format!("{}{}{}", top_left, horizontal.to_string().repeat(inner), top_right));

    // Title line
    let padding = (inner - title.len()) / 2;
    w.write_str(&format!("{}", vertical));
    w.write_str(&" ".repeat(padding));
    w.set_fg(Color::Yellow);
    w.bold();
    w.write_str(title);
    w.reset_color();
    w.set_fg(Color::LightCyan);
    w.write_str(&" ".repeat(inner - padding - title.len()));
    w.writeln(&format!("{}", vertical));

    // Bottom border
    w.writeln(&format!("{}{}{}", bottom_left, horizontal.to_string().repeat(inner), bottom_right));
    w.reset_color();

    w.flush()
}
```

### Anti-Patterns to Avoid
- **Async state machines:** Menu navigation is synchronous input/output; async adds complexity without benefit.
- **Global mutable state:** Pass `&mut MenuState` through handlers instead of static mut or unsafe globals.
- **Unbounded type-ahead buffer:** Cap at 10-20 characters to prevent memory exhaustion from rapid input.
- **Panic on invalid input:** Return `MenuAction::InvalidInput` and redraw menu, don't unwrap() or panic!().
- **Deeply nested match arms:** Extract submenu handling to separate methods, keep state machine logic flat.

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| FIFO input buffer | Custom circular buffer | `std::collections::VecDeque` | O(1) operations, battle-tested, zero dependencies |
| Random selection | Custom RNG or modulo arithmetic | `rand::choose()` | Uniform distribution, handles edge cases, standard crate |
| TOML parsing | String parsing and regex | `toml` + `serde` | Spec-compliant, error messages, type safety |
| Enum deserialization | Custom string matching | `#[serde(tag = "type")]` | Compile-time validation, exhaustive matching |
| Box-drawing characters | Hardcoded strings | `codepage_437` constants + AnsiWriter | Already in use, consistent rendering |

**Key insight:** Rust's standard library provides `VecDeque` specifically for this use case. The rand crate is ubiquitous (41M+ downloads). Don't reinvent these wheels.

## Common Pitfalls

### Pitfall 1: Race Conditions in Type-Ahead Buffer
**What goes wrong:** Keys arrive during menu transition, processed before new menu renders, causing invalid input handling.
**Why it happens:** Character-by-character input from websocket arrives asynchronously; menu state transitions aren't atomic with buffer consumption.
**How to avoid:** Process type-ahead buffer AFTER menu redraw completes, not during state transition.
**Warning signs:** Keys "disappear" when typing fast, or wrong menu items activate.
**Example:**
```rust
// BAD: Consume buffer during transition
*state = MenuState::Submenu { ... };
process_typeahead_buffer(&mut buffer, state);  // State changed, buffer keys are for OLD menu!

// GOOD: Transition, render, THEN consume buffer
*state = MenuState::Submenu { ... };
render_submenu(writer, state);
// NOW process buffered keys against the new menu
process_typeahead_buffer(&mut buffer, state);
```

### Pitfall 2: Untagged Enum Deserialization Slowness
**What goes wrong:** TOML parsing becomes slow or produces confusing errors with `#[serde(untagged)]`.
**Why it happens:** Serde tries to deserialize against EACH variant sequentially until one succeeds; with multiple variants, this means 3-4 failed attempts per menu item.
**How to avoid:** Use internally-tagged (`#[serde(tag = "type")]`) or adjacently-tagged enums for menu config.
**Warning signs:** Config loading takes >100ms for ~20 menu items, or error messages say "expected X, found Y" for wrong variant.

### Pitfall 3: Case-Sensitive Hotkey Matching
**What goes wrong:** User types 'q' but code only matches 'Q', or vice versa. BBS feels broken.
**Why it happens:** Direct `char` equality: `key == 'Q'` doesn't match 'q'.
**How to avoid:** Always use `key.eq_ignore_ascii_case(&'q')` or `key.to_ascii_uppercase() == 'Q'` for hotkey matching.
**Warning signs:** Commands work inconsistently, users report "Q doesn't quit".

### Pitfall 4: Forgetting Level-Gating During Rendering
**What goes wrong:** Menu items for Sysops appear for regular users, causing "access denied" errors when selected.
**Why it happens:** Level check happens on input handling, but rendering shows all items from config.
**How to avoid:** Filter menu items by user level BEFORE rendering the menu screen.
**Warning signs:** Users see options they can't access, confusion about available features.

### Pitfall 5: Mutating State During Rendering
**What goes wrong:** Rendering code changes `MenuState` or selects a new quote, causing re-entrancy issues or incorrect state.
**Why it happens:** Mixing rendering (should be pure) with state updates (should be in input handlers).
**How to avoid:** Pass `&MenuState` (immutable reference) to rendering functions, return `MenuAction` from input handlers, update state only in top-level loop.
**Warning signs:** Menu redraws change state, duplicate actions, hard-to-reproduce bugs.

## Code Examples

Verified patterns from official sources:

### Stoic Quote Rotation (Session-Scoped)
```rust
// Source: https://docs.rs/rand (version 0.9.2)
use rand::prelude::*;

pub struct QuoteRotator {
    quotes: Vec<&'static str>,
    current: String,
}

impl QuoteRotator {
    pub fn new() -> Self {
        let quotes = vec![
            "The obstacle is the way. — Marcus Aurelius",
            "It is not that we have a short time to live, but that we waste a lot of it. — Seneca",
            "He who fears death will never do anything worthy of a living man. — Seneca",
            "Waste no more time arguing what a good man should be. Be one. — Marcus Aurelius",
            "If it is not right, do not do it. If it is not true, do not say it. — Marcus Aurelius",
        ];
        let mut rng = rand::rng();
        let current = quotes.choose(&mut rng).unwrap_or(&quotes[0]).to_string();
        Self { quotes, current }
    }

    pub fn current_quote(&self) -> &str {
        &self.current
    }

    pub fn rotate(&mut self) {
        let mut rng = rand::rng();
        self.current = self.quotes.choose(&mut rng).unwrap_or(&self.quotes[0]).to_string();
    }
}
```

### Type-Ahead Buffer Integration
```rust
// Source: https://doc.rust-lang.org/std/collections/struct.VecDeque.html
use std::collections::VecDeque;

pub struct MenuSession {
    state: MenuState,
    typeahead: VecDeque<char>,
}

impl MenuSession {
    pub fn handle_char(&mut self, ch: char) -> MenuAction {
        // If currently processing (e.g., during redraw), buffer the input
        if self.is_processing() {
            self.typeahead.push_back(ch);
            return MenuAction::Buffered;
        }

        // Otherwise, process immediately
        self.process_key(ch)
    }

    pub fn process_buffered(&mut self) -> Vec<MenuAction> {
        let mut actions = Vec::new();
        while let Some(ch) = self.typeahead.pop_front() {
            let action = self.process_key(ch);
            actions.push(action);
            if matches!(action, MenuAction::LaunchService(_)) {
                break;  // Stop processing buffer when launching service
            }
        }
        actions
    }
}
```

### TOML Config Structure
```toml
# Source: https://serde.rs/enum-representations.html (internally-tagged)
# Main menu items
[[menu.main]]
type = "submenu"
hotkey = "G"
name = "Games"
submenu_key = "games"
order = 1

[[menu.main]]
type = "submenu"
hotkey = "M"
name = "Mail"
submenu_key = "mail"
order = 2

[[menu.main]]
type = "command"
hotkey = "P"
name = "Profile"
command = "profile"
order = 99

[[menu.main]]
type = "command"
hotkey = "Q"
name = "Quit"
command = "quit"
order = 100

# Games submenu items
[[menu.games]]
type = "service"
hotkey = "1"
name = "Galactic Wars"
service_name = "galactic_wars"
order = 1
min_level = 0

# Unbuilt items (commented out until phase ships)
# [[menu.games]]
# type = "service"
# hotkey = "2"
# name = "Word Ladder"
# service_name = "word_ladder"
# order = 2
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Type-state pattern FSMs | Enum-based state machines | 2018-2020 | Simpler for most cases; type-state overkill for 2-3 states |
| External tagging (default) | Internal/adjacent tagging | serde 1.0+ (2017) | Cleaner config files, more readable TOML |
| `Vec` for queues | `VecDeque` | Rust 1.0 (2015) | O(1) front operations vs O(n) with Vec::remove(0) |
| `rand` 0.7-0.8 | `rand` 0.9 | 2023 | `choose()` moved to `IndexedRandom` trait in prelude |

**Deprecated/outdated:**
- **Type-state pattern for menus:** Compile-time state validation is powerful but adds boilerplate for simple 2-level hierarchy. Enum state machines are the community standard for this complexity level.
- **`#[serde(untagged)]` for config:** Slow deserialization, poor error messages. Internally-tagged is the modern preference for config files.
- **`rand::thread_rng()`:** Deprecated in 0.9; use `rand::rng()` instead (returns thread-local RNG).

## Open Questions

Things that couldn't be fully resolved:

1. **Adaptive column layout breakpoint**
   - What we know: Phase 2 profile card uses single-column layout; multi-column layout is common in BBS menus.
   - What's unclear: Ideal breakpoint (how many items triggers 2-column split?). Depends on item text length.
   - Recommendation: Start with single-column for <8 items, two-column for 8+. Tune based on visual testing.

2. **Quote rotation trigger**
   - What we know: Can rotate per session (on login), per menu view, or time-based (hourly).
   - What's unclear: User preference from context is "rotating" but doesn't specify frequency.
   - Recommendation: Per-session (rotate on login, stays same throughout session) avoids complexity. Can enhance to time-based in later phase.

3. **Type-ahead buffer size limit**
   - What we know: Must be bounded to prevent memory exhaustion.
   - What's unclear: Real-world maximum for "fast typing" users.
   - Recommendation: 16 characters (conservative). Classic BBS systems had 10-15 char type-ahead buffers.

4. **Help text content**
   - What we know: '?' key shows help, marked as Claude's discretion.
   - What's unclear: Help format (inline vs full-screen), content detail level.
   - Recommendation: Full-screen help with box-drawing border (consistent style), list hotkeys + explanations, "[Enter] to continue".

## Sources

### Primary (HIGH confidence)
- [VecDeque documentation](https://doc.rust-lang.org/std/collections/struct.VecDeque.html) - O(1) FIFO operations, official stdlib docs
- [rand 0.9.2 documentation](https://docs.rs/rand) - choose() method, current version
- [toml 0.9.11 documentation](https://docs.rs/toml/latest/toml/) - Current version, serde integration
- [Serde enum representations](https://serde.rs/enum-representations.html) - Internally-tagged pattern

### Secondary (MEDIUM confidence)
- [Rust state machine patterns (enum-based)](https://blog.yoshuawuyts.com/state-machines-2/) - Modern enum approach
- [TOML configuration processing with serde](https://www.makeuseof.com/working-with-toml-files-in-rust/) - Hierarchical config patterns
- [Mystic BBS menu commands](https://wiki.mysticbbs.com/doku.php?id=menu_commands) - Historical BBS menu patterns for reference
- [Input buffering ring buffer patterns](https://ntietz.com/blog/whats-in-a-ring-buffer/) - VecDeque as ring buffer

### Tertiary (LOW confidence)
- [WebSearch: Stoic quotes implementations](https://github.com/storopoli/stoic-quotes) - Example quote lists, not authoritative
- [WebSearch: BBS terminal navigation](https://github.com/InterLinked1/lbbs) - Community BBS implementations for reference

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - All crates are current, well-documented, in active use
- Architecture: MEDIUM-HIGH - Enum state machines are proven, but menu-specific patterns are project-specific
- Pitfalls: MEDIUM - Derived from general Rust best practices + WebSearch findings, not menu-specific authoritative sources

**Research date:** 2026-01-27
**Valid until:** 2026-03-27 (60 days for stable stack - rand, serde, toml are mature)

**Notes:**
- Existing codebase already uses `AnsiWriter`, CP437 box-drawing, and `toml` 0.8 (compatible with 0.9).
- Phase 2 established double-line border pattern for headers; reuse for main menu.
- Service trait architecture supports MenuService as any other service.
- Prior phases use `#[serde(default)]` pattern; apply to menu items for optional fields.
