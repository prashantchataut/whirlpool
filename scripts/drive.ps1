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
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern IntPtr FindWindow(string cls, string title);
  [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h, uint msg, IntPtr w, IntPtr l);
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
function Wait-Window($title, $seconds = 30) {
    $until = [DateTime]::Now.AddSeconds($seconds)
    while ([DateTime]::Now -lt $until) {
        if ([U]::FindWindow($null, $title) -ne [IntPtr]::Zero) { Start-Sleep -Milliseconds 1500; return $true }
        Start-Sleep -Milliseconds 300
    }
    Log "window not found: $title"
    return $false
}
function Scenario($name, [scriptblock]$body) {
    Log "== $name"
    try { & $body } catch { Log "FAILED ${name}: $_" }
}

# The runner shows its agent console over the desktop; get it out of the frame.
(New-Object -ComObject Shell.Application).MinimizeAll()
Start-Sleep 2
# Dialogs the runner image raises on its own (the paging-file notice) are not part of the app.
foreach ($title in 'System Properties') {
    $h = [U]::FindWindow($null, $title)
    if ($h -ne [IntPtr]::Zero) { [void][U]::PostMessage($h, 0x0010, [IntPtr]::Zero, [IntPtr]::Zero); Log "closed runner dialog '$title'" }
}
Start-Sleep 1
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
    [void](Wait-Window 'Whirlpool Launchpad')
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
    [void](Wait-Window 'Whirlpool Launchpad')
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

