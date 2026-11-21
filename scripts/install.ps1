# Installs the latest Whirlpool release for the current user: no admin rights.
#   irm https://raw.githubusercontent.com/prashantchataut/whirlpool/main/scripts/install.ps1 | iex
$ErrorActionPreference = 'Stop'
$repo = 'prashantchataut/whirlpool'
$dir = Join-Path $env:LOCALAPPDATA 'Whirlpool'
$base = "https://github.com/$repo/releases/latest/download"

New-Item -ItemType Directory -Force $dir | Out-Null
Get-Process whirlpool, whirlpool-settings -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Milliseconds 400

# Every release publishes SHA256SUMS.txt; refuse to install anything that does not match.
$sums = (Invoke-WebRequest "$base/SHA256SUMS.txt" -UseBasicParsing).Content -split "`n" |
    Where-Object { $_.Trim() } | ForEach-Object { $p = $_.Trim() -split '\s+'; @{ Hash = $p[0]; Name = $p[-1] } }

foreach ($file in 'whirlpool.exe', 'whirlpool-settings.exe') {
    Write-Host "Downloading $file ..."
    $target = Join-Path $dir $file
    Invoke-WebRequest "$base/$file" -OutFile $target -UseBasicParsing
    $expected = ($sums | Where-Object { $_.Name -eq $file }).Hash
    $actual = (Get-FileHash $target -Algorithm SHA256).Hash.ToLower()
    if (-not $expected -or $actual -ne $expected) {
        Remove-Item $target -Force
        throw "Checksum mismatch for $file (expected $expected, got $actual). Nothing was installed."
    }
    Write-Host "  SHA-256 verified: $actual"
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
