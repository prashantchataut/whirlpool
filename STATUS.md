# Status

Release gate **not passed**. The 4.2.0-rc4 pre-release adds the session 5 work below; stable stays held back (signing is unavailable, the Windows desktop was only exercised on GitHub's runners, never a personal PC, and several items are partial).
Proof lives in `proof/`, `docs/captures/session5/` (runner captures) and the linked workflow runs.

## Session 5 (branch `feat/gaps-session-5`)

Earlier rows below are kept as they were; where session 5 supersedes one, this table wins.

| Item | Status | Proof | Notes |
| --- | --- | --- | --- |
| Study of a reference shell (black-box) | DONE, with limits | `STUDY.md`, `scripts/study.ps1` | Resource numbers measured. Many of its scenes were not triggered (shortcuts unknown). Its frames are not published on purpose. |
| 1. Island presets in layout and Settings | DONE | merged PR #10, island captures | Supersedes the B3 row. Four presets in the Menu bar page. |
| 1. Start grid as the real Start window | DONE | `06-launchpad-hotkey.jpg`; Settings preview | Dock button, hotkey, CLI. On a runner without OpenGL it appears after about 7 s. |
| 1. Drag motion: shadow, spring settle, snap preview | PARTIAL | `07-drag-snap-preview.jpg` | Supersedes the B4 row. Never transforms another program's window. Snap preview box still looks plain. |
| 2. Command palette (apps, files, actions, clipboard) | DONE | `08-palette-*.jpg` | Rounded and glass without relying on window alpha. Open: action icons are letters in circles; placeholder weak over a busy backdrop. |
| 3. Quick settings as an editable grid | DONE (editor logic unit-tested) | `12-quick-tiles.jpg`, `05-settings-menubar.jpg` | Ten tile kinds, small or wide. Drag reorder is tested in `whirlpool-core::tiles`, **not** exercised with real input in a capture. |
| 4. Notification centre with history | PARTIAL | `11-notifications.jpg` | Holds Whirlpool's and scripts' alerts (`whirlpool --notify`). Other programs' toasts cannot be read by an unpackaged app. Only the CLI feeds it today; the update reminder does not yet. |
| 4. Window overview | NOT DONE | | Hot corner and bar open Windows Task View. |
| 4. Hot corners | DONE | `13-corner-top-left.jpg`, `13-corner-top-right.jpg` | Top-left and top-right fired on real pointer input. Bottom corners and the lock/desktop actions not exercised. 120 ms timer only while a corner is set. |
| 5. Dock icon styles | DONE | `14-dock-*.jpg`, Dock page | Default, dark, clear, tinted. |
| 5. Glass refraction in our own shader | DONE as a CPU shader | `glass.rs` tests, captures | Runs once per panel open on a screen grab. It is not a GPU shader. Not applied to the dock itself or to the quick panel. |
| 6. Quit key, taskbar rescue key, install log, clean uninstall | DONE on the runner | `10-after-R.png`, `lifecycle.yml` run | Rescue and quit keys exercised with real keys. The lifecycle workflow passes; I did not re-read its log this session. |
| 7. Memory | PARTIAL, honest | `STUDY.md` table | Idle did **not** improve (22.5 to 22.8 MB private, 0 % CPU, both already small). After a panel closes the working set is now 0.9 / 10.8 MB instead of 14.9 / 21.9 MB. While a panel is open it is 297 to 333 MB private on the runner's software renderer, before and after. |
| Review of captures | 2 rounds, not 3 | `review-dock-round1-vs-round2.jpg` | Third round not done. |
| Reference's frames, side-by-side images | NOT DONE on purpose | `STUDY.md` | Would carry another company's names and icons. |
| Own window overview, file preview, window grouping | NOT DONE | `STUDY.md` gaps | |
| Real-desktop verification | NOT DONE | | Everything above ran on GitHub runners (x64; ARM64 for the earlier screenshots). |

## Earlier status (still accurate unless the table above says otherwise)

| Item | Status | Proof | Notes |
| --- | --- | --- | --- |
| B1 release workflow (x64 + ARM64, SHA256SUMS, commit notes, pre-release flag) | DONE | releases v4.2.0-rc1, v4.2.0-rc2 | Both published. |
| B1 updater: check, download, SHA-256 verify, install, relaunch | DONE | `proof/B1/e2e.txt`, run 45089601584 | rc1 updated itself to rc2 on a clean runner. Offline, rate limit, cancel, tamper are covered by unit tests, not by a live run. |
| B1 updater UI (progress, notes, rollback row, startup/N-day settings, privacy line) | PARTIAL | `proof/quick-settings/settings-system.png` | Built and compiled; the update flow was exercised through the CLI, not by clicking in the window. |
| B1 rollback | DONE | run 45090235367 (`Updater end to end`) | The first run failed (`proof/B1/e2e.txt`: stayed on rc2). After the fix, `--update --rollback` took rc2 back to rc1: "installed after rollback: 4.2.0-rc1 (wanted 4.2.0-rc1)". Earlier versions of this file still said unverified; that was out of date. |
| B1 signature verification in the updater | NOT DONE | | Needs a certificate. Hash only. |
| B2 FPS / system overlay | PARTIAL | `proof/B2/` | Hotkey command, three layouts, 0.013% CPU measured. Frame rate is the compositor's; per-game FPS, temperatures, power draw not offered. Drag, snap, hotkey press, fullscreen behaviour not exercised. |
| B3 Dynamic island, presets | PARTIAL | `crates/whirlpool-core/src/island.rs` (tests) | Geometry for Compact, Classic (= today), Wide, Expressive, opening curve and on-screen clamping, unit-tested. NOT connected to `menubar.rs` and not in Settings: no user can pick a preset yet. |
| B4 Window drag motion | PARTIAL | `crates/whirlpool-core/src/dragmotion.rs` (tests) | Spring follow, lean, lift and settle, unit-tested (no overshoot, stable at huge frames). NOT connected to any hook; Windows offers no supported way to transform another app's window, so the shell side needs a design decision (ghost overlay vs. none). |
| B5 Wallpaper rotation, per-monitor, fit, dark folder | PARTIAL | `proof/B5/` | Rotation steps verified on one display. Not done: custom crossfade, live preview, bundled-pack theme switching, separate sources per display, two-display test. |
| B6 Launchpad-style Start grid | PARTIAL | `crates/whirlpool-core/src/launchpad.rs` (tests) | Grid fitting, paging, slot rectangles, hit testing, swipe paging and ranked search, unit-tested. No window, no app list, no hotkey yet. |
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
| Update reminder | PARTIAL | `crates/whirlpool-update` tests, CI compile | Cadence (once per version, again after 3 days, 3 times max), clock-set-back and script-injection cases are unit-tested. The toast itself has NOT been seen on a real desktop; `whirlpool --update --remind-test` exists to check a machine. |
| Website without logo, honest roadmap | DONE | `docs/renders/site-4.2-*.jpg` | Rendered in headless Chromium; no logo image on the page (checked by script). |
