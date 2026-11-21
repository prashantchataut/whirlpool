# Architecture

Whirlpool is a Cargo workspace with two programs and three libraries. Nothing here
links a browser engine, a script runtime or a managed runtime.

```
apps/dock ───────────┐
                     ├── whirlpool-shell  (Windows shell integration, `windows` crate)
apps/settings ───────┤   whirlpool-core   (pure logic: layout, magnify, spring, genie)
                     └── whirlpool-config (config.json schema, load, save, migrate)
```

## Crates

| Crate | Depends on Windows | Responsibility |
| --- | --- | --- |
| `whirlpool-core` | No | Dock layout, cursor magnification, springs, bounce, genie geometry, roster ordering. Unit-tested on any OS. |
| `whirlpool-config` | Paths only | `Config` struct, defaults, atomic save, `%APPDATA%\Whirlpool\config.json`, legacy folder migration. |
| `whirlpool-shell` | Yes | Taskbar control, window tracking, icons and icon cache, pins, launcher, autostart, power, keyboard layout, recycle bin, the dock-to-settings link. |
| `whirlpool` (`apps/dock`) | Yes | Window creation, DirectComposition visuals, Direct2D drawing, menu bar, island, previews, drag and drop, watchdog. |
| `whirlpool-settings` | Yes | egui window. Edits the same config and tells the dock to reload. |

## Process model

1. `whirlpool.exe` starts, hides the Windows taskbar and starts a watchdog process.
2. The dock draws into DirectComposition visuals; Direct2D renders icon and label bitmaps once and the compositor animates them.
3. `whirlpool-settings.exe` writes `config.json` (debounced) and pings the dock through a named pipe. The dock re-reads the file and applies changes live.
4. If the dock exits for any reason, the watchdog restores the taskbar. `whirlpool.exe --restore` does the same by hand.

## Theme

Colour lives in two places and they are kept in step:

| Place | File |
| --- | --- |
| Dock glass, indicator, labels | `apps/dock/src/scene.rs`, `apps/dock/src/gfx.rs` |
| Menus and menu bar | `apps/dock/src/menu.rs`, `apps/dock/src/menubar.rs` |
| Settings window | `apps/settings/src/ui.rs` (`Palette`, `ACCENT`, `FOAM`, `install_theme`) |

| Token | Dark (Abyss) | Light (Foam) |
| --- | --- | --- |
| window | `#07151B` | `#EEF5F4` |
| sidebar | `#0A1D25` | `#E0ECEA` |
| card | `#0E2730` | `#FBFEFD` |
| border | `#1A3B46` | `#CBDDDA` |
| text | `#E7F3F0` | `#0B1F26` |
| accent | Tide `#0E9398` | Tide `#0E9398` |
| accent ink | Foam `#9FF0E6` | `#0A6E73` |

The settings window follows Windows light/dark. The dock follows the system theme and the
Windows transparency setting (glass when on, solid when off).

## Website and renders

`site/` is static HTML, CSS and JS with no build step. `site/assets/ui.js` draws the
dock and menu bar in CSS and is used both by the live hero demo and by the pages in
`site/renders`, which are screenshotted into `site/assets/screens`. To refresh them,
open each render at its stated size and capture it (the sizes are in `CONTRIBUTING.md`).

## Icon

`python scripts/make_icon.py` redraws `assets/brand/whirlpool-1024.png` and
`whirlpool.ico` analytically (numpy and pillow). Small sizes use a bolder variant.
