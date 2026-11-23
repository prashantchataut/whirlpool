# Decisions

Made without asking, in the order they came up.

1. **Fix the existing Rust app, do not rewrite it.** Whirlpool is a native Rust program (Win32, DirectComposition,
   egui for Settings). It already has one program, one installer and a correct LICENSE. The complaints that remain are
   behavior and design, so the work went there.
2. **Settings stays egui.** Rewriting it in a web view would add a runtime and a second renderer. The look was fixed
   through a new palette, a sidebar with grouped sections, search, a live preview column and drawn vector icons.
3. **Window fit comes from winit, not from a guess.** The old size came from Win32 metrics read before the window
   existed. The new `fit_window` reads the monitor size winit reports in the same units the window is sized in.
4. **Photographs are embedded.** Fourteen Pexels photographs (5.7 MB) live in the program: choosing one works offline
   and needs no hosted URL. The cost is a larger exe. The generated wallpapers remain in `whirlpool-core` (tested) but
   are no longer offered in Settings.
5. **Quick settings is a short-lived process.** `whirlpool.exe --quick-settings` is started by the bar and exits when it
   loses focus, so the dock holds no extra memory. Wi-Fi and Bluetooth really switch (WinRT Radios). Windows exposes no
   API to flip Cast, Night light or Battery saver, so those tiles open the Windows page that owns the setting.
6. **The upstream MIT notice stays.** Parts of the dock derive from an MIT-licensed project; MIT requires its
   copyright notice to travel with the code, so it stays in `THIRD_PARTY_LICENSES.md`. `LICENSE` is Prashant Chataut's.
7. **Version 4.2.0, not a re-cut 4.1.** v4.1.0 is already published; changes ship as 4.2.0.
8. **No fabricated proof.** Screenshots come from a workflow that runs the real build on a Windows runner. Anything
   that could not be run is listed as NOT DONE in `STATUS.md`.

## Session 3

9. **One update engine, two front ends.** `crates/whirlpool-update` holds all the logic behind an `Http` trait (tested
   on Linux and Windows with a fake server); Settings and `whirlpool --update` call it. The dock only records that
   a version exists, never downloads: the install always needs a click or an explicit command.
10. **Integrity, not yet authenticity.** The installer's SHA-256 comes from `SHA256SUMS.txt` of the same release over
    HTTPS, which defeats corruption and a bad mirror but not a compromised release. Signature checking waits for a
    certificate (SIGNING.md). The updater only follows links inside this repository.
11. **Rollback re-downloads.** Rollback fetches the previous release's installer (hash-checked) instead of keeping
    old binaries on disk. Releases before 4.2 do not restart Whirlpool after installing.
12. **ARM64 is cross-compiled** on the x64 runner. It compiles and packages; it has not run on ARM hardware.
13. **Overlay is a separate process** started by a global hotkey registered in the dock, so a hidden overlay costs
    nothing. Frame rate is the compositor's (DwmGetCompositionTimingInfo); per-game FPS needs ETW and administrator
    rights, so it is not offered. Temperatures and power draw are omitted because Windows exposes them only through
    vendor drivers.
14. **Wallpapers go through `IDesktopWallpaper`**, which Windows owns: no image stays in this program's memory, and
    per-monitor assignment, fit modes and the system's own transition come for free. There is no custom crossfade.
15. **Quick settings adapts by reading the wallpaper once per open** (a 4,000-pixel sample, in `whirlpool-core::tint`)
    and easing toward its colours; text colour is always chosen for 4.5:1 contrast.
16. **Cut for budget:** dynamic island presets and state machine (B3), window drag motion (B4), Launchpad-style Start grid
    (B6), the screenshot-regression set, website rebuild, and Phase 4 extras other than the overlay and the CLI. They
    are listed as NOT DONE in STATUS.md rather than shipped half-built.
