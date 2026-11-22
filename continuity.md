# Continuity

Living hand-off for Whirlpool. Read first, update last. (Rule 2 of [`AGENTS.md`](AGENTS.md).)

## What Whirlpool is

A standalone dock and menu bar for Windows 10/11. Two native Rust executables, no web view, no runtime:

- `whirlpool.exe`: the dock and menu bar (Win32 + DirectComposition + Direct2D), `apps/dock`. It also contains Settings: `whirlpool.exe --settings` runs the egui window from `apps/settings` (a library crate) in its own short-lived process.
- `installer/whirlpool.iss`: the Inno Setup script CI compiles into `Whirlpool-Setup.exe`
- `crates/whirlpool-core`: pure logic (layout, springs, genie, **procedural wallpapers**), tested anywhere
- `crates/whirlpool-shell`: Windows integration (taskbar takeover, wallpaper apply, clock, power...)
- `crates/whirlpool-config`: settings schema and persistence (`%APPDATA%\Whirlpool\config.json`)
- `site/`: the website (Vite + React + Tailwind), deployed to GitHub Pages

## Version 3.0.0: what changed

| Area | Change | Where |
| --- | --- | --- |
| Charging | Four styles: `surge`, `halo` (capsule), `edge` (liquid glass screen edges, colour by level), `tide`. Glows are drawn at 1/4 resolution and stretched by the compositor. | `apps/dock/src/charge.rs` |
| Menu bar | `menuBarTint`, `menuBarAlign`, `menuBarWidth`, `menuBarBorder`; width and alignment are applied in `inset_screen`. | `menubar.rs`, config |
| Wallpapers | Tidal, Halftone, Ripple, Silk styles; 66 presets. | `crates/whirlpool-core/src/wallpaper.rs` |
| Motion | Animation styles Fluid and Brisk, startup Drift and Dart, drop-down chooser. | config, `ui::choose` |
| Crab | Off by default, key renamed to `crab`. | config |
| Size | `reqwest` uses native TLS (no aws-lc); `opt-level = "s"`. | `apps/dock/Cargo.toml`, `Cargo.toml` |
| Settings | Error dialog if the window fails to open. | `apps/dock/src/main.rs` |
| License | Own notice in `LICENSE`; `NOTICE` kept and installed. | root |

### 3.0 verification notes

- Verified here (Linux): `whirlpool-core` and `whirlpool-config` tests and a contact sheet of the new wallpapers. Windows code is built and tested by CI only.
- **Not verified on a real desktop:** the four charging animations, the narrower floating bar, the drop-down widths in Settings.
- **Not done: menu bar on the bottom edge.** The bar, its panels and the island assume the top edge (`BAR_TOP`, `place()`, panel geometry). It needs its own pass before more bar options are added.
- Not started: dock themes and opacity, dynamic-island presets, per-monitor wallpapers, a wallpaper rotation timer, a charging preview button in Settings.
- Site: the charging section is a CSS preview of the motion, not a capture of Windows.

## Version 2.0.0: what changed

| Area | Change | Where |
| --- | --- | --- |
| One program | Settings is a library inside `whirlpool.exe`; `--settings` runs it, one window at a time (named mutex, raises the first). A second launch of the dock opens it. | `apps/settings/src/lib.rs`, `apps/dock/src/main.rs` |
| Installer | Per-user Setup (Inno Setup) built by the Release workflow; portable zip and `install.ps1` stay. | `installer/`, `.github/workflows/release.yml` |
| Logo | Settings uses `assets/brand/whirlpool-256.png` for its icon and header; the ICO has BMP frames for 16-128 px and PNG for 256. | `apps/settings/src/lib.rs`, `assets/brand/` |
| Charging | `charge.rs`: medallion, level ring, bolt, ripples, driven by compositor keyframes. `chargingStyle` = surge / halo. The crab is no longer used for the charger. | `apps/dock/src/charge.rs`, `app.rs::start_charge` |
| Menu bar | `menuBarStyle` (docked/floating), `menuBarSize`, `menuBarInset`, `menuBarGap`, `menuBarRadius`; height and gap are atomics read by the geometry code. | `menubar.rs` (`bar_height`, `inset_screen`, `float_px`) |
| Wallpapers | Aurora, Topo, Grid and Dither styles, 30 presets (54 total). | `crates/whirlpool-core/src/wallpaper.rs` |

## Version 1.0.0: what changed

