# Trust and verification

Whirlpool replaces part of your Windows shell, so you should be able to check what you run. This page says what we do,
what we cannot do yet, and how to verify a download.

## Why Windows may warn you

Microsoft SmartScreen and Defender judge a download by two things: whether it is **signed** with a certificate from a
trusted authority, and whether enough people have already run it (**reputation**). A brand-new open-source program has
neither. The warning means "unknown", not "malicious". We cannot buy reputation, but we can make the program easy to check.

## What we do

- **Open source, built in public.** Every release is built by GitHub Actions from a tagged commit
  ([`release.yml`](../.github/workflows/release.yml)); nobody uploads a binary from a laptop.
- **Provenance attestation.** Each release file has a signed GitHub build-provenance attestation that proves which
  workflow and commit produced it.
- **SHA-256 sums** in `SHA256SUMS.txt` on every release.
- **Honest binaries.** Full version information (publisher, product, original file name, copyright, link to this
  repository) and an `asInvoker` manifest: Whirlpool never asks for administrator rights. No packers, no obfuscation.
- **No telemetry.** Optional network use is listed in the README.
- **Safe by default.** If the dock exits, a watchdog gives the Windows taskbar back; `whirlpool.exe --restore` does it by hand.
- **PowerShell installer.** `irm .../install.ps1 | iex` downloads with PowerShell, which does not add the web
  Mark-of-the-Web to the files, so SmartScreen does not prompt. The script is short; read it first.

## Verify a download

```powershell
# 1. Hash
Get-FileHash .\whirlpool.exe -Algorithm SHA256        # compare with SHA256SUMS.txt

# 2. Provenance (needs the GitHub CLI)
gh attestation verify .\whirlpool.exe --repo prashantchataut/whirlpool
```

You can also upload the file to <https://www.virustotal.com> and compare the hash.

## If Defender or SmartScreen still warns

- SmartScreen: **More info → Run anyway**. It remembers the choice.
- Defender false positive: submit the file at <https://www.microsoft.com/wdsi/filesubmission> as "Incorrectly detected
  as malware". Open an issue so the report can be linked here.

## Code signing (the real fix)

An Authenticode certificate removes the "unknown publisher" prompt and lets reputation attach to the publisher
instead of each file. The release workflow already signs when two repository secrets exist:

| Secret | Value |
| --- | --- |
| `WINDOWS_CERT_PFX_BASE64` | the code-signing certificate (`.pfx`), base64 encoded |
| `WINDOWS_CERT_PASSWORD` | its password |

Free option for open source: apply to the [SignPath Foundation](https://signpath.org/) and replace the signing step
with their GitHub action. Until a certificate is in place, releases are unsigned and say so.
