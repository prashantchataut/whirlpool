# Photographs every Whirlpool surface at 1920x1080, 1:1, on a real photo wallpaper and a fresh profile.
# Also presses the hardware volume keys and records what appeared: our pill, or the stock flyout.
# Usage: pwsh scripts/showcase.ps1 -Exe target/release/whirlpool.exe -Out out
param([string]$Exe = 'target/release/whirlpool.exe', [string]$Out = 'out')
$ErrorActionPreference = 'Continue'
New-Item -ItemType Directory -Force $Out | Out-Null
$Exe = (Resolve-Path $Exe).Path
Add-Type -AssemblyName System.Windows.Forms, System.Drawing
Add-Type @"
using System; using System.Runtime.InteropServices; using System.Text; using System.Collections.Generic;
public static class Sc {
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern bool SystemParametersInfo(int a, int b, string c, int d);
  [DllImport("user32.dll")] public static extern void keybd_event(byte vk, byte scan, int flags, int extra);
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, int dx, int dy, uint d, UIntPtr e);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern IntPtr FindWindow(string c, string t);
  [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h, uint m, IntPtr w, IntPtr l);
  delegate bool Cb(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] static extern bool EnumWindows(Cb cb, IntPtr l);
  [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern int GetClassName(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] static extern bool GetWindowRect(IntPtr h, out RECT r);
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
  public static string Flyouts() {
    var found = new List<string>();
    EnumWindows((h, l) => {
      if (!IsWindowVisible(h)) return true;
      var s = new StringBuilder(128); GetClassName(h, s, 128);
      var c = s.ToString();
      if (c == "XamlExplorerHostIslandWindow" || c == "NativeHWNDHost") {
        RECT r; GetWindowRect(h, out r);
        int w = r.R - r.L, hh = r.B - r.T;
        if (w > 0 && hh > 0 && w <= 480 && hh <= 180) found.Add(c + " " + w + "x" + hh + " at " + r.L + "," + r.T);
      }
      return true; }, IntPtr.Zero);
    return string.Join("; ", found);
  }
}
"@
function Log($m) { "$([DateTime]::Now.ToString('HH:mm:ss')) $m" | Tee-Object -Append (Join-Path $Out 'showcase-log.txt') }
# Dialogs the runner image raises on its own (the paging-file notice) are not part of the app.
function Close-RunnerDialogs {
  foreach ($title in 'System Properties') {
    $h = [Sc]::FindWindow($null, $title)
    if ($h -ne [IntPtr]::Zero) { [void][Sc]::PostMessage($h, 0x0010, [IntPtr]::Zero, [IntPtr]::Zero); Log "closed runner dialog '$title'"; Start-Sleep -Milliseconds 700 }
  }
}
function Click-At($x, $y) {
  [void][Sc]::SetCursorPos([int]$x, [int]$y); Start-Sleep -Milliseconds 250
  [Sc]::mouse_event(0x0002, 0, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 70; [Sc]::mouse_event(0x0004, 0, 0, 0, [UIntPtr]::Zero)
}
function Shot($name) {
  Close-RunnerDialogs
  $b = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
  $bmp = New-Object System.Drawing.Bitmap $b.Width, $b.Height
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $g.CopyFromScreen($b.Location, [System.Drawing.Point]::Empty, $b.Size)
  $bmp.Save((Join-Path $Out "$name.png")); $bmp.Dispose()
  Log "shot $name $($b.Width)x$($b.Height)"
}
# Every scene gets a 4K picture from 4kwallpapers.com through the studio (the same code path as the
# Apply button). If the site refuses the runner (it sits behind a bot check), the scene falls back to
# the photo pack and the log says so: a frame never claims a source it did not use.
$Studio = @{
  'aurora-lake.jpg' = 27371; 'misty-peaks.jpg' = 27986; 'dune-sunrise.jpg' = 27362; 'blue-hour.jpg' = 28066
  'milky-way.jpg' = 27369; 'turquoise-surf.jpg' = 27374; 'city-lights.jpg' = 26734
}
$script:Sources = @()
function Wall($file) {
  $id = $Studio[$file]
  if ($id) {
    # whirlpool.exe is a GUI program: wait for it and read the exit code from the process object.
    $o = Join-Path $env:TEMP 'studio-out.txt'; $e = Join-Path $env:TEMP 'studio-err.txt'
    $p = Start-Process $Exe -ArgumentList '--studio-apply', $id -Wait -PassThru -WindowStyle Hidden -RedirectStandardOutput $o -RedirectStandardError $e
    Log "studio apply $id exit $($p.ExitCode): $((Get-Content $o, $e -ErrorAction SilentlyContinue) -join ' ')"
    $LASTEXITCODE = $p.ExitCode
    if ($p.ExitCode -eq 0) { $script:Sources += "$file => 4kwallpapers.com id $id (3840x2160)"; Start-Sleep 3; return }
    Log "studio exit $LASTEXITCODE for id $id; falling back to the photo pack ($file)"
  }
  $script:Sources += "$file => photo pack (Pexels)"
  [void][Sc]::SystemParametersInfo(20, 0, (Resolve-Path "assets/wallpapers/$file").Path, 3); Start-Sleep 2
}
# A desktop that is not flat: standard deviation of grey levels over a grid of samples.
function Spread($path) {
  $bmp = [System.Drawing.Bitmap]::FromFile($path); $v = @()
  for ($x = 40; $x -lt $bmp.Width - 40; $x += [int]($bmp.Width / 24)) { for ($y = 40; $y -lt $bmp.Height - 120; $y += [int]($bmp.Height / 14)) { $c = $bmp.GetPixel($x, $y); $v += ($c.R + $c.G + $c.B) / 3 } }
  $bmp.Dispose(); $m = ($v | Measure-Object -Average).Average
  [Math]::Round([Math]::Sqrt((($v | ForEach-Object { ($_ - $m) * ($_ - $m) }) | Measure-Object -Average).Average), 1)
}
# Windows an enthusiast would have open behind the shell: Explorer and a text file.
function Stage-Apps {
  $note = Join-Path $env:TEMP 'notes.txt'
  "Release checklist`r`n- 4K wallpapers: studio apply, accent`r`n- Island: lyrics, shelf`r`n- Dock: 32 px, running dots`r`n- Updater: rc to next, settings kept" | Set-Content $note
  Start-Process notepad.exe $note; Start-Process explorer.exe $env:USERPROFILE; Start-Sleep 4
}
function Unstage-Apps {
  Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force
  try { (New-Object -ComObject Shell.Application).Windows() | ForEach-Object { $_.Quit() } } catch {}
}
function Reset { Get-Process whirlpool -ErrorAction SilentlyContinue | Stop-Process -Force; Start-Sleep 2 }
function Panels {
  Get-CimInstance Win32_Process -Filter "Name='whirlpool.exe'" | Where-Object { $_.CommandLine -match '--(launchpad|palette|quick-settings|hub|notifications|settings|overview|studio)' } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }
  Start-Sleep 1
}
function Key($vk) { [Sc]::keybd_event($vk, 0, 0, 0); Start-Sleep -Milliseconds 40; [Sc]::keybd_event($vk, 0, 2, 0) }

