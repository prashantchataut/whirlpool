# Black-box study, round two. Runs one desktop program on a Windows runner with real input and
# measures what the screen, the process table and the accessibility tree show. It only observes:
# no file of the other program is read for code, text or art, nothing is decompiled.
#
#   -Target whirlpool   the build under test; frames and numbers are public
#   -Target reference   another program, installed from an installer given by -Installer; frames stay in
#                       a private artifact that lives one day, and logs never carry its names
param(
    [Parameter(Mandatory = $true)][ValidateSet('whirlpool', 'reference')][string]$Target,
    [Parameter(Mandatory = $true)][string]$Out,
    [string]$Exe = '',
    [string]$Installer = '',
    [string]$InstallArgs = '/S',
    [string]$SearchKeys = '12,20',      # virtual-key codes, hex, comma separated
    [string]$SearchText = '2+2*3'
)
$ErrorActionPreference = 'Continue'
New-Item -ItemType Directory -Force $Out | Out-Null
$logPath = Join-Path $Out 'log.txt'
function Log($m) {
    $line = "$([DateTime]::Now.ToString('HH:mm:ss.fff')) $m"
    Add-Content $logPath $line
    if ($Target -eq 'whirlpool') { Write-Host $line }
}
$num = [ordered]@{}   # the numbers that may be published
function Num($k, $v) { $num[$k] = $v; Log "num $k = $v" }

Add-Type -AssemblyName System.Windows.Forms, System.Drawing, UIAutomationClient, UIAutomationTypes
Add-Type -ReferencedAssemblies System.Drawing -TypeDefinition @"
using System; using System.Drawing; using System.Drawing.Imaging; using System.Runtime.InteropServices; using System.Diagnostics;
public static class S {
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, int dx, int dy, uint d, UIntPtr e);
  [DllImport("user32.dll")] public static extern void keybd_event(byte vk, byte scan, uint flags, UIntPtr e);
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  [DllImport("gdi32.dll")] static extern bool BitBlt(IntPtr dst, int x, int y, int w, int h, IntPtr src, int sx, int sy, int rop);
  [DllImport("user32.dll")] static extern IntPtr GetDC(IntPtr h);
  [DllImport("user32.dll")] static extern int ReleaseDC(IntPtr h, IntPtr dc);
  public static Bitmap Grab(int w, int h) {
    var bmp = new Bitmap(w, h); var g = Graphics.FromImage(bmp);
    IntPtr dst = g.GetHdc(); IntPtr src = GetDC(IntPtr.Zero);
    bool ok = BitBlt(dst, 0, 0, w, h, src, 0, 0, 0x40CC0020);
    ReleaseDC(IntPtr.Zero, src); g.ReleaseHdc(dst); g.Dispose();
    return bmp;
  }
  // 32x24 grid of mean brightness-ish samples, cheap enough to run every ~30 ms.
  public static double[] Grid(Bitmap b) {
    var r = new double[32 * 24]; var d = b.LockBits(new Rectangle(0, 0, b.Width, b.Height), ImageLockMode.ReadOnly, PixelFormat.Format32bppArgb);
    byte[] buf = new byte[d.Stride * b.Height]; Marshal.Copy(d.Scan0, buf, 0, buf.Length); b.UnlockBits(d);
    for (int gy = 0; gy < 24; gy++) for (int gx = 0; gx < 32; gx++) {
      double s = 0; int n = 0;
      for (int k = 0; k < 6; k++) { int x = (gx * b.Width / 32) + k * (b.Width / 192 + 1); int y = (gy * b.Height / 24) + k * (b.Height / 144 + 1);
        if (x >= b.Width || y >= b.Height) continue; int o = y * d.Stride + x * 4; s += buf[o] + buf[o + 1] + buf[o + 2]; n++; }
      r[gy * 32 + gx] = n == 0 ? 0 : s / n / 3;
    }
    return r;
  }
  public static double Dist(double[] a, double[] b) { double s = 0; for (int i = 0; i < a.Length; i++) s += Math.Abs(a[i] - b[i]); return s / a.Length / 255.0; }
  public static void Key(string codes) {
    var ks = codes.Split(','); var v = new System.Collections.Generic.List<byte>();
    foreach (var s in ks) v.Add(Convert.ToByte(s, 16));
    foreach (var k in v) { keybd_event(k, 0, 0, UIntPtr.Zero); System.Threading.Thread.Sleep(30); }
    v.Reverse(); foreach (var k in v) { keybd_event(k, 0, 2, UIntPtr.Zero); System.Threading.Thread.Sleep(30); }
  }
  public static void Click(int x, int y) { SetCursorPos(x, y); System.Threading.Thread.Sleep(60); mouse_event(2, 0, 0, 0, UIntPtr.Zero); mouse_event(4, 0, 0, 0, UIntPtr.Zero); }
}
"@
[void][S]::SetProcessDPIAware()
$scr = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
$W = $scr.Width; $H = $scr.Height
Log "screen ${W}x${H} arch=$env:PROCESSOR_ARCHITECTURE"

