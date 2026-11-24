# Session 7 (branch `feat/session-7-design`)

**Done and captured:** Start grid and quick settings redesigned (three directions each, A chosen for both), real
frosted backdrop (`whirlpool_core::glass::frost`, tested), shared `design.rs`, Inter loaded in the Start grid (it was
rendering in egui's default font), winget and scoop manifests (written, CI-checked, **not submitted**), OpenGL-to-DirectX
fallback for the panels, capture driver scenarios for all of it. Bottom bar vs dock: no overlap in the x64 capture
(`docs/captures/session7/round1-03-bar-bottom-with-dock.png`). Notch (island) opens from the top edge in the x64 capture.

**Not verified:** Start grid after the round-2 tweaks (rows centred, label shadow): the round-2 runner had no OpenGL 2.0
and the panel did not start, which led to the fallback above; it has not been captured since. The update toast: no capture
exists. ARM64 captures of the new screens exist only for round 1 build (compiled and ran green).

**Not done (blocked by the step budget of this session, not by a technical wall):** Wi-Fi/Bluetooth/volume/brightness HUDs,
the 100/150/200% DPI pass, the Settings redesign, wallpaper crossfade and per-display sources, the copy pass, the site
rebuild, and the release (see PR for the recommended order). Nothing has been tested on real hardware.

# Status

Release gate **not passed**. `4.2.0-rc4` is the intended next pre-release but is **not tagged yet** (newest tag and release: `v4.2.0-rc3`). Stable stays held back: signing is unavailable, nothing ran on a personal desktop, and the HUDs, the DPI pass and the Settings redesign are NOT DONE. PR #11 (session 5) is merged; session 6 is PR #12 (`feat/gaps-session-6`). Captures in `docs/captures/session6/` come from runner run 45403289477 (windows-latest, x64) on commit `027d24d`; they are screenshots of a running Windows Server 2025 session, not renders.
Proof lives in `proof/`, `docs/captures/session5/` (runner captures) and the linked workflow runs.

## Sessions 5 and 6 (PR #11 merged; PR #12 `feat/gaps-session-6`)

Earlier rows below are kept as they were; where session 5 supersedes one, this table wins.

| Item | Status | Proof | Notes |
| --- | --- | --- | --- |
| Study of a reference shell (black-box) | DONE, with limits | `STUDY.md`, `scripts/study.ps1` | Resource numbers measured. Many of its scenes were not triggered (shortcuts unknown). Its frames are not published on purpose. |
| 1. Island presets in layout and Settings | DONE on a runner capture | `docs/captures/session6/02-island-*` (run 45403289477) | Collapsed island with two providers now follows the preset (`menubar/layout.rs`, unit test). Differing pixels against Classic, closed: Compact 3,253, Wide 3,940, Expressive 2,083; open: 69,569 / 47,835 / 64,843 (fuzz 2%). Choosing the preset in Settings writes the config the live island reads. Not seen on a personal desktop or on two monitors. |
| 1. Start grid as the real Start window | DONE on runner captures | `06-launchpad-start-tile`, `-hotkey`, `-search`, `-navigated` (run 45403289477) | The dock Start tile (real click), `Ctrl+Alt+Space`, search, arrow-key selection (focus moved down and right from the first app) and the app list are captured. Fixed this session: the search placeholder was nearly invisible; it is now painted by the window and legible. Icons fill in about a second after opening on the runner (letter tiles first). Still called "Launchpad" in Settings. |
| 1. Drag motion: shadow, spring settle, snap preview | PARTIAL: works, shadow subtle | `07-drag-*` and `-control` (run 45403289477), `apps/dock/src/drag.rs` | The shadow is drawn behind the dragged window and aligned with it: beside its right edge the blue channel runs 167, 187, 199, 206 against about 216 for the wallpaper. In a downsized screenshot it is faint, even after raising its strength this session. The blue rounded snap preview sits correctly inside the zone. The large grey rectangle in the capture and in the control (motion off) is Windows own snap preview on this server image, not Whirlpool's; the earlier reading "offset, in front, dark fringe" was wrong. The spring settle is code plus before/after captures, not a motion capture. Never transforms another program's window. `core::dragmotion` is not used by `drag.rs` (it uses `snap` and `spring`). |
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

## Session 7 (branch `feat/gaps-session-6`, same PR #12; pre-release `4.2.0-rc5`)

Correction: the text above says `4.2.0-rc4` is not tagged; a `v4.2.0-rc4` pre-release exists on GitHub (I did not check which commit it was built from). Release gate still **not passed**.
Everything below ran on GitHub runners (x64; the ARM64 job started but its frames were not reviewed). Nothing ran on a personal desktop.
Captures: `docs/captures/session7/` (run 45409486018, commit 1e78315). Study numbers: run 45408962749.

| Item | Status | Proof | Notes |
| --- | --- | --- | --- |
| Window overview (own panel, live previews) | DONE on the runner, with limits | `15-overview-*.jpg`; driver log "overview Enter: overview still open=False; foreground changed=True" | `whirlpool --overview`: full-screen cards with live DWM previews (DWM composes them; Whirlpool never reads another window's pixels), caption, close button on the chosen card, arrows, Enter, Esc, click-away, closes on focus loss. Opened by the palette action and by a hot corner set to "window overview". Layout is tested (`whirlpool-core::overview`, 6 tests). **Not exercised with real input:** clicking a card, the close button, the hot-corner route. **Not done:** app icons beside titles; a Settings page and hotkey of its own; the menu-bar Task View button still opens Windows Task View on purpose (virtual desktops). The earlier "Window overview NOT DONE" rows are superseded. |
| Palette polish | DONE | `08-palette-*.jpg`, `review-palette-round1-vs-round3.jpg` | The search field was invisible in the earlier capture (egui hint colour); now a magnifier and a painted placeholder. Action icons are drawn glyphs (`apps/settings/src/glyphs.rs`), not letters. Selected row: full tint and border, no side bar. Still open: every action row repeats "Action". |
| Review rounds | 3 for palette and overview, 1 for the rest | `review-*.jpg` | Round 1: session 6 captures; round 2: first build with the overview; round 3: craft pass. The Start grid and drag ghost were looked at once and **not** re-captured after session 6's last commit. |
| Memory (item 7) | UNCHANGED, measured | `STUDY.md` | Idle 24.4 MB private, 0.04 % CPU (main: 23.9 MB, 0.08 %). A panel open: 297 to 336 MB private on the runner's software renderer; 23 to 25 MB after it closes. I changed nothing here. |
| Study rerun against the reference's latest release | DONE | `STUDY.md` | Numbers only. |
| Reference frames, side-by-side images | NOT PUBLISHED on purpose | `STUDY.md` | They carry another company's names and icons; the standing brief forbids naming it. Side-by-sides in the repo compare Whirlpool with Whirlpool (before and after). |

## Earlier status (still accurate unless the table above says otherwise)

| Item | Status | Proof | Notes |
| --- | --- | --- | --- |
| B1 release workflow (x64 + ARM64, SHA256SUMS, commit notes, pre-release flag) | DONE | releases v4.2.0-rc1, v4.2.0-rc2 | Both published. |
| B1 updater: check, download, SHA-256 verify, install, relaunch | DONE | `proof/B1/e2e.txt`, run 45089601584 | rc1 updated itself to rc2 on a clean runner. Offline, rate limit, cancel, tamper are covered by unit tests, not by a live run. |
| B1 updater UI (progress, notes, rollback row, startup/N-day settings, privacy line) | PARTIAL | `proof/quick-settings/settings-system.png` | Built and compiled; the update flow was exercised through the CLI, not by clicking in the window. |
| B1 rollback | DONE | run 45090235367 (`Updater end to end`) | The first run failed (`proof/B1/e2e.txt`: stayed on rc2). After the fix, `--update --rollback` took rc2 back to rc1: "installed after rollback: 4.2.0-rc1 (wanted 4.2.0-rc1)". Earlier versions of this file still said unverified; that was out of date. |
| B1 signature verification in the updater | NOT DONE | | Needs a certificate. Hash only. |
| B2 FPS / system overlay | PARTIAL | `proof/B2/` | Hotkey command, three layouts, 0.013% CPU measured. Frame rate is the compositor's; per-game FPS, temperatures, power draw not offered. Drag, snap, hotkey press, fullscreen behaviour not exercised. |
| B3 Dynamic island, presets | DONE on a runner capture | see the Session 5 and 6 table | Superseded: all four presets differ, closed and open. |
| B4 Window drag motion | PARTIAL | see the Session 5 and 6 table | Superseded by the row above. |
| B5 Wallpaper rotation, per-monitor, fit, dark folder | PARTIAL | `proof/B5/` | Rotation steps verified on one display. Not done: custom crossfade, live preview, bundled-pack theme switching, separate sources per display, two-display test. |
| B6 Launchpad-style Start grid | DONE on runner captures | see the Session 5 and 6 table | Superseded: tile click and arrow keys are captured validly now. |
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

## Session 6: requested work not done

| Item | Status | Reason |
| --- | --- | --- |
| HUDs (Wi-Fi, Bluetooth, volume, brightness), custom volume slider | NOT STARTED | Needs new Windows code (audio endpoint, radio and brightness change events, a new overlay window) and a runner capture each. Rust is built only in CI here, so each attempt is a 15 to 25 minute loop. |
| Bug pass at 100/150/200 percent DPI | NOT STARTED | The runner has one 1024x768 display at 100 percent and the driver cannot switch scale yet. `TESTING.md` section 5 covers it by hand. |
| Copy audit | PARTIAL | Searched the site and `en.json` for generic marketing phrases: none found. Fixed the site roadmap, which said presets, drag and the Start grid were "not yet connected". The rest of the app text was not re-read line by line. |
| Settings redesign | NOT STARTED | Depends on the HUD settings and the DPI pass. |
| Three look-and-fix rounds per screen | PARTIAL | Presets, Start grid and drag had two rounds (read the capture, fix, recapture). Other screens were not reviewed again. |
| `TESTING.md` | DONE | Ten sections: real desktop, several monitors, scale, ARM, battery. |
| Release | NOT DONE | Waiting for PR #12 to merge and a decision on tagging `v4.2.0-rc4`. |
