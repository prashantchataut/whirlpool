# Status

| Item | Status | Proof | Notes |
| --- | --- | --- | --- |
| Phase 0 audit | DONE | AUDIT.md | Reasoned from code plus runner captures; no hands-on Windows desktop. |
| Bug a) install | PARTIAL | AUDIT.md; release assets | Installer exists since 2.0; clean-machine install NOT DONE. |
| Bug b) Settings opens | PARTIAL | docs/captures/settings-*.jpg; whirlpool.log | Opens on a runner (falls back to DirectX when OpenGL is missing). Window fit and column overflow fixed after a real capture showed them. |
| Bug c) logo | PARTIAL | AUDIT.md | ICO has 16-256 frames, SVG exists; 100/150/200% DPI screenshots NOT DONE. |
| Bug d) mascot | PARTIAL | AUDIT.md; i18n diff | Surge/Halo have no mascot. The Settings hint still said the crab plugs in; fixed in en/es/fr. The opt-in crab feature remains. |
| Bug e) license | DONE | LICENSE; About page; installer LicenseFile | Upstream MIT notice kept in THIRD_PARTY_LICENSES.md (required). |
| Floating + auto-hide bar reachable | PARTIAL | edge_watch.rs; 2 unit tests pass in CI | Logic tested; live reveal NOT captured. |
| Bottom bar above dock | DONE | docs/captures/shell-bottom-floating.jpg | Real runner capture. Popups from a bottom bar and multi-monitor NOT tested. |
| Settings redesign, search, preview, About | PARTIAL | docs/captures/settings-*.jpg | Narrow layout captured at 1024x768; wide layout with preview column NOT captured. Light theme only captured. |
| Photo wallpapers | PARTIAL | assets/wallpapers; ASSETS_LICENSES.md; settings-personalize capture | Embedded and applied through SystemParametersInfo; grid rendering not in a capture (page scrolled off). |
| Quick settings (Wi-Fi/Bluetooth/Cast panel) | PARTIAL | docs/captures/quick-top.jpg | Renders. Switching radios NOT tested (runner has none). Bottom-opening NOT captured visibly. |
| Release 4.2 / v4.1 release | NOT DONE |  | v4.1.0 already exists. 4.2.0 not tagged: needs a final review by you. |
| ARM64 build | PARTIAL | CI job arm64 green | Compiles; native release zip comes with the next tag. Not run on ARM hardware. |
| Code signing | BLOCKED | SIGNING.md | Needs your certificate or SignPath application. |
| winget / scoop manifests | NOT DONE |  | Need release hashes. |
| Performance measurements | NOT DONE | PERFORMANCE.md | No desktop for measuring. |
| Website redesign, Lighthouse, live URL check | PARTIAL | site/ builds | Photo pack, honest labels, 4.2 notes. Full dither/glass redesign, real screenshots section, Lighthouse NOT done. |
| FPS overlay, island presets, Launchpad start, drag motion, wallpaper rotation, per-monitor wallpapers, hotkey manager, import/export, onboarding | NOT DONE |  | Not started. |
| Clean-machine install/uninstall test | NOT DONE |  |  |
