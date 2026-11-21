# Installs the latest Muelle release for the current user: no admin rights.
#   irm https://raw.githubusercontent.com/prashantchataut/muelle-native/main/scripts/install.ps1 | iex
$ErrorActionPreference = 'Stop'
$repo = 'prashantchataut/muelle-native'
$dir = Join-Path $env:LOCALAPPDATA 'Muelle'
$base = "https://github.com/$repo/releases/latest/download"

New-Item -ItemType Directory -Force $dir | Out-Null
Get-Process muelle, muelle-settings -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Milliseconds 400

foreach ($file in 'muelle.exe', 'muelle-settings.exe') {
    Write-Host "Downloading $file ..."
    Invoke-WebRequest "$base/$file" -OutFile (Join-Path $dir $file)
}

# Start menu entries: the dock and its settings.
$menu = Join-Path $env:APPDATA 'Microsoft\Windows\Start Menu\Programs'
$shell = New-Object -ComObject WScript.Shell
foreach ($entry in @(@('Muelle', 'muelle.exe'), @('Muelle Settings', 'muelle-settings.exe'))) {
    $link = $shell.CreateShortcut((Join-Path $menu "$($entry[0]).lnk"))
    $link.TargetPath = Join-Path $dir $entry[1]
    $link.WorkingDirectory = $dir
    $link.Save()
}

Start-Process (Join-Path $dir 'muelle.exe')
Write-Host "Muelle installed in $dir"
