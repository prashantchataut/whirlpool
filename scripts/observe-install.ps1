# Runs a program's own install and uninstall commands on a throw-away runner and records only what
# changed on the machine: folders, files, bytes, startup entries, scheduled tasks, services, time.
# It does not read the scripts or the program. Prompts are answered "no" through redirected input.
param(
    [Parameter(Mandatory = $true)][string]$PackageDir,
    [Parameter(Mandatory = $true)][string]$InstallCmd,
    [Parameter(Mandatory = $true)][string]$UninstallCmd,
    [Parameter(Mandatory = $true)][string]$Out
)
$ErrorActionPreference = 'Continue'
New-Item -ItemType Directory -Force $Out | Out-Null
$rep = Join-Path $Out 'install-observed.txt'
function Say($m) { Add-Content $rep $m }
$answers = Join-Path $env:RUNNER_TEMP 'no.txt'
(1..40 | ForEach-Object { 'n' }) | Set-Content $answers

function Snap {
    $dirs = foreach ($r in $env:LOCALAPPDATA, "$env:LOCALAPPDATA\Programs", $env:APPDATA, $env:ProgramData, $env:ProgramFiles, "$env:APPDATA\Microsoft\Windows\Start Menu\Programs") {
        Get-ChildItem $r -Directory -Force -ErrorAction SilentlyContinue | ForEach-Object FullName
    }
    $run = (Get-ItemProperty 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run' -ErrorAction SilentlyContinue).PSObject.Properties | Where-Object { $_.Name -notmatch '^PS' } | ForEach-Object { "$($_.Name)=$($_.Value)" }
    $soft = Get-ChildItem 'HKCU:\Software' -ErrorAction SilentlyContinue | ForEach-Object PSChildName
    $uninst = Get-ChildItem 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall', 'HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall' -ErrorAction SilentlyContinue | ForEach-Object PSChildName
    $tasks = Get-ScheduledTask -ErrorAction SilentlyContinue | ForEach-Object { "$($_.TaskPath)$($_.TaskName)" }
    $svc = Get-Service -ErrorAction SilentlyContinue | ForEach-Object Name
    $procs = Get-Process | ForEach-Object { "$($_.Id):$($_.Name)" }
    $shell = (Get-ItemProperty 'HKLM:\Software\Microsoft\Windows NT\CurrentVersion\Winlogon' -ErrorAction SilentlyContinue).Shell
    @{ dirs = @($dirs); run = @($run); soft = @($soft); uninst = @($uninst); tasks = @($tasks); svc = @($svc); procs = @($procs); shell = $shell }
}
function Diff($a, $b) {
    foreach ($k in 'dirs', 'run', 'soft', 'uninst', 'tasks', 'svc') {
        $added = @($b[$k] | Where-Object { $a[$k] -notcontains $_ })
        $removed = @($a[$k] | Where-Object { $b[$k] -notcontains $_ })
        Say "  ${k}: +$($added.Count) -$($removed.Count)"
        foreach ($x in $added) { Say "    + $x" }
        foreach ($x in $removed) { Say "    - $x" }
    }
    $newProcs = @($b.procs | Where-Object { $a.procs -notcontains $_ })
    Say "  new processes: $($newProcs.Count) [$($newProcs -join ' ')]"
    Say "  winlogon shell: $($b.shell)"
}
function FolderStats($paths) {
    $n = 0; $bytes = 0
    foreach ($p in $paths) { if (Test-Path $p) { Get-ChildItem $p -Recurse -Force -File -ErrorAction SilentlyContinue | ForEach-Object { $n++; $bytes += $_.Length } } }
    "$n files, $([math]::Round($bytes / 1MB, 1)) MB"
}
function RunTimed($cmd, $label) {
    $sw = [Diagnostics.Stopwatch]::StartNew()
    $p = Start-Process cmd.exe -ArgumentList '/c', $cmd -WorkingDirectory $PackageDir -RedirectStandardInput $answers -RedirectStandardOutput (Join-Path $Out "$label-stdout.txt") -RedirectStandardError (Join-Path $Out "$label-stderr.txt") -PassThru -WindowStyle Hidden
    if (-not $p.WaitForExit(420000)) { Say "${label}: still running after 420 s; stopped"; Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue }
    Say "${label}: exit code $($p.ExitCode) after $($sw.ElapsedMilliseconds) ms"
}

$before = Snap
RunTimed $InstallCmd 'install'
Start-Sleep 12
$mid = Snap
Say '== after install'; Diff $before $mid
$newDirs = @($mid.dirs | Where-Object { $before.dirs -notcontains $_ })
Say "  size of new folders: $(FolderStats $newDirs)"
$sh = Join-Path $Out 'after-install.png'
Add-Type -AssemblyName System.Windows.Forms, System.Drawing
$b = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
$bmp = New-Object System.Drawing.Bitmap $b.Width, $b.Height
$g = [System.Drawing.Graphics]::FromImage($bmp); $g.CopyFromScreen($b.Location, [System.Drawing.Point]::Empty, $b.Size); $bmp.Save($sh)
Start-Sleep 3
RunTimed $UninstallCmd 'uninstall'
Start-Sleep 8
$after = Snap
Say '== after uninstall (compared with before install)'; Diff $before $after
$left = @($newDirs | Where-Object { Test-Path $_ })
Say "  folders left behind: $($left.Count) [$($left -join ' ')]; $(FolderStats $left)"
Say "  explorer running: $([bool](Get-Process explorer -ErrorAction SilentlyContinue))"
