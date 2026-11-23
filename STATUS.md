# Status

Release gate **not passed**: B3, B4 and B6 are NOT DONE, so Phases 1 to 6 were not started, as the brief requires.
Proof lives in `proof/` (taken from GitHub's Windows runners, not a personal desktop) and in the linked workflow runs.

| Item | Status | Proof | Notes |
| --- | --- | --- | --- |
| B1 release workflow (x64 + ARM64, SHA256SUMS, commit notes, pre-release flag) | DONE | releases v4.2.0-rc1, v4.2.0-rc2 | Both published. |
| B1 updater: check, download, SHA-256 verify, install, relaunch | DONE | `proof/B1/e2e.txt`, run 45089601584 | rc1 updated itself to rc2 on a clean runner. Offline, rate limit, cancel, tamper are covered by unit tests, not by a live run. |
| B1 updater UI (progress, notes, rollback row, startup/N-day settings, privacy line) | PARTIAL | `proof/quick-settings/settings-system.png` | Built and compiled; the update flow was exercised through the CLI, not by clicking in the window. |
| B1 rollback | NOT DONE | `proof/B1/e2e.txt` | Failed in CI: version stayed rc2. Fix committed, unverified. |
| B1 signature verification in the updater | NOT DONE | | Needs a certificate. Hash only. |
| B2 FPS / system overlay | PARTIAL | `proof/B2/` | Hotkey command, three layouts, 0.013% CPU measured. Frame rate is the compositor's; per-game FPS, temperatures, power draw not offered. Drag, snap, hotkey press, fullscreen behaviour not exercised. |
| B3 Dynamic island, presets | NOT DONE | | Not started. |
| B4 Window drag motion | NOT DONE | | Not started. |
| B5 Wallpaper rotation, per-monitor, fit, dark folder | PARTIAL | `proof/B5/` | Rotation steps verified on one display. Not done: custom crossfade, live preview, bundled-pack theme switching, separate sources per display, two-display test. |
| B6 Launchpad-style Start grid | NOT DONE | | Not started. |
| B7 ARM64 build and installer | PARTIAL | release assets | Cross-compiled and packaged in CI; never run on ARM. |
| B7 signing wiring | PARTIAL | release logs (DRY RUN) | Dry run works; Azure/SignPath/PFX paths unexercised. BLOCKED on a certificate: see SIGNING.md. Nothing is claimed signed. |
| Quick settings redesign | PARTIAL | `proof/quick-settings/` | Wallpaper-adaptive, theme-aware. Tile blending bug found in the capture and fixed; not re-captured. |
| Phase 1 bug elimination, layout tests, screenshot regression, settings rework | NOT DONE | | Not started. Settings gained Updates, Overlay and Rotation sections only. |
| Phase 2 HUDs | NOT DONE | | |
| Phase 3 copy audit, site rebuild | NOT DONE | | README updated only. Site download links already use `releases/latest/download`. |
| Phase 4 extras | PARTIAL | | Only CLI (`--update`, `--wallpaper-next`, `--overlay-toggle`, `--version`) and the overlay hotkey. No IDEAS.md. |
| Phase 5 performance | PARTIAL | `PERFORMANCE.md` | Overlay cost measured. Release profile was already tuned. Idle CPU/RAM/startup not re-measured. |
| Phase 5 privacy + installer hygiene | PARTIAL | `docs/PRIVACY.md`, Settings text | Privacy stated in app and README. winget/scoop manifests NOT DONE (need a stable release hash). |
| Phase 6 stable v4.2.0 release | NOT DONE | | Held back: rollback unverified. Only pre-releases exist. |
| Phase 6 clean-install test from published artifact | PARTIAL | `proof/B1/e2e.txt` | Install, update, uninstall done on a runner from published installers. Not the full feature walk. |
