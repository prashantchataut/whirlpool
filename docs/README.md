# Whirlpool docs

| Document | What is in it |
| --- | --- |
| [THEME.md](THEME.md) | The Whirlpool palette and where each colour lives in the code |
| [CHANGELOG.md](CHANGELOG.md) | What changed from Muelle Native |
| [upstream/CHANGELOG-muelle.md](upstream/CHANGELOG-muelle.md) | Full history before the fork |
| [previews/](previews) | Design renders of the site and both apps |

## Architecture in one paragraph

`whirlpool-core` holds layout, springs, magnification and the genie curve. It has no Windows dependency and is unit-tested anywhere. `whirlpool-shell` wraps the Win32 and WinRT calls (windows, icons, taskbar, launching). `whirlpool-config` owns `%APPDATA%\Whirlpool\config.json`. The dock (`apps/dock`) renders with Direct2D and DirectComposition; the settings app (`apps/settings`) is an egui window. Settings and dock only communicate through the config file plus a registered window message.

## Moving from Muelle

On first run Whirlpool copies `%APPDATA%\Muelle\config.json` to `%APPDATA%\Whirlpool\` if there is no Whirlpool config yet. The Muelle folder is left untouched and both can be installed side by side.
