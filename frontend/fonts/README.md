# BBS Terminal Fonts

## Current Implementation

The terminal uses **IBM Plex Mono** loaded from Google Fonts CDN. This provides:
- Excellent CP437 box-drawing character support (all glyphs connect properly)
- Full Unicode coverage including extended ASCII
- Professional monospace design optimized for terminals
- Reliable cross-platform rendering
- Zero local hosting overhead

## Configuration

Font is loaded via Google Fonts in `index.html`:
```html
<link href="https://fonts.googleapis.com/css2?family=IBM+Plex+Mono:wght@400;700&display=swap" rel="stylesheet">
```

Applied in `terminal.ts`:
```typescript
fontFamily: "'IBM Plex Mono', 'Courier New', 'Courier', monospace"
```

## Future Enhancement

If you want to use an authentic DOS VGA font like "Perfect DOS VGA 437":
1. Download the TTF from: https://int10h.org/oldschool-pc-fonts/
2. Extract `VGA8.TTF` or similar from the pack
3. Place in this directory as `PerfectDOSVGA437.ttf`
4. Update `terminal.css` to add `@font-face` declaration
5. Update `terminal.ts` fontFamily to prioritize the local font

## Box-Drawing Character Test

IBM Plex Mono correctly renders all CP437 box-drawing characters:
```
┌─┬─┐  ╔═╦═╗  ╒═╤═╕
│ │ │  ║ ║ ║  │ │ │
├─┼─┤  ╠═╬═╣  ╞═╪═╡
│ │ │  ║ ║ ║  │ │ │
└─┴─┘  ╚═╩═╝  ╘═╧═╛
```
