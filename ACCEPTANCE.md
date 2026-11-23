# Acceptance tests

Each item is written before its code. "Outcome" is filled in from a real run; where it says
"pending" the item is not done.

## B1. Release pipeline and updater

**B1-a. Unit behaviour (runs anywhere, `cargo test -p whirlpool-update`)**
1. Version ordering: `4.2.0-rc1 < 4.2.0-rc2 < 4.2.0`, `4.10.0 > 4.9.9`.
2. Stable channel ignores pre-releases, drafts and links outside this repository; beta takes the highest.
3. A download whose SHA-256 differs from `SHA256SUMS.txt` is refused and deleted.
4. Cancel leaves no partial file. Offline, rate limit (with minutes) and HTTP errors have distinct messages.
5. ARM64 machines without an ARM64 installer get a clear message.

**B1-b. Release workflow**
1. Push tag `v4.2.0-rc1`. Expect: a GitHub pre-release with `Whirlpool-Setup.exe`, `Whirlpool-Setup-arm64.exe`, zips, `SHA256SUMS.txt`, notes listing the real commits.
2. Push tag `v4.2.0-rc2`. Expect: a second pre-release, newer than rc1.
3. The signing step reports `DRY RUN` and the release notes say the build is unsigned.

**B1-c. In-app update (workflow `Updater end to end`)**
1. Clean Windows runner, install rc1 from the published installer (hash checked).
2. `whirlpool --update --beta`: finds rc2, downloads, verifies SHA-256, starts Setup silently.
3. Installed version (uninstall registry key) becomes rc2 within 4 minutes; Whirlpool is running again.
4. `--update --beta --check-only` exits 0 (up to date).
5. `--update --rollback` installs rc1 again. Uninstall leaves no `whirlpool.exe`.
6. Settings > System > Updates shows current version, release notes, progress bar, Cancel, rollback row, and the privacy line.

## Outcomes (real runs on GitHub's Windows runners)

- **B1-a** `cargo test -p whirlpool-update`: 16 pass (Linux and Windows CI). PASS.
- **B1-b** tags `v4.2.0-rc1` and `v4.2.0-rc2` published as pre-releases with both installers, `SHA256SUMS.txt` and commit-based notes; signing step ran as DRY RUN and the notes say unsigned. PASS.
- **B1-c** workflow "Updater end to end", run 45089601584, `proof/B1/e2e.txt`: installed rc1; `--update --beta` found rc2, downloaded, verified SHA-256, started Setup; registry version became 4.2.0-rc2; Whirlpool was running again as 4.2.0-rc2 (`whirlpool-log-update.txt`); `--check-only` then exit 0; uninstall left no `whirlpool.exe`. PASS for update and relaunch.
  **FAIL for rollback:** `--update --rollback` exited 0 but the installed version stayed rc2. Suspected cause: the relaunched dock was still running and held the file. A fix (`/FORCECLOSEAPPLICATIONS`, Setup log `setup.log`) is committed and NOT yet verified. Signature verification is not built.
- **B2** run 45088081716, `proof/B2/`: the overlay started and stopped through the hotkey command for all three layouts; cost at 1 s interval: 0.013% of a 4-core machine, 18.6 MB working set, 7.9 MB private. The capture showed a label/value overlap (fixed afterwards, not re-captured) and 0 fps on an idle desktop (the compositor only presents frames when something changes). Hotkey press itself, drag and snap were not exercised on a desktop. PARTIAL.
- **B5** `proof/B5/overlay-and-rotation.txt`: three scripted steps over a 3-picture folder moved pic2 -> pic0 -> pic1 on a single display. Two displays, theme switching and the timer were not exercised. PARTIAL.
- **Quick settings** `proof/quick-settings/`: rendered in the real build, tinted by the wallpaper. The capture exposed a grey-tile blending bug; fixed, not re-captured.
