# Research notes

What informed the design of this pass, stated as decisions.

- **Palette.** Graphite neutrals with a faint sea tint, one teal accent. Photographs are the colorful element, so the
  chrome stays quiet. Dark: window `#121518`, card `#1c2126`, border `#2b3238`, text `#edf1f3`, muted `#95a2a9`.
  Light: window `#f3f5f6`, card `#ffffff`, border `#dbe1e4`, text `#13191d`, muted `#58666d`. Muted text is 5.6:1 on the
  dark card and 5.7:1 on the light one.
- **Type.** Inter Regular/SemiBold (bundled, OFL). Scale 10.5 (group labels) / 12.5 / 14 / 28 (page title).
- **Spacing and radius.** 4 px base: 8, 12, 16, 24, 32. Radius 8 controls, 10 nav, 12 cards, 16 panels.
- **Motion.** Switches animate 120 ms. The bar reveals in 180 ms and hides in 130 ms (existing cubic-bezier
  curves). No spring motion was added to Settings.
- **Preview beside controls.** Wide windows show a scaled sketch of the desktop driven by the live config, because
  the original complaint was a dead right side and because bar position/floating are hard to judge from labels.
- **Skills.** The catalog at open-design.ai lists skills; the repo already pins the relevant ones in `.agents/skills`
  (see SKILLS_USED.md).

Not done: browsing X, Pinterest and Reddit for references (no access from this environment).