function Shot($name) { $b = [S]::Grab($W, $H); $b.Save((Join-Path $Out "$name.png")); $b.Dispose(); Log "shot $name" }
function Move($x, $y) { [void][S]::SetCursorPos($x, $y) }
function Neutral { Move ([int]($W / 2)) ([int]($H / 2)); Start-Sleep -Milliseconds 500 }

# Sample the screen every ~30 ms after an action: first visible change and settle time.
function Timed($scene, [scriptblock]$action, $maxMs = 6000) {
    Neutral; Start-Sleep -Milliseconds 600
    $b0 = [S]::Grab($W, $H); $g0 = [S]::Grid($b0); $b0.Dispose()
    $sw = [Diagnostics.Stopwatch]::StartNew(); & $action
    $first = -1; $last = [S]::Grid([S]::Grab(64, 48)); $lastChange = 0; $prev = $g0; $best = $null
    while ($sw.ElapsedMilliseconds -lt $maxMs) {
        $b = [S]::Grab($W, $H); $g = [S]::Grid($b); $t = $sw.ElapsedMilliseconds
        if ($first -lt 0 -and [S]::Dist($g0, $g) -gt 0.004) { $first = $t }
        if ([S]::Dist($prev, $g) -gt 0.0015) { $lastChange = $t }
        $prev = $g; $best = $b
        if ($first -ge 0 -and ($t - $lastChange) -gt 500) { break }
        if ($t -gt 400 -and $first -lt 0 -and $t -gt 2500) { break }
        if ($best -ne $b) { $b.Dispose() }
    }
    Shot "$scene" | Out-Null
    $row = "{0},{1},{2}" -f $scene, $first, $lastChange
    Add-Content (Join-Path $Out 'timings.csv') $row
    Log "timed $row"
}

# ---------- accessibility tree: the visible text of a window, depth limited ----------
function Dump-Ui($label, $procIds) {
    $f = Join-Path $Out "ui-$label.txt"; $lines = New-Object System.Collections.Generic.List[string]
    try {
        $root = [System.Windows.Automation.AutomationElement]::RootElement
        $wins = $root.FindAll([System.Windows.Automation.TreeScope]::Children, [System.Windows.Automation.Condition]::TrueCondition)
        foreach ($w in $wins) {
            if ($procIds -notcontains $w.Current.ProcessId) { continue }
            $lines.Add("WINDOW '$($w.Current.Name)' class=$($w.Current.ClassName) rect=$($w.Current.BoundingRectangle)")
            $walker = [System.Windows.Automation.TreeWalker]::ControlViewWalker
            $stack = New-Object System.Collections.Stack; $stack.Push(@($w, 0)); $count = 0
            while ($stack.Count -gt 0 -and $count -lt 1500) {
                $item = $stack.Pop(); $el = $item[0]; $d = $item[1]; $count++
                $n = $el.Current.Name; $t = $el.Current.ControlType.ProgrammaticName -replace 'ControlType\.', ''
                if ($n) { $lines.Add(("  " * $d) + "$t '$n'") }
                if ($d -lt 14) {
                    $kids = @(); $c = $walker.GetFirstChild($el)
                    while ($c -ne $null) { $kids += $c; $c = $walker.GetNextSibling($c) }
                    [array]::Reverse($kids); foreach ($k in $kids) { $stack.Push(@($k, ($d + 1))) }
                }
            }
        }
    } catch { $lines.Add("UIA error: $_") }
    $lines | Set-Content $f; Log "ui $label lines=$($lines.Count)"
}

