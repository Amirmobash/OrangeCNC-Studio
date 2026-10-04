# Rebuild notes — 2026-10-04

This package was reconstructed from the public `Amirmobash/OrangeCNC-Studio` repository and reviewed against the project's stated architecture and CNC correctness constraints.

## High-impact fixes

- malformed fractional M-codes now fail instead of being silently ignored
- fractional tool numbers now fail instead of being rounded
- IJK full-circle arcs can be emitted without endpoint words
- arc tessellation closes on the exact canonical endpoint
- zero-length linear moves no longer create synthetic path segments
- stale toolpaths/reports are cleared after parser or geometry errors
- isometric viewport bounds use all eight 3D bounds corners
- rapid-time estimation uses machine-axis feed limits instead of a hard-coded speed
- feed simulation warns when an individual axis would exceed its configured maximum feed
- placeholder repository URLs were corrected

## Validation status

The execution environment used to produce this ZIP does not contain a Rust toolchain (`cargo`/`rustc` are unavailable), so `cargo fmt`, `cargo clippy`, and `cargo test` could not be executed here. The repository includes a pinned toolchain and CI workflow so those commands can run in a normal Rust/GitHub environment.
