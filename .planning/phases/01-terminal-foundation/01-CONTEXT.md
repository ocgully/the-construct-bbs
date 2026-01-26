# Phase 1: Terminal Foundation - Context

**Gathered:** 2026-01-26
**Status:** Ready for planning

<domain>
## Phase Boundary

Browser terminal that correctly renders authentic ANSI art over WebSocket, with a pluggable service architecture underneath. Delivers: xterm.js terminal with CP437 font, WebSocket I/O, mobile-responsive layout, paginated output, keyboard-driven input, and a data-driven service registry that can enable/disable modules via configuration.

</domain>

<decisions>
## Implementation Decisions

### ANSI Art Fidelity
- Pixel-perfect CP437 rendering — exact DOS character set with period-correct glyphs
- Original CGA 16-color palette — the actual DOS colors (brown not dark yellow, etc.)
- Full CRT simulation effects (scanlines, curvature, phosphor glow, flicker) — but toggleable with multiple levels (full CRT → subtle scanlines → clean)
- ANSI art will be AI-generated (by Claude) for all screens and menus — no sourcing from art packs

### Terminal Dimensions & Layout
- Classic 80x24 terminal dimensions — most ANSI art designed for this
- Minimal bezel around terminal in browser — dark border framing the terminal, no decorative monitor housing
- Status line lives on the bottom row of the terminal (line 24) — shows time remaining, username, current area
- No scrollback buffer — what's on screen is what you get, authentic to the era

### Service Plugin Interface
- Screen transition on service entry: clear screen + service draws its own header — the most authentic BBS approach (Claude picks exact behavior between clear+header and instant swap based on what real BBSes did)
- Service-specific exit patterns — each door/service has its own quit mechanism (like real BBS doors, not a universal key)
- BBS-style loading delay — "Entering door..." or "Loading..." message before service renders, for authentic feel
- Service configuration mechanism designed to be easy — underlying config that Phase 13 Sysop panel will wrap with a menu interface

### Mobile Terminal Experience
- Scale 80-column terminal to fit phone screen width — shrink font proportionally, all 80 columns visible even if small
- Portrait orientation preferred — user prioritized portrait for thumb-typing with on-screen keyboard
- Native mobile keyboard input — standard phone keyboard types directly into xterm.js (no floating input bar)
- Full CRT effects on mobile same as desktop (toggleable) — no reduced/disabled-by-default on phone

### Claude's Discretion
- Exact CRT shader implementation (WebGL vs CSS)
- CP437 font choice (Perfect DOS VGA 437 vs alternatives — verify in browsers)
- WebSocket framing protocol details (escape sequence buffering strategy)
- ANSI writer buffering implementation
- Service trait API design (method signatures, context injection pattern)
- Configuration file format for service registry (TOML vs JSON vs other)
- xterm.js addon selection and configuration

</decisions>

<specifics>
## Specific Ideas

- The CRT effect should be dial-able: full simulation → subtle scanlines → clean. User wants to be able to turn it down, not just on/off.
- Portrait mobile is the priority because the keyboard takes up screen space — user is thinking about thumb-typing on a phone.
- "Entering door..." delay is explicitly desired for atmosphere, not just a loading indicator.
- Service-specific exit commands match real BBS culture — LORD had its own quit, chat had its own exit. Don't homogenize.
- The bezel should feel like framing, not decoration. Think dark void around a glowing terminal, not a picture of a CRT monitor.

</specifics>

<deferred>
## Deferred Ideas

- Sysop menu for toggling services on/off — Phase 13 (Sysop Administration). Phase 1 builds the underlying config mechanism that Phase 13 will wrap with a UI.

</deferred>

---

*Phase: 01-terminal-foundation*
*Context gathered: 2026-01-26*
