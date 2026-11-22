# Code signing

Whirlpool is not signed yet. A signature needs a certificate that only you can request. Everything on the build side is
ready: `release.yml` signs `whirlpool.exe` and `Whirlpool-Setup.exe` with `signtool` as soon as the two secrets below
exist, and skips signing when they do not.

## Option A: SignPath Foundation (free for open source)

1. Apply at https://signpath.org/ (Foundation program). It asks for the repository, the license (MIT) and a statement
   that the project is not malware-adjacent. Review takes days to weeks.
2. When accepted, install the SignPath GitHub app, create a project and a `release-signing` policy, and add the
   organization ID, project slug and API token as repository secrets.
3. Replace the two `Sign` steps in `release.yml` with `signpath/github-action-submit-signing-request@v1`, passing the
   unsigned artifact. SignPath returns the signed file.

## Option B: Azure Trusted Signing (paid, per month)

1. Create an Azure account, a Trusted Signing account and a certificate profile; complete identity validation
   (individual developers are supported in some regions).
2. Add `AZURE_TENANT_ID`, `AZURE_CLIENT_ID`, `AZURE_CLIENT_SECRET` as secrets and use
   `azure/trusted-signing-action@v0` in place of the `Sign` steps.

## Option C: a certificate you buy (OV or EV)

Export it as a `.pfx`, base64 it (`[Convert]::ToBase64String([IO.File]::ReadAllBytes("cert.pfx"))`) and store it as
`WINDOWS_CERT_PFX_BASE64`, with the password as `WINDOWS_CERT_PASSWORD`. The existing workflow steps use these.

## What to expect

A valid signature gives the publisher name in the UAC and SmartScreen dialogs. SmartScreen reputation still builds with
downloads over time, for signed files too; an EV certificate removes the warm-up period, the others do not. Until then
the README tells people how to verify the SHA-256 sum and the build attestation.
