// Captures of the real program on a Windows runner, 1920x1080, 1:1. `scripts/site-release.mjs` and the
// showcase log decide the `wallpaper` line, so a caption never names a source that was not used.
export type Shot = { id: string; tab: string; file: string; caption: string };

export const WALLPAPER_NOTE = "";

export const SHOTS: Shot[] = [
  { id: "desktop", tab: "Desktop", file: "01-desktop", caption: "Dock and menu bar over the desktop. The taskbar is hidden." },
  { id: "windows", tab: "In use", file: "01b-desktop-apps", caption: "With windows open: Explorer and a text file." },
  { id: "hud", tab: "Volume", file: "02-volume-pill", caption: "Volume pill, 200 by 40. The Windows flyout stays closed." },
  { id: "start", tab: "Start grid", file: "03-start-grid", caption: "Start grid. Type to filter, arrows to move." },
  { id: "quick", tab: "Quick settings", file: "04-quick-settings", caption: "Quick settings, 320 px wide, 48 px tiles." },
  { id: "palette", tab: "Ctrl+K", file: "05-command-palette", caption: "Command palette: apps, files, windows, settings." },
  { id: "notifications", tab: "Notifications", file: "06-notification-center", caption: "Notification centre." },
  { id: "clock", tab: "Clock", file: "07-clock-panel", caption: "Clock, focus timer, file shelf." },
  { id: "studio", tab: "Wallpaper", file: "09-wallpaper-studio", caption: "Wallpaper studio. Previews load from 4kwallpapers.com; the hosted runner is refused by its bot check, so the tiles are empty here." },
  { id: "settings", tab: "Settings", file: "08-settings", caption: "Settings." },
];
