# Session 8 captures

`before/` holds the state of the app on the head of PR #16 (commit 24e5fba), before any redesign:
six screens at 100, 150 and 175 % on `windows-latest` (x64) and `windows-11-arm` (ARM64), downsized JPEGs of
the runner's 1024x768 (100 %) and 1920x1080 (150/175 %) frames. They are screenshots of a running Windows Server 2025
session, taken by `scripts/drive.ps1` in the DPI pass workflow (run 45585789817) and the Screenshots workflow.
200 % cannot be set on a hosted runner (see `docs/capture-matrix.json`, `blockedScales`).
