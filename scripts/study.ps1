# Black-box study driver. Runs one desktop program on a Windows runner with real mouse and keyboard
# input and measures what the screen and the process table show. It only observes: it never reads
# the program's files for code, text or art, and it publishes numbers, not the other program's frames.
#
#   -Target whirlpool   the build under test (frames go to the public artifact)
#   -Target reference   another shell, supplied through repository variables (frames stay private)
param(
    [Parameter(Mandatory = $true)][ValidateSet('whirlpool', 'reference')][string]$Target,
    [Parameter(Mandatory = $true)][string]$Exe,
    [Parameter(Mandatory = $true)][string]$Out,
    [string]$QuitKeys = '',
    [string]$QuitArgs = '--quit'
)
$ErrorActionPreference = 'Continue'
New-Item -ItemType Directory -Force $Out | Out-Null
$logPath = Join-Path $Out 'log.txt'
$csv = Join-Path $Out 'results.csv'
'target,scene,variant,change,latency_ms,duration_ms,f25,f50,f75,frames,fps' | Set-Content $csv
function Log($m) {
    $line = "$([DateTime]::Now.ToString('HH:mm:ss.fff')) $m"
    Add-Content $logPath $line
    if ($Target -eq 'whirlpool') { Write-Host $line }   # the other program's names never reach public logs
}

