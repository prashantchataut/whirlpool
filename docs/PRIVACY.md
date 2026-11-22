# Privacy

Whirlpool runs on your computer and talks to nothing unless you leave one switch on.

**What it does not do.** No telemetry, no analytics, no accounts, no crash upload, no advertising identifiers. It
does not read the contents of your windows or files. The window list, icons and thumbnails it shows never leave the
process.

**What it stores.** One settings file, `%APPDATA%\Whirlpool\config.json` (your choices, pinned apps, recent apps), and
a log, `%LOCALAPPDATA%\Whirlpool\whirlpool.log`, which holds errors only. Wallpapers you choose are copied to
`%LOCALAPPDATA%\Whirlpool\wallpapers\`.

**What it sends.** One optional request: with "Check for updates" on (the default), Settings asks
`api.github.com` for the latest release number of this repository. GitHub sees your IP address, as with any web
request. Turn it off in Settings > System and Whirlpool makes no network connections.

The optional AI usage island reads usage numbers from the tools you already run locally and shows them; it is off
unless you enable it.

**Removal.** Uninstalling runs `whirlpool.exe --restore`, which gives back the Windows taskbar, and removes the
startup entry. Delete `%APPDATA%\Whirlpool` to remove your settings.