| Area | Change | Where |
| --- | --- | --- |
| Brand | New logo: luminous three-arm vortex on a glass squircle. Generated, reproducible. | `scripts/make_icon.py`, `assets/brand/` |
| Brand | Standalone identity: no "fork of" wording in README, site, metadata. MIT notice kept in `LICENSE` as the license requires. | README, site, `Cargo.toml` |
| Screen fit | Settings window sizes itself to the monitor's work area and scale; declares per-monitor DPI v2 in its manifest (it had none). | `apps/settings/src/main.rs`, `build.rs` |
| Taskbar | Hiding the Windows taskbar now has a second layer (empty window region) and a 5 s repair check, so it stays hidden when Explorer resets it. Fully restored on exit / `--restore`. | `crates/whirlpool-shell/src/taskbar.rs`, `apps/dock/src/app.rs` |
| Wallpapers | 24 procedural wallpapers (vortex, mesh, horizon) rendered at the exact screen resolution, plus Shuffle. No downloads, no assets. | `crates/whirlpool-core/src/wallpaper.rs`, settings **Personalize** |
| Motion | `animationStyle` (Relaxed / Smooth / Snappy) scales every spring. | config, `app.rs::tick` |
| Startup | `startupAnimation` (Off / Glide / Rise / Swift): the dock rises from its edge. | `app.rs` |
| Quit | The dock sinks into its edge before exiting (`begin_smooth_quit`). | `app.rs` |
| Charging | `chargingAnimation` toggles the charging crab separately from other mascot moments. | `app.rs::power_moment` |
| Menu bar | Tint strength, show/hide date, 24h / 12h / Windows clock. | `menubar.rs`, `system::clock_text_with` |
| Trust | Version info + manifest in both exes, SHA-256 sums, build provenance attestation, `docs/TRUST.md`, optional Authenticode signing step. | `.github/workflows/release.yml` |
| Site | Rebuilt from scratch: glassmorphism + dithered "data" theme. | `site/` |
| Skills | Seven design skills vendored and made a rule. | `.agents/skills`, `AGENTS.md` |

## Verified vs not verified

- Verified in CI (Windows MSVC): format, `cargo test --workspace`, release build. See the latest run on `main`.
- Verified locally (Linux): `whirlpool-core` and `whirlpool-config` unit tests, including all wallpaper presets.
- **Not verified by a human on a real Windows desktop:** dock rise/sink animations, the taskbar region layer, wallpaper
  application, settings window sizing at unusual scales. Please run the release and report problems.
- README/site images are design renders from the app's own palette and layout, not captures.

## Known limits / honest notes

- Windows may still show SmartScreen for a new, unsigned app. Reputation builds with downloads; Authenticode signing removes it
  (see `docs/TRUST.md`). `scripts/install.ps1` downloads with PowerShell, which does not add the Mark-of-the-Web.
- The menu bar can float and change height, but it stays on the top edge (**bottom edge is not done**). A bottom or floating bar needs reworking `HEIGHT`-based geometry in `menubar.rs`
  (island notch, panels, work-area reservation); it is the first item below.
- Menu bar density (height) was prototyped in config and removed: resizing a live appbar needs a full re-reserve.

## Backlog: quality-of-life ideas (ranked)

1. Menu bar on the bottom edge: panels must open upward, and the island and reveal logic assume the top.
2. Dock close animation for a single app icon (poof / shrink with neighbours sliding).
3. Launchpad-style app grid as a replacement Start menu (`whirlpool:start` entry exists in the dock).
4. Task menu: custom right-click entries (e.g. "Always on top", "Move to monitor").
5. Wallpaper rotation on a coarse timer (every N minutes), per-monitor wallpapers, light/dark auto-switch.
6. Dock themes (tint, border, shadow strength) and per-app icon overrides.
7. Seconds in the menu bar clock via a 1 s timer only while the clock menu is open.
8. Windows-on-ARM build in the release matrix.

## Release checklist

1. `cargo fmt --all`, `cargo test --workspace`, `python scripts/make_icon.py` (if brand changed).
2. Bump `version` in `Cargo.toml`; update `CHANGELOG.md`.
3. Tag `vX.Y.Z`. The Release workflow builds on Windows, writes `SHA256SUMS.txt`, attests provenance, publishes.
4. Check the release page: exes, zip, `install.ps1`, sums. Update the site's download links if names change.

## 2.0 verification notes

- CI (Windows MSVC) builds and tests the workspace. The charging animation and floating bar are **not verified on a real desktop by a human**.
- Local (Linux): `whirlpool-core` and `whirlpool-config` tests, including all 54 wallpapers; the new styles were reviewed on a contact sheet.
- Windows Defender / SmartScreen: unsigned binaries from a new project are flagged by reputation, not by content. See `docs/TRUST.md`. Submit each release at https://www.microsoft.com/wdsi/filesubmission and, when funds allow, sign with SignPath or Azure Trusted Signing (the workflow signs automatically if a certificate secret exists).
