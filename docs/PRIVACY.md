# Privacy

Whirlpool runs on your computer and talks to nothing unless you leave a switch on (update check, on by default; synced lyrics, off by default).

**What it does not do.** No telemetry, no analytics, no accounts, no crash upload, no advertising identifiers. It
does not read the contents of your windows or files. The window list, icons and thumbnails it shows never leave the
process.

**What it stores.** One settings file, `%APPDATA%\Whirlpool\config.json` (your choices, pinned apps, recent apps), and
a log, `%LOCALAPPDATA%\Whirlpool\whirlpool.log`, which holds errors only. Wallpapers you choose are copied to
`%LOCALAPPDATA%\Whirlpool\wallpapers\`.

**What it sends.** The update check, optional: with "Check for updates" on (the default), Settings asks
`api.github.com` for the latest release number of this repository. GitHub sees your IP address, as with any web
request. Turn it off in Settings > System and Whirlpool makes no network connections.

**Synced lyrics (off by default).** Settings > Menu bar > Synced lyrics. When you turn it on and a song is playing,
Whirlpool sends the song title, artist, album and length to `lrclib.net`, a public lyrics service, once per song, so
the current line can replace the track title in the island. Nothing else goes with it: no account, no identifier. The
service sees your IP address, as any website does. The lyrics are held in memory for the current song and are never
written to disk. Turn the setting off and the request is never made. The request is made by `lyrics.rs` in the dock
and built by `crates/whirlpool-core/src/lrc.rs`.

**Clipboard history and the file shelf.** The palette's clipboard history lives in the dock's memory only, skips
items that password managers mark private, and is emptied when the setting is turned off. The file shelf stores file
*paths* (not contents) in `%APPDATA%\Whirlpool\shelf.txt`; the focus timer stores its state in `focus.txt` beside it.
Neither is ever sent anywhere.

The optional AI usage island reads usage numbers from the tools you already run locally and shows them; it is off
unless you enable it.

**The volume keys.** With the pop-ups on (Settings > Personalize > Volume and Wi-Fi pop-ups, then Handle the volume
keys, both on by default), Whirlpool installs a low-level keyboard hook so that the volume up, down and mute keys
change the volume and show Whirlpool's pill instead of the Windows flyout. The hook compares every key code with
those three values; any other key is passed on at once. **No key is stored, counted, logged or sent anywhere, not even
while debugging**, and the hook code (`crates/whirlpool-shell/src/mediakeys.rs`) contains no logging call. Turn
either switch off and the hook is removed; Windows then handles the keys itself. Brightness keys are not hooked:
Windows has no key code for them, the hardware reports them to the driver.

**Removal.** Uninstalling runs `whirlpool.exe --restore`, which gives back the Windows taskbar, and removes the
startup entry. Delete `%APPDATA%\Whirlpool` to remove your settings.
