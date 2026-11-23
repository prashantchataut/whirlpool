# Performance

**Not measured in this pass.** Idle CPU, idle RAM, GPU use and startup time need a real Windows desktop; none was
available. `scripts/measure.ps1` exists for it. The figures below are properties of the code, not measurements.

- The dock sleeps between events and paces animation with `DwmFlush` (existing design).
- This pass adds no timers to the dock. The bar's reveal check (100 ms) only runs while a floating bar is revealed.
- Settings asks for a repaint every 1.5 s while open, and exits when closed.
- Quick settings is a separate process that exits on focus loss.
- Binary size: the photo pack adds about 5.7 MB to `whirlpool.exe`.

## Measured in session 3 (GitHub Windows runner, 4 logical cores, `proof/B2/overlay-and-rotation.txt`)

- System overlay, detailed layout, 1 s interval, 30 s window: **0.013% CPU of the whole machine, 18.6 MB working set, 7.9 MB private.** Hidden: no process exists.
- Release build sizes (rc1): `whirlpool.exe` x64 15.9 MB, ARM64 14.6 MB; installers 11.7 MB (x64), 11.3 MB (ARM64).
- Update check: one HTTPS request to api.github.com; cost not measured.
- Not re-measured: idle dock CPU/RAM, startup time, GPU use during animations.
