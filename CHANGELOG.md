# Changelog / Cambios

Each release lists its changes in English and in Spanish. The release workflow
copies the section of the version being published into the GitHub release.

Cada versión lista sus cambios en inglés y en español. El flujo de publicación
copia la sección de la versión que se publica a la release de GitHub.

## 4.2.0-rc11

- Updater: `rc10` was ordered before `rc9` (plain text order), so anyone on rc9 or older was never offered rc10. Release numbers now compare by value. Install rc10 by hand once, from then on updates work.
- Desktop clock: a large time and the date on the desktop, behind the windows, click-through. Off by default. Settings, Clock, Desktop clock.
- Shorter wording for the lyrics hint and privacy line.
- Wallpaper studio check: a workflow applies a real 4K picture from 4kwallpapers.com through the studio on a Windows runner and reads the wallpaper back. No screenshot is taken or published.
- Captures: dock pinned with real apps, desktop clock on, `docs/captures/session14/`.

## 4.2.0-rc10

- Wallpaper studio: a tinted tile per group while a preview is missing (it was a blank white box).
- Quick settings: no "Plugged in" cluster on a PC without a battery; it ran into the two footer links.
- Search: shorter placeholder and empty text (the long one was cut off).
- Shorter wording for the update banner, clock subtitle, crab, focus and dark wallpaper hints.
- Captures: dark mode, a real media session with cover art in the island, the runner's own dialogs closed before every frame, `docs/captures/session13/`.

## 4.2.0-rc8

- Search: open windows, Settings entries and widgets join apps, files, actions and clipboard. `window:`, `setting:` and `widget:` narrow the list. Calculator rows now also convert units (`56 KB to MB`, `100 Mbps to MB/s`, `5'11" to cm`, `212 F to C`, `30 mpg to L/100 km`). Set `paletteHotkey` to `Ctrl+K` if you want that key (it is global, so it is not the default).
- Battery alerts: one Windows notification at 20, 10 and 5 % (`batteryAlertLevels`), once per level per discharge, only on computers with a battery, checked once a minute.
- `whirlpool --safe-mode`: starts with every optional feature off (hot corners, pop-ups, rotation, glass refraction, update checks, alerts). Nothing is written to your saved settings.
- `whirlpool --widgets export FILE` and `import FILE`: the pinned widget and favorites as a JSON profile.
- Tested building blocks without a screen yet: window tiling with undo, `.ics` reader, audio presets, workspaces, panel lifetime. No behaviour change from these in this release.
- Study 3 (`STUDY-3.md`): measured another shell next to Whirlpool. It showed that Whirlpool's Start grid process stays alive after it is dismissed (known issue, not fixed here).
- README Status section rewritten to match the captures.

## Unreleased

Nothing yet.

## [4.2.0-rc7]

Pre-release. Checked on GitHub runners only (x64 and ARM64, 100, 150 and 175 percent); nothing was run on a personal desktop or on ARM hardware (see TESTING.md). Installers are not signed.

### Changed / Cambiado
- **Start grid:** apps that have no icon get a lettered tile in a fixed colour palette; labels wrap to two lines.
  Cuadrícula de inicio: las aplicaciones sin icono muestran una ficha con su inicial; las etiquetas ocupan dos líneas.
- **Quick settings:** the volume control is one custom slider (track, fill, knob); tiles have a state dot; the footer buttons are outlined; tiles for Wi-Fi and Bluetooth that are not available stay readable (subtitles measured at about 5.9:1 on a runner capture).
  Ajustes rápidos: control de volumen propio, punto de estado en cada ficha, botones del pie con contorno y subtítulos legibles.

### Fixed / Corregido
- The website's download buttons pointed at v4.1.0 (GitHub's "latest" skips pre-releases). They now name the release.
  Los botones de descarga del sitio apuntaban a v4.1.0; ahora indican la versión exacta.

### Not in this release / Fuera de esta versión
- Brightness pop-up trigger, wallpaper crossfade and a source per display, the Settings sidebar at 1024 px, 200 percent scaling, signed installers, verification on ARM hardware. Details in STATUS.md.

## [4.2.0-rc6]

Pre-release. Checked on GitHub runners only; nothing was run on a personal desktop (see TESTING.md).

### Added / Añadido
- **Pop-ups for volume, Wi-Fi and Bluetooth.** A small glass pill (drawn speaker, gradient fill that glides to the new level, knob, number; Wi-Fi shows four signal steps, Bluetooth a switch) appears when the value changes and leaves after about two seconds. It lives in the dock process: no extra process, nothing running while it is away. A switch in Settings turns it off. `whirlpool --hud volume|muted|brightness|wifi|bluetooth <value>` shows one on demand. Brightness has the picture but no change notification feeds it yet.
  Ventanas emergentes de volumen, Wi-Fi y Bluetooth, con un interruptor en Ajustes. El brillo tiene el dibujo, pero todavía ningún evento lo activa.
- **One custom slider in Settings** (accent rail, growing knob, focus ring, arrow keys, value chip) in place of ten default sliders.
  Un único control deslizante propio en Ajustes.
- Start grid and quick settings redesigned (blurred backdrop, glass plates, Inter type, water-level volume capsule), OpenGL-to-DirectX fallback for the panels, winget and scoop manifests (written and CI-checked, not submitted).
- Window overview (`whirlpool --overview`), search palette with drawn icons.

### Fixed / Corregido
- The pop-up timer id collided with the wallpaper timer in the first build of this feature; found in a runner capture (the pill never left) and fixed before this release.

### Known gaps / Pendiente
- Settings redesign is partial: sliders only. At 1024 px the sidebar folds to icons and the first card runs under the scrollbar.
- Wallpaper crossfade and per-display sources, the 200 % display-scale pass (hosted runners top out at 175 %), signed installers, and a run on real hardware are not done.

## 4.2.0-rc5 (pre-release)

- Window overview: `whirlpool --overview` shows every open window as a live card (previews composed by DWM). Arrows, Enter, Esc, click, close button. Opened by the palette action "Show all windows" and by a hot corner set to window overview.
- Search palette: visible search field and placeholder, drawn icons for actions, files and clipboard, selection as a tint with a border.
- Runner captures and review notes in `docs/captures/session7/`. Not verified on a personal desktop.

## 4.2.0-rc4 (pre-release, not tagged yet)

- Quick settings tiles can be reordered by dragging, made wide or small, added and removed (Menu bar page).
- Notification history window and `whirlpool --notify "title" "text"`; history can be turned off.
- Hot corners with eight actions per corner (Windows page); they do nothing over full-screen apps.
- Dock icon styles: default, dark, clear, tinted (Dock page).
- Glass with refraction for search and notifications; strength in Personalize.
- Search and notification panels no longer show a black rectangle when the graphics driver has no OpenGL.
- Island presets (Compact, Classic, Wide, Expressive) change the live island, closed and open, with one provider or several.
- The dock's Start button and `Ctrl+Alt+Space` open the app grid (search, arrow keys, Enter). Its search placeholder is legible now.
- Dragging a window: a shadow behind it, a snap preview at screen edges and corners, and a spring settle on release. Windows does not allow tilting another program's window, so there is none. The shadow is stronger than in the first build.
- Website roadmap corrected to match what runs.
- Known gaps: no own window overview; panels use about 300 MB while open without OpenGL; volume, Wi-Fi, Bluetooth and brightness pop-ups, the display-scale pass and the Settings redesign are not done. Everything above was checked on GitHub runners only (`TESTING.md` lists what needs a real desktop).

## [4.2.0-rc3]

### Added
- **Update reminders.** When a background look finds a newer release, Windows shows one notification. It repeats after three days, three times at most per version, and never downloads or installs anything. Settings > System > Updates has a switch for it. "Also check when Whirlpool starts" is now on for new installs (existing settings keep their value). `whirlpool --update --remind-test` shows the notification for a made-up version, to prove it works on a machine.
- Tested engines in `whirlpool-core`, not yet connected to the shell: dynamic island presets (`island`), a Launchpad-style paged grid with search (`launchpad`) and spring motion for dragged windows (`dragmotion`).

### Changed
- The website no longer shows any logo image; the wordmark is plain text. "Not done yet" became a Roadmap that says what is written, tested and connected.
- Updater rollback is recorded as verified: rc2 rolled back to rc1 on a clean runner (run 45090235367).

### Not changed, and said plainly
- Builds are unsigned (SIGNING.md). The ARM64 build has not run on ARM hardware. The reminder notification has not been seen on a real desktop; its logic is unit-tested and the script is checked for injection.

### Añadido (resumen)
- **Recordatorios de actualización:** una notificación de Windows cuando hay una versión nueva; se repite a los tres días, como máximo tres veces por versión. Nunca descarga ni instala sola. Se desactiva en Ajustes > Sistema > Actualizaciones.
- Motores probados en `whirlpool-core`, aún sin conectar: presets de la isla, cuadrícula tipo Launchpad y movimiento de ventanas arrastradas.
- El sitio web ya no muestra ningún logotipo.

## [4.2.0]

### Fixed
- A floating menu bar set to auto-hide can be revealed again: the reveal strip now reaches the screen edge across the float gap.
- A menu bar on the bottom edge stands above a bottom dock instead of underneath it.
- Settings sizes itself from the monitor winit reports, so it no longer opens larger than the screen.

### Changed
- Settings redesigned: grouped sidebar with icons, search, a live preview column on wide windows, an icon-only sidebar on narrow ones, About page with the license text, new palette.
- Wallpapers are fourteen Pexels photographs stored in the program (set as Fill); the generated wallpapers are no longer offered.

