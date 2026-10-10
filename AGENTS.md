# Agent Instructions

Clean-room, pure-Rust photo library and raw developer targeting Adobe Lightroom parity. Native on Linux, macOS, Windows; WASM for web.

## Package Manager
Use **Cargo**: `cargo build`, `cargo check --workspace`, `cargo test`

## Commit Attribution
AI commits MUST include:
```
Co-Authored-By: (the agent model's name and attribution byline)
```

## File-Scoped Commands
| Task | Command |
|------|---------|
| Check crate | `cargo check -p <crate>` |
| Test crate | `cargo test -p <crate>` |
| Run single test | `cargo test -p <crate> <test_name> -- --exact` |
| Lint crate | `cargo clippy -p <crate> -- -D warnings` |
| Format | `cargo fmt --check` |
| Check layers | `cargo xtask layers` |
| Check assets | `cargo xtask assets` |
| Check parity | `cargo xtask parity` |
| Full CI | `cargo xtask ci` |

## Key Conventions
- **Never crash standard**: Non-test code must NEVER panic (`unwrap()`, `expect()`, `panic!`, `unreachable!`, `todo!`, `unimplemented!`, raw indexing `[i]`). Return crate `Result`. Every crash fix requires a regression test.
- **Pure Rust**: `#![forbid(unsafe_code)]` in all production crate roots. Isolated FFI only in `crates/sysmem`. No C/C++ dependencies.
- **Clean-room & zero Adobe assets**: Never read Adobe binaries or GPL/LGPL code (darktable, LibRaw, etc.). Never copy Adobe icons, fonts, presets, profiles. All assets must have entries in `assets/ATTRIBUTION.md`.
- **Crate layering**: Strict dependency layering enforced by `cargo xtask layers`:
  - L0: `geom`, `color`, `raster`, `tiff`, `fetch`, `heif`, `sysmem`
  - L1: `raw`, `codecs`, `meta`, `develop`, `scenes`, `faces`, `denoise-core`, `denoise`
  - L2: `pipeline`
  - L3: `catalog`, `gpu`, `merge`, `preview`, `segment`
  - L4: `engine`
  - L5: `ui-egui`, `mcp`
  - Apps: `lightcraft`, `lightcraft-cli`, `lightcraft-web`
- **Architecture**: Everything is an engine command (`lightcraft_engine`). All sliders are `develop` control specs.
- **UI & Controls**: egui text fields must use `crates/ui-egui/src/text_field.rs` (`TextField`), not `egui::TextEdit`. Drive UI via `--control <port>` JSON-lines protocol or headless snapshots (`lightcraft-cli snapshot`). Never send OS synthetic events.
- **Fonts**: Chinese/Japanese fonts are in optional `craft-fonts` repo (`CRAFT_FONTS_DIR=../craft-fonts`), never committed here.
