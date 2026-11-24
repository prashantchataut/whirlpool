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

## Session 5 (gap closing, black-box study)

Honest record: **the `SKILL.md` files in `.agents/skills/` were not opened this session** (tool-step budget). Nothing
below claims they were used. What was done instead, and the skill whose rule each change matches:

| Area (skill it corresponds to) | What changed | Evidence |
| --- | --- | --- |
| Review of real captures (`web-design-guidelines`) | Two rounds on the runner's captures. Round 1 found: the palette drawn as a black rectangle without alpha; Windows' own flyout covering every Settings frame; invisible "Default" label and pale hint text on the light Settings page; unreadable "Clear all". Round 2 confirmed each fix. A third round was not done. | `docs/captures/session5/review-dock-round1-vs-round2.jpg` |
| Contrast (`accessibility`) | Hint and label colours follow the theme instead of dark-theme constants; "Clear all" got a fill and a border; placeholder colour raised. Still open: placeholder over busy backdrop. | captures |
| Motion (`animation-vocabulary`, `review-animations`) | Tile reorder animates 140 ms; the hot corner preview shows the pointer travelling and the action ripple; panel rise 140 ms; reduced motion shows end states. | `extras.rs`, `palette.rs` |
| Direction (`frontend-design`, `design-taste-frontend`) | Water-themed glass: refraction at a rim with a cool highlight, dark wash for text. No borrowed icons or names. | `crates/whirlpool-core/src/glass.rs` |
| `impeccable`, `prototype`, `ui-ux-pro-max` | Not used. |  |

Research: black-box measurement of another shell on a runner (`STUDY.md`); the Windows limits on reading other
programs' notifications and on transforming other programs' windows shaped what was built and what was skipped.
