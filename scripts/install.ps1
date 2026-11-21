# Installs the latest Whirlpool release for the current user: no admin rights.
#   irm https://raw.githubusercontent.com/prashantchataut/whirlpool/main/scripts/install.ps1 | iex
$ErrorActionPreference = 'Stop'
$repo = 'prashantchataut/whirlpool'
$dir = Join-Path $env:LOCALAPPDATA 'Whirlpool'
$base = "https://github.com/$repo/releases/latest/download"

New-Item -ItemType Directory -Force $dir | Out-Null
Get-Process whirlpool, whirlpool-settings -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Milliseconds 400

foreach ($file in 'whirlpool.exe', 'whirlpool-settings.exe') {
    Write-Host "Downloading $file ..."
    Invoke-WebRequest "$base/$file" -OutFile (Join-Path $dir $file)
}

# Start menu entries: the dock and its settings.
$menu = Join-Path $env:APPDATA 'Microsoft\Windows\Start Menu\Programs'
$shell = New-Object -ComObject WScript.Shell
foreach ($entry in @(@('Whirlpool', 'whirlpool.exe'), @('Whirlpool Settings', 'whirlpool-settings.exe'))) {
    $link = $shell.CreateShortcut((Join-Path $menu "$($entry[0]).lnk"))
    $link.TargetPath = Join-Path $dir $entry[1]
    $link.WorkingDirectory = $dir
    $link.Save()
}

Start-Process (Join-Path $dir 'whirlpool.exe')
Write-Host "Whirlpool installed in $dir"
