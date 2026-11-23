# Skills used

Honest record for session 3. The web skill libraries named in the brief (open-design.ai, skills.sh, skillissue.in)
were not fetched in this session, so none of their text was applied and none is claimed. What informed the work:

| Area | What was used | What it changed |
| --- | --- | --- |
| Rust / windows crate | Reading the `windows` 0.62 API from the crate source, compiling against `x86_64-pc-windows-msvc` locally with `cargo check` before every push | Per-monitor wallpaper (`IDesktopWallpaper`), overlay counters (PDH, IP Helper, DWM), hotkey registration |
| Release engineering | GitHub Actions matrix, composite action, `gh release` | `release.yml`, `.github/actions/sign`, `updater-e2e.yml` |
| Colour and contrast | WCAG relative-luminance contrast | `whirlpool-core::tint`: the accent always carries text at 4.5:1 |
| Motion | egui `animate_bool_with_time`, reduced motion from the system setting | Quick settings tiles ease in 120-200 ms; no animation when Windows animations are off |

## Session 4 (4.2.0-rc3)

Honest record. Read from `.agents/skills/` before touching the site or its copy: `design-taste-frontend`, `copywriting`, `accessibility` (the repository's pinned copies). No new skills were downloaded this session: the pinned set already includes `agent-reach`, `animation-vocabulary` and the design skills, and nothing was fetched from skills.sh. Applied: copywriting (plain, specific, no "Not done yet" to-do voice on a public page), accessibility (the site keeps its skip link, focus styles and reduced-motion handling; nothing removed). Not applied: a reverse-engineering skill; no third-party project was decompiled or copied, and no code from FluidWin or similar projects was used.

