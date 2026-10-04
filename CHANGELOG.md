# Changelog

## 0.2.1 — 2026-10-04

- rebuilt and cleaned the Rust workspace without changing its layer boundaries
- fixed fractional M-code handling so malformed codes are no longer silently ignored
- added strict integer tool-number validation
- added full-circle IJK arc parsing and regression coverage
- preserved canonical arc endpoints exactly after tessellation
- stopped zero-length linear moves from creating fake toolpath segments
- cleared stale UI results after parse or geometry failures
- improved isometric preview bounds using all eight bounding-box corners
- added machine-profile-based rapid-time estimation and axis feed warnings
- corrected repository metadata and added CI/toolchain files

## 0.2.0 — 2026-09-19

- clean workspace architecture
- German desktop application
- new lexer/parser with modal state
- canonical moves and multi-plane arc geometry
- adaptive line/arc interpolation
- machine-envelope validation
- simulation diagnostics
- 2D/isometric toolpath preview
- project metadata for Amir Mobasheraghdam
