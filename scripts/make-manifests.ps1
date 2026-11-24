# Writes the winget manifests and the scoop manifest for one release from its SHA256SUMS.txt.
#   ./scripts/make-manifests.ps1 -Version 4.2.0 -Sums path\to\SHA256SUMS.txt
# Hashes are read from the release's own checksum file, never typed by hand.
param(
    [Parameter(Mandatory = $true)][string]$Version,
    [Parameter(Mandatory = $true)][string]$Sums
)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$hash = @{}
foreach ($line in Get-Content $Sums) {
    if ($line -match '^([0-9a-fA-F]{64})\s+\*?(.+)$') { $hash[$Matches[2].Trim()] = $Matches[1].ToUpper() }
}
foreach ($name in 'Whirlpool-Setup.exe', 'Whirlpool-Setup-arm64.exe', 'Whirlpool-x64.zip', 'Whirlpool-arm64.zip') {
    if (-not $hash.ContainsKey($name)) { throw "SHA256SUMS.txt has no entry for $name" }
}
$tag = "v$Version"
$base = "https://github.com/prashantchataut/whirlpool/releases/download/$tag"
$date = (Get-Date).ToString('yyyy-MM-dd')

$winget = Join-Path $root 'packaging/winget'
New-Item -ItemType Directory -Force $winget | Out-Null
$id = 'PrashantChataut.Whirlpool'
Set-Content -Encoding utf8 (Join-Path $winget "$id.yaml") @"
# yaml-language-server: `$schema=https://aka.ms/winget-manifest.version.1.6.0.schema.json
PackageIdentifier: $id
PackageVersion: $Version
DefaultLocale: en-US
ManifestType: version
ManifestVersion: 1.6.0
"@
Set-Content -Encoding utf8 (Join-Path $winget "$id.installer.yaml") @"
# yaml-language-server: `$schema=https://aka.ms/winget-manifest.installer.1.6.0.schema.json
PackageIdentifier: $id
PackageVersion: $Version
InstallerType: inno
Scope: user
InstallModes:
  - interactive
  - silent
  - silentWithProgress
UpgradeBehavior: install
ReleaseDate: $date
Installers:
  - Architecture: x64
    InstallerUrl: $base/Whirlpool-Setup.exe
    InstallerSha256: $($hash['Whirlpool-Setup.exe'])
  - Architecture: arm64
    InstallerUrl: $base/Whirlpool-Setup-arm64.exe
    InstallerSha256: $($hash['Whirlpool-Setup-arm64.exe'])
ManifestType: installer
ManifestVersion: 1.6.0
"@
Set-Content -Encoding utf8 (Join-Path $winget "$id.locale.en-US.yaml") @"
# yaml-language-server: `$schema=https://aka.ms/winget-manifest.defaultLocale.1.6.0.schema.json
PackageIdentifier: $id
PackageVersion: $Version
PackageLocale: en-US
Publisher: Prashant Chataut
PublisherUrl: https://github.com/prashantchataut
PackageName: Whirlpool
PackageUrl: https://github.com/prashantchataut/whirlpool
License: MIT
LicenseUrl: https://github.com/prashantchataut/whirlpool/blob/main/LICENSE
ShortDescription: A dock that moves like water: magnifying dock, menu bar with an island, Start grid and command palette for Windows 10 and 11.
Moniker: whirlpool
Tags:
  - dock
  - menu-bar
  - launcher
  - shell
  - customization
ReleaseNotesUrl: https://github.com/prashantchataut/whirlpool/releases/tag/$tag
ManifestType: defaultLocale
ManifestVersion: 1.6.0
"@

$scoop = Join-Path $root 'packaging/scoop'
New-Item -ItemType Directory -Force $scoop | Out-Null
$manifest = [ordered]@{
    version      = $Version
    description  = 'A dock that moves like water: magnifying dock, menu bar with an island, Start grid and command palette for Windows.'
    homepage     = 'https://github.com/prashantchataut/whirlpool'
    license      = 'MIT'
    architecture = [ordered]@{
        '64bit' = [ordered]@{ url = "$base/Whirlpool-x64.zip"; hash = $hash['Whirlpool-x64.zip'].ToLower() }
        'arm64' = [ordered]@{ url = "$base/Whirlpool-arm64.zip"; hash = $hash['Whirlpool-arm64.zip'].ToLower() }
    }
    bin          = 'whirlpool.exe'
    shortcuts    = , @('whirlpool.exe', 'Whirlpool')
    checkver     = [ordered]@{ github = 'https://github.com/prashantchataut/whirlpool' }
    autoupdate   = [ordered]@{
        architecture = [ordered]@{
            '64bit' = [ordered]@{ url = 'https://github.com/prashantchataut/whirlpool/releases/download/v$version/Whirlpool-x64.zip' }
            'arm64' = [ordered]@{ url = 'https://github.com/prashantchataut/whirlpool/releases/download/v$version/Whirlpool-arm64.zip' }
        }
    }
}
$manifest | ConvertTo-Json -Depth 6 | Set-Content -Encoding utf8 (Join-Path $scoop 'whirlpool.json')
Write-Host "Wrote winget and scoop manifests for $Version"