function Wheel($notches) { [U]::mouse_event(0x0800, 0, 0, [BitConverter]::ToUInt32([BitConverter]::GetBytes([int]($notches * -120)), 0), [UIntPtr]::Zero) }
function Type-Keys([byte[]]$vks) { foreach ($k in $vks) { [U]::keybd_event($k, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 30; [U]::keybd_event($k, 0, 2, [UIntPtr]::Zero); Start-Sleep -Milliseconds 40 } }

Scenario 'search palette: apps, calculator, actions, clipboard' {
    Start-Dock '{"menuBar":true,"menuBarPosition":"top"}'
    Set-Clipboard -Value 'water-blue 4a90d9'
    Start-Sleep 1
    Set-Clipboard -Value 'meeting notes: ripple the tide'
    Start-Sleep 1
    Key ([byte[]](0x11, 0x10, 0x20))     # Ctrl+Shift+Space
    [void](Wait-Window 'Whirlpool Search')
    Shot '08-palette-open'
    Type-Keys ([byte[]](0x43, 0x41, 0x4C))   # c a l
    Start-Sleep 2
    Shot '08-palette-apps'
    Key ([byte[]](0x11, 0x41)); Type-Keys ([byte[]](0x08, 0x32, 0x6B, 0x33, 0x6A, 0x34))   # select all, delete, 2+3*4
    Start-Sleep 2
    Shot '08-palette-calc'
    Key ([byte[]](0x11, 0x41)); Type-Keys ([byte[]](0x08)); Key ([byte[]](0x10, 0xBE))     # clear, then ">"
    Start-Sleep 2
    Shot '08-palette-actions'
    Key ([byte[]](0x11, 0x41)); Type-Keys ([byte[]](0x08)); Key ([byte[]](0x10, 0x33))     # clear, then "#"
    Start-Sleep 2
    Shot '08-palette-clipboard'
    Key ([byte[]](0x1B))
}

foreach ($page in 'windows', 'dock') {
    Scenario "settings previews ($page)" {
        KillAll
        $env:WHIRLPOOL_PAGE = $page
        $p = Start-Process $Exe -ArgumentList '--settings' -PassThru
        Start-Sleep 10
        Move-Mouse 520 400
        foreach ($n in 1, 2, 3) { Wheel 4; Start-Sleep -Milliseconds 700; Shot "09-settings-$page-scroll$n" }
        Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
        Start-Sleep 1
    }
}
Remove-Item Env:WHIRLPOOL_PAGE -ErrorAction SilentlyContinue

foreach ($pair in @(@('R', 0x52), @('Q', 0x51))) {
    Scenario "emergency key Ctrl+Alt+Shift+$($pair[0])" {
        Start-Dock '{"menuBar":true,"menuBarPosition":"top"}'
        Shot "10-before-$($pair[0])"
        $sw = [Diagnostics.Stopwatch]::StartNew()
        Key ([byte[]](0x11, 0x12, 0x10, [byte]$pair[1]))
        while ((Get-Process whirlpool -ErrorAction SilentlyContinue) -and $sw.ElapsedMilliseconds -lt 20000) { Start-Sleep -Milliseconds 50 }
        Log "key $($pair[0]): whirlpool gone after $($sw.ElapsedMilliseconds) ms; remaining=$(@(Get-Process whirlpool -ErrorAction SilentlyContinue).Count)"
        Start-Sleep 2
        $tray = [U]::FindWindow('Shell_TrayWnd', $null)
        Log "key $($pair[0]): Windows taskbar window present=$($tray -ne [IntPtr]::Zero)"
        Shot "10-after-$($pair[0])"
    }
}

KillAll
& $Exe --restore 2>$null
Copy-Item "$env:LOCALAPPDATA\Whirlpool\whirlpool.log" (Join-Path $Out 'whirlpool.log') -ErrorAction SilentlyContinue
Log 'done'

Scenario 'notification center: history, repeat folding, grouping' {
    Start-Dock '{"menuBar":true,"menuBarPosition":"top"}'
    Remove-Item (Join-Path $dir 'notifications.txt') -ErrorAction SilentlyContinue
    & $Exe --notify 'Backup finished' '12 files copied to D:\Backups'
    & $Exe --notify 'Build succeeded' 'whirlpool 4.2.0 in 3m 12s'
    & $Exe --notify 'Backup finished' '12 files copied to D:\Backups'
    & $Exe --notify 'Disk almost full' 'C: has 4 GB left'
    Start-Sleep 2
    & $Exe --notifications
    [void](Wait-Window 'Whirlpool Notifications')
    Shot '11-notifications'
    Key ([byte[]](0x1B))
    Start-Sleep 2
    & $Exe --notifications
    [void](Wait-Window 'Whirlpool Notifications')
    Shot '11-notifications-reopened'
    Key ([byte[]](0x1B))
}

Scenario 'quick settings with an edited tile grid' {
    Start-Dock '{"menuBar":true,"menuBarPosition":"top","quickTiles":[{"kind":"wifi","size":"wide"},{"kind":"bluetooth"},{"kind":"focus"},{"kind":"display"},{"kind":"sound","size":"wide"}]}'
    & $Exe --quick-settings ($screen.Width - 12) 40 top
    [void](Wait-Window 'Whirlpool Quick Settings')
    Shot '12-quick-tiles'
    Key ([byte[]](0x1B))
}

Scenario 'hot corners: top-left opens search, top-right opens notifications' {
    Start-Dock '{"menuBar":false,"hotCornerTopLeft":"palette","hotCornerTopRight":"notifications"}'
    Move-Mouse 300 300; Start-Sleep 1
    Move-Mouse 0 0
    [void](Wait-Window 'Whirlpool Search')
    Shot '13-corner-top-left'
    Key ([byte[]](0x1B)); Start-Sleep 2
    Move-Mouse 400 400; Start-Sleep 1
    Move-Mouse ($screen.Width - 1) 0
    [void](Wait-Window 'Whirlpool Notifications')
    Shot '13-corner-top-right'
    Key ([byte[]](0x1B))
}

foreach ($style in 'dark', 'clear', 'tinted') {
    Scenario "dock icon style $style" {
        Start-Dock "{`"menuBar`":true,`"menuBarPosition`":`"top`",`"dockIconStyle`":`"$style`"}"
        Move-Mouse ($screen.Width / 2) 300; Start-Sleep 1
        Shot "14-dock-$style"
    }
}
