# Asset licenses

Every image, font and icon that ships with Whirlpool or its site, and the terms it is used under.

## Wallpaper pack (`assets/wallpapers/`, embedded in `whirlpool.exe`, shown on the site)

Fourteen photographs from [Pexels](https://www.pexels.com), used under the [Pexels License](https://www.pexels.com/license/)
(free to use and modify, no attribution required; credited here anyway). Files were resized to 2560x1440 and
re-compressed; nothing else was changed.

| File | Title | Photographer | Source |
| --- | --- | --- | --- |
| `misty-forest.jpg` | Misty forest | Soubhagya Maharana | https://www.pexels.com/photo/18359798/ |
| `misty-peaks.jpg` | Misty peaks | Luca Cavallin | https://www.pexels.com/photo/39360611/ |
| `valley-fog.jpg` | Valley fog | Quang Nguyen Vinh | https://www.pexels.com/photo/14024556/ |
| `aurora-lake.jpg` | Aurora over a lake | Damir K | https://www.pexels.com/photo/31284411/ |
| `aurora-iceland.jpg` | Iceland aurora | Piotr Kowalonek | https://www.pexels.com/photo/15264067/ |
| `milky-way.jpg` | Milky Way | S S | https://www.pexels.com/photo/3083808/ |
| `blue-hour.jpg` | Blue hour skyline | Mohammed Alim | https://www.pexels.com/photo/30123502/ |
| `city-lights.jpg` | City lights | zhou shen | https://www.pexels.com/photo/18834566/ |
| `turquoise-surf.jpg` | Turquoise surf | Jess Loiterton | https://www.pexels.com/photo/9259704/ |
| `sea-foam.jpg` | Sea foam | Jess Loiterton | https://www.pexels.com/photo/7502481/ |
| `dune-sunrise.jpg` | Dune sunrise | Francesco Ungaro | https://www.pexels.com/photo/998635/ |
| `dune-sky.jpg` | Dune and sky | Mike Norris | https://www.pexels.com/photo/30739226/ |
| `dune-ripples.jpg` | Dune ripples | MART PRODUCTION | https://www.pexels.com/photo/8869381/ |
| `dune-night.jpg` | Dune at night | Marjan Taghipour | https://www.pexels.com/photo/13620627/ |

## Site photographs (`site/src/assets/wall-*.jpg`)

Pexels photographs under the same license, credited in the captions on the site.

## Fonts

- Inter (Regular, SemiBold), SIL Open Font License 1.1, `apps/settings/assets/fonts/Inter-OFL.txt`.
- The site's fonts are listed in `site/src/index.css` and are open-licensed (OFL).

## Logo and icons

- `assets/brand/*`: the Whirlpool mark, drawn for this project by `scripts/make_icon.py`. MIT, like the code.
- Settings icons (`apps/settings/src/icons.rs`) are drawn in code for this project. MIT.

## Screenshots

Images under `docs/captures/` are photographs of the real build running on a GitHub-hosted Windows runner, produced by
`.github/workflows/captures.yml`. They contain Pexels wallpapers only where noted above.
