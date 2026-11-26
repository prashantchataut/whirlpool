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
function Shot($name) {
  $b = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
  $bmp = New-Object System.Drawing.Bitmap $b.Width, $b.Height
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $g.CopyFromScreen($b.Location, [System.Drawing.Point]::Empty, $b.Size)
  $bmp.Save((Join-Path $Out "$name.png")); $bmp.Dispose()
  Log "shot $name $($b.Width)x$($b.Height)"
}
function Wall($file) { [void][Sc]::SystemParametersInfo(20, 0, (Resolve-Path "assets/wallpapers/$file").Path, 3); Start-Sleep 2 }
function Reset { Get-Process whirlpool -ErrorAction SilentlyContinue | Stop-Process -Force; Start-Sleep 2 }
function Panels {
  Get-CimInstance Win32_Process -Filter "Name='whirlpool.exe'" | Where-Object { $_.CommandLine -match '--(launchpad|palette|quick-settings|hub|notifications|settings|overview)' } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }
  Start-Sleep 1
}
function Key($vk) { [Sc]::keybd_event($vk, 0, 0, 0); Start-Sleep -Milliseconds 40; [Sc]::keybd_event($vk, 0, 2, 0) }

# Fresh profile: no config file, so the defaults apply (32 px dock).
$cfg = Join-Path $env:APPDATA 'Whirlpool'
Remove-Item -Recurse -Force $cfg -ErrorAction SilentlyContinue
Log "profile reset; display $([System.Windows.Forms.Screen]::PrimaryScreen.Bounds.Width)x$([System.Windows.Forms.Screen]::PrimaryScreen.Bounds.Height)"

Wall 'aurora-lake.jpg'
Reset; Start-Process $Exe; Start-Sleep 14
Shot '01-desktop'
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
Log 'done'
