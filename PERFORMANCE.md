# Performance

**Not measured in this pass.** Idle CPU, idle RAM, GPU use and startup time need a real Windows desktop; none was
available. `scripts/measure.ps1` exists for it. The figures below are properties of the code, not measurements.

- The dock sleeps between events and paces animation with `DwmFlush` (existing design).
- This pass adds no timers to the dock. The bar's reveal check (100 ms) only runs while a floating bar is revealed.
- Settings asks for a repaint every 1.5 s while open, and exits when closed.
- Quick settings is a separate process that exits on focus loss.
- Binary size: the photo pack adds about 5.7 MB to `whirlpool.exe`.
