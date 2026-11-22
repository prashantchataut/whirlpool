<p align="center">
  <img src="assets/brand/whirlpool-1024.png" width="132" alt="Whirlpool logo: a luminous three-arm vortex on a glass tile">
</p>

<h1 align="center">Whirlpool</h1>

<p align="center">
  <b>A dock that moves like water.</b><br>
  A magnifying dock and a customizable menu bar for Windows 10 and 11.<br>
  One small program with Settings built in. It does nothing while you are not using it.
</p>

<p align="center">
  <a href="https://github.com/prashantchataut/whirlpool/releases/latest"><b>Download</b></a> ·
  <a href="https://prashantchataut.github.io/whirlpool/">Website</a> ·
  <a href="docs/TRUST.md">Trust &amp; verification</a> ·
  <a href="CHANGELOG.md">Changelog</a>
</p>

<p align="center">
  <img src="docs/previews/site-hero.png" alt="The Whirlpool website: a dithered vortex behind a desktop with the dock and menu bar">
</p>

> **About the images.** The site images are screenshots of the website's live demo, drawn from the app's own palette and layout.
> The wallpaper sheets are the app's real renderer. None is a capture of a running Windows session; run the release and compare.

## New in 3.0

- **Four charging animations.** *Surge* (a small medallion with thin rings), *Halo* (a slim glass capsule), *Edge glow* (the screen edges glow like liquid glass: green above 60%, yellow in the middle, orange at 20% and below) and *Tide* (a line of light along the top edge up to your level). Or none.
- **A menu bar that is yours.** Docked or floating, six colours, width, left/centre/right position, radius, size and an optional edge highlight.
- **66 wallpapers.** Tidal waves, halftone spheres, ripples and silk join the dithered vortex.
- **More motion.** Five animation feels and six startup animations.
- **Smaller.** Network access uses the Windows TLS stack, the bundled crypto library is gone, and the build is optimised for size.
- **Settings tells you when it fails** to open (a missing OpenGL driver, say) instead of doing nothing.

<p align="center">
  <img src="docs/previews/wallpapers-3.jpg" alt="The twelve new wallpapers, rendered by the app" width="860">
</p>

## What you get

| | |
| --- | --- |
| **Glass dock** | Spring magnification, pins, groups, recents, live window previews, Genie minimize, drag and drop, auto-hide on any edge. |
| **Menu bar** | Clock, network, volume, battery, input language, tray, now playing and an AI plan-usage island. Tint strength, date and 12/24-hour clock are yours to set. |
| **Wallpapers** | 66 procedural wallpapers (vortex, mesh, horizon, aurora, topo, grid, dither, tidal, halftone, ripple, silk) drawn at the exact resolution of your screen, plus Shuffle. Nothing is downloaded. |
| **Motion** | Animation style (Relaxed, Smooth, Snappy), a startup animation (Off, Glide, Rise, Swift), a dock that sinks away when you quit, and a charging animation you can turn off on its own. |
| **Your taskbar, hidden for real** | The Windows taskbar is auto-hidden, made transparent and click-through, given an empty window region, and re-checked every few seconds. `whirlpool.exe --restore` always brings it back. |
| **Settings that fit** | A native window that sizes itself to your monitor and scale, in English, Español and Français, light and dark. |

<p align="center">
  <img src="docs/previews/site-customize.png" alt="The customization playground: sliders and segmented controls next to a live dock and the config.json it writes" width="860">
</p>

### Wallpapers

<p align="center">
  <img src="docs/previews/wallpapers.png" alt="Contact sheet of all 24 Whirlpool wallpapers" width="860">
</p>

### Screens

<p align="center">
  <img src="docs/previews/site-screens.png" alt="Design renders: dock over a desktop, the Personalize settings page, charging and startup states" width="860">
</p>

## Install

Windows 10 (1903+) or Windows 11, 64-bit.