Add-Type -AssemblyName System.Windows.Forms, System.Drawing
if ($PSVersionTable.PSEdition -eq 'Core') {
    # PowerShell 7: every managed assembly of the runtime, so the GDI+ implementation types resolve.
    $dirs = @([IO.Path]::GetDirectoryName([object].Assembly.Location), [IO.Path]::GetDirectoryName([System.Drawing.Bitmap].Assembly.Location)) | Select-Object -Unique
    $refs = foreach ($d in $dirs) {
        Get-ChildItem $d -Filter *.dll | Where-Object {
            try { [void][Reflection.AssemblyName]::GetAssemblyName($_.FullName); $_.Name -notmatch '^(Microsoft\.|System\.Management\.Automation)' } catch { $false }
        } | ForEach-Object FullName
    }
} else {
    $refs = @('System.Drawing')
}
try {
Add-Type -ReferencedAssemblies $refs -TypeDefinition @"
using System; using System.Collections.Generic; using System.Diagnostics; using System.Drawing;
using System.Drawing.Imaging; using System.Globalization; using System.Runtime.InteropServices; using System.Threading;
public static class Cap {
  [DllImport("user32.dll")] static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] static extern void mouse_event(uint f, int dx, int dy, uint d, UIntPtr e);
  [DllImport("user32.dll")] static extern void keybd_event(byte vk, byte scan, uint flags, UIntPtr e);
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  public static List<double> Times = new List<double>();
  public static List<double[]> Grids = new List<double[]>();
  public static List<Bitmap> Frames = new List<Bitmap>();
  public static double TriggerAt = 0;
  public static void Clear() { foreach (var f in Frames) f.Dispose(); Frames.Clear(); Grids.Clear(); Times.Clear(); TriggerAt = 0; }
  public static void Exec(string mode) {
    if (string.IsNullOrEmpty(mode)) return;
    int i = mode.IndexOf(':'); string kind = mode.Substring(0, i); string arg = mode.Substring(i + 1);
    if (kind == "key") {
      var keys = new List<byte>(); foreach (var s in arg.Split(',')) keys.Add(byte.Parse(s, NumberStyles.HexNumber));
      foreach (var k in keys) { keybd_event(k, 0, 0, UIntPtr.Zero); Thread.Sleep(30); }
      keys.Reverse();
      foreach (var k in keys) { keybd_event(k, 0, 2, UIntPtr.Zero); Thread.Sleep(30); }
    } else if (kind == "move") {
      var p = arg.Split(','); SetCursorPos(int.Parse(p[0]), int.Parse(p[1]));
    } else if (kind == "click") {
      var p = arg.Split(','); SetCursorPos(int.Parse(p[0]), int.Parse(p[1])); Thread.Sleep(60);
      mouse_event(0x0002, 0, 0, 0, UIntPtr.Zero); Thread.Sleep(50); mouse_event(0x0004, 0, 0, 0, UIntPtr.Zero);
    } else if (kind == "launch") {
      var psi = new ProcessStartInfo(arg); psi.WorkingDirectory = System.IO.Path.GetDirectoryName(arg); psi.UseShellExecute = false; Process.Start(psi);
    } else if (kind == "run") {
      var parts = arg.Split('|'); var psi = new ProcessStartInfo(parts[0], parts.Length > 1 ? parts[1] : ""); psi.UseShellExecute = false; psi.CreateNoWindow = true; Process.Start(psi);
    }
  }
  static double[] Grid(Bitmap b) {
    int gw = 128, gh = 96; var r = new double[gw * gh];
    var d = b.LockBits(new Rectangle(0, 0, b.Width, b.Height), ImageLockMode.ReadOnly, PixelFormat.Format32bppArgb);
    int stride = d.Stride; byte[] buf = new byte[stride * b.Height];
    Marshal.Copy(d.Scan0, buf, 0, buf.Length); b.UnlockBits(d);
    for (int gy = 0; gy < gh; gy++) for (int gx = 0; gx < gw; gx++) {
      int x = gx * b.Width / gw, y = gy * b.Height / gh; int k = y * stride + x * 4;
      r[gy * gw + gx] = 0.114 * buf[k] + 0.587 * buf[k + 1] + 0.299 * buf[k + 2];
    }
    return r;
  }
  static double Dist(double[] a, double[] b) { double s = 0; for (int i = 0; i < a.Length; i++) s += Math.Abs(a[i] - b[i]); return s / a.Length; }
  // Captures frames for `ms`; fires `mode` right after the first frame. intervalMs > 0 slows capture for long scenes.
  public static void Burst(int ms, int w, int h, string mode, int intervalMs) {
    Clear(); var sw = Stopwatch.StartNew(); bool fired = false; double next = 0;
    while (sw.Elapsed.TotalMilliseconds < ms && Frames.Count < 400) {
      if (intervalMs > 0 && sw.Elapsed.TotalMilliseconds < next) { Thread.Sleep(5); continue; }
      next = sw.Elapsed.TotalMilliseconds + intervalMs;
      var bmp = new Bitmap(w, h, PixelFormat.Format32bppArgb);
      using (var g = Graphics.FromImage(bmp)) g.CopyFromScreen(0, 0, 0, 0, new Size(w, h));
      Times.Add(sw.Elapsed.TotalMilliseconds); Frames.Add(bmp); Grids.Add(Grid(bmp));
      if (!fired) { fired = true; Exec(mode); TriggerAt = sw.Elapsed.TotalMilliseconds; }
    }
  }
  // CSV fields: change,latency,duration,f25,f50,f75,frames,fps. Progress p = 1 - dist(frame, final) / dist(first, final).
  public static string Analyze(string pngBase) {
    int n = Frames.Count; if (n < 3) return "0,,,,,,0,0";
    var fin = Grids[n - 1]; double d0 = Dist(Grids[0], fin);
    double fps = (n - 1) / ((Times[n - 1] - Times[0]) / 1000.0);
    Frames[n - 1].Save(pngBase + "-end.png", ImageFormat.Png);
    Frames[0].Save(pngBase + "-start.png", ImageFormat.Png);
    string head = d0.ToString("F2", CultureInfo.InvariantCulture);
    if (d0 < 0.8) return head + ",,,,,," + n + "," + fps.ToString("F1", CultureInfo.InvariantCulture);
    var p = new double[n]; for (int i = 0; i < n; i++) p[i] = 1 - Dist(Grids[i], fin) / d0;
    int s = -1; for (int i = 0; i < n; i++) if (p[i] >= 0.05) { s = i; break; }
    int e = n - 1; for (int i = n - 1; i >= 0; i--) { if (p[i] < 0.97) { e = Math.Min(i + 1, n - 1); break; } if (i == 0) e = 0; }
    if (s < 0) s = 0; if (e < s) e = s;
    double ts = Times[s], te = Times[e], dur = Math.Max(te - ts, 1);
    Func<double, string> frac = delegate(double level) {
      for (int i = s; i <= e; i++) if (p[i] >= level) return ((Times[i] - ts) / dur).ToString("F2", CultureInfo.InvariantCulture);
      return "";
    };
    int mid = s; for (int i = s; i <= e; i++) if (p[i] >= 0.5) { mid = i; break; }
    Frames[mid].Save(pngBase + "-mid.png", ImageFormat.Png);
    var ci = CultureInfo.InvariantCulture;
    return head + "," + (ts - TriggerAt).ToString("F0", ci) + "," + dur.ToString("F0", ci) + "," + frac(0.25) + "," + frac(0.5) + "," + frac(0.75) + "," + n + "," + fps.ToString("F1", ci);
  }
}
"@
} catch { Log "COMPILE FAILED: $($_.Exception.Message)"; foreach ($e in $Error) { Log ("  " + $e.ToString()) }; Log ("refs: " + ($refs -join ' ')); exit 2 }
[void][Cap]::SetProcessDPIAware()
$screen = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
$ScrW = $screen.Width; $ScrH = $screen.Height
Log "screen ${ScrW}x${ScrH} arch=$env:PROCESSOR_ARCHITECTURE cores=$env:NUMBER_OF_PROCESSORS"

