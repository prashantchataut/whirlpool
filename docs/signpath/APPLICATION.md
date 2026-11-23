# SignPath Foundation application text

**Project:** Whirlpool, a dock and menu bar for Windows 10/11 written in Rust.
**Repository:** https://github.com/prashantchataut/whirlpool (public, MIT license).
**What it does:** replaces the Windows taskbar with a dock and a menu bar; has a settings app, a system overlay and a
wallpaper changer. No telemetry; the only network request is an optional update check against api.github.com
([privacy statement](../PRIVACY.md)).
**Why signing:** releases are unsigned, so SmartScreen warns every user. Signing identifies the publisher.
**Build:** GitHub Actions on `windows-latest` from a version tag, no manual steps ([reproducible build notes](../REPRODUCIBLE_BUILD.md)).
**Contact / maintainer:** Prashant Chataut.
**Roles:** one maintainer approves signing requests; the release workflow submits them.
**Binaries to sign:** `whirlpool-x64.exe`, `whirlpool-arm64.exe`, `Whirlpool-Setup.exe`, `Whirlpool-Setup-arm64.exe`.
