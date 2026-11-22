# Audit (4.1.0 -> 4.2.0)

Method: read the source, the release assets (GitHub API) and the workflows; compile with `cargo check` for
`x86_64-pc-windows-msvc`; run the real build on a Windows runner (`captures.yml`). No Windows desktop was available
for hands-on testing, so items below say which findings are measured and which are reasoned from code.

## Architecture

- `apps/dock` builds `whirlpool.exe`: Win32 + DirectComposition + Direct2D dock and menu bar. It links `apps/settings`
  as a library; `whirlpool.exe --settings` runs the egui window in its own short-lived process (named mutex, one window).
- `crates/whirlpool-core` pure logic (layout, springs, genie); `whirlpool-shell` Windows integration;
  `whirlpool-config` schema and persistence (`%APPDATA%\Whirlpool\config.json`).
- `installer/whirlpool.iss` (Inno Setup, per-user, no admin) is compiled by `release.yml` into `Whirlpool-Setup.exe`.
- `site/` Vite + React + Tailwind, deployed by `pages.yml`.
- Settings is launched from the dock's context menu, the tray, a second launch of `whirlpool.exe`, and the Start-menu
  entry `whirlpool.exe --settings`.

## The five reported bugs

| Bug | Finding | Status |
| --- | --- | --- |
| a) not installable / "doesn't exist" after install | v1.0.0 and v0.3.0 shipped a bare `whirlpool.exe` plus a separate `whirlpool-settings.exe` and no installer (release assets, API). `Whirlpool-Setup.exe` exists since v2.0.0 and installs per user to `%LOCALAPPDATA%\Programs\Whirlpool` with a Start-menu entry. Not reproduced on a clean machine here. | Fixed in earlier releases; install path re-read, unchanged. Clean-machine test NOT DONE. |
| b) Settings does not open | Two causes found. (1) The OpenGL-to-DirectX retry shipped in code for 4.0 but 4.0 was never tagged. (2) New this session: the window size came from Win32 metrics read before the window existed; if the scale was wrong the window was larger than the screen, so part of it was off-screen. | (1) shipped in 4.1. (2) `fit_window` now sizes from winit's monitor size on the first frame. Needs a real-machine check. |
| c) logo low resolution | The ICO holds 16-256 px frames (`identify`), the SVG source exists, Settings drew the 256 px PNG at 34 px. | Settings draws the 256 px mark at 36 px (sharp at 200%). DPI screenshots NOT DONE. |
| d) Claude mascot in charging | `charge.rs` has no mascot code (grep). The crab in `mascot.rs` is an opt-in feature (`mascot`, off by default) that reacts to charging when enabled. | Surge/Halo are clean. The opt-in crab is left alone. |
| e) license from another project | `LICENSE` is MIT, "Prashant Chataut", and is what the installer shows (`LicenseFile=..\LICENSE`). The upstream MIT notice that parts of the dock derive from is kept in `THIRD_PARTY_LICENSES.md`: MIT requires it. | Already correct; About page now shows the license. |

## Bugs found in this session (user report)

1. **Floating + auto-hide bar unreachable.** `edge_watch` revealed the bar when the pointer was in the 2 px strip along
   `bounds.top`. A floating bar's `bounds.top` is `gap` pixels below the screen edge, so a pointer pressed against the
   edge was never inside it. Fix: the strip spans the gap (`reach`). Two unit tests added.
2. **Bottom bar under the dock.** The bar was laid out against the full screen rectangle; the dock floats over the same
   rows. Fix: `DOCK_LIFT`, set by the dock after it computes its layout, raises the bar by the dock's depth.
3. **Settings layout.** Content was capped at 680 px and left-aligned, leaving the right side empty when maximized; the
   row layout did not adapt to narrow windows. Fix: centered column, live preview column on wide windows, icon-only
   sidebar on narrow ones.
4. **Wallpapers drawn by code** looked wrong next to photographs. Replaced with a Pexels photo pack.
5. **Quick settings.** The bar only sent Win+A. It now opens an original panel with working radio switches.

## UI inconsistencies found

Four fonts sizes in rows, hint colors that failed contrast in light mode, nav items without icons, a hard-coded
"100% native Rust" footer, no empty state, no search, no About. Addressed through the palette, icon set and layout in
`apps/settings/src/{ui,icons,app}.rs`.
