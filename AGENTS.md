# Working on Whirlpool (rules for people and AI agents)

Whirlpool is a standalone dock and menu bar for Windows 10/11, written in Rust, with a marketing site in `site/`.
UI and UX are the most important part of this project. These rules are not optional.

## Rule 1: use the skills for every UI/UX task

The design skills live **in this repository** under [`.agents/skills/`](.agents/skills) and are pinned in
[`skills-lock.json`](skills-lock.json). Before you touch any visual or interaction work (the website, the settings
window, dock/menu-bar drawing, animation, icons, copy), read the relevant `SKILL.md` first and follow it.

| Skill | Use it for |
| --- | --- |
| `frontend-design` | Direction before code: palette, type, composition. Rejects generic templates. |
| `design-taste-frontend` | Visual variance / motion / density dials. Guards against default "AI look". |
| `ui-ux-pro-max` | Design-system queries, UX rules, accessibility, contrast, touch targets. |
| `web-design-guidelines` | A file:line UI review of finished work. |
| `accessibility` | The audit every public surface must pass (contrast, focus, reduced motion). |
| `impeccable` | Final craft pass: spacing, rhythm, polish. |
| `prototype` | Comparing several directions before committing to one. |

Workflow: **plan with `frontend-design`, query `ui-ux-pro-max`, build, review with `web-design-guidelines` +
`accessibility`, polish with `impeccable`.** Mention which skills you used in the PR description.

Keep the skills in the repo. Do not delete them, and add new ones with
`npx skills add <repo> --skill <name> --agent universal -y` (browse https://skills.sh).
The bulky sample photos of `design-taste-frontend` were pruned to keep the repo small; the skill works without them.

## Rule 2: read and update `continuity.md`

[`continuity.md`](continuity.md) is the living hand-off: what exists, what is verified, what is next. Read it before
starting; update it before you finish. If you add a user-visible feature, add it to the feature table.

## Rule 3: never ship what you have not compiled

`whirlpool.exe` only runs on Windows, and CI (`windows-latest`) is the source of truth. Run `cargo fmt --all`,
`cargo test --workspace` and `cargo build --release -p whirlpool -p whirlpool-settings` (or let CI do it) before merging.
Platform-independent logic belongs in `crates/whirlpool-core`, where it can be unit-tested anywhere.

## Rule 4: be honest about what was and was not verified

Screens in the README and site are **design renders** unless a file says it is a capture. Never describe a render as
a screenshot of a running Windows session. See `docs/TRUST.md` for the same standard applied to releases.

## Rule 5: energy first

The dock sleeps while idle (0% CPU) and paces animation with `DwmFlush`. Do not add polling timers faster than a few
seconds, do not keep threads alive between uses, and prefer Windows change notifications to polling.

## Rule 6: secrets never go in the repo

No tokens, keys or personal data in code, docs, commit messages or CI logs.

## Rule 7: the owner's standing brief (do not ask again)

Recorded so it is not repeated in every prompt. The owner wants, in the same session and without questions: professional tone everywhere public (no logo images or mascots on the website, no to-do-list copy; the wordmark is text); decisions made for them and explained afterwards; a pull request that includes renders; releases cut from the tag workflow (pre-release unless the gate in STATUS.md is passed); the app reminding users of updates; README, STATUS.md, CHANGELOG.md and continuity.md updated with the work. Nothing in them may claim more than was verified (Rule 4). Never write the owner's tokens anywhere (Rule 6); use them only through a per-command header.

