# Packaging

Manifests for the two package managers people ask for. They are generated, not hand-edited:

```powershell
./scripts/make-manifests.ps1 -Version 4.2.0 -Sums SHA256SUMS.txt
```

The hashes come from the release's own `SHA256SUMS.txt`, so a manifest can never disagree with the file it
points at. The copies checked in here are for the newest published release.

| Manager | Path | How to use it |
| --- | --- | --- |
| winget | `packaging/winget/` | `winget install --manifest packaging/winget` (local test), then submit to `microsoft/winget-pkgs`. |
| Scoop | `packaging/scoop/whirlpool.json` | `scoop install https://raw.githubusercontent.com/prashantchataut/whirlpool/main/packaging/scoop/whirlpool.json` |

**Status:** written and checked for shape by CI (`manifests.yml`). **Not yet submitted** to
`microsoft/winget-pkgs` and not yet in a Scoop bucket: winget review wants a stable release and a signed
installer (see `SIGNING.md`), and I would rather not open a submission that is rejected. Marked blocked, not done.
