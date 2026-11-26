# Registers a real Windows media session (System Media Transport Controls) with a title, an artist,
# cover art and a progress bar, so the island has something to show. Nothing is played: the session is
# declared as playing, which is what a player does. The cover is drawn here, not taken from a record.
# Usage: powershell -File scripts/stage-media.ps1 [-Seconds 240]   (Windows PowerShell 5.1: it needs WinRT)
param([int]$Seconds = 240, [string]$Cover = (Join-Path $env:TEMP 'whirlpool-cover.png'))
Add-Type -AssemblyName System.Drawing

# Cover art: 600x600, a sunset gradient with a sun and a road. Drawn, so it carries no licence.
$bmp = New-Object System.Drawing.Bitmap 600, 600
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.SmoothingMode = 'AntiAlias'
$rect = New-Object System.Drawing.Rectangle 0, 0, 600, 600
$grad = New-Object System.Drawing.Drawing2D.LinearGradientBrush $rect, ([System.Drawing.Color]::FromArgb(255, 255, 122, 89)), ([System.Drawing.Color]::FromArgb(255, 54, 24, 120)), 60
$g.FillRectangle($grad, $rect)
$sun = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(235, 255, 214, 153))
$g.FillEllipse($sun, 180, 190, 240, 240)
$ink = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(255, 22, 12, 52))
$g.FillRectangle($ink, 0, 360, 600, 240)
$pen = New-Object System.Drawing.Pen ([System.Drawing.Color]::FromArgb(150, 255, 214, 153)), 3
foreach ($i in 0..4) { $g.DrawLine($pen, 300 - $i * 60, 372 + $i * 38, 300 + $i * 60, 372 + $i * 38) }
$g.Dispose(); $bmp.Save($Cover, [System.Drawing.Imaging.ImageFormat]::Png); $bmp.Dispose()

[void][Windows.Media.Playback.MediaPlayer, Windows.Media, ContentType = WindowsRuntime]
[void][Windows.Media.MediaPlaybackType, Windows.Media, ContentType = WindowsRuntime]
[void][Windows.Media.SystemMediaTransportControlsTimelineProperties, Windows.Media, ContentType = WindowsRuntime]
[void][Windows.Storage.Streams.RandomAccessStreamReference, Windows.Storage.Streams, ContentType = WindowsRuntime]

$player = New-Object Windows.Media.Playback.MediaPlayer
$player.CommandManager.IsEnabled = $false
$smtc = $player.SystemMediaTransportControls
$smtc.IsEnabled = $true
$smtc.IsPlayEnabled = $true; $smtc.IsPauseEnabled = $true; $smtc.IsNextEnabled = $true; $smtc.IsPreviousEnabled = $true
$update = $smtc.DisplayUpdater
$update.Type = [Windows.Media.MediaPlaybackType]::Music
$update.MusicProperties.Title = 'Low Tide'
$update.MusicProperties.Artist = 'Meridian Lines'
$update.MusicProperties.AlbumTitle = 'Night Drive'
# A file URI is not readable by the media broker; a StorageFile is. WinRT calls are awaited through AsTask.
Add-Type -AssemblyName System.Runtime.WindowsRuntime
[void][Windows.Storage.StorageFile, Windows.Storage, ContentType = WindowsRuntime]
$asTask = ([System.WindowsRuntimeSystemExtensions].GetMethods() | Where-Object { $_.Name -eq 'AsTask' -and $_.GetParameters().Count -eq 1 -and $_.GetParameters()[0].ParameterType.Name -eq 'IAsyncOperation`1' })[0]
$task = $asTask.MakeGenericMethod([Windows.Storage.StorageFile]).Invoke($null, @([Windows.Storage.StorageFile]::GetFileFromPathAsync($Cover)))
[void]$task.Wait(-1)
$update.Thumbnail = [Windows.Storage.Streams.RandomAccessStreamReference]::CreateFromFile($task.Result)
$update.Update()
$smtc.PlaybackStatus = [Windows.Media.MediaPlaybackStatus]::Playing

$total = [TimeSpan]::FromSeconds(214)
$start = Get-Date
$end = $start.AddSeconds($Seconds)
while ((Get-Date) -lt $end) {
    $line = New-Object Windows.Media.SystemMediaTransportControlsTimelineProperties
    $line.StartTime = [TimeSpan]::Zero; $line.MinSeekTime = [TimeSpan]::Zero
    $line.EndTime = $total; $line.MaxSeekTime = $total
    $line.Position = [TimeSpan]::FromSeconds([Math]::Min(150, 74 + ((Get-Date) - $start).TotalSeconds))
    $smtc.UpdateTimelineProperties($line)
    Start-Sleep -Seconds 2
}
