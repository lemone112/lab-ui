# lab-ui

Enterprise design-system token generator. Rust core, zero runtime dependencies.

## Philosophy

- **Compact config** — define a single canonical colour per accent; four theme variants are derived algorithmically.
- **Algorithmic generation** — neutral scales use CAM16-UCS perceptual mixing; accents use APCA contrast inversion so accessibility is baked in, not eyeballed.
- **Zero runtime dependencies** in `labui-core` — the engine is a pure Rust library you can embed anywhere.

## Quick start

```bash
# 1. Edit your anchors and accents
$EDITOR config.yaml

# 2. Generate tokens
cargo run -p labui-cli

# 3. Output appears in dist/
#    dist/tokens.scss  — CSS custom properties, one block per selector
#    dist/tokens.json  — flat token map for JS bundlers
```

## Example config

```yaml
primitives:
  neutral:
    light: "#FFFFFF"
    base:  "#787880"
    dark:  "#101012"
    ic:
      light: "#FFFFFF"
      base:  "#72727A"
      dark:  "#000000"
    # Optional: tune the perceptual curve.
    # curve:
    #   lightness_ease: 1.7
    #   hue_ease: 0.6
    #   chroma_peak: 0.35

accents:
  brand: "#007AFF"
  red:   "#FF3B30"

# Optional: tune how accents adapt across themes.
# accent_theming:
#   dark_factor: 0.7   # dark-mode contrast as fraction of light-mode
#   ic_boost:    15.0  # extra APCA Lc for increased-contrast variants

output:
  scss: "dist/tokens.scss"
  json: "dist/tokens.json"
```

## How it works

### Neutral primitives
Neutral scales are generated in CAM16-UCS perceptual space. Each step is a perceptual mix between `light`, `base`, and `dark` anchors, producing smooth lightness ramps that remain hue-consistent.

### Accent theming (APCA inverse)
Instead of hand-picking four hex values per accent, you supply one **canonical** colour. The CLI derives the other three variants via APCA inverse contrast:

| Theme      | Derivation logic                                     |
|------------|------------------------------------------------------|
| Light      | Canonical colour as-is                               |
| Dark       | Lighter variant with contrast `Lc × dark_factor`     |
| IC Light   | Darker variant with contrast `Lc + ic_boost`         |
| IC Dark    | Lighter variant with contrast `-(Lc + ic_boost) × dark_factor` |

APCA inverse solves for a CAM16-UCS colour with the same hue/chroma as the canonical source that yields the target contrast against the theme background. If the exact colour falls outside the sRGB gamut, chroma is reduced while preserving hue and lightness.

### Output guarantees
- **Deterministic** — `BTreeMap` ordering and fixed selector sequence (`:root`, `.dark`, `.ic`, `.dark.ic`) guarantee byte-identical output across runs.
- **Deduplicated** — one CSS block per selector; accents are isolated from the primitive loop so adding more primitives never duplicates accent declarations.

## Project layout

```
crates/
  labui-core/     — CAM16-UCS engine + APCA + primitive/accent generators (Rust lib)
  labui-cli/      — CLI: reads config.yaml → dist/* (Rust bin)
schemas/
  tokens.schema.json — JSON Schema validating config.yaml
config.yaml       — Client DS input (primitives + accents + theming params)
```

## Tests

```bash
cargo test --workspace   # 37 tests: roundtrip, contrast contracts, determinism, deduplication
```
