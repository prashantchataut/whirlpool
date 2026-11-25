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

## Session 7 (window overview, palette polish)

Honest record. Opened this session: `impeccable` (SKILL.md, `reference/craft-floor.md`, `reference/animate.md`) and `animation-vocabulary`
(SKILL.md). **Not opened:** `frontend-design`, `design-taste-frontend`, `ui-ux-pro-max`, `web-design-guidelines`, `accessibility`, `prototype`.
The impeccable launcher scripts target web pages and were not run; checks were done by looking at runner captures.

| Skill | What it changed |
| --- | --- |
| `impeccable` craft floor ("Operate" mode for app UI) | The palette's selected row lost its one-sided accent bar (a listed refusal) for a full tint and border. Card depth became an offset, blurred shadow instead of a hard rectangle (on the near-black backdrop it is barely visible in the capture). Letters standing in for icons became drawn glyphs. Placeholder contrast raised. |
| `impeccable` animate reference | One motion for the overview: cards grow from 90 % while fading in, 200 ms, exponential ease-out; no stagger; none when Windows animations are off. |
| `animation-vocabulary` | Names used in review notes: scale-in, fade-in. Not done: origin-aware motion (cards do not fly out of their windows). |

Three looks at the real palette and overview captures (`docs/captures/session7/review-*.jpg`); findings are in `STATUS.md`.

## Session 8 (PR #17: Start grid and quick settings, round by round)

Honest record. **Opened this session:** `impeccable` (`SKILL.md` and `reference/craft-floor.md`), `accessibility` (contrast table, line 101 onward), `ui-ux-pro-max` (priority table in `SKILL.md`), `review-animations` (`SKILL.md` only; `STANDARDS.md` not read).
**Not opened:** `frontend-design`, `design-taste-frontend` (written for websites), `prototype`, `web-design-guidelines`, `theme-factory`, `animation-vocabulary`. The `impeccable` launcher scripts target web pages and were not run; checks were done by looking at runner captures. No skills were downloaded from outside the repository.

| Skill | What it changed |
| --- | --- |
| `impeccable` craft floor, Verify list ("batch the checks, fix what one look shows"; contrast 4.5:1; states incl. disabled) | Each round is one batched look at the captures, then one commit. Round 2 fixed the disabled tile state found in that look (Wi-Fi and Bluetooth tiles were dimmed to 55 % and unreadable; now 85 %). |
| `impeccable` craft floor, Refuse list (glass as decoration, one-sided accent bars, nested cards) | Kept: no side bars, shadows with offset and blur. Open: the Start grid still uses glass plates on every icon, which is close to the "glass as decoration" default. Not resolved. |
| `accessibility` (1.4.3 contrast) | Defines the 4.5:1 line the disabled tiles failed. Not yet measured with a tool: the check was by eye on the captures. |
| `ui-ux-pro-max` (priority 1 and 2: contrast, 44 px targets) | Read the table only; the 44 px touch-target rule was **not** audited against the tiles. |
| `review-animations` | Read the posture only. The entrance and slider motion were **not** reviewed against its standards this session. |
