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

## Session 9 (main, pre-release 4.2.0-rc6)

| Area | Change | Where |
| --- | --- | --- |
| HUDs | In-process DirectComposition pill for volume (from the menu bar's `AudioWatch` via `menu_bar_audio_changed`), Wi-Fi (second `NetworkWatch`, only offline/Wi-Fi flips), Bluetooth (`Radio.StateChanged`, one short thread at start). CPU-drawn bitmaps, compositor animation (inset clip for the fill). Timer 61 while up. Config `hudEnabled`. Brightness: drawn, no trigger. | `apps/dock/src/{hud,bluetooth,app}.rs`, `menubar/mod.rs::hud_volume` |
| CLI | `whirlpool --hud volume\|muted\|brightness\|wifi\|bluetooth N` posts `HUD_COMMAND` to the running dock (used by the capture driver). | `main.rs`, `app.rs::request_hud` |
| Settings | `ui::slider` replaces every `egui::Slider`. | `apps/settings/src/ui.rs` |
| Driver | Scenario `hud: ...` (matrix id `hud`), in the DPI pass as `hud`. | `scripts/drive.ps1`, `docs/capture-matrix.json` |
| Site | `Captures.tsx` with real runner screenshots; roadmap and headlines. | `site/` |

Gotcha: timer ids share one namespace per window; `app.rs` already used 41 (wallpaper). Next: brightness event source, Settings sidebar at narrow widths, wallpaper crossfade, app copy audit.

## Sessions 5 and 6 (PR #11 merged; PR #12 open; intended pre-release 4.2.0-rc4, not tagged yet)

| Area | Change | Where |
| --- | --- | --- |
| Core | `tiles` (grid pack, reorder, drop target), `notify` (bounded history, fold repeats, file format), `corners` (dwell state machine), `iconstyle` (four styles), `glass` (refraction shader on the CPU). All unit-tested (172 tests). | `crates/whirlpool-core/src/` |
| Config | `quickTiles`, `hotCorner*`, `dockIconStyle`, `dockIconTint`, `glassRefraction`, `glassStrength`, `notificationHistory`. Unknown values read as defaults. | `crates/whirlpool-config/src/lib.rs` |
| Windows | `chrome.rs`: `round_window` clips to a rounded region (no dependence on window alpha: the DirectX fallback draws black corners otherwise), `capture` grabs the screen behind a panel, `GlassPanel` refracts it. | `apps/settings/src/chrome.rs` |
| Panels | `--notifications` (history), `--notify "title" "body"` (script entry), quick panel reads `quickTiles`. | `notify_center.rs`, `quick.rs`, `apps/dock/src/main.rs` |
| Settings | Cards with live previews: tiles (Menu bar), hot corners (Windows), icon style (Dock), glass (Personalize). | `apps/settings/src/extras.rs` |
| Dock | Hot corner timer (only while a corner is set, never over full-screen), icon restyle at the single load point. | `apps/dock/src/app.rs`, `assets.rs` |
| Study | `scripts/study.ps1`, `observe-install.ps1`, `study.yml` (reference supplied by repository secrets; never named in the repo). `drive.ps1` waits for windows by title and dismisses stray Windows flyouts. | `scripts/`, `.github/workflows/` |

### Wiring state of presets, Launchpad and drag (checked against runner captures, run 45403289477, session 6)

| Area | State | Where |
| --- | --- | --- |
| Island presets | Closed and open island follow the preset (collapsed width with 2+ providers fixed in `trigger_layout_for`). All four captured and different. | `apps/dock/src/menubar/{layout,mod}.rs`, `docs/captures/session6/02-island-*` |
| Launchpad (Start grid) | Own process (`whirlpool --launchpad`, toggles). The dock Start tile (real click captured) and `launchpadHotkey` open it; search, arrow keys and the app list are captured. The placeholder is painted by us (egui's hint colour was invisible). | `apps/settings/src/launchpad.rs`, `apps/dock/src/{app,main}.rs` |
| Drag motion | `drag.rs`: thread only during a move, shadow window directly behind the dragged window, snap preview, spring settle. Shadow aligned, subtle (strength raised in session 6). The grey box in drag captures is Windows' own snap preview (it also shows with motion off). `core::dragmotion` unused. | `apps/dock/src/drag.rs`, `docs/captures/session6/07-drag-*` |

Runner driver: `scripts/drive.ps1` captures with BitBlt+CAPTUREBLT (layered windows included), clicks the real Start tile and sends real arrow keys. The Screenshots workflow runs on push only for older branches; for a new branch start it with `workflow_dispatch`.

Not started: HUDs (wifi, Bluetooth, volume, brightness), DPI pass at 100/150/200 percent, Settings redesign. Copy audit: partial (see STATUS.md). `4.2.0-rc4` is described above but not tagged yet.

### Next (ranked)

0. HUDs, DPI pass, Settings redesign (see STATUS.md). 1. Panel memory and start time without OpenGL (about 300 MB, 7 s). 2. Own window overview with DWM thumbnails. 3. Palette action icons and placeholder contrast; snap preview look. 4. Feed the update reminder and charging events into the history. 5. Rename "Launchpad". 6. Real-desktop run of everything above; signing.

Runner caveat: the screenshot runner shows its own system dialogs (paging file, performance options) behind panels; they appear refracted in the glass and are not Whirlpool.

## Session 7 (branch `feat/gaps-session-6`, PR #12, pre-release 4.2.0-rc5)

| Area | Change | Where |
| --- | --- | --- |
| Window overview | `whirlpool --overview` (toggles; `--overview-compat` for DirectX). Cards from `WindowTracker::list`; DWM thumbnails registered against the panel's own window (found by title); re-listed every 2 s while open. Packing and arrow steps are pure and tested in core. Hot-corner action `WindowOverview` and a palette action open it. | `apps/settings/src/overview.rs`, `crates/whirlpool-core/src/overview.rs`, `apps/dock/src/main.rs` (`open_overview`), `apps/dock/src/app.rs` |
| Glyphs | Shape-drawn icons, reusable as the fallback for windows and apps without an icon. | `apps/settings/src/glyphs.rs` |
| Palette | Painted placeholder and magnifier, glyph tiles, "Show all windows" action, selection as tint plus border. | `apps/settings/src/palette.rs` |
| Driver | Overview scenario (three Notepads and a PowerShell; open, Right, hover, Enter; logs whether it closed and the foreground changed). | `scripts/drive.ps1` |

Tooling note: the sandbox that wrote this ran `cargo test -p whirlpool-core` but cannot type-check the `windows` crate (out of memory), so CI compiled the Windows code. The docs of session 6 say rc4 is untagged; a `v4.2.0-rc4` pre-release exists.

### Next (ranked)

1. App icons in overview captions (reuse the Start grid icon loader); overview hotkey and a Settings card with a live preview. 2. Palette: drop the repeated "Action" label. 3. Panel memory and start time without OpenGL (about 300 MB, 7 s; unchanged). 4. Exit takes about 1.1 s (reference 0.3 s). 5. Re-capture and review the Start grid and drag ghost. 6. Real-desktop run; signing.

## Version 4.2.0-rc3: what changed

| Area | Change | Where |
| --- | --- | --- |
| Updates | Reminder toast through hidden Windows PowerShell with its own notification identity (`HKCU\\Software\\Classes\\AppUserModelId\\Whirlpool`). Cadence lives in `State::reminder_due` / `note_reminded` (`reminded`, `reminded_at`, `reminder_count` in `updates\\state.json`). Config `updateReminders` (default on); `updateOnStartup` default now on. | `crates/whirlpool-update/src/{remind,state}.rs`, `apps/settings/src/update.rs`, `apps/dock/src/app.rs` |
| Core | `island`, `launchpad`, `dragmotion`: pure and tested. (Island and Launchpad were wired later; see Session 5.) | `crates/whirlpool-core/src/` |
| Site | No logo images; Roadmap replaces "Not done yet"; version 4.2.0-rc3. | `site/` |

### Next (ranked)

1. Wire `island::Preset` into `menubar.rs` (`ISLAND_SINGLE_WIDTH`, `ISLAND_EXPANDED_WIDTH`, `ISLAND_ROW_HEIGHT`, `trigger_layout`) and add a Settings row; update the layout tests. 2. Launchpad window (separate process like the overlay; use `launchpad::{fit, hit, search}`). 3. Decide the drag-motion shell design. 4. Run the reminder on a real desktop. 5. Signing (SIGNING.md).

## Version 4.1.0: what changed

| Area | Change | Where |
| --- | --- | --- |
| Menu bar | `menuBarPosition: top \| bottom` (`BarPosition`). On the bottom the bar and its popup are two sibling containers (`bar_layer`, `popup_layer`) placed in one window; `layout_y` maps window-local pointer y back to the layout the bar is drawn in (bar at 0, popup below), so `island_hits`, zones and the usage viewport are untouched. `window_top`, `anchor_y`, `in_panel_rows` replace the old top-only arithmetic. | `apps/dock/src/menubar.rs` |
| Menu bar | `NCHITTEST` and the click-away hook use `BAR_AT_BOTTOM`, `PANEL_TOP`; reveal/hide slides toward the bar's own edge (`reveal_lift` is signed); AppBar reserves `Edge::Bottom`; the edge watcher reveals from the bottom (`set_menu_bar_bottom`). | `menubar.rs`, `edge_watch.rs` |
| Menu bar | Bottom edge forces the island to the Bubble style (a notch is welded to the top edge). | `draw_island` |
| Settings | "Screen edge" row; "Bar position" (alignment) is now "Alignment". | `apps/settings`, `i18n/*.json` |
| Notices | `NOTICE` is short; upstream MIT notice (required) moved to `THIRD_PARTY_LICENSES.md`, installed and zipped. | root, installer |
| Site | Four more Pexels photographs, Screen edge control and bottom-bar preview in the lab, caption fix. | `site/` |
| Skills | Added `theme-factory`, `webapp-testing`, `animation-vocabulary`, `review-animations`. | `.agents/skills/` |

### 4.1 verification notes

- Verified: `cargo fmt`, `cargo test --workspace` and `cargo build --release -p whirlpool` pass on `windows-latest` CI; the config test for `menuBarPosition` passes. The site was built and rendered in headless Chromium (renders in `docs/renders/`).
- **Not verified on a real desktop:** every bottom-bar behaviour (hit testing, popup placement, reveal strip, AppBar stacking with a bottom dock, flyout anchors). Treat as untested until someone runs it.
- Known gap: with the dock also on the bottom edge both reserve the edge; Windows stacks AppBars, but the dock's own placement was not adjusted for a floating bar.
- Settings not opening: 4.0 added the DirectX retry, but 4.0 was never tagged, so installed users still had 3.0. 4.1 is the first release containing it. Cause on affected machines is unconfirmed; `%LOCALAPPDATA%\Whirlpool\whirlpool.log` now records the failure.

### Next (ranked)

1. FPS and system-info overlay (one popup, 1 s timer only while visible; DWM `cFramesDisplayed` delta for fps). 2. Dynamic island presets (size, position, expansion). 3. Spring animation for dragged and floating windows. 4. Code signing (SignPath Foundation). 5. Wallpaper rotation.

## Version 4.0.0: what changed

| Area | Change | Where |
| --- | --- | --- |
| Charging | `halo` redrawn as a minimal pill; new `bloom` (bottom glow) and `aurora` (top ribbons). Both are CPU-drawn soft bitmaps animated by compositor keyframes, like the other styles. | `apps/dock/src/charge.rs` |
| Motion | `AnimationStyle::{Dreamy, Rapid}`, `StartupAnimation::Float`. | `crates/whirlpool-config` |
| Menu bar | `BarTint::{Violet, Mint, Sunset, Slate, Sand}`. | `crates/whirlpool-config`, Settings |
| Settings | eframe has both `glow` and `wgpu` (DX12 only). `whirlpool.exe --settings` tries OpenGL; on error or panic it starts `--settings-compat` (DirectX). | `apps/dock/src/main.rs`, `apps/settings/src/lib.rs` |
| Site | New site: live desktop/dock/menu bar renders, a working Settings lab, six charging previews, trust section. Source in `site/`. | `site/` |
| Docs | README rewritten with renders (`docs/renders/`, generated from the site, **not** captures of Windows). | README |

### 4.0 verification notes

- Verified here (Linux): `whirlpool-config` tests (29 pass). The dock and settings crates were **not** compiled on this machine (the `windows` crate needs more RAM than the sandbox has); CI on `windows-latest` is the compiler.
- **Not verified on a real desktop:** Bloom, Aurora, the new Halo, the DirectX fallback. They are written against the same helpers as Edge and Tide.
- The site's charging previews are CSS, not captures.

### Bottom-edge menu bar: the plan (not started in code)

The bar window is one `visual_height` tall: bar at `y = 0`, popup panel at `y = bar_height + PANEL_GAP` (`popup_surface`, `IslandGeometry::top`). For a bottom bar:
1. Put the bar visuals and the popup in two sibling containers; offset the bar by `visual_height - bar_height`, the popup above it.
2. Map pointer `y` in `hover`/`click` (`island_hits`, lines using `bounds.bottom`) through one `to_layout_y`.
3. Mirror `BAR_TOP/BAR_BOTTOM/PANEL_BOTTOM` for bottom, `Edge::Bottom` in `place()`, and the edge watcher's reveal strip.
4. Config key `menuBarPosition: top|bottom`; Settings row under *Menu bar*.
It touches ~15 sites and needs a real desktop to verify, hence it was not guessed at.

### Next (ranked)

1. Bottom-edge bar (above). 2. FPS / system-info overlay (one popup, 1 s timer only while visible). 3. Charging preview button (settings -> dock IPC). 4. Wallpaper rotation. 5. Launchpad-style start grid.

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

## Session 7 hand-off

Read `STATUS.md` first. New: `apps/settings/src/design.rs` (tokens, easing, stagger, `Backdrop`), `glass::frost` in core,
`Direction` A/B/C switches for Start and quick settings (A is the default; B and C stay behind the env vars until someone
deletes them). The capture driver takes `-Only <text>` and the Screenshots workflow has an `only` input, so one
scenario can be re-run in minutes: `gh workflow run screenshots.yml -f only=redesign`. Next: re-capture the Start grid on
the fallback build, then HUDs, DPI, Settings, crossfade, site.
