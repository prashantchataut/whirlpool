# Theme: water column

Light at the surface, abyss at the bottom. One bright accent (foam), one warm signal colour (coral) used only for attention.

| Token | Value | Use |
| --- | --- | --- |
| Abyss | `#071419` | Settings window (dark) |
| Sidebar | `#0a1b22` | Settings sidebar |
| Card | `#0e252d` | Settings cards |
| Border | `#1b3a45` | Hairlines |
| Current | `#0ea5a8` | Accent: switches, sliders, selection |
| Foam | `#7aeede` | Highlights on dark; running-app dot on the dark dock |
| Coral | `#f2644a` | Update badge, errors |
| Shallows | `#eef6f7` | Settings window (light) |

## Where it lives

| Surface | File |
| --- | --- |
| Settings palette, vortex mark, waterline | `apps/settings/src/ui.rs` |
| Settings sidebar and nav | `apps/settings/src/app.rs` |
| Dock glass tint, separator, running dot | `apps/dock/src/scene.rs` |
| Context-menu tint | `apps/dock/src/menu.rs` |
| Hover labels, folder tile, fallback icon | `apps/dock/src/gfx.rs` |
| Mascot palette | `apps/dock/src/mascot.rs` |
| App icon (regenerate with `python scripts/make_icon.py`) | `assets/brand/` |

The vortex is two spiral arms drawn from vector strokes, in Rust (`ui::vortex`) and in the website (`site/src/components/Mark.tsx`), so both stay sharp at any size.
