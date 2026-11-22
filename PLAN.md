# Plan

Written after the audit, before the feature code of this pass.

1. Menu bar: edge reveal across the float gap; bottom bar above the dock (small, testable, high impact).
2. Settings: palette, layout, search, preview, About, vector icons.
3. Wallpapers: photo pack embedded; Fill style; shuffle.
4. Quick settings: original panel, real Wi-Fi/Bluetooth switches.
5. Proof: `captures.yml` photographs the real build; `cargo check` for the Windows target locally; CI for tests.
6. Site: photo pack, honest labels, 4.2 notes. Docs: PRIVACY, SIGNING, ASSETS_LICENSES, STATUS.
7. Release 4.2.0 from a tag; ARM64 zip.

Deferred on purpose (not started, see STATUS.md): FPS overlay, island presets, Launchpad start grid, window-drag motion,
wallpaper rotation and per-monitor wallpapers, hotkey manager, import/export, winget/scoop manifests.