# Click each named button/tab/list item once, to see what the settings window holds (reference only).
function Crawl-Ui($procIds, $max = 30) {
    $done = @{}
    try {
        $root = [System.Windows.Automation.AutomationElement]::RootElement
        for ($round = 0; $round -lt $max; $round++) {
            $target = $null
            $wins = $root.FindAll([System.Windows.Automation.TreeScope]::Children, [System.Windows.Automation.Condition]::TrueCondition)
            foreach ($w in $wins) {
                if ($procIds -notcontains $w.Current.ProcessId) { continue }
                if ($w.Current.BoundingRectangle.Width -lt 300) { continue }
                $cond = New-Object System.Windows.Automation.OrCondition(
                    (New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ControlTypeProperty, [System.Windows.Automation.ControlType]::Button)),
                    (New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ControlTypeProperty, [System.Windows.Automation.ControlType]::TabItem)),
                    (New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ControlTypeProperty, [System.Windows.Automation.ControlType]::ListItem)),
                    (New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ControlTypeProperty, [System.Windows.Automation.ControlType]::Hyperlink)))
                foreach ($e in $w.FindAll([System.Windows.Automation.TreeScope]::Descendants, $cond)) {
                    $n = $e.Current.Name
                    if (-not $n -or $done.ContainsKey($n) -or $n -match '(?i)close|quit|exit|uninstall|minimi|maximi|donate|paypal|update|restore') { continue }
                    $r = $e.Current.BoundingRectangle
                    if ($r.Width -lt 4 -or $r.Height -lt 4 -or $r.X -lt 0) { continue }
                    $target = @($n, $r); break
                }
                if ($target) { break }
            }
            if (-not $target) { break }
            $done[$target[0]] = $true; $r = $target[1]
            [S]::Click([int]($r.X + $r.Width / 2), [int]($r.Y + $r.Height / 2)); Start-Sleep -Milliseconds 900
            $safe = ($target[0] -replace '[^A-Za-z0-9]+', '_'); if ($safe.Length -gt 30) { $safe = $safe.Substring(0, 30) }
            Shot ("crawl-{0:D2}-{1}" -f $round, $safe)
            Dump-Ui ("crawl-{0:D2}" -f $round) $procIds
        }
    } catch { Log "crawl error: $_" }
    Log ("crawl clicked: " + ($done.Keys -join ' | '))
}

$ignore = 'pwsh', 'powershell', 'conhost', 'dllhost', 'WmiPrvE', 'WmiPrvSE', 'RuntimeBroker', 'backgroundTaskHost', 'SearchHost', 'ShellExperienceHost', 'TextInputHost', 'Runner.Worker', 'Runner.Listener', 'Runner.PluginHost', 'dotnet', 'ctfmon', 'smartscreen', 'msdtc', 'TrustedInstaller', 'TiWorker', 'SgrmBroker', 'audiodg', 'CompatTelRunner', 'MsMpEng', 'MpDefenderCoreService', 'NisSrv', 'wuauclt', 'UsoClient', 'WaaSMedic', 'spoolsv', 'SearchIndexer', 'SearchProtocolHost', 'SearchFilterHost', 'ApplicationFrameHost', 'StartMenuExperienceHost', 'LockApp', 'OpenConsole', 'WindowsTerminal', 'cmd', 'sppsvc', 'MoUsoCoreWorker', 'msiexec', 'taskhostw', 'sihost', 'svchost', 'MicrosoftEdgeUpdate', 'setup', 'Update', 'elevation_service', 'identity_helper', 'msedge'
$baseline = @{}; Get-Process | ForEach-Object { $baseline[$_.Id] = $true }
function Tracked { Get-Process | Where-Object { -not $baseline.ContainsKey($_.Id) -and ($ignore -notcontains $_.Name) } }

# ---------- install (reference only) ----------
if ($Target -eq 'reference' -and $Installer) {
    $item = Get-Item $Installer
    Num 'installer_bytes' $item.Length
    Num 'installer_mb' ([math]::Round($item.Length / 1MB, 2))
    $dirs = @($env:LOCALAPPDATA, $env:ProgramFiles, ${env:ProgramFiles(x86)}, $env:APPDATA, $env:ProgramData)
    function DirBytes($p) { if (Test-Path $p) { (Get-ChildItem $p -Recurse -File -Force -ErrorAction SilentlyContinue | Measure-Object Length -Sum).Sum } else { 0 } }
    $before = @{}; foreach ($d in $dirs) { Get-ChildItem $d -Directory -Force -ErrorAction SilentlyContinue | ForEach-Object { $before[$_.FullName] = $true } }
    $sw = [Diagnostics.Stopwatch]::StartNew()
    $p = Start-Process $Installer -ArgumentList $InstallArgs -PassThru -Wait
    Num 'install_seconds' ([math]::Round($sw.Elapsed.TotalSeconds, 1))
    $new = @(); foreach ($d in $dirs) { Get-ChildItem $d -Directory -Force -ErrorAction SilentlyContinue | Where-Object { -not $before.ContainsKey($_.FullName) } | ForEach-Object { $new += $_.FullName } }
    $total = 0; foreach ($n in $new) { $total += (DirBytes $n) }
    Num 'installed_new_dirs' $new.Count
    Num 'installed_mb' ([math]::Round($total / 1MB, 1))
    $found = $null
    foreach ($n in $new) { $e = Get-ChildItem $n -Recurse -Filter *.exe -ErrorAction SilentlyContinue | Where-Object { $_.Name -notmatch '(?i)unins|uninstall|setup|update|crashpad|webview' } | Sort-Object Length -Descending | Select-Object -First 1; if ($e) { $found = $e.FullName; break } }
    if ($found) { $Exe = $found; Log "installed exe found" } else { Log "no exe found after install; new dirs: $($new -join ';')" }
    Get-ChildItem ($new | Select-Object -First 3) -Recurse -File -ErrorAction SilentlyContinue | Select-Object -First 80 | ForEach-Object { "$($_.FullName) $($_.Length)" } | Set-Content (Join-Path $Out 'installed-files.txt')
}
if (-not $Exe -or -not (Test-Path $Exe)) { Log "no executable to run"; $num | ConvertTo-Json | Set-Content (Join-Path $Out 'numbers.json'); exit 1 }

