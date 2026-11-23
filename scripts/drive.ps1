# Drives the built Whirlpool on a Windows runner with real mouse and keyboard input and photographs the
# screen. Used by .github/workflows/screenshots.yml. Every scenario is independent: one that fails
# is written to the log and the rest still run, so a missing feature shows up as a missing image.
param(
    [Parameter(Mandatory = $true)][string]$Exe,
    [Parameter(Mandatory = $true)][string]$Out
)
$ErrorActionPreference = 'Continue'
New-Item -ItemType Directory -Force $Out | Out-Null
$log = Join-Path $Out 'drive-log.txt'
function Log($m) { "$([DateTime]::Now.ToString('HH:mm:ss')) $m" | Tee-Object -Append $log }

Add-Type -AssemblyName System.Windows.Forms, System.Drawing
Add-Type @"
using System; using System.Runtime.InteropServices;
public static class U {
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, int dx, int dy, uint d, UIntPtr e);
  [DllImport("user32.dll")] public static extern void keybd_event(byte vk, byte scan, uint flags, UIntPtr e);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
}
"@
[void][U]::SetProcessDPIAware()

$screen = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
Log "screen $($screen.Width)x$($screen.Height) arch=$env:PROCESSOR_ARCHITECTURE"
$dir = Join-Path $env:APPDATA 'Whirlpool'
New-Item -ItemType Directory -Force $dir | Out-Null

