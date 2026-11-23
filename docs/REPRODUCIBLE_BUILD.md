# Build notes

Every release is built by GitHub Actions (`.github/workflows/release.yml`) on `windows-latest` with the stable Rust
toolchain, from a tag `vX.Y.Z[-suffix]`. The workflow sets the workspace version from the tag, runs
`cargo test --workspace` (x64), `cargo build --release --target <triple> -p whirlpool`, and compiles the Inno Setup
script `installer/whirlpool.iss`.

To rebuild a release yourself: check out the tag, install Rust stable and Inno Setup 6, set `version` in the
`[workspace.package]` of `Cargo.toml` to the tag, run the same cargo command, then
`iscc /DAppVersion=<version> /DNumericVersion=<a.b.c.0> /DArch=x64 installer\whirlpool.iss`.
Dependencies are fixed by `Cargo.lock`. The release profile is `lto = true`, `codegen-units = 1`, `opt-level = "s"`,
`panic = "abort"`, `strip = true`. Byte-identical output is not guaranteed (the PE timestamp and the installer
compressor are not pinned); compare the program's behaviour and the build provenance attestation instead:
`gh attestation verify <file> --repo prashantchataut/whirlpool`.
ARM64 binaries are cross-compiled on the x64 runner (`aarch64-pc-windows-msvc`).
