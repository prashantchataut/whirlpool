# Testing what CI cannot see (about 10 minutes)

CI builds the program, runs the unit tests and photographs it on GitHub's Windows runners (x64 and ARM64).
Those runners are single-monitor servers at 1024x768 with no battery, no touchpad and nobody looking. This list
covers the rest. Do it on a real Windows 10 or 11 machine, ideally with a second monitor.
Write the result of each line in the table at the bottom and paste it into the release notes or the PR.

Install the build under test from the release assets (`Whirlpool-Setup.exe`, or the ARM64 one on ARM).
Open `%LOCALAPPDATA%\Whirlpool\whirlpool.log` afterwards: it should hold no error lines.

## 1. First run and Start (1 min)
- [ ] The dock and the menu bar appear; the Windows taskbar is hidden; nothing flickers.
- [ ] Click the dock's first tile: the Launchpad opens. Click it again or press Esc: it closes.
- [ ] `Ctrl+Alt+Space` opens it. Type two letters: the grid filters. Use the arrow keys: the highlight moves. Enter opens the app.
- [ ] Turn the Launchpad off in Settings: the first tile opens the Windows Start menu again.

## 2. Window drag motion (1 min)
- [ ] Settings > Windows > Dragging windows ("Shadow, snap preview and settle") is on. Drag a normal window by its title bar: a soft shadow sits behind it and follows it.
- [ ] Drag to the left or right edge and to a corner: a snap preview appears where it would land.
- [ ] Release: the window settles with a short spring and no overshoot. Release in the middle of the screen: nothing moves.
- [ ] Resize by an edge: no shadow, no preview.
- [ ] Repeat with "Show window contents while dragging" off (Windows Performance Options, `SystemPropertiesPerformance.exe`). Note what happens (the system then draws only an outline).

## 3. Island presets (1 min)
- [ ] Settings > Menu bar: switch Compact, Classic, Wide, Expressive. The collapsed island changes width each time, with one provider and with two.
- [ ] Open the island in each preset: the panel width and row height change and nothing is clipped.

## 4. Several monitors (2 min)
- [ ] The dock and menu bar appear on the monitors you chose; the other one is untouched.
- [ ] Drag a window to the second monitor: the shadow and the snap preview follow, with the right size.
- [ ] Move the pointer to the top edge of the second monitor: the island reacts there, not on the first.
- [ ] Unplug the second monitor while running, then plug it back in: no crash, windows are not lost.
- [ ] Put monitors at different scale (for example 100% and 150%): text and tiles stay sharp on both.

## 5. Scale (1 min)
- [ ] At 100%, 150% and 200% (Windows Settings > System > Display): the dock, menu bar, island, Launchpad and Settings are not clipped, text is sharp, hit areas line up with what you see.
- [ ] Sign out and in after changing the scale; check again.

## 6. Hotkeys and recovery (1 min)
- [ ] `Ctrl+Shift+Space` opens search. `Ctrl+Shift+F10` toggles the overlay.
- [ ] `Ctrl+Alt+Shift+R` brings the Windows taskbar back and quits Whirlpool (under two seconds). Start it again.
- [ ] `whirlpool --quit` and `whirlpool --restore` leave the desktop usable.

## 7. Battery and power (2 min, laptop)
- [ ] Unplug and plug the charger: the charging animation plays once and stops. Try each style you ship.
- [ ] Task Manager, Details: with nothing moving, `whirlpool.exe` sits at 0% CPU for a minute.
- [ ] The battery tile in quick settings shows the same percentage as the Windows battery icon.
- [ ] Battery saver on: animation still works and nothing spins in the background.

## 8. ARM64 hardware (1 min, only on an ARM PC)
- [ ] Task Manager shows `whirlpool.exe` as Arm64, not x64 emulated.
- [ ] Items 1, 2 and 6 above pass.

## 9. Updates and notices (1 min)
- [ ] `whirlpool --update --remind-test` shows one Windows notification with Whirlpool's name and no script text.
- [ ] `whirlpool --notify "Test" "Hello"` lands in the notification history.
- [ ] Settings > System > Updates > Check now reports the current version or a newer one; nothing downloads by itself.

