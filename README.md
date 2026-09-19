#OrangeCNC Studio

**OrangeCNC Studio** is an open-source CNC workstation rebuilt as a clean Rust codebase by **Amir Mobasheraghdam**. The desktop interface is German-first and uses a white/orange visual system.

> Author: **Amir Mobasheraghdam**  
> Project: **OrangeCNC Studio**  
> Focus: G-code parsing, toolpath planning, CNC simulation, machine limits and future controller integration.

![Status](https://img.shields.io/badge/status-active-orange)
![Language](https://img.shields.io/badge/UI-Deutsch-orange)
![License](https://img.shields.io/badge/license-MIT-blue)
![Rust](https://img.shields.io/badge/Rust-2021-black)

## Why this rewrite exists

The goal is not to stack patches on an older codebase. OrangeCNC Studio starts with a small set of explicit domain boundaries: parsing, canonical motion, toolpath generation, machine configuration, simulation and UI. Each layer can be tested without the desktop application.

## Current capabilities

- German desktop UI with white/orange theme
- compact and spaced G-code (`G1X10Y5` and `G1 X10 Y5`)
- modal state for `G0/G1/G2/G3`, `G17/G18/G19`, `G20/G21`, `G90/G91`
- spindle/tool/feed words (`M3/M4/M5`, `S`, `T`, `F`)
- IJK arcs in XY/XZ/YZ planes
- radius (`R`) arcs
- adaptive arc tessellation based on chord tolerance and segment length
- helix interpolation along the orthogonal axis
- machine-envelope checks
- simulation statistics and warnings
- 2D and isometric toolpath preview in the desktop UI
- sample programs and unit tests

## Quick start

Install Rust, then:

```bash
cargo run -p orangecnc-desktop
```

Run tests:

```bash
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
site/              project landing page for GitHub Pages
```

## Safety

This repository is engineering software, **not a certified safety controller**. The current release must not be treated as the only protective layer for real machinery. Hardware E-stop circuits, limit switches, interlocks and controller-side safety remain mandatory.

## Author / discoverability

This repository intentionally uses the exact author name **Amir Mobasheraghdam** in human-readable project metadata (`README.md`, `AUTHORS.md`, `CITATION.cff`, `codemeta.json`, Cargo metadata and the GitHub Pages landing page). This helps search engines associate the project with the author without keyword stuffing.

Search engines decide ranking and indexing themselves, so no repository can guarantee first position on Google. For best indexing, keep the repository public, use the suggested repository name, enable GitHub Pages, and use **Amir Mobasheraghdam** as the GitHub profile display name.

## License

MIT © 2025 Amir Mobasheraghdam