### Added
- **Updater that updates.** Settings > System > Updates and `whirlpool --update` check GitHub Releases, compare versions (pre-releases ordered correctly), download the installer for this CPU (x64 or ARM64), refuse it unless its SHA-256 matches `SHA256SUMS.txt`, install silently and relaunch. Progress bar, release notes, cancel, rollback to the previous version, clear messages for offline and rate limits. Settings for check on startup, check every N days and release candidates. The old button only opened the release page and the dock's daily check did nothing.
- **System overlay** (Ctrl+Shift+F10 by default): CPU, memory, GPU load, video memory, network, battery and the desktop compositor's frame rate; three layouts, four corners with snapping, drag, opacity, size, click-through, update interval, hide over full-screen apps. A separate process: nothing runs while it is hidden.
- **Wallpaper rotation**: off by default; every 5 minutes to 1 day, sequential or shuffled, from a folder or the bundled photographs, a folder for dark mode, fill/fit/stretch/tile/center, a different picture on each display. `whirlpool --wallpaper-next`.
- Quick settings redesigned: colours follow the wallpaper and the system theme, animated tiles, battery drawn to scale, keyboard focus and labels.
- Release workflow: x64 and ARM64 installers, `SHA256SUMS.txt`, notes from the commits, pre-releases for tags with a suffix, signing step that dry-runs without secrets. Installer: `/relaunch=1`, ARM64 variant.
- Quick settings panel from the bar's network/volume area: Wi-Fi and Bluetooth switches, Airplane mode, volume, shortcuts to Cast, Night light and Battery saver.
- ARM64 build job; `WHIRLPOOL_PAGE` opens a Settings page; capture workflow; PRIVACY, SIGNING, AUDIT, ASSETS_LICENSES.

## [4.1.0]

### Added
- **Menu bar on the bottom edge.** Settings > Menu bar > Screen edge. The bar, its island and its popups, the hide and reveal motion, the click-away hook, the pointer reveal strip and the AppBar reservation all follow the chosen edge. On the bottom the island is drawn as a bubble that opens upward.
- Website: four more free-to-use photographs, a Screen edge control in the live Settings lab, and a bottom-bar desktop preview.
- `THIRD_PARTY_LICENSES.md`: all third-party notices in one file, shipped with the installer and the zip.
- Skills: `theme-factory`, `webapp-testing`, `animation-vocabulary`, `review-animations` added under `.agents/skills/`.

### Changed
- `NOTICE` is short and leads with Whirlpool; the upstream notice that the MIT licence requires moved to `THIRD_PARTY_LICENSES.md`.
- Menu bar alignment is labelled "Alignment" (it was "Bar position", which now means the edge).
- A side dock no longer leaves room at the top when the bar is on the bottom.

### Not done (see continuity.md)
- FPS and system-info overlay, a full dynamic island with presets, motion for dragged windows. These need a real desktop to design and verify.

### Cambios (resumen)
- **Barra de menús abajo.** Ajustes > Barra de menús > Borde de la pantalla. La isla se abre hacia arriba como una burbuja.
- Sitio: cuatro fotografías libres más y el control de borde en el laboratorio de Ajustes.
- `THIRD_PARTY_LICENSES.md` reúne los avisos de terceros.

## [4.0.0]

### Added
- Charging: two new animations, **Bloom** (a soft glow rises behind the dock) and **Aurora** (slow ribbons of colour across the top). Six styles in total.
- Motion: **Dreamy** and **Rapid** animation speeds, and a **Float** start-up.
- Menu bar: five new tints (Violet, Mint, Sunset, Slate, Sand).
- Settings: if OpenGL cannot start, Settings retries with DirectX (`--settings-compat`) so the window always opens.
- Site: rebuilt around live renders of the real Settings window and the six charging scenes; README renders in `docs/renders/`.

### Changed
- **Halo** is now one small glass pill: a breathing dot in the battery colour and the level. No bolt, no meter.
- Notices: `LICENSE` is Whirlpool's own MIT licence; third-party credits live in `NOTICE`.

### Not done (see continuity.md)
- Menu bar on the bottom edge, FPS / system info overlay.

## [Unreleased]

## [3.0.0]

### Added
- **Four charging animations.** Surge (a small medallion and two thin rings), Halo (a slim glass capsule), Edge glow (the screen edges glow like liquid glass: green above 60%, yellow in the middle, orange at 20% and below) and Tide (a line of light along the top edge up to the level). Pick one in Settings, or turn charging animations off.
- **Menu bar options:** colour (Ocean, Graphite, Midnight, Forest, Amber, Rose), width and left/centre/right position for the floating bar, and an optional edge highlight.
- **12 wallpapers in four new styles:** Tidal, Halftone, Ripple and Silk (66 in total).
- **More motion:** animation styles Fluid and Brisk; startup animations Drift and Dart, chosen from drop-downs.
- If Settings cannot open its window (no OpenGL, for example) it says why and points to the log.

### Changed
- Halo is a small capsule instead of a large glow; Surge is smaller with thinner rings.
- The crab is off by default and stored under a new key, so earlier files that enabled it do not bring it back.
- Network access uses the Windows TLS stack (Schannel). The bundled crypto library is gone: a smaller program with a plainer import table.
- The release build is optimised for size.
- `LICENSE` carries Whirlpool's own notice; `NOTICE` lists bundled components and is installed alongside it.

## [2.0.0]

### Fixed
- **Settings did not open.** The dock looked for a separate `whirlpool-settings.exe` that was missing for anyone who downloaded only `whirlpool.exe`. Settings is now part of `whirlpool.exe`.
- **Old logo in Settings.** The window drew an old hand-made mark. It now shows the real logo, in the sidebar and as the window icon.
- **No logo on the executable.** The icon file is rebuilt (classic bitmap frames for small sizes, PNG for 256 px) and the installer carries it too.
- **The crab appeared when charging.** Charging has its own animation now.

### Added
- `Whirlpool-Setup.exe`, a per-user installer (Inno Setup) with Start menu entries, optional start with Windows, uninstall and taskbar restore.
- Charging animation (*Surge* or *Halo*) with the real battery level.
- Menu bar style (docked or floating), size (compact, regular, large), side margin, top gap and corner radius.
- 30 wallpapers in four new styles (aurora, topographic, grid, dither): 54 in total.
- Starting Whirlpool again while it runs opens Settings.

### Changed
- One executable instead of two. The release no longer contains `whirlpool-settings.exe`.

## [1.0.0]

### English

- **A new mark.** A luminous three-arm vortex on a glass squircle, generated from maths (`scripts/make_icon.py`) so it is reproducible. New `.ico` (16 to 256 px, bolder at small sizes) and SVG.
- **Wallpapers.** 24 procedural wallpapers (vortex, mesh gradients, horizons) drawn at the exact resolution of your screen, with Shuffle. Nothing is downloaded.
- **Personalize page** in Settings: animation style (Relaxed / Smooth / Snappy), startup animation (Off / Glide / Rise / Swift), a separate charging-animation switch, menu bar tint strength, date on/off and a 24h / 12h / Windows clock.
- **Smoother motion.** The dock rises from its edge on start and sinks into it when you quit.
- **Settings fits your screen.** The window sizes itself to the monitor's work area and scale and declares per-monitor DPI awareness, so it is sharp and never taller than the display.
- **The Windows taskbar stays hidden.** A second layer (an empty window region) and a five-second repair check keep it hidden when Explorer brings it back. `whirlpool.exe --restore` still returns it.
- **Trust.** Both executables carry full version information and a manifest; releases ship `SHA256SUMS.txt` and a build-provenance attestation; see `docs/TRUST.md`.
- **Website rebuilt from scratch** with a glass and dithered-data look.

### Español

- **Un nuevo logo**, un vórtice luminoso de tres brazos sobre un cuadrado de cristal, generado por código.
- **Fondos de pantalla.** 24 fondos procedurales a la resolución exacta de tu pantalla, con modo aleatorio. No se descarga nada.
- **Página Personalizar** en Ajustes: estilo de animación, animación de inicio, animación de carga y apariencia de la barra de menús (opacidad, fecha y reloj de 24 h / 12 h).
- **Movimiento más suave.** El dock sube desde el borde al iniciar y se hunde al cerrar.
- **Ajustes se adapta a tu pantalla** y declara DPI por monitor.
- **La barra de Windows sigue oculta**, con una segunda capa y una comprobación cada cinco segundos.
- **Confianza.** Información de versión en los ejecutables, sumas SHA-256 y atestación de procedencia.


## [0.3.0]

### English

- **Renamed to Whirlpool**, with a new ocean identity: Abyss (dark) and Foam (light) palettes,
  a tide-teal accent, a vortex mark and a regenerated icon. Dock glass, labels, menus, the
  menu bar, group glow and the active-app indicator are tinted to match.
- **Rebuilt settings window.** Embedded Inter typeface with a real semibold, themed sliders,
  combo boxes and buttons, a sidebar with a leading tide mark, and a drawn vortex logo.
- **Settings carry over.** `%APPDATA%\Muelle` (and the older `WinDock`) is moved to
  `%APPDATA%\Whirlpool` on first run.
- New website (`site/`) deployed to GitHub Pages, architecture and contributing docs.
- Release pipeline publishes the changelog section as the release notes.
- Dock behaviour and the shell and core crates are otherwise unchanged from Muelle Native 0.2.0.

### Español

- **Ahora se llama Whirlpool**, con identidad de océano: paletas Abyss (oscura) y Foam (clara),
  acento verde marino, nueva marca en espiral e icono regenerado.
- **Ajustes rediseñados**: tipografía Inter incrustada, controles con tema propio y barra lateral nueva.
- **Tus ajustes se conservan**: la carpeta `Muelle` se mueve a `Whirlpool` en el primer inicio.

*Entries below predate the rename; the product name in them was changed mechanically from "Muelle" to "Whirlpool".*

## [0.2.0]

### English

- **Fully Rust-native.** The Tauri + React settings app is gone. Settings is now a
  native Rust (egui) program: no web view, no Node, no installer runtime. Both programs
  are plain `.exe` files.
- New settings look: sidebar navigation, light/dark following Windows, live dock and
  island previews drawn over your own wallpaper, animated switches, debounced saving.
- Update checks use Windows' own HTTP client and open the release page; the executables
  are replaced by `scripts/install.ps1` or a manual download.
