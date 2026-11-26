# Black-box study, round three. Launches one desktop program on a Windows runner and records what the
# screen and the process table show: idle cost, processes it starts, what stays alive after its panel is
# dismissed, and frames for a short list of key presses. It only observes: no file of the program under
# study is read for code, text or art.
#
#   -Target whirlpool   the build under test; frames and numbers are public
#   -Target reference   another program; frames stay in a private artifact that lives one day
param(
    [Parameter(Mandatory = $true)][ValidateSet('whirlpool', 'reference')][string]$Target,
    [Parameter(Mandatory = $true)][string]$Exe,
    [Parameter(Mandatory = $true)][string]$Out,
    [string]$Keys = 'ctrl+k;ctrl+alt+a;ctrl+alt+space',
    [string]$ExeArgs = '',
    [string]$OpenExe = '',
    [string]$OpenArgs = ''
)
$ErrorActionPreference = 'Continue'
New-Item -ItemType Directory -Force $Out | Out-Null
Add-Type -AssemblyName System.Windows.Forms, System.Drawing
Add-Type -ReferencedAssemblies System.Drawing -TypeDefinition @"
using System; using System.Drawing; using System.Runtime.InteropServices;
public static class S3 {
  [DllImport("user32.dll")] public static extern void keybd_event(byte vk, byte scan, uint flags, UIntPtr e);
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  [DllImport("gdi32.dll")] static extern bool BitBlt(IntPtr dst, int x, int y, int w, int h, IntPtr src, int sx, int sy, int rop);
  [DllImport("user32.dll")] static extern IntPtr GetDC(IntPtr h);
  [DllImport("user32.dll")] static extern int ReleaseDC(IntPtr h, IntPtr dc);
  public static void Grab(string path, int w, int h) {
    var bmp = new Bitmap(w, h); var g = Graphics.FromImage(bmp); IntPtr dst = g.GetHdc(); IntPtr src = GetDC(IntPtr.Zero);
    BitBlt(dst, 0, 0, w, h, src, 0, 0, 0x40CC0020); ReleaseDC(IntPtr.Zero, src); g.ReleaseHdc(dst); g.Dispose();
    bmp.Save(path, System.Drawing.Imaging.ImageFormat.Png); bmp.Dispose();
  }
}
"@
[S3]::SetProcessDPIAware() | Out-Null
$w = [System.Windows.Forms.SystemInformation]::PrimaryMonitorSize.Width
$h = [System.Windows.Forms.SystemInformation]::PrimaryMonitorSize.Height
$num = [ordered]@{ target = $Target; os = [Environment]::OSVersion.Version.ToString() }
$logPath = Join-Path $Out 'log.txt'
function Log($m) { $l = "$([DateTime]::Now.ToString('HH:mm:ss')) $m"; Add-Content $logPath $l; if ($Target -eq 'whirlpool') { Write-Host $l } }
function Shot($name) { [S3]::Grab((Join-Path $Out "$name.png"), $w, $h); Log "frame $name" }
function Press($combo) {
    $vk = @{ ctrl = 0x11; alt = 0x12; shift = 0x10; win = 0x5B; space = 0x20; k = 0x4B; a = 0x41; escape = 0x1B }
    $parts = $combo.Split('+'); $codes = $parts | ForEach-Object { [byte]$vk[$_] }
    foreach ($c in $codes) { [S3]::keybd_event($c, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 40 }
    [array]::Reverse($codes)
    foreach ($c in $codes) { [S3]::keybd_event($c, 0, 2, [UIntPtr]::Zero); Start-Sleep -Milliseconds 40 }
}
function Snapshot { Get-Process | Where-Object { $_.Id -ne $PID } | Select-Object Id, ProcessName, @{n='priv';e={$_.PrivateMemorySize64}}, @{n='ws';e={$_.WorkingSet64}}, CPU, MainWindowTitle }

Shot '00-desktop'
$skip = 'powershell|conhost|pwsh|MsMpEng|svchost|WmiPrvSE|dllhost|backgroundTask|RuntimeBroker|SearchHost|TextInputHost|ShellExperienceHost|StartMenuExperienceHost|vctip|MSBuild|VBCSCompiler|git|cmd|Runner|Worker'
$elapsed = 0
$before = @{}; Snapshot | ForEach-Object { $before[$_.Id] = $_ }
$sw = [Diagnostics.Stopwatch]::StartNew()
$startArgs = @{ FilePath = $Exe; PassThru = $true; WorkingDirectory = (Split-Path $Exe) }
if ($ExeArgs) { $startArgs.ArgumentList = $ExeArgs }
$proc = Start-Process @startArgs
Start-Sleep -Seconds 20
Shot '01-launched'
$mine = @(Snapshot | Where-Object { -not $before.ContainsKey($_.Id) -and $_.ProcessName -notmatch $skip })

$num.processes_after_launch = $mine.Count
$num.private_mb_total_idle = [math]::Round((($mine | Measure-Object priv -Sum).Sum) / 1MB, 1)
$num.working_set_mb_total_idle = [math]::Round((($mine | Measure-Object ws -Sum).Sum) / 1MB, 1)
Log "processes: $(($mine | ForEach-Object { '{0}({1:N0}MB)' -f $(if ($Target -eq 'whirlpool') { $_.ProcessName } else { 'p' + $_.Id }), ($_.priv / 1MB) }) -join ', ')"
# idle CPU over 10 s
$ids = $mine.Id; $c0 = ($ids | ForEach-Object { (Get-Process -Id $_ -ErrorAction SilentlyContinue).CPU } | Measure-Object -Sum).Sum
Start-Sleep -Seconds 10
$c1 = ($ids | ForEach-Object { (Get-Process -Id $_ -ErrorAction SilentlyContinue).CPU } | Measure-Object -Sum).Sum
$num.idle_cpu_percent_one_core = [math]::Round((($c1 - $c0) / 10) * 100, 2)
if ($OpenExe) {
    # A second window of the program (its own panel) opened by hand, as its documentation describes.
    $sa = @{ FilePath = $OpenExe; PassThru = $true; WorkingDirectory = (Split-Path $OpenExe) }
    if ($OpenArgs) { $sa.ArgumentList = $OpenArgs }
    $open = Start-Process @sa
    Start-Sleep -Seconds 12
    Shot '05-panel-opened'
    $both = @(Snapshot | Where-Object { -not $before.ContainsKey($_.Id) -and $_.ProcessName -notmatch $skip })
    $num.panel_open_processes = $both.Count
    $num.panel_open_private_mb_total = [math]::Round((($both | Measure-Object priv -Sum).Sum) / 1MB, 1)
    $num.panel_open_working_set_mb_total = [math]::Round((($both | Measure-Object ws -Sum).Sum) / 1MB, 1)
    Log "panel open processes: $(($both | ForEach-Object { '{0:N0}MB' -f ($_.priv / 1MB) }) -join ', ')"
    Press 'escape'; Start-Sleep -Seconds 1
    foreach ($t in 5, 20, 20) {
        Start-Sleep -Seconds $t
        $now = @(Snapshot | Where-Object { -not $before.ContainsKey($_.Id) -and $_.ProcessName -notmatch $skip })
        $elapsed += $t
        $num["after_hide_${elapsed}s_processes"] = $now.Count
        $num["after_hide_${elapsed}s_private_mb"] = [math]::Round((($now | Measure-Object priv -Sum).Sum) / 1MB, 1)
    }
}
$i = 2
foreach ($k in $Keys.Split(';')) {
    Log "press $k"
    Press $k; Start-Sleep -Seconds 4
    Shot ('{0:D2}-{1}' -f $i, ($k -replace '\+', '-'))
    $open = @(Snapshot | Where-Object { -not $before.ContainsKey($_.Id) -and $_.ProcessName -notmatch $skip })
    $num["private_mb_total_after_$($k -replace '\+','_')"] = [math]::Round((($open | Measure-Object priv -Sum).Sum) / 1MB, 1)
    $num["processes_after_$($k -replace '\+','_')"] = $open.Count
    Press 'escape'; Start-Sleep -Seconds 1
    $i++
}
# does anything heavy go away after the panels are dismissed? sample at 5 s and 40 s
Start-Sleep -Seconds 5
$s5 = @(Snapshot | Where-Object { -not $before.ContainsKey($_.Id) -and $_.ProcessName -notmatch $skip })
Start-Sleep -Seconds 40
$s40 = @(Snapshot | Where-Object { -not $before.ContainsKey($_.Id) -and $_.ProcessName -notmatch $skip })
$num.processes_5s_after_dismiss = $s5.Count
$num.processes_45s_after_dismiss = $s40.Count
$num.private_mb_5s_after_dismiss = [math]::Round((($s5 | Measure-Object priv -Sum).Sum) / 1MB, 1)
$num.private_mb_45s_after_dismiss = [math]::Round((($s40 | Measure-Object priv -Sum).Sum) / 1MB, 1)
Shot '09-end'
$num | ConvertTo-Json | Set-Content (Join-Path $Out 'numbers.json')
Write-Host ($num | ConvertTo-Json)
Snapshot | Where-Object { -not $before.ContainsKey($_.Id) -and $_.ProcessName -notmatch $skip } | ForEach-Object { try { Stop-Process -Id $_.Id -Force } catch {} }
exit 0
