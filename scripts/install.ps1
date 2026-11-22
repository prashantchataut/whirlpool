# Installs the latest Whirlpool release for the current user: no admin rights.
#   irm https://raw.githubusercontent.com/prashantchataut/whirlpool/main/scripts/install.ps1 | iex
# Prefer the Setup program from the Releases page if you want Apps & Features and an uninstaller.
$ErrorActionPreference = 'Stop'
$repo = 'prashantchataut/whirlpool'
$dir = Join-Path $env:LOCALAPPDATA 'Whirlpool'
$base = "https://github.com/$repo/releases/latest/download"

New-Item -ItemType Directory -Force $dir | Out-Null
Get-Process whirlpool -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Milliseconds 400

# Every release publishes SHA256SUMS.txt; refuse to install anything that does not match.
$sums = (Invoke-WebRequest "$base/SHA256SUMS.txt" -UseBasicParsing).Content -split "`n" |
    Where-Object { $_.Trim() } | ForEach-Object { $p = $_.Trim() -split '\s+'; @{ Hash = $p[0]; Name = $p[-1] } }

$file = 'whirlpool.exe'
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

# An old two-program install leaves this behind.
Remove-Item (Join-Path $dir 'whirlpool-settings.exe') -Force -ErrorAction SilentlyContinue

$menu = Join-Path $env:APPDATA 'Microsoft\Windows\Start Menu\Programs'
$shell = New-Object -ComObject WScript.Shell
foreach ($entry in @(@('Whirlpool', ''), @('Whirlpool Settings', '--settings'))) {
    $link = $shell.CreateShortcut((Join-Path $menu "$($entry[0]).lnk"))
    $link.TargetPath = $target
    $link.Arguments = $entry[1]
    $link.IconLocation = "$target,0"
    $link.WorkingDirectory = $dir
    $link.Save()
}

Start-Process $target
Write-Host "Whirlpool installed in $dir"
