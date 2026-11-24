# Install, run, uninstall: proves what Setup leaves behind. Used by .github/workflows/lifecycle.yml.
param([Parameter(Mandatory = $true)][string]$Setup, [Parameter(Mandatory = $true)][string]$Out)
$ErrorActionPreference = 'Continue'
$Out = (New-Item -ItemType Directory -Force $Out).FullName   # absolute: the uninstaller's second phase runs elsewhere
$Setup = (Resolve-Path $Setup).Path
$report = Join-Path $Out 'lifecycle.txt'
function Say($m) { $m | Tee-Object -Append $report }

function Snapshot {
    $roots = @("$env:LOCALAPPDATA\Programs", "$env:LOCALAPPDATA\Whirlpool", "$env:APPDATA\Whirlpool", "$env:APPDATA\Microsoft\Windows\Start Menu\Programs", "$env:ProgramFiles\Whirlpool", "$env:USERPROFILE\Desktop")
    $files = foreach ($r in $roots) { if (Test-Path $r) { Get-ChildItem $r -Recurse -Force -File -ErrorAction SilentlyContinue | Where-Object { $_.FullName -match 'Whirlpool' } | ForEach-Object FullName } }
    $keys = @()
    $run = Get-ItemProperty 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run' -ErrorAction SilentlyContinue
    if ($run) { $keys += $run.PSObject.Properties | Where-Object { $_.Name -match 'Whirlpool' } | ForEach-Object { "Run\$($_.Name)" } }
    foreach ($k in 'HKCU:\Software\Classes\AppUserModelId\Whirlpool', 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\{B7D0A2F4-5C1E-4C55-9E4B-3F1A6A0C7A11}_is1', 'HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\{B7D0A2F4-5C1E-4C55-9E4B-3F1A6A0C7A11}_is1') { if (Test-Path $k) { $keys += $k } }
    $procs = Get-Process -Name whirlpool -ErrorAction SilentlyContinue | ForEach-Object { "process $($_.Id)" }
    $tray = if (Get-Process explorer -ErrorAction SilentlyContinue) { 'explorer running' } else { 'EXPLORER NOT RUNNING' }
    @{ files = @($files | Sort-Object); keys = @($keys); procs = @($procs); tray = $tray }
}
function Show($title, $s) {
    Say "--- $title"
    foreach ($f in $s.files) { Say "  file  $f" }
    foreach ($k in $s.keys) { Say "  key   $k" }
    foreach ($p in $s.procs) { Say "  $p" }
    Say "  $($s.tray)"
}

Show 'before install' (Snapshot)
$sw = [Diagnostics.Stopwatch]::StartNew()
$p = Start-Process $Setup -ArgumentList '/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART', '/TASKS=autostart', "/LOG=$Out\setup-raw.log" -PassThru -Wait
Say "install exit code $($p.ExitCode) in $($sw.ElapsedMilliseconds) ms"
$after = Snapshot; Show 'after install' $after

$exe = ($after.files | Where-Object { $_ -match 'whirlpool\.exe$' } | Select-Object -First 1)
if ($exe) {
    Start-Process $exe; Start-Sleep 12
    Say "running after start: $(@(Get-Process -Name whirlpool -ErrorAction SilentlyContinue).Count) process(es)"
    $log = $after.files | Where-Object { $_ -match 'install\.log$' } | Select-Object -First 1
    if ($log) { Say '--- install.log content'; Get-Content $log | ForEach-Object { Say "  $_" } } else { Say 'NO install.log FOUND' }
}
$unins = ($after.files | Where-Object { $_ -match 'unins\d+\.exe$' } | Select-Object -First 1)
if ($unins) {
    $sw.Restart()
    $u = Start-Process $unins -ArgumentList '/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART', "/LOG=$Out\uninstall-raw.log" -PassThru -Wait
    Say "uninstall exit code $($u.ExitCode) in $($sw.ElapsedMilliseconds) ms"
    Start-Sleep 4
} else { Say 'NO UNINSTALLER FOUND' }
$gone = Snapshot; Show 'after uninstall' $gone
$left = @($gone.files).Count + @($gone.keys).Count + @($gone.procs).Count
Say "LEFTOVERS: $left"
if ($left -gt 0 -or $gone.tray -ne 'explorer running') { exit 1 }
