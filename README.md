# lab-ui

Enterprise design-system token generator.

## Quick start

```bash
# 1. Edit your anchors
cat config.yaml
# primitives:
#   neutral:
#     light: "#FFFFFF"
#     base:  "#787880"
#     dark:  "#101012"
# semantic:
#   bg-surface:     "neutral-0"
#   text-primary:   "neutral-12"

# 2. Generate tokens
cargo run -p labui-cli

# 3. Output appears in dist/
#    dist/tokens.scss  — CSS custom properties
#    dist/tokens.json  — token map for JS bundlers
```

## Project layout

```
crates/
  labui-core/     — CAM16-UCS engine + primitive generators (Rust lib)
  labui-cli/      — CLI: reads config.yaml → dist/* (Rust bin)
  labui-wasm/     — WASM bindings for Figma / JS runtimes (Rust lib)
schemas/
  tokens.schema.json — JSON Schema validating config.yaml
packages/
  css/            — Generated CSS variables (publish to npm)
config.yaml       — Client DS input (primitives + semantic aliases)
```

## Tests

```bash
cargo test          # Rust parity + roundtrip tests
```
