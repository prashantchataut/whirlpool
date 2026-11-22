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