- Release pipeline builds both programs on GitHub Actions and attaches `whirlpool.exe`,
  `whirlpool-settings.exe`, a zip and SHA-256 sums.
- Dock, shell and core crates are unchanged from upstream 0.1.18.

### Español

- **100 % nativo en Rust.** Se eliminó la app de ajustes con Tauri y React: ahora es un
  programa nativo en Rust (egui), sin web view ni Node. Ambos programas son `.exe` sueltos.


## [0.1.18]

### English

- A folder or window picker left open no longer re-sends its unchanged visuals
  to the compositor every 50–60 ms: once open, it redraws only when the item
  under the pointer changes, so Windows does no composition work while it rests.
- Fixed island popup edge cases: AI cards stay within the display at high DPI,
  with clipped scrolling and reachable buttons. Volume drags cancel safely when
  the layout changes; audio-only updates reuse current system/provider data.
  Popups without room for their trigger close, repeated refresh clicks are
  disabled while busy, and failed media reads no longer retain the previous song.
  Scrolling reuses GPU visuals without extra idle polling or quota requests.
- Music now shows circular Windows media artwork in the island and its popup,
  with a bundled offline fallback. The popup adds system volume and mute controls;
  the thumb and percentage respond immediately, with coalesced audio writes and
  retained visuals. Audio events avoid unrelated system queries and full redraws
  for percentage-only changes. Release/cancellation does not steal app focus.
- Dock and island-style previews use the current monitor's wallpaper when available.
  Notch/bubble illustrations have clearer content and unobstructed selection marks. Nested
  options and sliders have fewer icons, and the GitHub star is filled yellow.
- Settings now illustrate Dock placement, island styles and Genie speeds, with
  native keyboard-selectable options, clearer icons and shorter descriptions.
  A permanent sidebar link opens Whirlpool on GitHub to explore and star the project.
- Settings → Dock → Position on screen: Left, Bottom or Right, as on macOS.
  Bottom stays the default. On a side the dock runs top to bottom below the
  menu bar, icons grow and show their names inward, menus, groups and window
  previews open beside the icon, the genie flies sideways into the dock, a
  hidden dock comes out of its side edge (on any display), and maximized
  windows end beside it. No top position: that edge belongs to the menu bar.
- Separated manual SignPath test builds from public releases and updater output.
  Signature verification now records Windows' detailed result and the file hash,
  checks the expected signer, and keeps strict integrity checks in both modes.
  Test-certificate trust is temporary and restricted to GitHub-hosted runners.
- Settings → Menu bar → Show what's playing (issue #28). The music half of
  the island could not be turned off; now it can, and with AI usage off too
  the menu bar shows no island at all. Off, Whirlpool also stops following media
  sessions. On by default, as before.
