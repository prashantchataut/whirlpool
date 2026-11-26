# Study 3: a second shell that restyles Explorer's taskbar

**What this is.** A black-box and source-reading comparison against another open-source Windows dock ("the reference"). Its source is public but carries **no license**, so it is treated as all rights reserved: nothing of its code, text or artwork is copied here, and its screenshots are **not** committed (see "Captures"). The repository secret `STUDY3_SOURCE` holds its location so no workflow, log or file in this repository names it.

**How it was run.** `.github/workflows/study3.yml` on GitHub-hosted Windows (Server 2025, build 10.0.26100, 1024x768, no GPU, no radios). Whirlpool is built from `main`; the reference is built from its public source on a `windows-2025-vs2026` runner (native controller with CMake, dashboard with `dotnet publish`; both built, exit code 0). `scripts/study3.ps1` launches each, waits, samples private memory and CPU, sends key presses, screenshots, then samples again after dismissal. Run 45726707534 (second pass) and 45725577295 (first pass). Numbers: `docs/captures/study3/*-numbers.json`.

## What it does differently, in one paragraph

It does not replace the taskbar: it **loads code into `explorer.exe`** to restyle Explorer's own taskbar, is documented as working on **one Windows 11 build only**, and falls back to drawing in free space on Explorer's taskbar. Whirlpool will not do this (fragile across builds, and in-process injection into the shell looks like malware to Defender). Whirlpool hides the taskbar, draws its own dock and restores the taskbar on quit. The consequence on the runner: the reference's native controller **exited at once** (build 26100 is not the build it targets; zero processes after launch, the stock taskbar unchanged), so the styled taskbar was never observed. Only its documented `--tray` mode and its dashboard window were measured.

## Feature table

| Idea (reference) | Whirlpool before | Whirlpool now (this session) | Verified how |
| --- | --- | --- | --- |
| Global search over windows, apps, settings, widgets | Palette: apps, files, actions, clipboard, calculator | Adds open windows, Settings entries and widgets, with `window:`, `setting:`, `widget:` filters | Ranking and filters unit-tested (core); UI compiled in CI, **not captured** |
| Inline unit and math calculator (`56 KB to MB`, `100 Mbps to MB/s`, `5'11" to cm`, `30 mpg to L/100 km`) | Math only | Units module: data (decimal and binary), rates, length, mass, volume, time, speed, temperature, fuel economy | 8 unit tests |
| Pinned widget plus up to four favorites, JSON profile | none | Model with pin-swap, reorder, normalising import; `whirlpool --widgets export|import FILE` | 7 unit tests; **no widget UI yet** |
| Per-app audio output routing and audio presets | none | Preset model and a plan that waits for apps without a stream | Unit-tested model only; **no Windows audio code, nothing routes audio** |
| Workspaces: reopen folder, URLs, chosen windows; tile side by side, stacked, grid; undo | none | Tiling geometry and bounded undo (tested, no overlaps for 1 to 9 windows); workspace data model | Unit-tested; **windows are not moved yet** |
| Battery alerts at set levels | none | Alerts at 20, 10, 5 % by default (configurable), one per level per discharge, reset on charger; Windows toast through hidden PowerShell with text restricted to plain characters | State machine and toast text tested; timer wired, **not seen on a laptop** |
| Meetings from a local `.ics` | none | Reader with join links for five known hosts only | 5 unit tests; **no widget UI yet** |
| Downloads widget, drag-out clipboard and file shelf | file shelf and clipboard history exist | unchanged | |
| Heavy panels close shortly after hiding, tiny background process | panels exit when closed; resident dock about 25 MB | `lifetime` state machine and `panelIdleCloseSecs` setting | State machine tested; **not wired** (see numbers) |
| Safe mode, installer update and rollback, release lifecycle test | updater with rollback; lifecycle and updater workflows | `whirlpool --safe-mode`: every optional feature off, in memory only | Config transform unit-tested; run **not captured** |
| Chat assistants, games, voice memos, local database | n/a | **Skipped on purpose** (needs a larger runtime) | |

## Measured on the runner

| | Whirlpool (x64, run 45726707534) | Reference (tray mode plus dashboard) |
| --- | --- | --- |
| Processes after launch | 3 | 4 |
| Private memory, idle, all its processes | 27.6 MB | 50.7 MB |
| Working set, idle | 13.2 MB | 153.4 MB |
| Idle CPU (10 s, one core) | 0 % | 0.31 % |
| With its panel open | Start grid: 38.8 MB total | dashboard: 61.8 MB private, 235 MB working set |
| 45 s after the panel was dismissed | **5 processes, 340 MB private (the panel was still alive)** | 4 processes, 52 MB private (dashboard still alive at 45 s in this run) |

Read honestly: Whirlpool's resident cost is about half of the reference's, but its **Start grid process stays alive and heavy (about 310 to 340 MB on the software renderer) after Escape**. The reference's documentation says its dashboard exits about 20 seconds after hiding; this run did not show that, but the hide key used may not be the one it expects, so that claim is neither confirmed nor refuted. The Whirlpool figure is a real defect worth fixing first (gap 1 below). One run each, one machine type, no GPU.

## Captures

`docs/captures/study3/whirlpool-start-grid.png` and `whirlpool-launched.png` are Whirlpool frames from the same run. The reference's frames were taken (they show Explorer's unchanged taskbar and a console window on the runner, because its controller exits on this build and its dashboard needs a signed-in user session) but are kept in a one-day private artifact: they carry another project's artwork and it is unlicensed. A side-by-side with its frames would redistribute that artwork, so none is published.

## Ranked gap list (value per megabyte of RAM)

1. **Close the Start grid, overview and search panels after hiding.** Measured 340 MB after dismissal. Wire `lifetime::Lifetime` (grace from `panelIdleCloseSecs`, default 20 s) into the panel processes. Zero extra resident cost.
2. **Search over windows, settings, widgets plus units**: done in logic and UI, needs a capture review and a Settings deep link (Setting rows open Settings, not the page). Costs nothing at idle (runs in the palette process).
3. **Widget slot and favorites UI** over the finished profile model, plus meetings and downloads widgets: model done; panels not built.
4. **Workspaces**: Win32 window moving, folder and URL launch over the tiling model; undo already modelled.
5. **Per-app audio routing**: needs the undocumented audio policy interface, which differs by Windows build. This is the same fragility we rejected for the taskbar, so it is the lowest-value, highest-risk item and is **not started**.
6. Battery alerts: wire a hardware check on a laptop.
7. Installer-level update action and a CI release lifecycle test: the updater end-to-end workflow already covers update and rollback; a lifecycle test of a tagged release exists as `lifecycle.yml`.