function Neutral { [Cap]::Exec("move:$([int]($ScrW/2)),$([int]($ScrH/2))"); Start-Sleep -Milliseconds 400 }
function Scene($scene, $variant, $mode, $ms, $interval = 0, $cleanup = $null) {
    $base = Join-Path $Out ("{0}-{1}" -f $scene, $variant)
    try {
        Neutral; Start-Sleep -Milliseconds 700
        [Cap]::Burst($ms, $ScrW, $ScrH, $mode, $interval)
        $row = [Cap]::Analyze($base)
        Add-Content $csv "$Target,$scene,$variant,$row"
        Log "scene $scene/$variant -> $row"
    } catch { Log "FAILED $scene/${variant}: $_" }
    if ($cleanup) { [Cap]::Exec($cleanup); Start-Sleep -Milliseconds 500 }
    [Cap]::Exec('key:1B'); Start-Sleep -Milliseconds 600   # Escape closes whatever opened
}

$ignore = 'pwsh', 'powershell', 'conhost', 'dllhost', 'WmiPrvSE', 'RuntimeBroker', 'backgroundTaskHost', 'SearchHost', 'ShellExperienceHost', 'TextInputHost', 'notepad', 'Runner.Worker', 'Runner.Listener', 'Runner.PluginHost', 'dotnet', 'ctfmon', 'smartscreen', 'msdtc', 'TrustedInstaller', 'TiWorker', 'SgrmBroker', 'audiodg', 'CompatTelRunner', 'MsMpEng', 'MpDefenderCoreService', 'NisSrv', 'wuauclt', 'UsoClient', 'WaaSMedic', 'spoolsv', 'SearchIndexer', 'SearchProtocolHost', 'SearchFilterHost', 'ApplicationFrameHost', 'StartMenuExperienceHost', 'LockApp', 'OpenConsole', 'WindowsTerminal', 'cmd', 'sppsvc', 'MoUsoCoreWorker', 'msiexec', 'taskhostw', 'sihost', 'svchost'
$baseline = @{}; Get-Process | ForEach-Object { $baseline[$_.Id] = $true }
function Tracked { Get-Process | Where-Object { -not $baseline.ContainsKey($_.Id) -and ($ignore -notcontains $_.Name) } }
function Idle-Sample($seconds) {
    $before = @{}; foreach ($p in Tracked) { try { $before[$p.Id] = $p.TotalProcessorTime.TotalSeconds } catch {} }
    Start-Sleep $seconds
    $ws = 0; $priv = 0; $cpu = 0; $names = @()
    foreach ($p in Tracked) {
        try {
            $p.Refresh(); $ws += $p.WorkingSet64; $priv += $p.PrivateMemorySize64; $names += $p.Name
            if ($before.ContainsKey($p.Id)) { $cpu += $p.TotalProcessorTime.TotalSeconds - $before[$p.Id] }
        } catch {}
    }
    $ci = [Globalization.CultureInfo]::InvariantCulture
    return @{ ws = ($ws / 1MB).ToString('F1', $ci); priv = ($priv / 1MB).ToString('F1', $ci); cpu = ($cpu / $seconds * 100).ToString('F2', $ci); procs = $names.Count; names = ($names -join ' ') }
}

