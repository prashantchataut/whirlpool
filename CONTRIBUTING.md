# Contributing

Thanks for helping. Whirlpool is MIT licensed and stays 100 % Rust.

## Setup

- Windows 10/11, Rust (MSVC), Visual Studio C++ build tools.
- `cargo run -p whirlpool` runs the dock. The dock hides the taskbar; `Ctrl+C` restores it, or run `cargo run -p whirlpool -- --restore`.
- `cargo run -p whirlpool-settings` runs settings.

## Before a pull request

```powershell
cargo fmt --all -- --check
cargo test --workspace
cargo build --release -p whirlpool -p whirlpool-settings
```

On Linux you can type-check everything with
`cargo check --target x86_64-pc-windows-gnu -p whirlpool -p whirlpool-settings`
and run the platform-independent tests with `cargo test -p whirlpool-core -p whirlpool-config`.

## Style

- No `unwrap`/`expect` in non-test code (the workspace lints warn on them).
- Keep drawing code allocation-free in the hot path; the dock redraws while the pointer moves.
- UI colour comes from the palette (`apps/settings/src/ui.rs`) or `scene.rs`; do not add one-off hex values.
- Add strings to all three files in `apps/settings/i18n`.

## Renders for the website

`site/renders/*.html` are screenshotted at: dock 1600x1000, menubar 1600x620, settings 1400x900
(`settings.html?light` for the Foam theme), device scale factor 1.5, into `site/assets/screens`.
