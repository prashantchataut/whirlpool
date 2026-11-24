# Status

Release gate **not passed**. `4.2.0-rc4` is the intended next pre-release but is **not tagged yet** (the newest tag and release is `v4.2.0-rc3`). Stable stays held back: signing is unavailable, nothing ran on a personal desktop, HUDs, the DPI pass, the copy audit and the Settings redesign are NOT DONE, and the drag ghost is visibly wrong in the runner capture.
Proof lives in `proof/`, `docs/captures/session5/` (runner captures) and the linked workflow runs.

## Session 5 (branch `feat/gaps-session-5`)

Earlier rows below are kept as they were; where session 5 supersedes one, this table wins.

| Item | Status | Proof | Notes |
| --- | --- | --- | --- |
| Study of a reference shell (black-box) | DONE, with limits | `STUDY.md`, `scripts/study.ps1` | Resource numbers measured. Many of its scenes were not triggered (shortcuts unknown). Its frames are not published on purpose. |
| 1. Island presets in layout and Settings | PARTIAL | `02-island-*` (run 45387185126) | Settings row and live preview exist; the **opened** panel follows the preset. The **collapsed** island does not with two or more providers (see the B3 row). Earlier text here said DONE; the pixel comparison says otherwise. |
| 1. Start grid as the real Start window | PARTIAL | `06-launchpad-hotkey.jpg` | Window, search, grid, hotkey captured. The dock Start tile opens it in code (`Outside::OpenStart`) but no capture clicks it, and the arrow-key capture is invalid (see the B6 row). On a runner without OpenGL it appears after about 7 s. |
| 1. Drag motion: shadow, spring settle, snap preview | PARTIAL, looks wrong | `07-drag-*` (run 45387185126) | See the B4 row: ghost offset from the window and in front of it, dark fringe. Never transforms another program's window. |
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
| B3 Dynamic island, presets | PARTIAL | `02-island-*` (run 45387185126) | Wired through `menubar/layout.rs::island_metrics` and a Settings row. Opened panel: measured 43k to 66k differing pixels against Classic for Compact, Wide, Expressive. Collapsed island: width is a fixed 134 px per provider when two or more are shown, so the four `02-island-*-closed.png` are pixel-identical (0 differing). Only the one-provider width follows the preset (unit test, no capture). |
| B4 Window drag motion | PARTIAL, looks wrong | `07-drag-*` (run 45387185126), `apps/dock/src/drag.rs` | Hooked on `EVENT_SYSTEM_MOVESIZESTART`/`END`; thread exists only during a move; layered ghost overlay with shadow and snap preview; spring settle on release; `dragMotion` setting and Settings preview. Reviewed capture: held, the overlay is offset from the dragged window and drawn in front of it, with a dark fringe on its border; the snap preview position is sensible but has the fringe. `core::dragmotion` is not used by `drag.rs`. |
| B5 Wallpaper rotation, per-monitor, fit, dark folder | PARTIAL | `proof/B5/` | Rotation steps verified on one display. Not done: custom crossfade, live preview, bundled-pack theme switching, separate sources per display, two-display test. |
| B6 Launchpad-style Start grid | PARTIAL | `06-launchpad-*` (run 45387185126), `apps/settings/src/launchpad.rs` | Separate-process window (`--launchpad`, toggles), search, paged grid of Start Menu apps (real icons for some, letter tiles for others), focus ring, Escape closes, Ctrl+Alt+Space opens it (captured). Dock Start tile: code only. Arrow-key capture invalid: the driver typed "c1" and the shot reads "No apps match". |
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
