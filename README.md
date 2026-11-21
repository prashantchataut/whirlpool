<div align="center">

# Muelle Native

**A fast, macOS-inspired dock and menu bar for Windows 10 and 11, in 100 % native Rust.**

</div>

Muelle Native is a fork of [crafter-station/Muelle](https://github.com/crafter-station/Muelle)
(MIT). The dock was already native Direct2D/DirectComposition Rust; this fork removes the last
non-native piece, the Tauri/React settings app, and ships everything as two plain executables.

| Program | What it is |
| --- | --- |
| `muelle.exe` | The dock and optional top menu bar (Win32 + DirectComposition + Direct2D) |
| `muelle-settings.exe` | Native settings window (egui/glow). Edits the same `config.json` and tells the dock to reload |

## Features

- Glass dock with cursor-based magnification, pinning, grouping, reordering, recent apps.
- Live window previews, Genie minimize effect (three speeds), minimized-window thumbnails.
- Menu bar with clock, network, volume, battery, input language, tray icons, now-playing and AI plan usage island.
- Drag and drop onto apps, Downloads and the Recycle Bin; shortcuts folder menu; optional auto-hide.
- Settings in English, Spanish and French, light and dark, with previews drawn over your wallpaper.
- A watchdog restores the Windows taskbar if the dock exits unexpectedly.

## Install

Windows 10 (1903+) or Windows 11, 64-bit.

**One line (no admin, per-user):**

```powershell
irm https://raw.githubusercontent.com/prashantchataut/muelle-native/main/scripts/install.ps1 | iex
```

**Or manually:** download `muelle.exe` and `muelle-settings.exe` from
[Releases](https://github.com/prashantchataut/muelle-native/releases/latest), keep them in the same
folder and run `muelle.exe`. The binaries are unsigned, so SmartScreen may ask you to choose
**More info → Run anyway**. Checksums are in `SHA256SUMS.txt`.

If the dock ever misbehaves, `muelle.exe --restore` brings the Windows taskbar back.

## Build from source

Requires Rust (MSVC toolchain) and the Visual Studio C++ build tools.

```powershell
cargo build --release -p muelle -p muelle-settings
# target\release\muelle.exe and target\release\muelle-settings.exe
```

```powershell
cargo fmt --all -- --check
cargo test --workspace
```

## Layout

| Path | Purpose |
| --- | --- |
| `apps/dock` | Native Windows dock and menu bar |
| `apps/settings` | Native settings window (egui) |
| `crates/muelle-core` | Platform-independent layout and animation logic |
| `crates/muelle-shell` | Windows shell integration |
| `crates/muelle-config` | Shared settings and persistence |

## Privacy

No telemetry. Optional network use: the settings window asks GitHub for the latest release when
you open it (turn off with *Check for updates automatically*), and the dock's AI-usage island asks
the provider you are already signed in to for plan limits (turn off in Menu bar settings).

## License

MIT. Original work © Muelle contributors; changes in this fork are released under the same license.
