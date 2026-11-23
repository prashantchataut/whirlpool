# Code signing

**Whirlpool builds are not signed.** No certificate is held by this project and none has been bought. The release
workflow has everything wired; it signs the moment one of the three secret sets below exists, and until then it runs
in **dry-run** mode: the log says `SIGNING DRY RUN`, lists the files it would sign, changes nothing, and the release
notes say the build is unsigned. The workflow never self-signs and never reports `signed=true` unless Windows
verifies the signature (`Get-AuthenticodeSignature` must say `Valid`).

The wiring is `.github/actions/sign/action.yml`, called twice from `release.yml` (the program, then the installer).

## What you do, in order

1. **Pick one route.** SignPath Foundation is free for open source (slow to approve). Azure Trusted Signing is paid
   per month and fast. A purchased OV/EV certificate works too.
2. **SignPath Foundation**
   1. Apply at https://signpath.org/ using the text in [`docs/signpath/APPLICATION.md`](docs/signpath/APPLICATION.md).
      The repository already has what they ask for: a license (MIT), a [privacy statement](docs/PRIVACY.md),
      [reproducible build notes](docs/REPRODUCIBLE_BUILD.md) and a [signing policy](docs/signpath/POLICY.md).
   2. After approval, install the SignPath GitHub app and create the project `whirlpool`, the signing policy
      `release-signing` and an artifact configuration `default` that signs `*.exe` (Authenticode).
   3. Add repository secrets `SIGNPATH_API_TOKEN` and `SIGNPATH_ORGANIZATION_ID`. If your slugs differ, edit the
      `signpath-*` defaults in `.github/actions/sign/action.yml`.
3. **Azure Trusted Signing** (instead of step 2): create a Trusted Signing account and certificate profile, finish
   identity validation, then add secrets `AZURE_SIGNING_ENDPOINT`, `AZURE_SIGNING_ACCOUNT`, `AZURE_SIGNING_PROFILE`,
   `AZURE_TENANT_ID`, `AZURE_CLIENT_ID`, `AZURE_CLIENT_SECRET`.
4. **Purchased certificate** (instead of step 2): export a `.pfx`, run
   `[Convert]::ToBase64String([IO.File]::ReadAllBytes("cert.pfx"))`, store it as `WINDOWS_CERT_PFX_BASE64` and the
   password as `WINDOWS_CERT_PASSWORD`.
5. **Test** by pushing a tag such as `v4.2.1-rc1`. In the run, the two "Sign" steps must say `Signing with: <mode>`
   and the "Verify and report" step must print `Valid`.
6. **Then** remove the "not code-signed" sentence from the README (the release notes switch by themselves) and make
   the in-app updater verify the signature as well as the hash (not built yet: `STATUS.md`).

The SignPath and Azure steps have not been exercised: no account exists. Only the dry-run path has run in CI.

## What a signature does and does not do

It shows your publisher name in SmartScreen and UAC and ties the file to you. SmartScreen *reputation* still builds
with downloads over time. Until then, verify downloads by hash: see the README.
