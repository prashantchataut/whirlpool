# Drives the built Whirlpool on a Windows runner with real mouse and keyboard input and photographs the
# screen. Used by .github/workflows/screenshots.yml. Every scenario is independent: one that fails
# is written to the log and the rest still run, so a missing feature shows up as a missing image.
param(
    [Parameter(Mandatory = $true)][string]$Exe,
    [Parameter(Mandatory = $true)][string]$Out,
    [string]$Only = '',
    # Proof that the gate can fail: 'overview' never starts the overview, 'blank' never starts the Start grid
    # for the last Start capture. Used once by hand (workflow_dispatch input), never by default.
    [string]$Sabotage = ''
)
$ErrorActionPreference = 'Continue'
New-Item -ItemType Directory -Force $Out | Out-Null
$log = Join-Path $Out 'drive-log.txt'
# Out-Host: Log is called inside functions whose return value is used; its text must not become that value.
function Log($m) { "$([DateTime]::Now.ToString('HH:mm:ss')) $m" | Tee-Object -Append $log | Out-Host }

# Session 8 gate. A scenario whose window never appears, or whose frame is the bare desktop, is a
# failure: it is written to FAILED.txt and the script exits 1 at the end. Before this, an empty
# desktop was saved as a capture and the job was green (that is how a broken Start grid looked fine).
$script:Failures = New-Object System.Collections.Generic.List[string]
function Fail($m) { Log "GATE FAIL: $m"; $script:Failures.Add($m) }

Add-Type -AssemblyName System.Windows.Forms, System.Drawing
Add-Type @"
public static class Px {
  // Share of pixels (4 bytes each) whose largest channel difference is above tol.
  public static double Diff(byte[] a, byte[] b, int tol) {
    int n = System.Math.Min(a.Length, b.Length) / 4; if (n == 0) return 0; long hit = 0;
    for (int i = 0; i < n; i++) {
      int o = i * 4;
      int d = System.Math.Abs(a[o] - b[o]); int e = System.Math.Abs(a[o + 1] - b[o + 1]); int f = System.Math.Abs(a[o + 2] - b[o + 2]);
      if (d > tol || e > tol || f > tol) hit++;
    }
    return (double)hit / n;
  }
}
"@
Add-Type @"
using System; using System.Runtime.InteropServices;
public static class U {
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, int dx, int dy, uint d, UIntPtr e);
  [DllImport("user32.dll")] public static extern void keybd_event(byte vk, byte scan, uint flags, UIntPtr e);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern IntPtr FindWindow(string cls, string title);
  [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h, uint msg, IntPtr w, IntPtr l);
  [DllImport("gdi32.dll")] public static extern bool BitBlt(IntPtr dst, int x, int y, int w, int h, IntPtr src, int sx, int sy, int rop);
  [DllImport("user32.dll")] public static extern IntPtr GetDC(IntPtr h);
  [DllImport("user32.dll")] public static extern int ReleaseDC(IntPtr h, IntPtr dc);
  public delegate bool EnumProc(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc p, IntPtr l);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern int GetClassName(IntPtr h, System.Text.StringBuilder s, int n);
  // Visible top-level windows of one process: "class|left,top,right,bottom".
  public static System.Collections.Generic.List<string> Of(uint pid) {
    var r = new System.Collections.Generic.List<string>();
    EnumWindows(delegate (IntPtr h, IntPtr l) {
      uint p; GetWindowThreadProcessId(h, out p);
      if (p == pid && IsWindowVisible(h)) {
        RECT rc; GetWindowRect(h, out rc);
        var sb = new System.Text.StringBuilder(256); GetClassName(h, sb, 256);
        r.Add(sb.ToString() + "|" + rc.L + "," + rc.T + "," + rc.R + "," + rc.B);
      }
      return true;
    }, IntPtr.Zero);
    return r;
  }
  [DllImport("user32.dll")] public static extern bool SystemParametersInfo(uint action, uint param, IntPtr value, uint flags);
  [DllImport("user32.dll")] public static extern bool SystemParametersInfo(uint action, uint param, ref int value, uint flags);
}
"@
[void][U]::SetProcessDPIAware()

