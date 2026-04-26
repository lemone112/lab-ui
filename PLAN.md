# Plan: Accent + Tint Matrix Support

## 1. Data Analysis (from Figma MCP extraction)

### Collections
| Collection | Vars | Modes |
|-----------|------|-------|
| `4.1 Primitives` | 114 | Light, Dark, Light-IC, Dark-IC |
| `4.2 Semantic` | 148 | Light, Dark, Light-IC, Dark-IC |

### Accent Bases — Exact Hex
Derived from Figma variable RGB values. All 12 accents have **4 distinct hex values** (Apple-style adaptive).

| Accent | Light | Dark | IC Light | IC Dark |
|--------|-------|------|----------|---------|
| Brand | `#007AFF` | `#4A8FFF` | `#0040DD` | `#409CFF` |
| Red | `#FF3B30` | `#FF3A3A` | `#D70015` | `#FF6161` |
| Orange | `#FFA100` | `#FF9008` | `#C93400` | `#FFA940` |
| Yellow | `#FFD000` | `#FFD60A` | `#B25000` | `#FFD426` |
| Green | `#34C759` | `#30D158` | `#248A3D` | `#30DB5B` |
| Teal | `#5AC8FA` | `#64D2FF` | `#0071A4` | `#70D7FF` |
| Mint | `#00C7BE` | `#63E6E2` | `#0C817B` | `#6CEBE7` |
| Blue | `#3E87FF` | `#5696FF` | `#0050CF` | `#95C0FF` |
| Indigo | `#5856D6` | `#5E5CE6` | `#3634A3` | `#7D7AFF` |
| Purple | `#AF52DE` | `#BF5AF2` | `#8944AB` | `#DA8FFF` |
| Pink | `#FF2D55` | `#FF2D55` | `#D30F45` | `#FF6482` |

### Key Principles (NOT copied values)
1. **Accent bases are theme-dependent** — dark mode shifts lightness up for readability; IC shifts hue/saturation for contrast inversion
2. **Derivables** (`X/Y@Z`) are algorithmic — they mix a source step with a background at Z% opacity. They are NOT manual hex entries.
3. **Shadow** uses semantic color tokens (`FX/Shadow/*`) + EFFECT styles (blur/spread). Shadow color = `Dark/Dark@N` (neutral derivable on dark).
4. **Materials** (`Materials/Light|Dark/Soft|Base|Elevated|Muted|Subtle`) are elevation-aware background fills. They map to semantic `Backgrounds/*` tokens.
5. **Interactive states** (`Interactive/Accent/Default|Hover|Pressed`) use accent tint at different strengths on component surfaces. These are semantic, not primitive.

---

## 2. Scope — THIS PR ONLY

### IN
1. `tint.rs` — CAM16-UCS `perceptual_mix(fg, bg, strength)`
2. `accent.rs` — `AccentConfig` (String | 4-theme struct), `build_tint_matrix()`
3. CLI — parse `accents` + `tint.strengths`, generate `--accent-*` + `--tint-*-*-on-neutral-*`
4. Schema — `tokens.schema.json` updates
5. Tests — roundtrip, edge cases, config parsing

### OUT (future PRs)
- Neutral derivables (algorithmic, requires understanding of bg+opacity relationship)
- Semantic tokens (Labels, Fills, Border, FX, Misc)
- Shadow/materials/interactive state generation
- Typography / dimensions / z-index

---

## 3. Architecture

### Config
```yaml
primitives:
  neutral: { light: "#fff", base: "#787880", dark: "#000", ic: {...} }

accents:
  brand: "#007AFF"                          # simple: all themes
  # OR explicit per-theme:
  brand:
    light: "#007AFF"
    dark: "#4A8FFF"
    ic_light: "#0040DD"
    ic_dark: "#409CFF"

tint:
  strengths: [2, 4, 8, 12, 20, 32, 52, 72]   # from Figma derivable patterns
```

### Output
```css
:root {
  --accent-brand: #007aff;
  --tint-brand-72-on-neutral-0: #b3d7ff;
  --tint-brand-72-on-neutral-1: #b5d8ff;
  /* ... 13 bgs × 8 strengths per accent ... */
}
.dark { /* recalculated with dark accent base + dark neutral bg */ }
.ic { /* recalculated with IC accent base + IC neutral bg */ }
.dark.ic { /* ... */ }
```

### Size
- 13 bgs × 12 accents × 8 strengths = 1,248 hex per theme
- × 4 themes = ~5K hex total
- Raw CSS ~35KB, gzipped ~5KB

---

## 4. Files

| File | Action |
|------|--------|
| `crates/labui-core/src/tint.rs` | CREATE |
| `crates/labui-core/src/accent.rs` | CREATE |
| `crates/labui-core/src/lib.rs` | MODIFY — add exports |
| `crates/labui-cli/src/main.rs` | MODIFY — parse + generate |
| `schemas/tokens.schema.json` | MODIFY — add defs |
| `config.yaml` | MODIFY — sample accents |

---

## 5. Tests

- `tint.rs`: 0% = bg, 100% = fg, 50% perceptually ≠ sRGB midpoint, reject OOB
- `accent.rs`: `from_hex()` clones to all themes, `resolve_base()` mapping
- CLI: config roundtrip, output contains expected CSS vars, JSON keys

---

## 6. Open Questions for Future PRs

1. **Neutral derivables**: What is the exact mixing function? Is it `mix(neutral-X, neutral-bg, Z%)` or something else? Need to derive algorithm from Figma values.
2. **Materials elevation**: How do `Soft/Base/Elevated/Muted/Subtle` map to `Backgrounds/Materials/*`? Are they fixed mixes or relative to current bg?
3. **Interactive states**: Are hover/pressed colors fixed tint strengths (e.g., `@72` → `@52` → `@32`) or do they use a different mixing curve?