(New-Object -ComObject Shell.Application).MinimizeAll(); Start-Sleep 2
foreach ($title in 'System Properties') { $dlg = (Get-Process | Where-Object { $_.MainWindowTitle -eq $title } | Select-Object -First 1); if ($dlg) { [void]$dlg.CloseMainWindow() } }
Start-Sleep 1
if (Test-Path "$env:APPDATA\Whirlpool\config.json") { Remove-Item "$env:APPDATA\Whirlpool\config.json" -Force }

# ---------- startup: frames from the launch until the screen stops changing ----------
$startedAt = Get-Date
Scene 'startup' 'launch' "launch:$Exe" 22000 250
$alive = @(Tracked).Count
Log "after startup: $alive tracked processes; names: $((Tracked | ForEach-Object Name | Sort-Object -Unique) -join ' ')"
$targetNames = @(Tracked | ForEach-Object Name | Sort-Object -Unique)
function Own { Get-Process | Where-Object { $targetNames -contains $_.Name -and -not $baseline.ContainsKey($_.Id) } }
Start-Sleep 5

# ---------- idle cost ----------
$idle = Idle-Sample 40
Add-Content (Join-Path $Out 'idle.csv') "target,ws_mb,private_mb,cpu_pct_of_one_core,processes`n$Target,$($idle.ws),$($idle.priv),$($idle.cpu),$($idle.procs)"
Log "idle: ws=$($idle.ws) MB private=$($idle.priv) MB cpu=$($idle.cpu)% procs=$($idle.procs) [$($idle.names)]"

