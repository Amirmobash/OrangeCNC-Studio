# OrangeCNC Studio

**OrangeCNC Studio** is an open-source CNC workstation in Rust by **Amir Mobasheraghdam**. The desktop application is German-first and uses a white/orange visual system.

> Author: **Amir Mobasheraghdam**  
> Project: **OrangeCNC Studio**  
> Focus: G-code parsing, canonical motion, toolpath planning, CNC simulation, machine limits and future controller integration.

## Architecture

OrangeCNC Studio keeps CNC responsibilities in a one-way pipeline:

```text
text / G-code
  -> orangecnc-gcode
  -> CanonicalCommand
  -> orangecnc-motion
  -> Toolpath
  -> orangecnc-machine
  -> orangecnc-simulation
  -> apps/desktop
```

The UI does not parse G-code, and the parser does not depend on egui. Internal geometry is normalized to millimetres at the parser boundary.

## Current capabilities

- German desktop UI with white/orange theme
- compact and spaced G-code (`G1X10Y5` and `G1 X10 Y5`)
- modal state for `G0/G1/G2/G3`, `G17/G18/G19`, `G20/G21`, `G90/G91`
- spindle/tool/feed words (`M3/M4/M5`, `S`, `T`, `F`)
- IJK arcs in XY/XZ/YZ planes, including full-circle IJK arcs
- radius (`R`) arcs with major/minor arc selection by radius sign
- adaptive arc tessellation based on chord tolerance and maximum segment length
- helix interpolation along the orthogonal axis
- machine-envelope checks and axis-feed warnings
- simulation statistics and machine-profile-based rapid-time estimation
- 2D and isometric toolpath preview in the desktop UI
- sample programs and regression tests

## Quick start

Install Rust 1.78 or newer, then run:

```bash
cargo run -p orangecnc-desktop
```

Quality checks:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

## Repository layout

```text
apps/desktop       German desktop application
crates/domain      stable CNC domain types
crates/gcode       lexer, parser and modal state
crates/motion      canonical geometry and interpolation
crates/machine     machine envelope and axis data
crates/simulation  dry-run simulation and diagnostics
docs/              architecture and development notes
examples/          sample G-code programs
site/              project landing page for GitHub Pages
```

## Safety

This repository is engineering software, **not a certified safety controller**. It must not be the only protective layer for real machinery. Hardware E-stop circuits, limit switches, interlocks and controller-side safety remain mandatory.

## License

MIT © 2026 Amir Mobasheraghdam