function Shot($name) {
    $bmp = New-Object System.Drawing.Bitmap $screen.Width, $screen.Height
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.CopyFromScreen($screen.Location, [System.Drawing.Point]::Empty, $screen.Size)
    $bmp.Save((Join-Path $Out "$name.png"))
    $g.Dispose(); $bmp.Dispose()
    Log "shot $name"
}
function KillAll {
    & $Exe --quit 2>$null
    Start-Sleep 1
    Get-Process whirlpool -ErrorAction SilentlyContinue | Stop-Process -Force
    Start-Sleep 2
}
function Move-Mouse($x, $y) { [void][U]::SetCursorPos([int]$x, [int]$y); Start-Sleep -Milliseconds 120 }
function Click($x, $y) {
    Move-Mouse $x $y; Start-Sleep -Milliseconds 200
    [U]::mouse_event(0x0002, 0, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 70
    [U]::mouse_event(0x0004, 0, 0, 0, [UIntPtr]::Zero)
}
function Key([byte[]]$keys) {
    foreach ($k in $keys) { [U]::keybd_event($k, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 40 }
    [array]::Reverse($keys)
    foreach ($k in $keys) { [U]::keybd_event($k, 0, 2, [UIntPtr]::Zero); Start-Sleep -Milliseconds 40 }
}
function Start-Dock($json) {
    Set-Content -Path (Join-Path $dir 'config.json') -Value $json
    KillAll
    $env:WHIRLPOOL_DEMO = '1'
    $script:dock = Start-Process $Exe -PassThru
    Start-Sleep 14
    Log "dock running=$([bool](Get-Process whirlpool -ErrorAction SilentlyContinue))"
}
function Scenario($name, [scriptblock]$body) {
    Log "== $name"
    try { & $body } catch { Log "FAILED ${name}: $_" }
}

# The runner shows its agent console over the desktop; get it out of the frame.
(New-Object -ComObject Shell.Application).MinimizeAll()
Start-Sleep 2
Shot '00-runner-desktop'

Scenario 'bar and dock, top bar' {
    Start-Dock '{"menuBar":true,"menuBarPosition":"top","menuBarMusic":false}'
    Move-Mouse ($screen.Width / 2) ($screen.Height / 2); Start-Sleep 1
    Shot '01-bar-dock-top'
}

foreach ($preset in 'compact', 'classic', 'wide', 'expressive') {
    Scenario "island $preset" {
        Start-Dock "{`"menuBar`":true,`"menuBarPosition`":`"top`",`"menuBarMusic`":false,`"menuBarIslandPreset`":`"$preset`"}"
        Move-Mouse ($screen.Width / 2) ($screen.Height / 2); Start-Sleep 2
        Shot "02-island-$preset-closed"
        Click ($screen.Width / 2) 12
        Start-Sleep 2
        Shot "02-island-$preset-open"
    }
}

Scenario 'bar and dock, bottom floating bar over a bottom dock' {
    Start-Dock '{"menuBar":true,"menuBarPosition":"bottom","menuBarStyle":"floating","dockPosition":"bottom"}'
    Move-Mouse ($screen.Width / 2) 300; Start-Sleep 1
    Shot '03-bar-bottom-with-dock'
}

Scenario 'quick settings and volume' {
    Start-Dock '{"menuBar":true,"menuBarPosition":"top"}'
    Click ($screen.Width - 88) 12
    Start-Sleep 3
    Shot '04-quick-settings'
}

foreach ($page in 'dock', 'menubar', 'personalize', 'windows', 'system', 'about') {
    Scenario "settings $page" {
        KillAll
        $env:WHIRLPOOL_PAGE = $page
        $p = Start-Process $Exe -ArgumentList '--settings' -PassThru
        Start-Sleep 9
        Shot "05-settings-$page"
        Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
        Start-Sleep 1
    }
}
Remove-Item Env:WHIRLPOOL_PAGE -ErrorAction SilentlyContinue

Scenario 'launchpad: dock button, hotkey, search, keys' {
    Start-Dock '{"menuBar":true,"menuBarPosition":"top"}'
    # Hotkey: Ctrl+Alt+Space
    Key ([byte[]](0x11, 0x12, 0x20))
    Start-Sleep 3
    Shot '06-launchpad-hotkey'
    Key ([byte[]](0x43))     # c
    Key ([byte[]](0x61))     # a (numpad 1 key code is not a letter; harmless)
    Start-Sleep 1
    Shot '06-launchpad-search'
    Key ([byte[]](0x27))     # right arrow
    Key ([byte[]](0x28))     # down arrow
    Start-Sleep 1
    Shot '06-launchpad-navigated'
    Key ([byte[]](0x1B))     # escape
    Start-Sleep 1
    Log "launchpad closed by Escape: running=$([bool](Get-Process whirlpool -ErrorAction SilentlyContinue | Where-Object { $_.Id -ne $script:dock.Id }))"
    & $Exe --launchpad
    Start-Sleep 3
    Shot '06-launchpad-cli'
    Key ([byte[]](0x1B))
}

Scenario 'drag motion: shadow, snap preview, settle' {
    Start-Dock '{"menuBar":true,"menuBarPosition":"top","dragMotion":true}'
    $np = Start-Process notepad.exe -PassThru
    Start-Sleep 4
    $np.Refresh()
    $r = New-Object U+RECT
    [void][U]::GetWindowRect($np.MainWindowHandle, [ref]$r)
    Log "notepad rect $($r.L),$($r.T) $($r.R),$($r.B)"
    $gx = [int](($r.L + $r.R) / 2); $gy = $r.T + 14
    Move-Mouse $gx $gy
    [U]::mouse_event(0x0002, 0, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 200
    foreach ($i in 1..12) { Move-Mouse ($gx - $i * 14) ($gy + $i * 9); Start-Sleep -Milliseconds 40 }
    Start-Sleep -Milliseconds 400
    Shot '07-drag-held'
    foreach ($i in 1..30) { Move-Mouse (($gx - 168) - $i * 12) ($gy + 108 + $i * 4); Start-Sleep -Milliseconds 20 }
    Move-Mouse 3 400; Start-Sleep -Milliseconds 700
    Shot '07-drag-snap-preview'
    [U]::mouse_event(0x0004, 0, 0, 0, [UIntPtr]::Zero)
    Start-Sleep -Milliseconds 120
    Shot '07-drag-release-120ms'
    Start-Sleep 1
    Shot '07-drag-settled'
    Stop-Process -Id $np.Id -Force -ErrorAction SilentlyContinue
}

KillAll
& $Exe --restore 2>$null
Copy-Item "$env:LOCALAPPDATA\Whirlpool\whirlpool.log" (Join-Path $Out 'whirlpool.log') -ErrorAction SilentlyContinue
Log 'done'
