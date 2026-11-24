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

## Session 7 (redesign of Start and quick settings)

Honest record. This session the `SKILL.md` files in `.agents/skills/` were opened and read: `impeccable` (and its
`craft-floor.md`), `design-taste-frontend`, `animation-vocabulary`, `review-animations`, `ui-ux-pro-max`. The web
libraries named in the brief (open-design.ai, skills.sh, skillissue.in) were **not** fetched; nothing from them is claimed.
`frontend-design`, `prototype`, `theme-factory`, `vercel-react-best-practices` and `webapp-testing` are web skills and
were not applied. The skills are written for web pages, so what was used is the rule, not the tooling (no browser
tools exist for a native egui panel).

| Skill | Rule applied | What it changed |
| --- | --- | --- |
| `impeccable` craft floor | "Glass and blur as decoration rather than as a specific effect" is a default to refuse; "unicode glyphs standing in for an icon system"; contrast 4.5:1 | The blur is a real picture of the screen behind the panel (blurred once), not a translucent rectangle. Icons stay the drawn set in `icons.rs`. Start labels got a shadow because the first capture showed grey labels on a busy blur. |
| `impeccable` craft floor | "Declare elevation once"; shadows carry an offset | `design::drop_shadow` always has a downward offset; no zero-offset halos. |
| `review-animations` | Sub-300 ms, ease-out, no `scale(0)`, 30-80 ms stagger, reduced motion shows end states, frequent actions get no motion | Tiles enter from 90% scale with a back-ease (not from zero), rising 14-20 pt, staggered 34-40 ms on a diagonal, capped at 320 ms of delay. Paging and typing in search do not re-stagger. Hover and selection ease in 120-140 ms. Everything returns end states when Windows animations are off. Nothing repaints once settled. |
| `animation-vocabulary` | Terms | The brief's "springy stagger" is a *stagger* of *pop-in* entrances; implemented as `design::stagger` and `design::out_back`. |
| `design-taste-frontend` | One system, plain copy, no caption under an image | Both screens share one language (glass plate, soft shadow, one accent). Copy is "Search apps", "{n} apps", "In Settings"; no taglines. |
| `ui-ux-pro-max` | Touch/pointer target size and visible focus | Tiles stay at least 68 pt tall; keyboard focus ring kept on tiles and the volume slider. |
| `copywriting` | Specific over generic | Not applied this session: the site and in-app copy pass is **not done** (see STATUS.md). |
| `accessibility` | Contrast | Checked by eye on captures only; no contrast measurement was run on the new surfaces. Quick settings option C failed that eye check (small labels over bright blur), which is one reason it lost. |