**Setup (recommended).** Download `Whirlpool-Setup.exe` from [Releases](https://github.com/prashantchataut/whirlpool/releases/latest) and run it. No administrator rights.

**One line, per user, no admin.** The script verifies the SHA-256 of the file before it installs anything:

```powershell
irm https://raw.githubusercontent.com/prashantchataut/whirlpool/main/scripts/install.ps1 | iex
```

**Portable.** Download `whirlpool.exe` (or the zip) and run it. Open settings from the dock's right-click menu, or run `whirlpool.exe --settings`.

### Windows warns me. Is it safe?

A brand-new open-source app has no signature and no download reputation yet, so SmartScreen says *unknown publisher*.
That means unknown, not malicious. Here is how to check, and what we do about it ([full notes](docs/TRUST.md)):

```powershell
Get-FileHash .\whirlpool.exe -Algorithm SHA256                          # compare with SHA256SUMS.txt
gh attestation verify .\whirlpool.exe --repo prashantchataut/whirlpool  # proves which workflow and commit built it
```

- Releases are built by GitHub Actions from a tagged commit, with a signed build-provenance attestation and `SHA256SUMS.txt`.
- Both executables carry full version information and ask for **no administrator rights**.
- No telemetry. If the dock exits, a watchdog gives you the taskbar back.
- Code signing is wired into the release workflow and turns on when a certificate is added.

## Customize

Everything is in the Settings window (right-click the dock, or *Whirlpool Settings* in the Start menu; it runs as `whirlpool.exe --settings`), which writes
`%APPDATA%\Whirlpool\config.json` and tells the dock to reload. New in 1.0:

| Key | Values |
| --- | --- |
| `animationStyle` | `relaxed`, `smooth`, `snappy` |
| `startupAnimation` | `off`, `glide`, `rise`, `swift` |
| `chargingAnimation` | `true` / `false` |
| `chargingStyle` | `surge`, `halo`, `edge`, `tide` |
| `menuBarTint` | `ocean`, `graphite`, `midnight`, `forest`, `amber`, `rose` |
| `menuBarAlign`, `menuBarWidth` | `left`/`center`/`right`, `40` to `100` |
| `menuBarBorder` | `true` / `false` |
| `crab` | `true` / `false` (default off) |
| `menuBarOpacity` | `10` to `100` |
| `menuBarDate` | `true` / `false` |
| `menuBar24h` | `null` (Windows), `true`, `false` |
| `wallpaper` | id of the last applied wallpaper, e.g. `"abyss"` |

Older config files keep working; missing keys take their defaults.

## Build from source

Requires Rust (MSVC toolchain) and the Visual Studio C++ build tools.

```powershell
cargo build --release -p whirlpool
# target\release\whirlpool.exe (Settings is inside it)
cargo fmt --all -- --check
cargo test --workspace
python scripts/make_icon.py      # regenerates the logo (needs numpy and pillow)
```

## Layout

| Path | Purpose |
| --- | --- |
| `apps/dock` | The dock and menu bar (Win32, DirectComposition, Direct2D) |
| `apps/settings` | The settings window (egui) |
| `crates/whirlpool-core` | Platform-independent layout, springs, genie and the wallpaper engine, tested anywhere |
| `crates/whirlpool-shell` | Windows shell integration |
| `crates/whirlpool-config` | Settings schema and persistence |
| `site/` | The website (Vite, React, Tailwind), deployed to GitHub Pages |
| `.agents/skills/` | The design skills this project works by (see below) |

## Privacy

No telemetry. Optional network use: the settings window asks GitHub for the latest release when it opens
(turn off with *Check for updates automatically*), and the AI-usage island asks the provider you are already signed in to
for plan limits (turn off in Menu bar settings).

## Contributing

Read [`AGENTS.md`](AGENTS.md) and [`continuity.md`](continuity.md) first. UI and UX come first here: design work follows the
skills vendored in [`.agents/skills`](.agents/skills) (frontend-design, design-taste-frontend, ui-ux-pro-max,
web-design-guidelines, accessibility, impeccable, prototype), and `continuity.md` tracks what is verified and what is next.

## License

MIT. See [`LICENSE`](LICENSE).