# ---------- scenes ----------
$bottom = $ScrH - 24
if ($Target -eq 'whirlpool') {
    Scene 'dock-hover' 'icon' "move:470,$bottom" 1200
    foreach ($y in 600, 650, 700) { [Cap]::Exec("move:470,$y"); Start-Sleep -Milliseconds 40 }
    Scene 'dock-hover' 'edge-sweep' "move:420,$bottom" 1200
    Scene 'search' 'bar-icon' 'click:819,14' 1500
    Scene 'quick-settings' 'bar-icon' 'click:853,14' 2500
    Scene 'notifications' 'bar-clock' 'click:930,14' 2500
    Scene 'start-grid' 'hotkey' 'key:11,12,20' 6000 0 'key:11,12,20'
    Scene 'window-overview' 'bar-icon' 'click:717,14' 2000
    foreach ($c in @(@('tl', 0, 0), @('tr', ($ScrW - 1), 0), @('bl', 0, ($ScrH - 1)), @('br', ($ScrW - 1), ($ScrH - 1)))) {
        Scene 'hot-corner' $c[0] "move:$($c[1]),$($c[2])" 1500
    }
} else {
    $cx = [int]($ScrW / 2)
    Scene 'dock-hover' 'bottom-a' "move:$cx,$bottom" 1200
    Scene 'dock-hover' 'bottom-b' "move:$($cx - 60),$($bottom - 8)" 1200
    foreach ($k in @(@('alt-space', '12,20'), @('ctrl-space', '11,20'), @('win-space', '5B,20'), @('win-s', '5B,53'), @('ctrl-alt-space', '11,12,20'))) {
        Scene 'search' $k[0] "key:$($k[1])" 1800
    }
    foreach ($k in @(@('ctrl-up', '11,26'), @('f3', '72'), @('win-tab', '5B,09'), @('ctrl-alt-up', '11,12,26'), @('win-up', '5B,26'))) {
        Scene 'overview' $k[0] "key:$($k[1])" 2000
    }
    foreach ($k in @(@('f4', '73'), @('ctrl-alt-l', '11,12,4C'), @('win-a', '5B,41'))) {
        Scene 'start-grid' $k[0] "key:$($k[1])" 2000
    }
    foreach ($dx in 40, 80, 120, 160, 200, 250) {
        Scene 'bar-icon' "x-$dx" "click:$($ScrW - $dx),12" 2000
    }
    foreach ($c in @(@('tl', 0, 0), @('tr', ($ScrW - 1), 0), @('bl', 0, ($ScrH - 1)), @('br', ($ScrW - 1), ($ScrH - 1)))) {
        Scene 'hot-corner' $c[0] "move:$($c[1]),$($c[2])" 1500
    }
}


# ---------- memory while a window is open, and after it closes ----------
function MemNow($label) {
    $mine = @(Own); $ws = ($mine | Measure-Object WorkingSet64 -Sum).Sum; $pv = ($mine | Measure-Object PrivateMemorySize64 -Sum).Sum
    Log ("mem {0}: processes={1} ws={2:F1} MB private={3:F1} MB" -f $label, $mine.Count, ($ws / 1MB), ($pv / 1MB))
    Add-Content (Join-Path $Out 'memory.csv') ("{0},{1},{2},{3:F1},{4:F1}" -f $Target, $label, $mine.Count, ($ws / 1MB), ($pv / 1MB))
}
'target,state,processes,ws_mb,private_mb' | Set-Content (Join-Path $Out 'memory.csv')
MemNow 'before-open'
if ($Target -eq 'whirlpool') {
    foreach ($w in @(@('start-grid', 'key:11,12,20'), @('palette', 'key:11,10,20'), @('quick-settings', 'click:853,14'))) {
        [Cap]::Exec($w[1]); Start-Sleep 5; MemNow "$($w[0])-open"
        [Cap]::Exec('key:1B'); Start-Sleep 3; [Cap]::Exec('move:512,400'); Start-Sleep 4; MemNow "$($w[0])-closed"
    }
}

# ---------- exit ----------
$sw = [Diagnostics.Stopwatch]::StartNew()
if ($QuitKeys) { [Cap]::Exec("key:$QuitKeys") } else { [Cap]::Exec("run:$Exe|$QuitArgs") }
while (@(Own).Count -gt 0 -and $sw.ElapsedMilliseconds -lt 20000) { Start-Sleep -Milliseconds 50 }
$exitMs = $sw.ElapsedMilliseconds
Log "exit: $exitMs ms until the program's own processes ended; remaining=$(@(Own).Count)"
Add-Content (Join-Path $Out 'exit.csv') "target,exit_ms,remaining`n$Target,$exitMs,$(@(Own).Count)"
Start-Sleep 3
[Cap]::Burst(300, $ScrW, $ScrH, '', 0); [void][Cap]::Analyze((Join-Path $Out 'after-exit'))
Get-Process | Where-Object { -not $baseline.ContainsKey($_.Id) -and ($ignore -notcontains $_.Name) } | Stop-Process -Force -ErrorAction SilentlyContinue
if (-not (Get-Process explorer -ErrorAction SilentlyContinue)) { Start-Process explorer.exe; Log 'explorer was not running after exit; started it' }
Log 'done'