# ---------- launch: first change and settle ----------
$sw = [Diagnostics.Stopwatch]::StartNew()
$b0 = [S]::Grab($W, $H); $g0 = [S]::Grid($b0); $b0.Dispose()
$proc = Start-Process $Exe -PassThru
$first = -1; $lastChange = 0; $prev = $g0
while ($sw.ElapsedMilliseconds -lt 25000) {
    $b = [S]::Grab($W, $H); $g = [S]::Grid($b); $b.Dispose(); $t = $sw.ElapsedMilliseconds
    if ($first -lt 0 -and [S]::Dist($g0, $g) -gt 0.004) { $first = $t }
    if ([S]::Dist($prev, $g) -gt 0.0015) { $lastChange = $t }
    $prev = $g
    if ($first -ge 0 -and ($t - $lastChange) -gt 1500) { break }
    Start-Sleep -Milliseconds 25
}
Num 'launch_first_change_ms' $first
Num 'launch_settled_ms' $lastChange
Start-Sleep 6
Shot '01-desktop'
$mine = @(Tracked); $ids = @($mine | ForEach-Object Id)
Log ("processes after launch: " + (($mine | ForEach-Object Name | Sort-Object -Unique) -join ' '))
Dump-Ui 'launch' $ids

# ---------- idle cost over 40 s ----------
function Cpu-Of($set) { ($set | ForEach-Object { try { $_.TotalProcessorTime.TotalSeconds } catch { 0 } } | Measure-Object -Sum).Sum }
$s0 = @(Tracked); $c0 = Cpu-Of $s0; $t0 = Get-Date
Start-Sleep 40
$s1 = @(Tracked); $c1 = Cpu-Of $s1; $secs = ((Get-Date) - $t0).TotalSeconds
Num 'idle_cpu_pct_of_one_core' ([math]::Round((($c1 - $c0) / $secs) * 100, 2))
Num 'idle_working_set_mb' ([math]::Round((($s1 | Measure-Object WorkingSet64 -Sum).Sum) / 1MB, 1))
Num 'idle_private_mb' ([math]::Round((($s1 | Measure-Object PrivateMemorySize64 -Sum).Sum) / 1MB, 1))
Num 'idle_processes' $s1.Count
Num 'idle_threads' (($s1 | ForEach-Object { $_.Threads.Count } | Measure-Object -Sum).Sum)

# ---------- scenes (same coordinates for both programs) ----------
$cx = [int]($W / 2)
Timed '02-dock-hover-edge' { Move $cx ($H - 3) }
Timed '03-dock-hover-icon' { Move ($cx - 40) ($H - 40) }
Timed '04-top-centre-hover' { Move $cx 2 }
Timed '05-top-centre-click' { [S]::Click($cx, 10) }
[S]::Key('1B'); Start-Sleep 800
Timed '06-search-hotkey' { [S]::Key($SearchKeys) }
foreach ($ch in $SearchText.ToCharArray()) {
    $vk = switch ($ch) { '+' { 'BB' } '*' { '6A' } '-' { 'BD' } '/' { '6F' } default { '{0:X2}' -f [int][char]([string]$ch).ToUpper() } }
    if ($ch -eq '+') { [S]::Key('10,BB') } elseif ($ch -eq '*') { [S]::Key('10,38') } else { [S]::Key($vk) }
    Start-Sleep -Milliseconds 120
}
Start-Sleep -Milliseconds 1500
Shot '07-search-calc'
[S]::Key('1B'); Start-Sleep 800
$mine = @(Tracked); $ids = @($mine | ForEach-Object Id)
Dump-Ui 'after-scenes' $ids

# ---------- settings window ----------
if ($Target -eq 'whirlpool') {
    Start-Process $Exe -ArgumentList '--settings' | Out-Null
} else {
    Start-Process $Exe | Out-Null     # a second start usually shows the settings of a tray program
}
Start-Sleep 6; Neutral
Shot '08-settings'
$mine = @(Tracked); $ids = @($mine | ForEach-Object Id)
Dump-Ui 'settings' $ids
if ($Target -eq 'reference') { Crawl-Ui $ids 30 }

# ---------- exit time ----------
$num | ConvertTo-Json | Set-Content (Join-Path $Out 'numbers.json')
Log 'done'
