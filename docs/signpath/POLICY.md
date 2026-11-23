# Signing policy

- Only binaries built by `.github/workflows/release.yml` from a `v*` tag of this repository are submitted.
- The maintainer (Prashant Chataut) approves every signing request for a stable release; pre-release tags (with a
  `-` suffix) may be signed with the test policy only.
- No third-party binaries are signed. Dependencies come from crates.io, locked by `Cargo.lock`.
- Signed files are published with `SHA256SUMS.txt` and a build-provenance attestation.