## 10. Uninstall (end)
- [ ] Uninstall from Windows Settings > Apps. The Windows taskbar is back, the startup entry is gone, `%LOCALAPPDATA%\Whirlpool` keeps only what the uninstaller says it keeps.

## Result

| Date | Build | Machine (CPU, GPU, monitors, scale, battery) | Sections passed | Sections failed and what you saw |
| --- | --- | --- | --- | --- |
| | | | | |

If something fails, attach `whirlpool.log` and a screenshot to an issue. Do not describe a capture from a CI
runner as a real-desktop result (AGENTS.md, Rule 4).

## Session 7: what only real hardware can check

- **Backdrop blur on a GPU desktop and at 125/150/200% scaling.** Captured only at 1024x768, 100%, software rendering. The screen grab is taken in physical pixels and the blur factor scales with width, but a high-DPI multi-monitor desktop has not been seen.
- **Frame pacing of the staggered entrance.** The runner has no GPU, so the motion was captured as two frames (mid-entrance and settled), not measured. Check it feels like ~0.5 s of arrival, with no dropped frames, on real hardware.
- **Start grid with 300+ apps** (page count, dot row width) and with Store apps whose icons are missing.
- **Volume capsule dragging with a real mouse, wheel and touch**, and muting from the speaker icon.
- **Update toast**: no runner capture exists. It is still unverified.
- **Light system theme**: both panels are always dark glass; check they still look intentional on a light desktop.

## Added in 4.2.0-rc6 (needs a real desktop)

- Press the volume keys: the pill should appear once per change, glide, and leave after about two seconds. Mute shows the crossed speaker. Opening the menu bar volume popup should show no pill.
- Turn Wi-Fi off and on from Windows: the Wi-Fi pill appears on connect and on loss. Turn Bluetooth off and on: the switch pill follows.
- `whirlpool --hud brightness 60` shows the brightness pill; changing brightness with the keys does not yet (not wired).
- Settings > Personalize: the "Volume and Wi-Fi pop-ups" switch turns all of them off. Drag and arrow-key every slider.
- 200 % display scale for every panel.
## Session 8: what only real hardware can check

- **200 % scale.** Hosted runners stop at 175 % (1920x1080); see `blockedScales` in `docs/capture-matrix.json`. Open the Start grid, quick settings, palette and overview at 200 % on a 4K or QHD+ display and check clipping and text sharpness.
- **Quick settings tiles with working radios.** On the runner Wi-Fi and Bluetooth read "Not available". Check the on, off and unavailable states on a laptop; the unavailable state was raised from 55 % to 85 % opacity in round 2 and has been seen only on the runner.
- **Legacy icons on the Start plates.** Old 32 px icons with their own square frame (for example 7-Zip) show that frame inside the plate. Seen in captures, not yet fixed.
- **Slider feel.** The custom volume slider in quick settings: drag, wheel, keyboard arrows and touch, with a real audio device. The runner has no audio endpoint, so the 50 % shown is a default.
- **Update toast, notch and bottom bar vs dock.** The notch (island) opens in runner captures. The toast and a real bottom-bar-plus-dock layout on a physical display remain unverified.


## Added for 4.2.0-rc7 (needs real hardware)
- [ ] Install `Whirlpool-Setup.exe` from the rc7 release on a personal desktop and use it for a day; read `%LOCALAPPDATA%\Whirlpool\whirlpool.log`.
- [ ] Start grid: monogram tiles for apps without an icon, two-line labels, search and arrow keys.
- [ ] Quick settings: the volume slider moves the real volume; the Wi-Fi and Bluetooth tiles read correctly when the radio is off or missing.
- [ ] Windows on ARM: run `Whirlpool-Setup-arm64.exe` on an ARM device (never run on ARM hardware yet).
- [ ] 200 percent scaling on a display of about 2048 x 1536 or larger.
- [ ] Brightness: no pop-up appears yet, by design (no trigger).
