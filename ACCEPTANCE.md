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