# Fresh profile: no config file, so the defaults apply (32 px dock).
$cfg = Join-Path $env:APPDATA 'Whirlpool'
Remove-Item -Recurse -Force $cfg -ErrorAction SilentlyContinue
Log "profile reset; display $([System.Windows.Forms.Screen]::PrimaryScreen.Bounds.Width)x$([System.Windows.Forms.Screen]::PrimaryScreen.Bounds.Height)"

# A daily-driver desktop: no build watermark, no icon clutter. Both are per-user settings read by Explorer,
# so Explorer is restarted once, before Whirlpool exists. What the log says is what happened.
try {
  Set-ItemProperty 'HKCU:\Control Panel\Desktop' -Name PaintDesktopVersion -Value 0 -Type DWord
  Set-ItemProperty 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced' -Name HideIcons -Value 1 -Type DWord
  Get-Process explorer -ErrorAction SilentlyContinue | Stop-Process -Force; Start-Sleep 6
  if (-not (Get-Process explorer -ErrorAction SilentlyContinue)) { Start-Process explorer.exe; Start-Sleep 5 }
  Log 'desktop: version text off, icons hidden, explorer restarted'
} catch { Log "desktop tidy failed: $_" }
Log "test signing: $((bcdedit /enum '{current}' 2>&1 | Select-String -Pattern 'testsigning') -join ' ')"
Close-RunnerDialogs

