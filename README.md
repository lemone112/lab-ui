# lab-ui

Enterprise design-system token generator. Rust core, zero runtime dependencies.

## Quick start

```bash
# 1. Edit your anchors and semantic aliases
$EDITOR config.yaml

# 2. Generate tokens
cargo run -p labui-cli

# 3. Output appears in dist/
#    dist/tokens.scss  — CSS custom properties
#    dist/tokens.json  — token map for JS bundlers
```

## Example config

```yaml
primitives:
  neutral:
    light: "#FFFFFF"
    base:  "#787880"
    dark:  "#101012"
    # Optional: tune the perceptual curve.
    # Omit to use defaults tuned for sRGB average surround.
    # curve:
    #   lightness_ease: 1.7   # power ease exponent for J'
    #   hue_ease: 0.6         # how fast white's hue snaps to base
    #   chroma_peak: 0.35     # sinusoid peak position (0..1)
    #   chroma_boost: 1.2     # overshoot above base chroma

semantic:
  bg-surface:       "neutral-0"
  bg-surface-hover: "neutral-1"
  text-primary:     "neutral-12"
  text-secondary:   "neutral-8"

output:
  scss: "dist/tokens.scss"
  json: "dist/tokens.json"
```

## Project layout

```
crates/
  labui-core/     — CAM16-UCS engine + primitive generators (Rust lib)
  labui-cli/      — CLI: reads config.yaml → dist/* (Rust bin)
schemas/
  tokens.schema.json — JSON Schema validating config.yaml
config.yaml       — Client DS input (primitives + semantic aliases)
```

## Tests

```bash
cargo test          # Rust unit tests (roundtrip + semantic invariants)
```