$screen = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
Log "screen $($screen.Width)x$($screen.Height) arch=$env:PROCESSOR_ARCHITECTURE"
$dir = Join-Path $env:APPDATA 'Whirlpool'
New-Item -ItemType Directory -Force $dir | Out-Null

function Close-RunnerDialogs {
    # The paging-file notice can appear after the resolution change, not only at the start (it covered
    # the 150% frames of the previous pass). It is not part of the app, so it never stays in a capture.
    foreach ($title in 'System Properties') {
        $h = [U]::FindWindow($null, $title)
        if ($h -ne [IntPtr]::Zero) { [void][U]::PostMessage($h, 0x0010, [IntPtr]::Zero, [IntPtr]::Zero); Log "closed runner dialog '$title'"; Start-Sleep -Milliseconds 700 }
    }
}
function Shot($name, $to = $Out) {
    Close-RunnerDialogs
    $bmp = New-Object System.Drawing.Bitmap $screen.Width, $screen.Height
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    # BitBlt with CAPTUREBLT (0x40000000) also copies layered windows, such as the drag overlay.
    $copied = $false
    try {
        $dst = $g.GetHdc(); $src = [U]::GetDC([IntPtr]::Zero)
        $copied = [U]::BitBlt($dst, 0, 0, $screen.Width, $screen.Height, $src, $screen.X, $screen.Y, 0x40CC0020)
        [void][U]::ReleaseDC([IntPtr]::Zero, $src); $g.ReleaseHdc($dst)
    } catch { Log "BitBlt failed: $_" }
    if (-not $copied) { $g.CopyFromScreen($screen.Location, [System.Drawing.Point]::Empty, $screen.Size) }
    $bmp.Save((Join-Path $to "$name.png"))
    $g.Dispose(); $bmp.Dispose()
    Log "shot $name"
}
function Get-Pixels($path) {
    $bmp = New-Object System.Drawing.Bitmap $path
    $rect = New-Object System.Drawing.Rectangle 0, 0, $bmp.Width, $bmp.Height
    $data = $bmp.LockBits($rect, [System.Drawing.Imaging.ImageLockMode]::ReadOnly, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $bytes = New-Object byte[] ($data.Stride * $bmp.Height)
    [System.Runtime.InteropServices.Marshal]::Copy($data.Scan0, $bytes, 0, $bytes.Length)
    $bmp.UnlockBits($data); $bmp.Dispose()
    return , $bytes
}
$refDir = Join-Path $env:TEMP 'whirlpool-ref'
New-Item -ItemType Directory -Force $refDir | Out-Null
# A panel capture must differ from the dock-only frame of the same scenario in at least $min of its pixels.
function Shot-Panel($name, $min = 0.05, [scriptblock]$Retrigger = $null) {
    $ref = Join-Path $refDir 'dock-only.png'
    if (-not (Test-Path $ref)) { Shot $name; Fail "no dock-only reference frame to compare $name with"; return }
    # A pop-up started by a command (the HUD pills) can be sent before a slow runner's dock listens: send it
    # again, up to three times, and only then call the frame blank. A pill that never paints still fails.
    $tries = if ($Retrigger) { 3 } else { 1 }
    for ($i = 1; $i -le $tries; $i++) {
        Shot $name
        $d = [Px]::Diff((Get-Pixels $ref), (Get-Pixels (Join-Path $Out "$name.png")), 24)
        Log ("painted {0}: {1:P1} of pixels differ from the dock-only frame (needs {2:P0}, try {3} of {4})" -f $name, $d, $min, $i, $tries)
        if ($d -ge $min) { return }
        if ($i -lt $tries) { Start-Sleep -Milliseconds 1500; & $Retrigger; Start-Sleep -Milliseconds 800 }
    }
    Fail "blank capture: $name differs from the dock-only frame in only $([math]::Round($d * 100, 2)) % of pixels"
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
    # A click on empty desktop closes any Windows flyout left open by an earlier scene.
    Click 600 ($screen.Height - 220); Start-Sleep 1
    Log "dock running=$([bool](Get-Process whirlpool -ErrorAction SilentlyContinue))"
    if (-not (Get-Process whirlpool -ErrorAction SilentlyContinue)) { Fail 'the dock process is not running after start' }
    Clear-FirstRun
    Shot 'dock-only' $refDir
}
function Type-Keys([byte[]]$vks) { foreach ($k in $vks) { [U]::keybd_event($k, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 30; [U]::keybd_event($k, 0, 2, [UIntPtr]::Zero); Start-Sleep -Milliseconds 40 } }
function Diagnose($title) {
    # What was running and what crashed, so a missing window is explained and not just reported.
    $procs = @(Get-Process whirlpool -ErrorAction SilentlyContinue | ForEach-Object { "$($_.Id):'$($_.MainWindowTitle)'" }) -join ' '
    Log "diagnose ${title}: whirlpool processes = [$procs]"
    try {
        Get-WinEvent -FilterHashtable @{ LogName = 'Application'; StartTime = [DateTime]::Now.AddMinutes(-3) } -ErrorAction Stop |
            Where-Object { $_.Message -match 'whirlpool' -and $_.LevelDisplayName -in 'Error', 'Warning' } | Select-Object -First 3 |
            ForEach-Object { Log ("diagnose event: " + (($_.Message -split "`n")[0..3] -join ' | ')) }
    } catch { Log "diagnose: no crash events ($($_.Exception.Message))" }
}
function Wait-Gone($title, $seconds = 15) {
    # A panel that was just closed is still a window for a moment; starting the next one now would meet its mutex and exit.
    $until = [DateTime]::Now.AddSeconds($seconds)
    while ([DateTime]::Now -lt $until) {
        if (-not (Get-Process whirlpool -ErrorAction SilentlyContinue | Where-Object { $_.MainWindowTitle -eq $title })) { Start-Sleep -Milliseconds 800; return }
        Start-Sleep -Milliseconds 300
    }
    Log "window still open after $seconds s: $title"
}
function Wait-Window($title, $seconds = 30) {
    $until = [DateTime]::Now.AddSeconds($seconds)
    while ([DateTime]::Now -lt $until) {
        if (Get-Process whirlpool -ErrorAction SilentlyContinue | Where-Object { $_.MainWindowTitle -eq $title }) { Start-Sleep -Milliseconds 1500; return $true }
        Start-Sleep -Milliseconds 300
    }
    Fail "window never appeared: $title"
    Diagnose $title
    return $false
}
function Scenario($name, [scriptblock]$body) {
    # -Only takes one or more comma-separated fragments (a scenario runs if its name contains any of them).
    if ($Only -and -not (@($Only -split ',' | Where-Object { $_ } | Where-Object { $name -like "*$_*" }).Count)) { return }
    Log "== $name"
    try { & $body } catch { Fail "scenario ${name} threw: $_" }
}

# The windows-11-arm image can sit on the Windows first-run page ("Choose privacy settings for your device") over the
# whole desktop. Whirlpool correctly hides under a full-screen window, so every frame was of that page and the
# driver's keys were typed into it. Close its host, and list what is on screen so the next failure explains itself.
function Clear-FirstRun {
    $host_ = @(Get-Process -ErrorAction SilentlyContinue | Where-Object { $_.ProcessName -match '^(CloudExperienceHost|CloudExperienceHostBroker|WWAHost|OOBE|oobe.*)$' })
    if ($host_.Count) { Log "first-run page host found ($(($host_ | ForEach-Object { $_.ProcessName }) -join ', ')); closing it"; $host_ | Stop-Process -Force -ErrorAction SilentlyContinue; Start-Sleep 2 }
    $wins = @(Get-Process -ErrorAction SilentlyContinue | Where-Object { $_.MainWindowTitle } | ForEach-Object { "$($_.ProcessName):'$($_.MainWindowTitle)'" })
    Log "windows on the desktop: $($wins -join ' | ')"
}
Clear-FirstRun
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
    & $Exe --quick-settings ($screen.Width - 12) 40 top
    [void](Wait-Window 'Whirlpool Quick Settings')
    Shot-Panel '04-quick-settings'
}

Scenario 'hud: volume, brightness, Wi-Fi and Bluetooth pills' {
    Start-Dock '{"menuBar":true,"menuBarPosition":"top"}'
    # The pill is small, so it needs fewer differing pixels than a panel to count as painted.
    & $Exe --hud volume 35; Start-Sleep -Milliseconds 800
    Shot-Panel '30-hud-volume-35' 0.008 { & $Exe --hud volume 35 }
    & $Exe --hud volume 85; Start-Sleep -Milliseconds 130
    Shot '30-hud-volume-glide'
    Start-Sleep -Milliseconds 700
    Shot-Panel '30-hud-volume-85' 0.008 { & $Exe --hud volume 85 }
    & $Exe --hud muted 85; Start-Sleep -Milliseconds 700
    Shot '30-hud-volume-muted'
    Start-Sleep -Milliseconds 2600
    Shot '30-hud-gone'
    & $Exe --hud brightness 60; Start-Sleep -Milliseconds 800
    Shot-Panel '30-hud-brightness' 0.008 { & $Exe --hud brightness 60 }
    Start-Sleep -Milliseconds 2600
    & $Exe --hud wifi 3; Start-Sleep -Milliseconds 800
    Shot-Panel '30-hud-wifi' 0.004 { & $Exe --hud wifi 3 }
    & $Exe --hud wifi 0; Start-Sleep -Milliseconds 700
    Shot '30-hud-wifi-off'
    & $Exe --hud bluetooth 1; Start-Sleep -Milliseconds 800
    Shot-Panel '30-hud-bluetooth' 0.004 { & $Exe --hud bluetooth 1 }
    & $Exe --hud bluetooth 0; Start-Sleep -Milliseconds 700
    Shot '30-hud-bluetooth-off'
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

Scenario 'launchpad: dock Start tile, hotkey, keys, search' {
    Start-Dock '{"menuBar":true,"menuBarPosition":"top"}'
    $startX = 0; $startY = $screen.Height - 41
    $wins = [U]::Of([uint32]$script:dock.Id)
    foreach ($w in $wins) { Log "dock process window: $w" }
    # The dock pill is the shortest window near the bottom edge (a taller sibling holds popups); its first tile is the Start tile.
    $best = $null
    foreach ($w in $wins) {
        $n = ($w.Split('|')[1]).Split(',') | ForEach-Object { [int]$_ }
        $width = $n[2] - $n[0]; $height = $n[3] - $n[1]
        if ($n[3] -ge ($screen.Height - 160) -and $height -lt 200 -and $width -lt ($screen.Width - 40) -and $width -gt 150) {
            if ($null -eq $best -or $height -lt $best.H) { $best = @{ L = $n[0]; T = $n[1]; R = $n[2]; B = $n[3]; W = $width; H = $height } }
        }
    }
    if ($best) {
        $startX = $best.L + 34; $startY = [int](($best.T + $best.B) / 2)
        Log "dock rect $($best.L),$($best.T) $($best.R),$($best.B)"
    } else { Log 'dock window not identified; clicking the default position' }
    Log "clicking the Start tile at $startX,$startY"
    Click $startX $startY
    $opened = Wait-Window 'Whirlpool Launchpad' 20
    Log "launchpad opened by the Start tile: $opened"
    Shot-Panel '06-launchpad-start-tile'
    Key ([byte[]](0x1B)); Start-Sleep 2
    # Hotkey: Ctrl+Alt+Space
    Key ([byte[]](0x11, 0x12, 0x20))
    [void](Wait-Window 'Whirlpool Launchpad')
    Shot-Panel '06-launchpad-hotkey'
    # Arrow keys on the full grid: the focus ring must leave the first tile (Right, Right, Down).
    Type-Keys ([byte[]](0x27, 0x27, 0x28))
    Start-Sleep 1
    Shot '06-launchpad-navigated'
    # Letters: C A L
    Type-Keys ([byte[]](0x43, 0x41, 0x4C))
    Start-Sleep 1
    Shot '06-launchpad-search'
    Key ([byte[]](0x1B))
    Start-Sleep 1
    Key ([byte[]](0x1B))
    Start-Sleep 1
    Log "launchpad closed by Escape: running=$([bool](Get-Process whirlpool -ErrorAction SilentlyContinue | Where-Object { $_.MainWindowTitle -eq 'Whirlpool Launchpad' }))"
    Wait-Gone 'Whirlpool Launchpad'
    if ($Sabotage -ne 'blank') { & $Exe --launchpad }
    [void](Wait-Window 'Whirlpool Launchpad')
    Shot-Panel '06-launchpad-cli'
    Key ([byte[]](0x1B))
}

foreach ($mode in @(@{ tag = ''; on = 'true' }, @{ tag = '-control'; on = 'false' })) {
Scenario "drag motion$($mode.tag): shadow, snap preview, settle" {
    # Server images drag a bare outline and leave the window where it is. Real desktops drag the whole
    # window, which is the case Whirlpool decorates, so turn it on and say what the system reports.
    [void][U]::SystemParametersInfo(0x0025, 1, [IntPtr]::Zero, 3)
    $full = 0; [void][U]::SystemParametersInfo(0x0026, 0, [ref]$full, 0)
    Log "full-window drag (SPI_GETDRAGFULLWINDOWS) = $full"
    Start-Dock "{`"menuBar`":true,`"menuBarPosition`":`"top`",`"dragMotion`":$($mode.on)}"
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
    Shot "07-drag-held$($mode.tag)"
    foreach ($i in 1..30) { Move-Mouse (($gx - 168) - $i * 12) ($gy + 108 + $i * 4); Start-Sleep -Milliseconds 20 }
    Move-Mouse 3 400; Start-Sleep -Milliseconds 700
    Shot "07-drag-snap-preview$($mode.tag)"
    [U]::mouse_event(0x0004, 0, 0, 0, [UIntPtr]::Zero)
    Start-Sleep -Milliseconds 120
    Shot "07-drag-release-120ms$($mode.tag)"
    Start-Sleep 1
    Shot "07-drag-settled$($mode.tag)"
    Stop-Process -Id $np.Id -Force -ErrorAction SilentlyContinue
}
}

function Wait-Window-Fast($title, $seconds = 20) {
    $until = [DateTime]::Now.AddSeconds($seconds)
    while ([DateTime]::Now -lt $until) {
        if (Get-Process whirlpool -ErrorAction SilentlyContinue | Where-Object { $_.MainWindowTitle -eq $title }) { return $true }
        Start-Sleep -Milliseconds 40
    }
    Fail "window never appeared: $title"
    return $false
}

# Session 7: the three directions of the Start grid and of quick settings, each settled and (for the
# first of each) caught while the tiles are still arriving. WHIRLPOOL_START_DESIGN and
# WHIRLPOOL_QUICK_DESIGN pick the direction; the chosen one is the default without them.
Scenario 'redesign directions: Start grid and quick settings' {
    Start-Dock '{"menuBar":true,"menuBarPosition":"top"}'
    foreach ($d in 'a', 'b', 'c') {
        $env:WHIRLPOOL_START_DESIGN = $d
        & $Exe --launchpad
        [void](Wait-Window-Fast 'Whirlpool Launchpad')
        Start-Sleep -Milliseconds 120
        if ($d -eq 'a') { Shot "20-start-$d-entering" }
        Start-Sleep -Milliseconds 2200
        Shot-Panel "20-start-$d"
        Type-Keys ([byte[]](0x27, 0x27, 0x28)); Start-Sleep -Milliseconds 700
        Shot "20-start-$d-selected"
        Type-Keys ([byte[]](0x53, 0x45, 0x54)); Start-Sleep -Milliseconds 900
        Shot "20-start-$d-search"
        Key ([byte[]](0x1B)); Start-Sleep 1; Key ([byte[]](0x1B)); Start-Sleep 1
        # Opening is a toggle and Wait-Window-Fast accepts the dying window: wait until it is really gone (the 175% x64 race).
        Wait-Gone 'Whirlpool Launchpad' 20
    }
    Remove-Item Env:WHIRLPOOL_START_DESIGN -ErrorAction SilentlyContinue
    foreach ($d in 'a', 'b', 'c') {
        $env:WHIRLPOOL_QUICK_DESIGN = $d
        & $Exe --quick-settings ($screen.Width - 12) 40 top
        [void](Wait-Window-Fast 'Whirlpool Quick Settings')
        Start-Sleep -Milliseconds 90
        if ($d -eq 'a') { Shot "21-quick-$d-entering" }
        Start-Sleep -Milliseconds 1800
        Shot-Panel "21-quick-$d"
        # Drag the volume to the middle and a bit further, then photograph the result.
        Move-Mouse ($screen.Width - 140) 330; Start-Sleep -Milliseconds 300
        Shot "21-quick-$d-hover"
        Key ([byte[]](0x1B)); Start-Sleep 1
        Wait-Gone 'Whirlpool Quick Settings' 20
    }
    Remove-Item Env:WHIRLPOOL_QUICK_DESIGN -ErrorAction SilentlyContinue
}

function Wheel($notches) { [U]::mouse_event(0x0800, 0, 0, [BitConverter]::ToUInt32([BitConverter]::GetBytes([int]($notches * -120)), 0), [UIntPtr]::Zero) }

Scenario 'search palette: apps, calculator, actions, clipboard' {
    Start-Dock '{"menuBar":true,"menuBarPosition":"top"}'
    Set-Clipboard -Value 'water-blue 4a90d9'
    Start-Sleep 1
    Set-Clipboard -Value 'meeting notes: ripple the tide'
    Start-Sleep 1
    Key ([byte[]](0x11, 0x10, 0x20))     # Ctrl+Shift+Space
    [void](Wait-Window 'Whirlpool Search')
    Shot-Panel '08-palette-open'
    Type-Keys ([byte[]](0x43, 0x41, 0x4C))   # c a l
    Start-Sleep 2
    Shot '08-palette-apps'
    Key ([byte[]](0x11, 0x41)); Type-Keys ([byte[]](0x08, 0x32, 0x6B, 0x33, 0x6A, 0x34))   # select all, delete, 2+3*4
    Start-Sleep 2
    Shot '08-palette-calc'
    Key ([byte[]](0x11, 0x41)); Type-Keys ([byte[]](0x08)); Key ([byte[]](0x10, 0xBE))     # clear, then ">"
    Start-Sleep 2
    Shot-Panel '08-palette-actions'
    Key ([byte[]](0x11, 0x41)); Type-Keys ([byte[]](0x08)); Key ([byte[]](0x10, 0x33))     # clear, then "#"
    Start-Sleep 2
    Shot '08-palette-clipboard'
    Key ([byte[]](0x1B))
}

Scenario 'window overview: live cards, arrow keys, Enter switches' {
    Start-Dock '{"menuBar":true,"menuBarPosition":"top"}'
    $apps = @()
    foreach ($i in 1..3) {
        $np = Start-Process notepad.exe -PassThru
        $apps += $np
        Start-Sleep 3
        Type-Keys ([byte[]](0x57, 0x41, 0x56, 0x45, 0x20, [byte](0x30 + $i)))   # "wave N" so each preview differs
    }
    $apps += Start-Process powershell.exe -ArgumentList '-NoExit', '-Command', '$Host.UI.RawUI.WindowTitle = ''Tide log''' -PassThru
    Start-Sleep 3
    if ($Sabotage -ne 'overview') { Start-Process $Exe -ArgumentList '--overview' }
    $up = Wait-Window 'Whirlpool Overview'
    Log "overview opened: $up"
    Start-Sleep 1
    Shot-Panel '15-overview-open'
    Key ([byte[]](0x27)); Start-Sleep -Milliseconds 600       # Right arrow
    Shot '15-overview-selected'
    $card = ($screen.Width / 2); Move-Mouse $card ($screen.Height / 2); Start-Sleep -Milliseconds 700
    Shot '15-overview-hover'
    $before = [U]::GetForegroundWindow()
    Key ([byte[]](0x0D)); Start-Sleep 2                         # Enter
    $after = [U]::GetForegroundWindow()
    $left = [bool](Get-Process whirlpool -ErrorAction SilentlyContinue | Where-Object { $_.MainWindowTitle -eq 'Whirlpool Overview' })
    Log "overview Enter: overview still open=$left; foreground changed=$($before -ne $after)"
    Shot '15-overview-after-enter'
    foreach ($a in $apps) { Stop-Process -Id $a.Id -Force -ErrorAction SilentlyContinue }
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
    Wait-Gone 'Whirlpool Notifications'
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

Scenario 'island pill: clock and focus timer' {
    # A focus session that is already running (the clock panel keeps it in focus.txt) and the clock moved into the island.
    $now = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
    Set-Content -Path (Join-Path $dir 'focus.txt') -Value "mode=focus`ndone=0`nrunning=1`nremaining_ms=1499000`nsaved_ms=$now`n"
    Start-Dock '{"menuBar":true,"menuBarPosition":"top","menuBarMusic":false,"aiUsage":false,"clockPlace":"island","focusIsland":true}'
    Move-Mouse ($screen.Width / 2) ($screen.Height / 2); Start-Sleep 2
    Shot '31-island-pill'
    Click ($screen.Width / 2) 12
    [void](Wait-Window 'Whirlpool Clock')
    Shot-Panel '31-hub-focus'
    Key ([byte[]](0x1B)); Start-Sleep 1
    foreach ($tab in 'clock', 'shelf') {
        KillAll
        $p = Start-Process $Exe -ArgumentList '--hub', $tab -PassThru
        [void](Wait-Window 'Whirlpool Clock')
        Start-Sleep 1
        Shot "31-hub-$tab"
        Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
    }
    Remove-Item (Join-Path $dir 'focus.txt') -ErrorAction SilentlyContinue
}

# The update reminder is one Windows notification. It has never been photographed: this asks Windows to show
# the made-up reminder (`--update --remind-test`) and checks that something appeared that was not in the
# dock-only frame. If a runner's notification platform shows nothing, this scenario fails and says so.
Scenario 'update toast: reminder notification' {
    Start-Dock '{"menuBar":true,"menuBarPosition":"top"}'
    & $Exe --update --remind-test | Out-Null
    Start-Sleep -Milliseconds 2500
    Shot-Panel '32-update-toast' 0.003 { & $Exe --update --remind-test | Out-Null }
}

Scenario 'settings previews (menubar)' {
    KillAll
    $env:WHIRLPOOL_PAGE = 'menubar'
    $p = Start-Process $Exe -ArgumentList '--settings' -PassThru
    Start-Sleep 10
    Move-Mouse 520 400
    # The page is long and every earlier pass shot only its last stretch (the same bottom frame seven times),
    # so the clock card was never seen. Shoot from the top, every second step, to the bottom.
    Shot '09-settings-menubar-scroll00'
    foreach ($n in 1..18) { Wheel 5; Start-Sleep -Milliseconds 400; if ($n % 2 -eq 0) { Shot ("09-settings-menubar-scroll{0:D2}" -f $n) } }
    Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
    Remove-Item Env:WHIRLPOOL_PAGE -ErrorAction SilentlyContinue
    Start-Sleep 1
}

# Gate: any missing window, blank panel frame or thrown scenario fails the job (artifacts still upload).
if ($script:Failures.Count -gt 0) {
    $script:Failures | Set-Content (Join-Path $Out 'FAILED.txt')
    Log "GATE: $($script:Failures.Count) failure(s); exiting 1"
    exit 1
}
Log 'GATE: every expected window appeared and every panel frame was painted'