# The runner's own Windows Terminal (its agent log) sits on the desktop: minimise it, never close it.
try { (New-Object -ComObject Shell.Application).MinimizeAll() } catch {}
Start-Sleep 2
Wall 'aurora-lake.jpg'
Reset; Start-Process $Exe; Start-Sleep 14
Shot '01-desktop'
Log "desktop spread (grey-level deviation, flat < 12): $(Spread (Join-Path $Out '01-desktop.png'))"
# A real media session (title, artist, cover, progress) so the island has a track; then a real click opens it.
$media = Start-Process powershell.exe -ArgumentList '-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', (Resolve-Path 'scripts/stage-media.ps1').Path, '-Seconds', '300' -PassThru -WindowStyle Hidden
Start-Sleep 8
Click-At ([System.Windows.Forms.Screen]::PrimaryScreen.Bounds.Width / 2) 12; Start-Sleep 3
Shot '01c-island-media'
Click-At ([System.Windows.Forms.Screen]::PrimaryScreen.Bounds.Width / 2) 400; Start-Sleep 1
Stage-Apps; Shot '01b-desktop-apps'; Unstage-Apps
& $Exe --studio; Start-Sleep 12; Shot '09-wallpaper-studio'; Panels
Get-Process whirlpool -ErrorAction SilentlyContinue | Where-Object { $_.MainWindowTitle -eq 'Wallpaper' } | Stop-Process -Force
Log "saved config after first run: $(if (Test-Path (Join-Path $cfg 'config.json')) { (Get-Content (Join-Path $cfg 'config.json') -Raw) -replace '\s+',' ' | ForEach-Object { $_.Substring(0, [Math]::Min(400, $_.Length)) } } else { 'none (defaults only)' })"

# Hardware volume keys: the pill should appear and the stock flyout should not.
Shot '02a-before-keys'
Log "stock flyout windows before keys: [$([Sc]::Flyouts())]"
1..5 | ForEach-Object { Key 0xAF; Start-Sleep -Milliseconds 120 }
Start-Sleep -Milliseconds 400
$seen = [Sc]::Flyouts()
Shot '02b-after-volume-up'
Log "stock flyout windows after volume up x5: [$seen]"
Key 0xAD; Start-Sleep -Milliseconds 500
Log "stock flyout windows after mute: [$([Sc]::Flyouts())]"
Shot '02c-after-mute'
# Whirlpool's own pill, forced with the command so it is always photographed.
& $Exe --hud volume 64; Start-Sleep -Milliseconds 900; Shot '02-volume-pill'

Wall 'misty-peaks.jpg'; Start-Sleep 1
& $Exe --launchpad; Start-Sleep 5; Shot '03-start-grid'
Panels

Wall 'dune-sunrise.jpg'
Start-Process $Exe -ArgumentList '--quick-settings', '1900', '34', 'top'; Start-Sleep 9; Shot '04-quick-settings'
Panels

Wall 'blue-hour.jpg'
Start-Process $Exe -ArgumentList '--palette'; Start-Sleep 5; Shot '05-command-palette'
Panels

Wall 'milky-way.jpg'
& $Exe --notify 'Build succeeded' 'whirlpool finished in 3m 12s'; & $Exe --notify 'Backup finished' '12 files copied'; Start-Sleep 1
Start-Process $Exe -ArgumentList '--notifications'; Start-Sleep 5; Shot '06-notification-center'
Panels

Wall 'turquoise-surf.jpg'
Start-Process $Exe -ArgumentList '--hub', 'clock'; Start-Sleep 5; Shot '07-clock-panel'
Panels

Wall 'city-lights.jpg'
Start-Process $Exe -ArgumentList '--settings'; Start-Sleep 9; Shot '08-settings'
Reset
& $Exe --restore
$script:Sources | ForEach-Object { Log "source: $_" }
if ($media -and -not $media.HasExited) { Stop-Process -Id $media.Id -Force }
Log 'done'
