# Architecture

OrangeCNC Studio uses a one-way domain pipeline:

```text
text file
  ↓
orangecnc-gcode       lexer + modal parser
  ↓
CanonicalCommand      stable domain boundary
  ↓
orangecnc-motion      geometry + interpolation
  ↓
Toolpath              machine-independent segments
  ↓
orangecnc-machine     envelope / limits
  ↓
orangecnc-simulation  dry-run diagnostics
  ↓
apps/desktop          German UI and visualization
```

The UI never parses G-code itself. The parser never knows about egui. This keeps geometry and parser tests fast and allows future headless or controller applications.

## Design choices

- Internal dimensions are millimetres.
- G-code units are converted at the parser boundary.
- Arcs remain canonical until motion planning; only then are they tessellated.
- Arc resolution is controlled by chord tolerance and maximum segment length, not a fixed segment count.
- Canonical arc endpoints are preserved exactly at the final tessellated segment.
- Machine limits belong to a machine profile rather than parser state.
- Simulation can warn when resolved axis feed exceeds the profile's axis feed limit.
- Parse or geometry failures must invalidate previous UI analysis results rather than leaving stale toolpaths visible.

Author: Amir Mobasheraghdam
