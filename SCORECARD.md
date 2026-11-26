# Scorecard

Feature by feature against the closest comparable shell: a free Windows notch, dock and desktop-widget
tool, v0.1.1 (24 Nov 2026), studied from its public website, release notes and installer metadata.
Black-box only: nothing was copied, and this repository does not name it.

**Have It**: Whirlpool does this today. **Beat It**: Whirlpool does this and goes further. **Skip It**: not built, with the reason.
**Gap**: the reference has it and Whirlpool does not yet. Every row below was checked against the code in this
repository, not against a claim. "Seen" means a runner capture exists (see `docs/captures/session12/`).

| Feature | Reference | Whirlpool | Verdict |
| --- | --- | --- | --- |
| Top island: idle pill that opens | Yes | Four presets (Compact, Classic, Wide, Expressive), opens from the top edge | Have It |
| Island media: cover art, transport, progress | Yes | Now playing with transport in the island | Have It |
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

## What is behind the staging

The reference's pictures look like a real PC because they show a full desktop on a photograph, with the glass
refracting it. Ours looked like CI because the runner's desktop was flat or empty and the crops were tight.
The capture driver now applies a 4K picture through the studio, checks the desktop is not flat, opens real
windows behind the shell and shoots the whole 1920x1080 screen. When the site refuses the runner the driver
falls back to the bundled photos and writes that into the log, so a frame never claims a source it did not use.
