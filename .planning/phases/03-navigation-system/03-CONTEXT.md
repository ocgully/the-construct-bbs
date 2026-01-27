# Phase 3: Navigation System - Context

**Gathered:** 2026-01-27
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can navigate BBS using Wildcat-style numbered/lettered menus with ANSI art. Hierarchical menu structure (2 levels: main menu and submenus), hotkey navigation, config-driven menu definitions, and ANSI art headers. This phase delivers the navigation framework — actual services (Mail, Chat, Games) are built in later phases but menu entries are defined in config.

</domain>

<decisions>
## Implementation Decisions

### Menu Structure
- 2-level hierarchy: main menu -> submenus (no deeper nesting)
- Simplified grouping: Games, Chat, Mail, Profile, Goodbye (not full classic Wildcat set)
- Config-driven menus: menu items defined in config.toml with sysop control
- Full ordering support: config entries have an `order` field for sysop-controlled display order
- Typed menu items: each config entry has a `type` field (service, submenu, command)
- Full screen redraw when entering/exiting submenus (clear screen, render new menu)
- Each submenu has its own unique ANSI art header
- Level-gated access: menu items have minimum user level, hidden if user doesn't qualify
- Config includes commented-out entries for future/unbuilt services so sysop knows what's coming
- Mixed hotkeys: numbers for services, letters for commands (Q=Quit, P=Profile)
- Explicit [Q] Back to Main Menu listed as last item in every submenu
- Global commands (Quit, Profile) only available from main menu, not submenus
- Invalid input redraws the full menu (clear + redraw)
- Command stacking supported: e.g., type 'G1' at main menu to go Games > item 1
- Type-ahead buffer: keys typed during navigation are buffered and processed at next menu
- No location context in prompt — just "Your choice? "
- Time display deferred to Phase 4
- Enter alone redraws current menu
- '?' at any menu shows help text explaining available keys

### Visual Style
- Box-framed title headers (bordered box with title text inside, consistent with Phase 2 login header)
- Mixed box-drawing: double-line borders for main menu header, single-line for submenus — visual hierarchy through border weight
- Uniform color scheme across all menus (no per-submenu accent colors) — consistent with CGA palette in use
- 80-column width matching Phase 2 screens (login header, profile card)
- Title only in header box (no user info in the header)
- Blank line separator between header box and menu items
- Adaptive column layout: single column when few items, two columns when many

### Input Behavior
- Single keypress instant navigation (no Enter required)
- Type-ahead buffer for command stacking (keys buffered during screen transitions)
- Key echo: selected key appears on screen before navigation
- Case-insensitive input ('q' and 'Q' both work)
- Brief pause (~200ms) on transitions between menus
- No arrow key navigation — hotkeys only (authentic BBS)
- Enter alone redraws current menu

### Menu Content
- All planned sections defined in config (Games, Mail, Chat, News, Profile, Goodbye) — unbuilt items hidden until their phase ships
- Status badges next to items when available (e.g., unread mail count, players online)
- MOTD area between header and menu items — auto-generated rotating Stoic/stoicism quotes for contemplation
- Help text accessible via '?' key at any menu

### Claude's Discretion
- Exact config.toml schema structure for menu definitions
- Type-ahead buffer implementation approach
- Stoic quote selection and rotation mechanism
- Adaptive column layout breakpoint (how many items triggers two-column)
- Help text content and formatting
- Status badge formatting and alignment

</decisions>

<specifics>
## Specific Ideas

- MOTD should be auto-generated rotating Stoic/stoicism quotes — brief messages for contemplation, not configurable text files. Thematic for "The Construct" atmosphere.
- Command stacking works like classic Wildcat type-ahead: press keys fast and they buffer through menu transitions (e.g., G1 = Games > first game)
- Menu prompt text: "Your choice? " (friendly, not "Command? ")
- Profile and Quit use letter hotkeys [P] and [Q]; services use numbers [1], [2], etc.
- Submenus always end with explicit [Q] Back to Main Menu entry

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 03-navigation-system*
*Context gathered: 2026-01-27*