- Added "End task" beside "Quit" in a running app's context menu, including apps
  inside dock groups (issue #26). Force termination runs outside the UI thread
  against process handles captured for that menu. Explorer, critical/shared hosts
  and inaccessible processes are excluded; cancelling the menu changes nothing.
- Hardened "End task" against windows closed or added while its menu is open,
  including independent windows outside the dock's current desktop. All selected
  processes are checked before termination; an outdated selection is refused
  instead of ending a background process or partially applying the request.
- Settings → Windows → Genie speed (issue #27): Normal, Fast or Faster.
  Normal is the speed the genie always had (0.5 s, close to the macOS Dock)
  and stays the default; Fast takes 0.35 s and Faster 0.25 s, minimizing and
  restoring alike. The change applies to the next window at once.
- Added a persistent red attention badge for apps that request attention through
  Windows, including grouped apps and minimized-window thumbnails (issue #22,
  first stage). It clears on window activation/closure and follows Windows' badge
  preference live. Updates reuse visuals without rebuilding the dock, additional
  polling, notification-content access, or invented unread-message counts.
- Fixed (issue #25): an uninstalled app could stay in Recent Apps with no way
  to remove it. Recent apps that no longer exist now leave the dock on their
  own, as pinned ones already did, and clicking one that was just uninstalled
  removes it instead of failing. A recent app's menu also has "Remove from
  Dock", to dismiss it while recent apps stay on; it comes back the next time
  you open and close it.
- Fixed: a Logitech mouse on a receiver (reported with a G305 on its LIGHTSPEED
  receiver) did not show its battery in the menu bar, or showed it only for a
  moment. The mouse sleeps a few seconds after it stops moving and then does not
  answer, and every check dropped it. A device behind a receiver now keeps its
  last reading for 30 minutes while it sleeps, as Logitech's own tools do, and
  each request waits longer for a reply over the radio. Unplugging the
  receiver still removes it at once.
- OpenCode shows only measured Go quotas and their reset times, including the
  monthly window. Zen's local spending estimate and balance link are removed,
  together with the history queries and bundled SQLite dependency. A Zen-only
  connection shows no OpenCode indicator; missing quota data never becomes 0%.
- Fixed OpenCode Go account discovery: read its `opencode-go` credential, respect
  `XDG_DATA_HOME`, and fall back from an empty environment key to the saved login.
  Only saved Go keys or an explicit environment key are used for quota requests;
  saved Zen keys are not queried.
- Fixed a blocking "Ordinal 381 Not Found" system dialog when launching applications
  with Uninstall Tool (`utshellext.dll`) or other third-party shell extensions installed
  (issue #23). `whirlpool.exe` now embeds an application manifest declaring a dependency on
  `Microsoft.Windows.Common-Controls` 6.0.0.0 (along with PerMonitorV2 DPI awareness and
  Windows 10/11 compatibility), enabling shell extensions to resolve modern ComCtl32 exports
  such as `LoadIconWithScaleDown`.
- Process-level error mode now suppresses critical-error and file-not-found system message
  boxes (`SEM_FAILCRITICALERRORS | SEM_NOOPENFILEERRORBOX`) across the dock, the one-shot
  shell worker, and application launch, and `ShellExecuteExW` requests `SEE_MASK_FLAG_NO_UI`.
  Faulty or incompatible shell extensions no longer block or delay application opening;
  mandatory UAC elevation prompts remain visible.

### Español

- Una carpeta o un selector de ventanas abierto ya no reenvía al compositor sus
  visuales sin cambios cada 50–60 ms: una vez abierto, solo se redibuja cuando
  cambia el elemento bajo el puntero, así que Windows no compone nada mientras reposa.
- Corregidos casos límite de los popups: las tarjetas de IA caben en la pantalla
  con escala alta, con desplazamiento y botones limitados al área visible. El
  arrastre de volumen se cancela si cambia su posición; los cambios de audio
  reutilizan los datos del sistema y proveedores. Los popups sin espacio para
  su acceso se cierran, actualizar se desactiva mientras está ocupado y los
  errores de lectura multimedia no conservan la pista anterior. Desplazar la
  lista reutiliza visuales GPU, sin sondeo en reposo ni consultas de cuota extra.
- La isla y el popup de música muestran la carátula circular de Windows, con
  una imagen incluida como respaldo. El popup incorpora volumen del sistema y
  silencio: tirador y porcentaje responden al instante, reutilizando sus visuales
  y agrupando las escrituras de audio. Los cambios de porcentaje evitan consultas
  ajenas al audio y redibujar la barra. El arrastre no roba el foco.
- Las previews del Dock y de estilos de isla usan el fondo del monitor actual.
  Muesca y burbuja tienen contenido más claro y la marca de selección queda fuera
  de la imagen. Si el fondo no está disponible, mantienen el fondo ilustrado.
  Se quitan iconos de subopciones y sliders; la estrella de GitHub es amarilla.
- Ajustes incorpora vistas de la posición del Dock, los estilos de isla y la
  velocidad del genio, con selección por teclado, iconos y textos más breves.
  Un enlace fijo al pie del sidebar abre Whirlpool en GitHub para conocer y apoyar
  el proyecto con una estrella.
- Ajustes → Dock → Posición en la pantalla: Izquierda, Abajo o Derecha, como
  en macOS. Abajo sigue siendo la predeterminada. En un lateral el dock va de
  arriba abajo bajo la barra superior, los iconos crecen y muestran su nombre
  hacia dentro, los menús, grupos y vistas previas se abren junto al icono, el
  genio vuela de lado hacia el dock, el dock oculto sale por su borde lateral
  (en cualquier pantalla) y las ventanas maximizadas terminan a su lado. No hay
  posición arriba: ese borde es de la barra superior.
- Separadas las pruebas manuales de SignPath de las releases y el actualizador.
  La verificación registra el detalle de Windows y el hash, comprueba el firmante
  previsto y exige firmas válidas en ambos modos. La confianza del certificado
  de prueba es temporal y se limita a runners alojados de GitHub.
- Ajustes → Barra superior → Mostrar lo que se reproduce (issue #28). La parte
  de música de la isla no se podía apagar; ahora sí, y si también desactivas el
  uso de IA la barra superior no muestra ninguna isla. Apagado, Whirlpool deja
  además de seguir las sesiones multimedia. Activado por defecto, como antes.
- Añadido «Finalizar tarea» junto a «Salir» en el menú de una app abierta, también
  dentro de grupos del dock (issue #26). La finalización forzada se ejecuta fuera
  del hilo de interfaz y usa los procesos capturados para ese menú. Se excluyen
  Explorer, procesos críticos/hosts compartidos y procesos sin permisos; cancelar
  el menú no realiza ninguna acción.
- Reforzado «Finalizar tarea» si se cierran o aparecen ventanas mientras el menú
  sigue abierto, también fuera del escritorio que muestra el dock. Se comprueban
  todos los procesos antes de finalizar: una selección antigua se rechaza en vez
  de terminar un proceso en segundo plano o aplicar parcialmente la petición.
- Ajustes → Ventanas → Velocidad del genio (issue #27): Normal, Rápida o Muy
  rápida. Normal es la velocidad de siempre (0,5 s, cercana al Dock de macOS)
  y sigue siendo la predeterminada; Rápida dura 0,35 s y Muy rápida 0,25 s,
  tanto al minimizar como al restaurar. Se aplica al instante a la siguiente
  ventana.
- Añadido un badge rojo persistente para las apps que solicitan atención mediante
  Windows, también en grupos y miniaturas minimizadas (issue #22, primera etapa).
  Se quita al activar/cerrar la ventana y sigue en caliente la preferencia de
  badges de Windows. Se reutilizan los visuales sin reconstruir el dock, sin
  sondeos adicionales, sin leer mensajes y sin inventar contadores de no leídos.
- Corregido (issue #25): una app desinstalada podía quedarse en Apps recientes
  sin forma de quitarla. Las apps recientes que ya no existen salen solas del
  dock, como ya pasaba con las fijadas, y al hacer clic en una recién
  desinstalada se quita en vez de fallar. El menú de una app reciente tiene
  además «Quitar del Dock» para descartarla sin desactivar las recientes;
  vuelve la próxima vez que la abras y la cierres.
- Corregido: un ratón Logitech con receptor (reportado con un G305 y su receptor
  LIGHTSPEED) no mostraba su batería en la barra superior, o solo un momento. El
  ratón se duerme a los pocos segundos de no moverlo y entonces no responde, y
  cada comprobación lo quitaba. Un dispositivo tras receptor conserva ahora su
  última lectura 30 minutos mientras duerme, como las herramientas de Logitech, y
  cada petición espera más la respuesta por radio. Quitar el
  receptor lo sigue borrando al instante.
- Corregido un diálogo de error bloqueante ("No se encuentra el ordinal 381") al abrir
  aplicaciones con Uninstall Tool (`utshellext.dll`) u otras extensiones del shell de
  terceros instaladas (issue #23). `whirlpool.exe` ahora incrusta un manifiesto de aplicación
  que declara la dependencia de `Microsoft.Windows.Common-Controls` 6.0.0.0 (junto con
  reconocimiento DPI PerMonitorV2 y compatibilidad con Windows 10/11), permitiendo a las
  extensiones resolver funciones modernas de ComCtl32 como `LoadIconWithScaleDown`.
- El modo de error del proceso ahora suprime los cuadros de diálogo de errores críticos
  del sistema (`SEM_FAILCRITICALERRORS | SEM_NOOPENFILEERRORBOX`) en el dock, el proceso
  auxiliar del shell y el lanzador, y `ShellExecuteExW` usa `SEE_MASK_FLAG_NO_UI`. Las
  extensiones del shell defectuosas o incompatibles ya no bloquean ni retrasan la apertura
  de aplicaciones; las solicitudes de elevación UAC se mantienen visibles.
- WinGet manifests now use `/S` for both unattended modes and declare the
  Visual C++ x64 and WebView2 runtime dependencies. This addresses first-install
  UI and missing-runtime risks; Microsoft installation revalidation is pending.

### Español

- Corregida la detección de cuentas OpenCode Go: se lee la credencial `opencode-go`,
  se respeta `XDG_DATA_HOME` y una variable de clave vacía permite usar la sesión
  guardada. Las consultas usan únicamente la clave guardada de Go o una clave
  explícita de entorno; no se consulta con claves guardadas de Zen.
- OpenCode muestra solo cuotas medidas de Go y sus reinicios, incluido el mes.
  Se retiran el gasto local de Zen y el enlace al saldo, las consultas al historial
  y la dependencia SQLite. Una conexión solo Zen no muestra el indicador de
  OpenCode; la ausencia de cuotas nunca se convierte en 0 %.
- Los manifiestos de WinGet usan `/S` en ambos modos desatendidos y declaran
  las dependencias de Visual C++ x64 y WebView2. Se corrigen riesgos de interfaz
  en la primera instalación y de runtimes ausentes; falta revalidar con Microsoft.

## [0.1.17]

### English

- Added Google Search Console ownership verification to the website.
- Automated WinGet manifest generation and submission for the first package and
  future stable releases, with manual retries, native validation, final installer
  checksum checks, duplicate detection and explicit missing-token failures.
- Added bounded graphics-device recovery and recovery of an existing dock when
  Whirlpool is opened again. The watchdog closes the old instance before restarting;
  a blocked recovery close has an eight-second deadline. Repeated device loss
  within the first minute after recovery stops automatic restarts and restores
  the Windows taskbar. Graphics health reuses the existing watchdog heartbeat.
- An error updating the top bar no longer prevents the dock from refreshing or
  returning after full-screen mode.
- Added read-only USB adapters for SteelSeries Nova 5/5X and more Nova 7/7P
  variants, Logitech G PRO X 2 (Centurion), selected G PRO/G533 HID++ variants,
  HyperX Cloud Alpha Wireless and Kingston Cloud II Wireless. Voltage-only and
  discrete readings remain unknown; HP/DTS Cloud II revisions with a reported
  connection-query side effect are excluded. Physical validation is pending.
- Added Logitech Lightspeed and selected Nano receiver IDs, including the G305's
  C53F, and HID++ Battery Status 1000 reads. Short and long report collections are
  now queried together once per receiver. The 60-second interval and opt-in
  setting remain; physical validation of the expanded coverage is pending.
- Fixed three Clippy violations in the accessory diagnostic's serialization tests.
  Tag releases now run the full CI checks before building and signing the installer.

### Español

- Añadida la verificación de propiedad de Google Search Console en la web.
- Automatizada la generación y el envío de manifiestos de WinGet para la primera
  publicación y las próximas releases estables, con reintentos manuales, validación
  nativa, comprobación del hash final, detección de duplicados y error si falta el token.
- Añadida recuperación limitada ante pérdida del dispositivo gráfico y al abrir
  Whirlpool de nuevo cuando ya existe una instancia. El guardián cierra la anterior
  antes de reiniciarla; si el cierre de recuperación se bloquea, tiene un plazo de
  ocho segundos. Un nuevo fallo gráfico durante el primer minuto tras recuperar
  detiene los reinicios automáticos y devuelve la barra de Windows. La revisión
  gráfica aprovecha la comprobación de salud que ya hacía el guardián.
- Un error al actualizar la barra superior ya no impide actualizar el dock ni
  hacerlo reaparecer al salir de pantalla completa.
- Añadidos adaptadores USB de lectura para SteelSeries Nova 5/5X y más variantes
  Nova 7/7P, Logitech G PRO X 2 (Centurion), variantes G PRO/G533 HID++, HyperX
  Cloud Alpha Wireless y Cloud II Wireless Kingston. Voltajes y niveles discretos
  siguen sin porcentaje; se excluyen revisiones HP/DTS de Cloud II cuya consulta
  de conexión tiene un efecto secundario documentado. Falta validación física.
- Añadidos receptores Logitech Lightspeed y Nano identificados, incluido C53F del
  G305, y lectura de batería HID++ 1000. Las colecciones de informes cortos y largos
  se consultan juntas una sola vez por receptor. Se mantienen los 60 segundos y
  la activación opcional; la ampliación queda pendiente de validación física.
- Corregidos tres errores de Clippy en las pruebas de serialización del diagnóstico
  de accesorios. Las releases por etiqueta ejecutan ahora todos los controles de CI
  antes de compilar y firmar el instalador.

## [0.1.16]

### English

- Initial automatic USB battery adapters for allowlisted Logitech Unifying/Bolt
  receivers and SteelSeries Nova 7 variants. Queries run in disposable processes,
  pause with the hidden menu bar and use a 60-second interval with error backoff.
  Approximate levels stay unknown. Physical validation is still pending; radio
  link changes may take until the next query to appear.
- Optional mouse/headset battery indicators in Settings → Menu bar, off by default.
  Enabling them detects connected accessories automatically; missing battery data
  displays a dash. Disabling them stops the isolated reader and all accessory
  queries. The landing's Settings demo includes the switch and inline indicators.
  Hardware compatibility still needs validation.
- The menu bar now follows Windows' Battery percentage setting, showing the
  charge next to the battery icon when enabled. Preference changes apply
  without restarting Whirlpool, through registry notifications rather than polling.
- Fixed icons remaining over the desktop after dragging an app out of a dock
  group. Drag tracking now recovers missed pointer messages without taking
  focus, stops on release or Escape, and rejects late clicks after cancellation.
- Grouping now requires dropping over the target icon vertically as well as
  horizontally. Recovered or collapsed groups no longer duplicate pinned apps
  in recent apps.
- Fixed (issue #15): the Recycle Bin could keep its full icon after being
  emptied, even after restarting Whirlpool. Right after a change, Explorer may
  still draw the previous state, and that icon was cached for the day under
  the other state. The empty and full icons now come from where Windows names
  them (`DefaultIcon`, including an icon you chose in Desktop icon settings),
  and wrongly cached ones are replaced on their own.

### Español

- Adaptadores USB iniciales para receptores Logitech Unifying/Bolt y variantes
  SteelSeries Nova 7 identificadas. Consultas en procesos desechables, pausadas
  con la barra oculta, con intervalo de 60 segundos y espera tras errores.
  Los niveles aproximados siguen sin porcentaje. Falta validación física; los
  cambios del enlace radio pueden tardar hasta la consulta siguiente.
- Indicadores opcionales de batería del ratón y auriculares en Ajustes → Barra
  superior, desactivados por defecto. Al activarlos se detectan los accesorios
  conectados automáticamente; un guion indica batería no disponible. Desactivarlos
  termina el lector aislado y todas las consultas de accesorios. La demo de Ajustes
  de la landing incluye el interruptor y los indicadores en la barra.
  La compatibilidad con hardware real queda pendiente de validar.
- La barra superior respeta la opción Porcentaje de batería de Windows y
  muestra la carga junto al icono cuando está activada. Los cambios se aplican
  sin reiniciar Whirlpool, mediante notificaciones del registro, sin sondeo.
- Corregido el icono que quedaba flotando sobre el escritorio al sacar una app
  de un grupo del dock. El arrastre recupera mensajes del puntero perdidos sin
  tomar el foco, termina al soltar o pulsar Escape y descarta clics tardíos tras
  una cancelación.
- Agrupar requiere soltar sobre el icono también en vertical. Recuperar o
  reducir un grupo ya no duplica apps fijadas en las apps recientes.
- Corregido (issue #15): la Papelera podía seguir con el icono de llena tras
  vaciarla, incluso al reiniciar Whirlpool. Justo después de un cambio, Explorer
  puede dibujar aún el estado anterior, y ese icono se guardaba en caché todo
  el día bajo el otro estado. Los iconos de vacía y llena se leen ahora de
  donde Windows los define (`DefaultIcon`, incluido un icono elegido en la
  configuración de iconos del escritorio), y los guardados mal se reemplazan solos.

## [0.1.15]

### English

- Dragging apps out of the dock or a group now completes or cancels reliably
  even when a non-activating window misses mouse input outside its bounds.
  Press Escape to cancel a drag.
- Dock groups: drag a pinned app onto the middle of another one (a blue frame
  shows it) to put both in a group that takes a single place in the dock;
  drop more apps on the group to add them. A group shows its first apps'
  icons on a framed tile, and lists their names on hover. Clicking it fans
  its apps out above it (a grid when there are many); drag one out of the
  fan to take it out of the group, into the dock where it is dropped.
  Right-click a group to ungroup it.
- Shortcuts folder: choose a folder in Settings → Dock and it shows at the
  right of the dock. Clicking it opens its contents as a menu, as a Windows 10
  taskbar toolbar did: folders first, each one a submenu, then files, with
  Explorer's names and icons; choosing an item opens it. Right-click it to
  open the folder, change it or remove it.
- French: the dock, Settings and the installer are now also in French. Choose
  it in Settings → System, or leave Automatic to follow Windows. A language
  Whirlpool does not know in the settings file now reads as Automatic instead of
  making the file invalid.
- Mascot showcase: the website now previews the crab's reactions to AI limits,
  charging and energy saver, and the hero crab moves between the Notes window
  and dock. Choose a moment to play its scene or let the preview run; reduced
  motion is respected.
- Apps that showed a blank page instead of their icon now show it, as on the
  taskbar: LibreOffice (its windows belong to `soffice.bin`; the dock now
  shows LibreOffice Calc, Writer… with their names and icons), and programs
  whose icon the Windows shell cannot draw although it is valid (one made
  only of PNG images, as some Electron tools write). Icons are read again
  once after updating.

### Español

- Arrastrar apps fuera del dock o de un grupo ahora termina o se cancela de
  forma fiable aunque una ventana que no toma el foco pierda los eventos del
  mouse fuera de sus límites. Pulsa Esc para cancelar el arrastre.
- Grupos en el dock: arrastra una app fijada al centro de otra (un marco azul
  lo indica) para reunirlas en un grupo que ocupa un solo lugar en el dock;
  suelta más apps sobre el grupo para añadirlas. Un grupo muestra los iconos
  de sus primeras apps en un mosaico enmarcado, y sus nombres al pasar el
  puntero. Al hacer clic se despliegan en abanico encima (en cuadrícula si son
  muchas); arrastra una fuera del abanico para sacarla del grupo, al lugar del
  dock donde la sueltes. Clic derecho en un grupo para desagruparlo.
- Carpeta de accesos directos: elige una carpeta en Ajustes → Dock y aparece
  a la derecha del dock. Al hacer clic, su contenido se abre como un menú,
  como una barra de herramientas de la barra de tareas de Windows 10: primero
  las carpetas, cada una un submenú, luego los archivos, con los nombres e
  iconos del Explorador; al elegir un elemento se abre. Clic derecho para
  abrir la carpeta, cambiarla o quitarla.
- Francés: el dock, Ajustes y el instalador también están en francés.
  Elígelo en Ajustes → Sistema, o deja Automático para seguir a Windows. Un
  idioma que Whirlpool no conoce en el archivo de ajustes ahora se lee como
  Automático en lugar de invalidar el archivo.
- Vista previa de la mascota: la web muestra las reacciones del cangrejo ante
  los límites de IA, la carga y el ahorro de energía; además, en la portada se
  mueve entre la ventana de Notas y el dock. Elige un momento para ver su escena
  o deja que la vista previa avance; se respeta la opción de reducir movimiento.
- Las apps que mostraban una página en blanco en lugar de su icono ahora lo
  muestran, como en la barra de tareas: LibreOffice (sus ventanas pertenecen
  a `soffice.bin`; el dock ahora muestra LibreOffice Calc, Writer… con sus
  nombres e iconos) y los programas cuyo icono el shell de Windows no sabe
  dibujar aunque sea válido (uno hecho solo de imágenes PNG, como escriben
  algunas herramientas de Electron). Los iconos se vuelven a leer una vez
  tras actualizar.

## [0.1.14]

### English

- Clicking an app whose minimized window is still reported as foreground now
  restores it on the first click, including with one app open and minimized
  previews disabled (#12). File drag-hover uses the same native-state check.

- Automatic startup retries up to three times after a failed launch. Existing
  enabled tasks receive the recovery policy without enabling disabled tasks or
  changing the installation they launch. Settings checks the real registration.
- If Explorer has not created its taskbar yet, Whirlpool retries taking it over
  without blocking the interface or saving a false original taskbar setting.
- Startup failures before graphics initialization are now recorded in the log.
- Dock window previews keep their positions when switching or minimizing
  windows. Newly discovered windows append; closed windows are removed. Normal
  icon activation continues to use the most recently used window.

### Español

- Pulsar el icono de una app minimizada que Windows aún indica como activa
  ahora la restaura al primer clic, incluso con una sola app abierta y las
  miniaturas de minimizadas desactivadas (#12). El arrastre sobre un icono
  utiliza la misma comprobación del estado real de la ventana.

- El inicio automático reintenta hasta tres veces si falla el arranque. Las
  tareas activadas reciben esta recuperación sin reactivar tareas deshabilitadas
  ni cambiar la instalación que ejecutan. Ajustes consulta el registro real.
- Si Explorer todavía no ha creado su barra, Whirlpool vuelve a intentar ocultarla
  sin bloquear la interfaz ni guardar un estado original incorrecto.
- El registro también recoge fallos anteriores a la inicialización gráfica.

- Las previsualizaciones del dock conservan su posición al cambiar de ventana
  o minimizarla. Las ventanas nuevas se añaden al final y las cerradas se retiran.
  El clic normal del icono sigue recuperando la ventana usada más recientemente.

## [0.1.13]

### English

- The menu bar's keyboard indicator opens a themed layout menu with the
  current keyboard marked and a link to Windows language settings. Changes
  are requested for the original typing window instead of simulating Win+Space.
- Changing Whirlpool's language refreshes menu bar text even when its data has
  not changed. Menu bar controls retain the standard arrow cursor.
- Menus return focus after a selection without overriding another app the
  user switched to while dismissing them. Auto-hide keeps the menu bar open
  while its keyboard selector is in use.

- With auto-hide enabled, rest the pointer at a display's bottom edge to
  bring the Dock to that display (#9, #11).
- Menu bar redraws no longer cancel a pending top-edge reveal (#10).
- Screen-edge detection reads the cursor on a dedicated timer thread while
  auto-hide is active, including when the cursor is already at the edge.
- Auto-hidden menu bars verify the cursor position before closing on
  mouse-leave notifications, with a pointer check only while the bar is revealed.
- The menu bar reveals after a shorter edge dwell and slides/fades in and
  out using the Windows compositor. Returning during its exit reverses the
  transition; Windows' disabled animation setting is respected.
- Show/hide transitions wait for compositor completion and keep hidden content
  transparent, preventing cached-frame flashes. Auto-hide also applies before
  the menu bar's first frame at startup.
- Closing a dock context menu resets hover and resumes auto-hide when the
  pointer is outside the dock. File drags keep the dock open.
- Dock and Settings writes are serialized. Changing a preference preserves
  recent pin changes and the dock's latest auto-hide setting. Closing
  Settings waits for pending edits to be saved.
- Taking over the Windows taskbar after resume or an Explorer restart runs
  in the background, as it does at startup.

### Español

- El indicador de teclado de la barra abre un menú con el estilo de Whirlpool,
  marca el teclado activo y permite abrir los ajustes de idioma de Windows.
  El cambio se solicita a la ventana donde se escribía, sin simular Win+Espacio.
- Cambiar el idioma de Whirlpool actualiza los textos de la barra aunque no
  cambien sus datos. Los controles de la barra mantienen el cursor de flecha.
- Los menús devuelven el foco tras una selección sin imponerse a otra app
  elegida al cerrarlos. La barra permanece visible mientras se usa su
  selector de teclado, incluso con ocultación automática.

- Con la ocultación automática activada, deja el puntero en el borde inferior
  de una pantalla para mostrar el Dock allí (#9, #11).
- Actualizar la barra superior ya no cancela su aparición al dejar el puntero
  en el borde (#10).
- La detección del borde consulta el puntero desde un hilo con temporizador
  mientras está activa la ocultación automática, incluso si ya está en el borde.
- Los avisos de salida de la barra superior verifican la posición real del
  puntero antes de cerrarla, con una comprobación solo mientras está desplegada.
- La barra aparece con una espera menor en el borde y transiciones de
  deslizamiento y opacidad mediante el compositor de Windows. Volver durante
  la salida la invierte; respeta la desactivación de animaciones de Windows.
- La salida espera a que termine realmente la animación del compositor y el
  contenido permanece transparente al ocultarse, evitando destellos de fotogramas
  anteriores. La auto-ocultación se aplica desde el primer fotograma al iniciar.
- Al cerrar un menú contextual fuera del dock se restablece el puntero y
  vuelve a funcionar la ocultación automática. Al arrastrar archivos, el
  dock permanece abierto.
- El dock y Ajustes guardan por turnos. Cambiar una preferencia conserva
  los pines y el estado de ocultación más recientes. Ajustes guarda los
  cambios pendientes antes de cerrarse.
- Recuperar la barra de tareas tras reanudar Windows o reiniciar el
  Explorador se ejecuta en segundo plano, igual que en el inicio.

## [0.1.12]

### English

- Unplugging the charger now plays a short mascot reaction above the dock,
  including when unplugging also enables energy saver. It replaces an ongoing
  charging reaction and temporarily reveals an auto-hidden dock.
- The first interactive installation opens Whirlpool Settings when it completes.

### Español

- Al desconectar el cargador, la mascota también aparece sobre el dock,
  incluso si se activa el ahorro de energía a la vez. Sustituye la reacción
  de carga si todavía está en curso y muestra temporalmente el dock oculto.
- La primera instalación interactiva abre Ajustes de Whirlpool al terminar.

## [0.1.11]

### English

- Removed the manual option to import apps pinned to the Windows taskbar from
  the dock's context menu. Changes to Windows taskbar pins still sync
  automatically.
- Clicking an app while a shell flyout has focus accepts native pointer
  activation before opening it. Minimized windows are restored even if the
  subsequent foreground request is refused. Hidden stale windows are skipped.
- Dock button clicks resolve the current pointer position instead of relying
  on a potentially coalesced mouse-move event.
- Minimization flights keep their capture when Windows focuses the next app.
  Deferred restores still cancel when the user switches to a different app.
- The mascot respects the Windows animation preference and stops when
  disabled, entering full screen or changing display geometry. Initial
  power notifications establish a baseline. Power acts start after the
  dock refresh, and hover reactions preserve the act's exit animation.
  Failed creation no longer reveals an auto-hidden dock or consumes the
  charging cooldown. AI quota cues ignore failed or older measurements
  and comparisons between different quota windows.
- Energy-saver notifications now update the dock and menu bar directly from
  the state sent by Windows. Standard and high-savings modes both disable
  glass, and turning the saver off restores the transparency preference.
- Whirlpool uses an interactive logon task to avoid Windows' discretionary delay
  of `Run` entries. Windows still controls when the process can start.
  Registration uses the Task Scheduler API with normal priority, battery
  support and a `Run` fallback. Enabled registrations migrate only for the
  matching installation. Migration and Settings changes are serialized,
  and task-removal failures are reported.
- Logon tasks use the account's SID, which survives account renames.
  Changes to autostart run off the Settings UI thread.
- Fixed: after turning the PC on, the dock and the menu bar could stay hidden
  and the Windows taskbar showed instead. Right after signing in, the window
  in front is often a hidden shell window the size of the screen (Start,
  Search or the text input host), and the dock took it for a full-screen app.
  Hidden and cloaked windows no longer count as full screen.
- Diagnostics are written off the UI thread through a bounded queue.
  `%LOCALAPPDATA%\Whirlpool\whirlpool.log` and its rotated backup each hold up to
  256 KB. The log records errors, full-screen transitions and initialization
  time measured from entering the dock initialization routine.

### Español

- Se eliminó del menú contextual del dock la opción manual para añadir las
  apps fijadas en la barra de tareas de Windows. Los cambios en esos pines
  siguen sincronizándose automáticamente.
- Al pulsar una app mientras un panel del sistema conserva el foco, el dock
  acepta la activación nativa del clic antes de abrirla. Las ventanas
  minimizadas se restauran aunque se rechace la petición posterior de foco.
  Las ventanas ocultas que hayan quedado en la lista se descartan.
- Los clics del dock consultan la posición actual del puntero, evitando usar
  el icono de un movimiento anterior cuando Windows agrupa eventos del ratón.
- El efecto de minimización conserva la captura cuando Windows enfoca la
  siguiente app. Las restauraciones aplazadas se cancelan si el usuario
  cambia a otra aplicación.
- La mascota respeta la opción de animaciones de Windows y se cancela al
  desactivarla, entrar en pantalla completa o cambiar la geometría de pantalla.
  Los avisos iniciales de energía establecen el estado de referencia.
  Sus escenas comienzan después de actualizar el dock y las reacciones al
  puntero conservan la animación de salida. Un fallo al crearla ya no revela
  el dock oculto ni consume la espera entre avisos del cargador. Los avisos
  de IA descartan lecturas fallidas, antiguas o de ventanas de cuota distintas.
- Las notificaciones de ahorro de energía actualizan el dock y la barra
  superior con el estado enviado por Windows. Tanto el modo estándar como
  el ahorro máximo desactivan el cristal. Al desactivarlos, se recupera la
  preferencia de transparencia del sistema.
- Whirlpool usa una tarea de inicio de sesión interactiva para evitar el retraso
  que Windows puede aplicar a las entradas `Run`. Windows sigue decidiendo
  cuándo puede arrancar el proceso. La tarea se registra con la API del
  Programador, prioridad normal, soporte para batería y respaldo mediante
  `Run`. Se migran los registros activos de la misma instalación. Los cambios
  desde Ajustes y la migración se ejecutan de forma serializada y los errores
  al eliminar tareas se comunican.
- Las tareas usan el SID de la cuenta, que se conserva al cambiar su nombre.
  Cambiar el inicio automático se ejecuta fuera del hilo de Ajustes.
- Corregido: al encender el PC, el dock y la barra superior podían quedarse
  ocultos y se veía la barra de tareas de Windows. Justo al iniciar sesión, la
  ventana al frente suele ser una ventana oculta del shell del tamaño de la
  pantalla (Inicio, Búsqueda o el host de entrada de texto), y el dock la
  tomaba por una app a pantalla completa. Las ventanas ocultas ya no cuentan.
- El registro de diagnóstico se escribe fuera del hilo de la interfaz mediante
  una cola acotada. `%LOCALAPPDATA%\Whirlpool\whirlpool.log` y su copia anterior
  ocupan hasta 256 KB cada uno. Guarda errores, cambios de pantalla completa
  y el tiempo medido desde que comienza la inicialización del dock.

## [0.1.10]

### English

- Right-clicking an empty area of the dock now opens Task Manager or Taskbar
  settings.
- Apps with multiple windows have a dedicated preview picker with more
  reliable hover and foreground handling.
- The Genie effect follows Windows' "Animation effects" setting, which
  "Adjust for best performance" also turns off, and did so silently: the
  switch in Settings stayed on and nothing happened. Settings now says when
  Windows animations are off and opens the right Windows page to turn them on.
- A pixel crab comes out for moments worth it, and only then: it hops with a
  heart under an AI provider whose limit just reset, trembles with a drop
  past 90 %, falls asleep at 100 %, scurries along the dock when the charger
  is plugged in, and dozes off on it when energy saver starts. Hover over it
  and it reacts. An auto-hidden dock comes up for the moment, as the macOS
  Dock does. While no crab is out it costs nothing; its acts run in the system
  compositor. It can be turned off in Settings → Dock.
- Fixed (issue #8): apps pinned from a Windows shortcut with arguments, such
  as `wscript.exe "script.vbs"`, a browser profile or a web app, opened the
  program alone. They now open through their shortcut, exactly as from
  Explorer or the taskbar, and show the shortcut's name and icon. A shortcut
  dropped on the dock is kept in `%APPDATA%\Whirlpool\Pins`, so the pin survives
  deleting the original; shortcuts to scripts or documents can be pinned too.
- Dragging a file over the dock reads a dropped shortcut once, not on every
  mouse move.

### Español

- Al hacer clic derecho en un espacio vacío del dock, ahora puedes abrir el
  Administrador de tareas o la configuración de la barra de tareas.
- Las apps con varias ventanas tienen un selector de vistas previas propio,
  con un manejo más fiable del puntero y de la ventana en primer plano.
- El efecto genio sigue el ajuste «Efectos de animación» de Windows, que también
  apaga «Ajustar para obtener el mejor rendimiento», y lo hacía en silencio: el
  interruptor de Ajustes seguía activo y no pasaba nada. Ahora Ajustes avisa
  cuando las animaciones de Windows están desactivadas y abre la página de
  Windows correcta para activarlas.
- Un cangrejo pixelado aparece en los momentos que lo merecen, y solo
  entonces: salta con un corazón bajo el proveedor de IA cuyo límite se acaba
  de reiniciar, tiembla con una gota al pasar del 90 %, se duerme al llegar al
  100 %, cruza el dock corriendo al enchufar el cargador y se echa una siesta
  encima al activarse el ahorro de energía. Si pasas el puntero por encima,
  reacciona. El dock oculto sube durante ese momento, como hace el Dock de
  macOS. Mientras no hay cangrejo no consume nada; sus escenas las ejecuta el
  compositor del sistema. Se desactiva en Ajustes → Dock.
- Corregido (issue #8): las apps fijadas desde un acceso directo de Windows con
  argumentos, como `wscript.exe "script.vbs"`, un perfil de navegador o una
  app web, abrían solo el programa. Ahora se abren a través de su acceso
  directo, igual que desde el Explorador o la barra de tareas, y muestran su
  nombre e icono. Un acceso directo soltado en el dock se guarda en
  `%APPDATA%\Whirlpool\Pins`, así que la app fijada sigue funcionando aunque se
  borre el original; también se pueden fijar accesos a scripts o documentos.
- Al arrastrar un archivo sobre el dock, un acceso directo se lee una sola vez,
  no en cada movimiento del ratón.

## [0.1.9]

### English

- Fixed: bringing back a minimized app from the dock could leave its window
  behind the one in front, as if it had closed again (Warp, for one). Since
  0.1.8 the dock restored the window without taking the focus first, and
  Windows then places it behind the active window.
- GitHub Copilot usage was removed from the menu bar island. Its sign-in is
  only stored encrypted, and decrypting another app's secrets is what made
  antivirus programs flag Whirlpool; Whirlpool no longer reads the Windows
  Credential Manager either.
- Fixed: some antivirus programs flagged Whirlpool 0.1.6–0.1.8 as a trojan and
  deleted it, often right after updating. Two things looked like malware: to
  show GitHub Copilot and Cursor usage, Whirlpool decrypted VS Code's stored
  secrets and read Cursor's database (what password stealers do), and since
  0.1.7 it set itself up to start with Windows by running `schtasks.exe`
  hidden. Whirlpool no longer does either: it starts from the usual `Run`
  registry entry (an existing startup task is moved there once), Copilot
  usage comes only from the Copilot CLI sign-in or `GH_TOKEN`/`GITHUB_TOKEN`,
  and Cursor usage was removed. The dock is also 28% smaller.
- Fixed: after choosing an item in the dock's menu (Whirlpool Settings, for
  one), the dock could stay magnified and stop reacting to the pointer.
- Alt+Tab: the hidden dock shows only for Alt+Tab, no longer for Task View,
  whose virtual desktops it covered at the bottom of the screen; and it no
  longer flashes for a moment after cancelling Alt+Tab with Esc.
- Alt+Tab: moving the mouse or clicking an empty spot no longer stops the
  dock's magnifier from following the selection, and closing a window from
  the switcher keeps it on the right app. Only the arrow keys, which move the
  selection in ways Whirlpool cannot see, make it step aside.
- Automatic updates: once a day, a couple of minutes after you turn on the
  computer (or every 24 hours if it stays on), Whirlpool installs new versions by
  itself; the dock restarts in a moment. Never while an app is full screen. It can be turned off in Settings → System → "Install
  updates automatically". The dock itself still never uses the network.
- Windows 10: the menu bar no longer shows a Widgets button, which Windows
  10 does not have; it opened Windows Ink instead.
- Glass follows energy saver: turning it on or off (by hand or at low battery)
  switches the dock, the menu bar and the dock menu between glass and solid
  at once.

### Español

- Corregido: al recuperar una app minimizada desde el dock, su ventana podía
  quedar detrás de la que estaba delante, como si se hubiera cerrado otra vez
  (Warp, por ejemplo). Desde la 0.1.8 el dock restauraba la ventana sin tomar
  antes el foco, y Windows la coloca entonces detrás de la ventana activa.
- Se quitó el uso de GitHub Copilot de la isla de la barra superior. Su sesión
  solo se guarda cifrada, y descifrar los secretos de otra app es lo que hacía
  que los antivirus marcaran Whirlpool; Whirlpool tampoco lee ya el Administrador de
  credenciales de Windows.
- Corregido: algunos antivirus marcaban Whirlpool 0.1.6–0.1.8 como troyano y lo
  borraban, a menudo justo al actualizar. Dos cosas parecían malware: para
  mostrar el uso de GitHub Copilot y Cursor, Whirlpool descifraba los secretos
  guardados de VS Code y leía la base de datos de Cursor (lo que hacen los
  ladrones de contraseñas), y desde la 0.1.7 se configuraba para iniciar con
  Windows ejecutando `schtasks.exe` oculto. Whirlpool ya no hace ninguna de las
  dos: arranca desde la entrada habitual `Run` del registro (una tarea de
  inicio existente se traslada ahí una vez), el uso de Copilot sale solo de la
  sesión de Copilot CLI o de `GH_TOKEN`/`GITHUB_TOKEN`, y se quitó el uso de
  Cursor. Además, el dock ocupa un 28 % menos.
- Corregido: tras elegir una opción del menú del dock (por ejemplo, Ajustes
  de Whirlpool), el dock podía quedarse ampliado y dejar de responder al puntero.
- Alt+Tab: el dock oculto aparece solo con Alt+Tab, ya no con la Vista de
  tareas, cuyos escritorios virtuales tapaba abajo; y ya no parpadea un
  momento después de cancelar Alt+Tab con Esc.
- Alt+Tab: mover el ratón o hacer clic en un hueco ya no impide que la lupa
  del dock siga la selección, y cerrar una ventana desde el selector la
  mantiene en la app correcta. Solo las flechas, que mueven la selección de
  formas que Whirlpool no ve, hacen que se aparte.
- Actualizaciones automáticas: una vez al día, un par de minutos después de
  encender el ordenador (o cada 24 horas si sigue encendido), Whirlpool instala
  las versiones nuevas por sí solo; el dock se reinicia en un momento. Nunca
  con una app a pantalla completa. Se puede desactivar en Ajustes →
  Sistema → «Instalar actualizaciones automáticamente». El dock en sí sigue
  sin usar la red.
- Windows 10: la barra superior ya no muestra un botón de Widgets, que Windows
  10 no tiene; abría Windows Ink en su lugar.
- El cristal sigue al ahorro de energía: al activarlo o desactivarlo (a mano o
  con batería baja), el dock, la barra superior y el menú del dock cambian
  entre cristal y sólido al momento.

## [0.1.8]

### English

- Opening apps and shell file operations run in temporary helper processes so
  slow shell handlers and native progress dialogs keep the dock responsive.
  Transfers run in order; ordinary launch helpers time out after 30 seconds,
  while file operations and administrator launches keep their native user controls.
- Recycle Bin notifications share one drive scan at a time, preserving changes
  that arrive during a scan without starting overlapping disk queries.
- Shell icons and app names load in the background through a bounded queue,
  keeping the dock responsive while the shell is slow. Repeated requests share
  the same load, and unused pixel buffers are released.
- Minimize animations prepare restore images off the UI thread and reuse
  thumbnail dimensions between window events. Batch minimization starts one
  capture; a stalled capture no longer keeps an overlay and frame loop alive.
- Drag and drop into apps: hold a file (or text, or an image) over an open
  app in the dock and its window comes to the front, so you can drop it there,
  as with the Windows taskbar. Passing over icons does not switch windows.
- Fixed: with "Automatically hide and show the menu bar" on, the bar covered the
  minimize, maximize and close buttons of maximized windows and took clicks
  meant for browser tabs at the top edge. The hidden bar no longer keeps any
  window on screen; it appears after the pointer rests briefly on the top
  edge, and never over a window's title-bar buttons.
- The top-right corner of the menu bar no longer shows the desktop: throwing
  the pointer there to close a window minimized everything. "Show Desktop"
  remains in the dock menu.
- Fixed: clicking the menu bar's "^" again left the hidden-icons popup open
  and moved the pointer to its first icon. The second click now closes it.
- The dock's menu looks like a macOS menu: frosted glass in the light or dark
  theme, compact rows and the selection in your Windows accent color. It
  works fully from the keyboard; with a screen reader or high contrast on,
  the native Windows menu is used instead.
- An auto-hidden dock shows while Alt+Tab (or Task View) is open, so the
  open apps are in view, and hides again when you let go. As you press Tab,
  the dock magnifies the app of the selected window. Windows does not report
  the selection, so it follows Tab and Shift+Tab; after arrow keys or the
  mouse in the switcher, the magnifier steps aside rather than guess.
- The menu bar no longer shows a location icon. Widgets keeps location in use
  for the weather all the time, and Windows shows no icon for that either.
- Windows 10: apps pinned to the taskbar from the Microsoft Store (Store, Mail
  and others) now come to the dock. Versions before 0.1.7 could not import
  them, and updating did not add them afterwards.

### Español

- Abrir apps y ejecutar operaciones de archivos del shell se hace en procesos
  auxiliares temporales para mantener la respuesta del dock mientras el shell
  tarda o muestra el progreso. Las transferencias se ejecutan en orden; las
  aperturas normales tienen un límite de 30 segundos, mientras las operaciones
  de archivos y las aperturas como administrador conservan sus controles nativos.
- Los avisos de Papelera comparten una sola lectura de discos a la vez y
  conservan los cambios que lleguen durante esa lectura, sin consultas solapadas.
- Los iconos y nombres del shell se cargan en segundo plano con una cola
  limitada para que el dock responda mientras el shell tarda. Las solicitudes
  repetidas comparten la misma carga y se liberan los píxeles que ya no se usan.
- Las animaciones de minimizar preparan la imagen de restauración fuera del
  hilo de interfaz y reutilizan las dimensiones de las miniaturas entre avisos
  de ventanas. Al minimizar en lote se inicia una captura; una captura atascada
  ya no mantiene una vista previa y un ciclo de fotogramas indefinidos.
- Arrastrar y soltar en las apps: mantén un archivo (o texto, o una imagen)
  sobre una app abierta del dock y su ventana pasa al frente para que lo
  sueltes ahí, como con la barra de tareas de Windows. Pasar por encima de los
  iconos no cambia de ventana.
- Corregido: con «Ocultar y mostrar la barra superior automáticamente», la barra tapaba
  los botones de minimizar, maximizar y cerrar de las ventanas maximizadas y
  se quedaba con los clics en las pestañas del navegador del borde superior.
  La barra oculta ya no deja ninguna ventana en pantalla: aparece cuando el
  puntero se detiene un momento en el borde superior, y nunca encima de los
  botones de una ventana.
- La esquina superior derecha de la barra ya no muestra el escritorio: lanzar
  el puntero ahí para cerrar una ventana minimizaba todo. «Mostrar escritorio»
  sigue en el menú del dock.
- Corregido: volver a pulsar «^» en la barra superior dejaba abierto el
  panel de iconos ocultos y movía el puntero a su primer icono. Ahora el
  segundo clic lo cierra.
- El menú del dock se ve como un menú de macOS: cristal esmerilado en tema
  claro u oscuro, filas compactas y la selección en el color de énfasis de
  Windows. Funciona por completo con el teclado; con un lector de pantalla o
  alto contraste activos se usa el menú nativo de Windows.
- El dock oculto aparece mientras Alt+Tab (o la Vista de tareas) está abierto,
  para ver las apps abiertas, y se vuelve a ocultar al soltar. Al pulsar Tab,
  el dock amplía la app de la ventana seleccionada. Windows no informa de la
  selección, así que sigue Tab y Mayús+Tab; si se usan las flechas o el ratón
  en el selector, la lupa se aparta en vez de adivinar.
- La barra superior ya no muestra el icono de ubicación. Widgets mantiene la
  ubicación en uso todo el tiempo para el clima, y Windows tampoco muestra un
  icono por eso.
- Windows 10: las apps de Microsoft Store fijadas en la barra de tareas
  (Store, Correo y otras) ahora llegan al dock. Las versiones anteriores a la
  0.1.7 no podían importarlas, y al actualizar no se añadían después.

## [0.1.7]

### English

- Faster startup: Whirlpool now uses a Windows sign-in task instead of the Run
  key, and keeps the taskbar hidden when Windows shuts down or restarts.
- Taskbar pins stay in sync with the dock, including changes made while Whirlpool
  is closed. Imported pins keep their order and include Store apps; removed
  apps no longer remain as broken dock icons.
- The dock menu can import apps pinned to the Windows taskbar. Unchecking
  "Keep in Dock" also unpins the app from the taskbar.
- Middle-click or Shift-click opens another app window; Ctrl+Shift-click or
  the dock menu opens an app as administrator. The menu bar's right edge and
  the dock menu can show the desktop.
- The menu bar shows the active input language and privacy indicators for
  microphone, camera, and location use. Its clock follows time and time-zone
  changes, and switching virtual desktops updates the dock immediately.
- Genie: maximized and snapped windows now fly using the area they occupy on
  screen, without jumping to a different size.

### Español

- Inicio más rápido: Whirlpool ahora usa una tarea de Windows al iniciar sesión
  en lugar de la clave Run y mantiene oculta la barra de tareas al apagar o
  reiniciar Windows.
- Las apps fijadas en la barra de tareas se sincronizan con el dock, incluso
  si el cambio ocurre mientras Whirlpool está cerrado. Las apps importadas
  conservan su orden e incluyen las de la Store; las desinstaladas ya no quedan
  como iconos rotos.
- El menú del dock permite importar las apps fijadas en la barra de tareas.
  Desmarcar «Mantener en el Dock» también desancla la app de la barra.
- El clic central o Mayús+clic abre otra ventana; Ctrl+Mayús+clic o el menú del
  dock abre una app como administrador. El borde derecho de la barra superior
  y el menú del dock permiten mostrar el escritorio.
- La barra superior muestra el idioma de entrada activo e indicadores cuando
  se usan el micrófono, la cámara o la ubicación. El reloj sigue los cambios
  de hora y zona horaria, y al cambiar de escritorio virtual el dock se
  actualiza de inmediato.
- Genio: las ventanas maximizadas o acopladas vuelan usando el área que ocupan
  en pantalla, sin saltar a otro tamaño.

## [0.1.6]

### English

- Windows 10 support (version 1903 or later). The dock used to close right
  after starting there. On Windows 10 the dock and menu bar are solid instead
  of glass, since it has no backdrop blur for desktop apps.
- The menu bar's "^" button opens Windows' hidden notification-area icons.
- A curved menu bar island shows usage for Claude, Codex, GitHub Copilot,
  Cursor, OpenCode Go, and Gemini or Antigravity while they are running. It can
  be turned off in Settings → Menu bar.
- Settings has a clearer layout with separate Dock, Windows, Menu bar and
  System sections, and its window can be resized or maximized.
- Minimized windows now fly into their app's dock icon by default. Live
  thumbnails on the right side of the dock remain available as an option.
- The active-app click option was removed: clicking the app in front minimizes
  it, as on the Windows taskbar.
- The dock's Whirlpool icon is sharper, and the Start button uses the Windows
  logo.
- Settings can turn off automatic update checks. The website adds privacy and
  code-signing policy pages.

### Español

- Soporte para Windows 10 (versión 1903 o posterior). Antes el dock se cerraba
  nada más arrancar ahí. En Windows 10 el dock y la barra superior son sólidos
  en lugar de cristal, porque no ofrece desenfoque de fondo a las apps de
  escritorio.
- El botón «^» de la barra superior abre los iconos ocultos del área de
  notificación de Windows.
- Una isla curva en la barra superior muestra el uso de Claude, Codex, GitHub
  Copilot, Cursor, OpenCode Go y Gemini o Antigravity mientras están abiertos.
  Se puede desactivar en Ajustes → Barra superior.
- Ajustes tiene secciones separadas para Dock, Ventanas, Barra superior y
  Sistema; su ventana se puede redimensionar o maximizar.
- Por defecto, las ventanas minimizadas vuelan hacia el icono de su app en el
  dock. Las miniaturas en vivo a la derecha siguen disponibles como opción.
- Se quitó la opción de clic en la app activa: al hacer clic en la app que está
  delante, se minimiza, como en la barra de Windows.
- El icono de Whirlpool se ve más nítido en el dock y el botón Inicio usa el logo
  de Windows.
- Ajustes permite desactivar la búsqueda automática de actualizaciones. La web
  incorpora páginas de privacidad y firma de código.

## [0.1.5]

### English

- The dock appears sooner at startup (about 170 ms instead of 260-450 ms).
- Lower idle CPU: windows changing their title (terminals, browsers) no longer
  make the dock re-read every window.
- Genie starts sooner after minimizing: the screen copy no longer waits extra
  frames.
- Fixed: the dock and menu bar could stay black after energy saver; without
  transparency they are now solid gray, as macOS draws them with "Reduce
  transparency".
- Fixed: the menu bar could miss an update and showed no app name until the
  first window switch.
- Menu bar: the battery shows a charging bolt.
- Settings: options that depend on another fold away under it.

### Español

- El dock aparece antes al arrancar (unos 170 ms en lugar de 260-450 ms).
- Menos CPU en reposo: las ventanas que cambian de título (terminales,
  navegadores) ya no hacen que el dock relea todas las ventanas.
- El genio empieza antes al minimizar: la copia de pantalla ya no espera
  fotogramas de más.
- Corregido: el dock y la barra superior podían quedarse negros tras el ahorro
  de energía; sin transparencia ahora son grises sólidos, como macOS con
  «Reducir transparencia».
- Corregido: la barra superior podía perder una actualización y no mostraba el
  nombre de la app hasta el primer cambio de ventana.
- Barra superior: la batería muestra un rayo mientras carga.
- Ajustes: las opciones que dependen de otra se pliegan bajo ella.

## [0.1.4]

### English

- Maintenance release: align Genie default documentation and tests with the
  shipped setting.
- Website: add an Open Graph preview image and document the observed idle
  memory reading.

### Español

- Versión de mantenimiento: alinear la documentación y las pruebas del efecto
  Genio con el ajuste publicado.
- Web: añadir una imagen de vista previa Open Graph y documentar la lectura
  observada de memoria en reposo.

## [0.1.2]

### English

- Updates are announced in Settings: a banner above every section with
  "Update now", "Later" and "What's new", download progress, and a badge on
  System while an update waits.
- Smoother Genie effect: the system compositor now runs the whole animation
  at the display's refresh rate, and it no longer skips its first frames.
- Genie: no more flash when minimizing, and a window restored with Alt+Tab or
  from the app itself animates again the next time it is minimized.
- Fixed: the Dock section of Settings could not scroll to its last options.
- Fixed: opening Settings twice opened two windows; it now brings the open one
  to the front.
- Fixed: with an app running as administrator in front (Task Manager, an admin
  terminal, an installer), the dock's Start button and the menu bar buttons did
  nothing, because Windows drops simulated keys aimed at such apps.

### Español

- Las actualizaciones se anuncian en Ajustes: un aviso encima de cada sección
  con «Actualizar ahora», «Más tarde» y «Novedades», progreso de la descarga y
  una insignia en Sistema mientras haya una pendiente.
- Efecto genio más fluido: el compositor del sistema ejecuta toda la animación
  a la frecuencia del monitor y ya no se salta sus primeros fotogramas.
- Genio: ya no hay destello al minimizar, y una ventana restaurada con Alt+Tab o
  desde la propia app vuelve a animarse la siguiente vez que se minimiza.
- Corregido: la sección Dock de Ajustes no dejaba bajar hasta las últimas opciones.
- Corregido: abrir Ajustes dos veces abría dos ventanas; ahora trae al frente la abierta.
- Corregido: con una app de administrador delante (Administrador de tareas, una
  terminal de administrador, un instalador), el botón Inicio del dock y los botones
  de la barra superior no hacían nada, porque Windows descarta las teclas simuladas
  dirigidas a esas apps.

## [0.1.1]

### English

- Menu bar: network (Wi-Fi strength, Ethernet or offline) and volume, updated
  as they change. Search, Task View and Widgets buttons appear when they are on
  in your Windows taskbar settings.
- Clicking the app that is already in front minimizes it, as the Windows
  taskbar does; click again to bring it back. Settings can switch this to
  "do nothing", as on a Mac.
- Apps opened from the dock come to the front reliably instead of sometimes
  opening behind other windows.
- Minimized apps that run as administrator, such as Task Manager, can be brought
  back from the dock.
- Settings: restore defaults, and options to show open-app indicators, show
  recent apps and animate opening apps.

### Español

- Barra superior: red (intensidad del Wi-Fi, Ethernet o sin conexión) y volumen,
  actualizados al momento. Los botones de búsqueda, Vista de tareas y Widgets
  aparecen si los tienes activados en la barra de tareas de Windows.
- Clic en la app que ya está delante la minimiza, como la barra de Windows; otro
  clic la vuelve a traer. En Ajustes se puede cambiar a «no hacer nada», como en Mac.
- Las apps abiertas desde el dock aparecen delante siempre, en vez de abrirse a
  veces detrás de otras ventanas.
- Las apps minimizadas que se ejecutan como administrador, como el Administrador
  de tareas, se pueden recuperar desde el dock.
- Ajustes: restablecer valores y opciones para mostrar los indicadores de apps
  abiertas, mostrar las apps recientes y animar al abrir apps.

## [0.1.0]

### English

- First release: glass dock with magnification, live minimized-window
  thumbnails, optional Genie effect, menu bar, Settings app in English and
  Spanish, one-step installer and in-app updates.

### Español

- Primera versión: dock de cristal con ampliación, miniaturas en vivo de las
  ventanas minimizadas, efecto genio opcional, barra superior, app de Ajustes en
  español e inglés, instalador de un paso y actualizaciones desde la app.
