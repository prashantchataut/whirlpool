# Continuity

Living hand-off for Whirlpool. Read first, update last. (Rule 2 of [`AGENTS.md`](AGENTS.md).)

## What Whirlpool is

A standalone dock and menu bar for Windows 10/11. Two native Rust executables, no web view, no runtime:

- `whirlpool.exe`: the dock and menu bar (Win32 + DirectComposition + Direct2D). `apps/dock`
- `whirlpool-settings.exe`: native settings (egui). `apps/settings`
- `crates/whirlpool-core`: pure logic (layout, springs, genie, **procedural wallpapers**), tested anywhere
- `crates/whirlpool-shell`: Windows integration (taskbar takeover, wallpaper apply, clock, power...)
- `crates/whirlpool-config`: settings schema and persistence (`%APPDATA%\Whirlpool\config.json`)
- `site/`: the website (Vite + React + Tailwind), deployed to GitHub Pages

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
- The menu bar stays on the top edge. A bottom or floating bar needs reworking `HEIGHT`-based geometry in `menubar.rs`
  (island notch, panels, work-area reservation); it is the first item below.
- Menu bar density (height) was prototyped in config and removed: resizing a live appbar needs a full re-reserve.

## Backlog: quality-of-life ideas (ranked)

1. Menu bar position (bottom / floating inset) and density.
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
