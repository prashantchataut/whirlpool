# Sets the primary display's resolution and scale on a CI runner, and says what really applied.
# Used by .github/workflows/dpi.yml before scripts/drive.ps1. Best effort on purpose: the scale API
# (DisplayConfigSetDeviceInfo, packet type -4) is undocumented, and a runner with a virtual display
# may refuse it. The result is written to GITHUB_ENV as DPI_APPLIED so the job can fail honestly
# instead of capturing at 100% and calling it 150%.
param([int]$Scale = 100, [string]$Resolution = '1920x1080', [string]$Out = 'out')
$ErrorActionPreference = 'Continue'
New-Item -ItemType Directory -Force $Out | Out-Null
$log = Join-Path $Out 'dpi-log.txt'
function Log($m) { "$([DateTime]::Now.ToString('HH:mm:ss')) $m" | Tee-Object -Append $log }
Add-Type @"
using System; using System.Runtime.InteropServices; using System.Collections.Generic;
public static class Dpi {
  [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Ansi)] public struct DEVMODE {
    [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 32)] public string dmDeviceName;
    public short dmSpecVersion, dmDriverVersion, dmSize, dmDriverExtra; public int dmFields;
    public int dmPositionX, dmPositionY, dmDisplayOrientation, dmDisplayFixedOutput;
    public short dmColor, dmDuplex, dmYResolution, dmTTOption, dmCollate;
    [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 32)] public string dmFormName;
    public short dmLogPixels; public int dmBitsPerPel, dmPelsWidth, dmPelsHeight, dmDisplayFlags, dmDisplayFrequency;
    public int dmICMMethod, dmICMIntent, dmMediaType, dmDitherType, dmReserved1, dmReserved2, dmPanningWidth, dmPanningHeight;
  }
  [DllImport("user32.dll", CharSet = CharSet.Ansi)] static extern bool EnumDisplaySettingsA(string dev, int mode, ref DEVMODE dm);
  [DllImport("user32.dll", CharSet = CharSet.Ansi)] static extern int ChangeDisplaySettingsA(ref DEVMODE dm, int flags);
  [StructLayout(LayoutKind.Sequential)] struct Header { public int Type; public uint Size; public uint Low; public int High; public uint Id; }
  [StructLayout(LayoutKind.Sequential)] struct GetDpi { public Header H; public int Min, Cur, Max; }
  [StructLayout(LayoutKind.Sequential)] struct SetDpi { public Header H; public int Rel; }
  [DllImport("user32.dll")] static extern int GetDisplayConfigBufferSizes(uint flags, out uint paths, out uint modes);
  [DllImport("user32.dll")] static extern int QueryDisplayConfig(uint flags, ref uint paths, byte[] pathBuf, ref uint modes, byte[] modeBuf, IntPtr topo);
  [DllImport("user32.dll")] static extern int DisplayConfigGetDeviceInfo(ref GetDpi p);
  [DllImport("user32.dll")] static extern int DisplayConfigSetDeviceInfo(ref SetDpi p);
  static readonly int[] Steps = { 100, 125, 150, 175, 200, 225, 250, 300, 350, 400, 450, 500 };
  public static string Current() { var dm = new DEVMODE(); dm.dmSize = (short)Marshal.SizeOf(typeof(DEVMODE)); EnumDisplaySettingsA(null, -1, ref dm); return dm.dmPelsWidth + "x" + dm.dmPelsHeight; }
  public static List<string> Modes() {
    var set = new SortedSet<string>(); var dm = new DEVMODE(); dm.dmSize = (short)Marshal.SizeOf(typeof(DEVMODE));
    for (int i = 0; EnumDisplaySettingsA(null, i, ref dm); i++) if (dm.dmBitsPerPel >= 32) set.Add(dm.dmPelsWidth + "x" + dm.dmPelsHeight);
    return new List<string>(set);
  }
  public static string SetMode(int w, int h) {
    var dm = new DEVMODE(); dm.dmSize = (short)Marshal.SizeOf(typeof(DEVMODE));
    if (!EnumDisplaySettingsA(null, -1, ref dm)) return "EnumDisplaySettings failed";
    dm.dmPelsWidth = w; dm.dmPelsHeight = h; dm.dmFields = 0x80000 | 0x100000;
    int test = ChangeDisplaySettingsA(ref dm, 2); if (test != 0) return "mode test refused (code " + test + ")";
    return "ChangeDisplaySettings returned " + ChangeDisplaySettingsA(ref dm, 1);
  }
  // Returns "applied=<pct>" or "unavailable: ..." with the range the display offers.
  public static string SetScale(int pct) {
    uint np, nm; if (GetDisplayConfigBufferSizes(2, out np, out nm) != 0) return "unavailable: GetDisplayConfigBufferSizes failed";
    var pb = new byte[np * 72]; var mb = new byte[nm * 64];
    if (QueryDisplayConfig(2, ref np, pb, ref nm, mb, IntPtr.Zero) != 0 || np == 0) return "unavailable: QueryDisplayConfig failed";
    var h = new Header { Low = BitConverter.ToUInt32(pb, 0), High = BitConverter.ToInt32(pb, 4), Id = BitConverter.ToUInt32(pb, 8) };
    var g = new GetDpi { H = h }; g.H.Type = -3; g.H.Size = (uint)Marshal.SizeOf(typeof(GetDpi));
    int rc = DisplayConfigGetDeviceInfo(ref g); if (rc != 0) return "unavailable: get scale failed (code " + rc + ")";
    int rec = -g.Min; int idx = Array.IndexOf(Steps, pct); if (idx < 0) return "unavailable: " + pct + "% is not a Windows scale step";
    string range = "offered " + Steps[rec + g.Min] + "% to " + Steps[Math.Min(Steps.Length - 1, rec + g.Max)] + "%, current " + Steps[rec + g.Cur] + "%";
    int rel = idx - rec; if (rel < g.Min || rel > g.Max) return "unavailable: " + pct + "% (" + range + ")";
    var s = new SetDpi { H = h, Rel = rel }; s.H.Type = -4; s.H.Size = (uint)Marshal.SizeOf(typeof(SetDpi));
    rc = DisplayConfigSetDeviceInfo(ref s); if (rc != 0) return "unavailable: set scale failed (code " + rc + "; " + range + ")";
    g.H.Type = -3; DisplayConfigGetDeviceInfo(ref g);
    return "applied=" + Steps[rec + g.Cur] + " (" + range + ")";
  }
}
"@
Log "runner display before: $([Dpi]::Current()); modes at 32 bit: $(([Dpi]::Modes()) -join ' ')"
$applied = $false
if ($Scale -eq 100) { $applied = $true; Log 'scale 100: nothing to change' }
else {
    $w, $h = $Resolution -split 'x'
    Log "set resolution ${Resolution}: $([Dpi]::SetMode([int]$w, [int]$h))"
    Start-Sleep 3
    Log "runner display now: $([Dpi]::Current())"
    $r = [Dpi]::SetScale($Scale)
    Log "set scale ${Scale}%: $r"
    $applied = $r -eq "applied=$Scale" -or $r.StartsWith("applied=$Scale ")
    Start-Sleep 4
}
Log "DPI_APPLIED=$applied (requested $Scale%)"
if ($env:GITHUB_ENV) { "DPI_APPLIED=$applied" | Add-Content $env:GITHUB_ENV }
