# Realm of Ralnar: Asset Conversion Agent Prompt

Use this prompt to instruct a Claude agent to implement the asset conversion pipeline and export PNGs at all scales.

---

## Task: Implement Realm of Ralnar Asset Conversion Pipeline

You are implementing the asset conversion tools for "The Realm of Ralnar" JRPG port.

### Context
- Original game: QBasic, VGA Mode 13h (320x200, 256 colors)
- Target: Web-based (WASM + Canvas), playable via BBS DOOR or standalone
- Assets located in: Bg_rpg/Bg_rpg/
- Full specification: docs/realm_of_ralnar_specification.md

### Your Deliverables

1. **VGA Palette** (palette.rs)
   - Implement full 256-color VGA palette
   - 6-bit to 8-bit conversion: `rgb8 = rgb6 * 4 + 3`
   - Reference: Bg_rpg/Bg_rpg/monster/CHRIS.PAL if available

2. **pic2png** - Convert .PIC files to PNG
   - Input: Text file, 400 lines, one palette index per line
   - Reading order: column-major (x increments first, then y)
   - Transparency: -1 → alpha=0
   - Output: PNG at 1x, 2x, 3x, 4x, 5x scales (nearest-neighbor)
   - Test: pics/TREE.PIC should show green tree on transparent background

3. **mmi2png** - Convert .MMI files to PNG + extract attributes
   - Input: QBasic GET array format (203 integers + 1 attribute)
   - Line 1: width*8 (should be 160 for 20px wide)
   - Line 2: height (should be 20)
   - Lines 3-203: packed pixel data (2 pixels per integer, low byte first)
   - Line 204: tile attribute
   - Output: PNG at 1x-5x scales + attributes.json
   - Note: MMIs are already composited tiles (layered PICs flattened)

4. **mon2png** - Convert .MON files to PNG
   - Input: Binary, 8-byte header + raw pixels
   - Header: LE16 version, LE16 frames, LE16 width*8, LE16 height
   - Pixel data: 1 byte per pixel, 0xFF = transparent
   - Output: PNG at 1x-5x scales
   - Variable sizes: 40x40, 60x80, 100x100

5. **mmm2json / nmf2json** - Convert maps to JSON
   - MMM: text format ("name", width, height, enemies_flag, [tile_idx, attr] pairs)
   - NMF: binary format (LE16 header + LE16 tile data)
   - Output JSON with per-map tileset (tile names, not global indices)
   - Map tile indices to names via mmi/MMIFILES.TXT lookup
   - See specification section 1.6 for target JSON format

### Output Directory Structure
```
assets/
├── tiles/
│   ├── 1x/  (20x20)
│   ├── 2x/  (40x40)
│   ├── 3x/  (60x60)
│   ├── 4x/  (80x80)
│   └── 5x/  (100x100)
├── monsters/
│   └── [1x-5x structure]
├── sprites/
│   └── [1x-5x structure]
├── maps/
│   └── *.json
└── metadata/
    ├── tiles.json      (attributes from MMI files)
    └── palette.json    (VGA palette for reference)
```

### Scaling for Scanlines
- Virtual resolution is ALWAYS 320x200
- At any scale, scanlines are calculated on virtual lines
- Example: 3x scale (960x600) has 200 virtual scanlines, each 3px tall
- Darken every other virtual scanline by 30%

### File Format Quick Reference

```
PIC: text, one integer per line, 20x20 grid = 400 lines (+1 trailing)
     Value -1 = transparent, 0-255 = VGA palette index
     Reading order: column-major (x first, then y)

MMI: QBasic GET array format (composited tile from layered PICs)
     Line 1: width*8 (160 for 20px)
     Line 2: height (20)
     Lines 3-203: packed pixel data (2 pixels per 16-bit integer)
     Line 204: tile attribute byte

MMM: "name"\nwidth\nheight\nenemies_flag\n[tile_idx,attr pairs]
NMF: binary, LE16 width, LE16 height, header, LE16 tile data

MON: binary, 8-byte header, raw pixel data, 0xFF = transparent
     Header: LE16 version (1), LE16 frames, LE16 width*8, LE16 height
```

### Batch Conversion Checklist

Run all converters and verify:
- [ ] All PIC files converted (check pics/ folder)
- [ ] All MMI files converted (check mmi/ folder)
- [ ] All MON files converted (check monster/ folder)
- [ ] All map files converted (check maps/ folder)
- [ ] Visual spot-check: TREE, MEDOW, WATER1, SPIDER look correct
- [ ] Transparency works (no black backgrounds where should be transparent)
- [ ] All 5 scale variants exist for each asset

### Important Notes
- Use nearest-neighbor interpolation for scaling (preserve pixel art crispness)
- Preserve exact colors - no anti-aliasing or smoothing
- PIC transparency is -1, MON transparency is 0xFF
- MMI files may use palette index 0 for transparency (check visually)
- Tile attribute values: 0=passable, 1=blocked, 2=water, etc.
- The global tile list is in mmi/MMIFILES.TXT (410 tiles)
- Maps reference tiles by index into this global list
