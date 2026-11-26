# Scorecard

Feature by feature against the closest comparable shell: a free Windows notch, dock and desktop-widget
tool, v0.1.1 (24 Nov 2026), studied from its public website, release notes and installer metadata.
Black-box only: nothing was copied, and this repository does not name it.

**Have It**: Whirlpool does this today. **Beat It**: Whirlpool does this and goes further. **Skip It**: not built, with the reason.
**Gap**: the reference has it and Whirlpool does not yet. Every row below was checked against the code in this
repository, not against a claim. "Seen" means a runner capture exists (see `docs/captures/session13/`).

| Feature | Reference | Whirlpool | Verdict |
| --- | --- | --- | --- |
| Top island: idle pill that opens | Yes | Four presets (Compact, Classic, Wide, Expressive), opens from the top edge | Have It |
| Island media: cover art, transport, progress | Yes | Cover art, title, artist and transport in the island (seen, `docs/captures/session13/01c-island-media.jpg`, staged with a real system media session); no progress bar in the open panel | Have It, progress open |
| Island media: animated spectrum bars | Yes | Not built (needs a loopback tap on the output device) | Gap |
| Synced lyrics | Floating karaoke window, lrclib.net | In the island, opt-in, one request per song, off by default | Have It |
| File shelf: drop in, drag out | Yes | Clock panel shelf, native shell drag out into any window | Have It |
| System HUDs: volume, brightness, battery | Yes | Volume, Wi-Fi, Bluetooth pills (seen); brightness drawn, no event source; battery alerts as toasts | Have It, brightness open |
| Stock volume flyout suppressed | Not documented | `WH_KEYBOARD_LL` takes only volume up, down, mute; fallback hides a stray flyout; nothing logged; Settings switch | Beat It |
| Dock with genie minimize | Yes, Direct3D11 | Genie on the compositor, seven speeds | Have It |
| Dock icon size | Not documented | 32 px default, 1.25x magnification, 44 px bar; saved sizes are never changed | Have It |
| Dock icons from 256 px sources | n/a | `IShellItemImageFactory` through the icon cache, scaled down with the style and tint options | Have It |
| Four icon styles and a tint | No | Yes | Beat It |
| Dock on any edge, auto-hide, multi-monitor | Bottom | Bottom, top, left, right; per-display choice | Beat It |
| Desktop clock widget | Three styles | Clock panel, clock in bar, dock or island; no desktop-layer clock yet | Gap |
| Vinyl music, calendar, sticky memo widgets | Four widgets | Models and `.ics` reader exist; no desktop widget layer | Gap |
| Command palette | Alt+Space spotlight | `Ctrl+K` (or `Ctrl+Shift+Space`): apps, files, windows, settings, widgets, clipboard, actions, unit conversion, calculator | Beat It |
| Launchpad / Start grid | Win-key tap | Dock tile and hotkey, filter, arrow keys | Have It |
| Clipboard history | Yes | Yes, in the palette | Have It |
| Quick settings | n/a | 320 px panel, 48 px tiles, editable tiles, volume row | Have It |
| Wallpaper picker | Not shown | Wallpaper studio: 20 curated 4K pictures from 4kwallpapers.com, credit and page link on every tile, one-click apply, optional accent match; plus 14 bundled photos and rotation | Beat It |
| Per-app volume mixer | Yes | Not built: needs an undocumented per-build interface | Skip It |
| Game auto-hide | Yes | Not built | Gap |
| Live CPU, RAM, network in the island | Yes | Separate overlay process (`Ctrl+Shift+F10`): CPU, memory, GPU, VRAM, network, battery, frame rate; nothing runs while hidden | Have It |
| Hot corners, notification centre | No | Yes | Beat It |
| Updater | Not documented | SHA-256 checked download, install, relaunch, one-version rollback; tested end to end on a clean runner | Beat It |
| Privacy | No telemetry; lyrics via lrclib.net | No telemetry; update check; opt-in lyrics; studio contacts 4kwallpapers.com only when opened | Have It |
| Installer size | 3.5 MB | 11.9 MB x64, 11.5 MB ARM64 (includes 14 photographs) | Skip It: the photo pack is the cost |
| Idle memory | about 35 MB | 24 MB private, 0.04 % CPU (runner, `STUDY.md`) | Have It |
| ARM64 | Not offered | Built in CI; not run on ARM hardware | Beat It, unverified on hardware |
| Signed installer | Not documented | No: unsigned, see `SIGNING.md` | Gap |

## Reference study (black box, session 13)

Read from the reference's public page and its release listing (one 3.5 MB installer, 24 Nov 2026); nothing was
decompiled, copied or reused. What its page sells: four desktop widgets (clock in three styles, a vinyl music player,
a calendar with `.ics`, a sticky memo), a floating karaoke window, a notch with shelf, clipboard, per-app mixer and game
auto-hide, a Direct3D genie, and a short idle-cost table (about 35 MB, about 0 % CPU). Its screenshots are whole desktops
on a photograph; ours were tight crops on a flat runner. That is the part we changed.

Where it is ahead: the desktop widget layer and the installer size. Where we are ahead: volume-key suppression, edges
and displays, updater with rollback, `Ctrl+K` scope, notification centre, hot corners, wallpaper studio. Not built, so
not claimed: spectrum bars, desktop widgets, per-app mixer, game auto-hide.

## Licence of the wallpaper studio

4kwallpapers.com states that its pictures belong to their authors and are cleared for personal use as a desktop
wallpaper; any other use needs the author's permission. So the studio downloads a picture to the user's own cache
only when Apply is pressed, credits the site on every tile and links to each page. This repository, the releases and
the site do not redistribute any of them, and the public captures use the licensed photographs in `assets/wallpapers`
(see `ASSETS_LICENSES.md`). The site's bot check also refuses hosted runners (HTTP 403), so the studio could not be
photographed with its own pictures; its tiles show a tint per group when a preview is unavailable.

## What is behind the staging

The capture driver sets a dark theme, hides the build text and desktop icons it can, applies a photograph, registers a
real system media session with a drawn cover, opens real windows and shoots the whole 1920x1080 screen. It closes the
runner's own paging-file dialog before every frame. One thing cannot be removed: the runner has test signing on, so
Windows draws a "Test Mode" watermark at the bottom right of every frame. It is left in, not retouched.
