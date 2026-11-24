; Whirlpool Setup. Per-user, no administrator rights, one program.
; Built by CI:  iscc /DAppVersion=2.0.0 installer\whirlpool.iss
#ifndef AppVersion
  #define AppVersion "0.0.0"
#endif
; Four numbers for the file version resource (4.2.0-rc1 -> 4.2.0.0).
#ifndef NumericVersion
  #define NumericVersion "0.0.0.0"
#endif
; x64 (default) or arm64. CI passes /DArch=arm64 /DSourceExe=... for the ARM64 build.
#ifndef Arch
  #define Arch "x64"
#endif
#ifndef SourceExe
  #define SourceExe "..\target\release\whirlpool.exe"
#endif

[Setup]
AppId={{B7D0A2F4-5C1E-4C55-9E4B-3F1A6A0C7A11}
AppName=Whirlpool
AppVersion={#AppVersion}
AppVerName=Whirlpool {#AppVersion}
AppPublisher=Whirlpool
AppPublisherURL=https://github.com/prashantchataut/whirlpool
AppSupportURL=https://github.com/prashantchataut/whirlpool/issues
AppUpdatesURL=https://github.com/prashantchataut/whirlpool/releases
VersionInfoVersion={#NumericVersion}
VersionInfoProductName=Whirlpool
VersionInfoDescription=Whirlpool Setup
DefaultDirName={autopf}\Whirlpool
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
OutputDir=..\dist
#if Arch == "arm64"
OutputBaseFilename=Whirlpool-Setup-arm64
#else
OutputBaseFilename=Whirlpool-Setup
#endif
SetupIconFile=..\assets\brand\whirlpool.ico
UninstallDisplayIcon={app}\whirlpool.exe
UninstallDisplayName=Whirlpool
LicenseFile=..\LICENSE
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
#if Arch == "arm64"
ArchitecturesAllowed=arm64
ArchitecturesInstallIn64BitMode=arm64
#else
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
#endif
MinVersion=10.0.18362
SetupLogging=yes
CloseApplications=no

[Tasks]
Name: "autostart"; Description: "Start Whirlpool when I sign in"; GroupDescription: "Options:"
Name: "desktopicon"; Description: "Create a desktop shortcut"; GroupDescription: "Options:"; Flags: unchecked

[Files]
Source: "{#SourceExe}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\NOTICE"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\THIRD_PARTY_LICENSES.md"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\Whirlpool"; Filename: "{app}\whirlpool.exe"; IconFilename: "{app}\whirlpool.exe"
Name: "{autoprograms}\Whirlpool Settings"; Filename: "{app}\whirlpool.exe"; Parameters: "--settings"; IconFilename: "{app}\whirlpool.exe"
Name: "{autodesktop}\Whirlpool"; Filename: "{app}\whirlpool.exe"; IconFilename: "{app}\whirlpool.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\whirlpool.exe"; Parameters: "--autostart on"; Flags: runhidden; Tasks: autostart
; The in-app updater passes /relaunch=1 so Whirlpool comes back after a silent update.
Filename: "{app}\whirlpool.exe"; Flags: nowait; Check: RelaunchRequested
Filename: "{app}\whirlpool.exe"; Description: "Start Whirlpool now"; Flags: nowait postinstall skipifsilent

[UninstallDelete]
Type: files; Name: "{app}\install.log"
Type: files; Name: "{app}\install-setup.log"

[UninstallRun]
Filename: "{app}\whirlpool.exe"; Parameters: "--autostart off"; Flags: runhidden; RunOnceId: "AutostartOff"
Filename: "{app}\whirlpool.exe"; Parameters: "--restore"; Flags: runhidden; RunOnceId: "RestoreTaskbar"

[Code]
function RelaunchRequested: Boolean;
begin
  Result := ExpandConstant('{param:relaunch|0}') = '1';
end;

// Quit a running copy first so its file can be replaced.
function PrepareToInstall(var NeedsRestart: Boolean): String;
var
  Code: Integer;
begin
  if FileExists(ExpandConstant('{app}\whirlpool.exe')) then
  begin
    Exec(ExpandConstant('{app}\whirlpool.exe'), '--quit', '', SW_HIDE, ewWaitUntilTerminated, Code);
    Sleep(600);
  end;
  Result := '';
end;

// ---- Install log ---------------------------------------------------------------------------------
// A short, readable record next to the program: what was installed, where, with which options.
// The full Setup log is copied beside it when Windows lets us read it.
procedure WriteInstallLog;
var
  Lines: TArrayOfString;
begin
  SetArrayLength(Lines, 7);
  Lines[0] := 'Whirlpool {#AppVersion} installed ' + GetDateTimeString('yyyy-mm-dd hh:nn:ss', '-', ':');
  Lines[1] := 'Folder: ' + ExpandConstant('{app}');
  Lines[2] := 'Start when I sign in: ' + IntToStr(Ord(WizardIsTaskSelected('autostart')));
  Lines[3] := 'Desktop shortcut: ' + IntToStr(Ord(WizardIsTaskSelected('desktopicon')));
  Lines[4] := 'Start menu: ' + ExpandConstant('{autoprograms}') + '\Whirlpool';
  Lines[5] := 'Settings and logs live in %APPDATA%\Whirlpool and %LOCALAPPDATA%\Whirlpool';
  Lines[6] := 'Uninstall removes all of the above (Settings > Apps, or unins000.exe).';
  SaveStringsToFile(ExpandConstant('{app}\install.log'), Lines, False);
  FileCopy(ExpandConstant('{log}'), ExpandConstant('{app}\install-setup.log'), False);
end;

procedure CurStepChanged(CurStep: TSetupStep);
begin
  if CurStep = ssPostInstall then
    WriteInstallLog;
end;

// ---- Clean uninstall ------------------------------------------------------------------------------
// Give the taskbar back, stop every copy of the program (the dock, its watchdog and any open window),
// then remove what Whirlpool created outside its folder. Silent uninstalls remove settings too unless
// /KEEPDATA is passed; an interactive one asks.
function KeepData: Boolean;
begin
  if UninstallSilent then
    Result := ExpandConstant('{param:keepdata|0}') = '1'
  else
    Result := MsgBox('Keep your Whirlpool settings and logs for a later reinstall?', mbConfirmation, MB_YESNO or MB_DEFBUTTON2) = IDYES;
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
var
  Code: Integer;
begin
  if CurUninstallStep = usUninstall then
  begin
    Exec(ExpandConstant('{app}\whirlpool.exe'), '--restore', '', SW_HIDE, ewWaitUntilTerminated, Code);
    Sleep(1200);
    Exec(ExpandConstant('{sys}\taskkill.exe'), '/F /IM whirlpool.exe', '', SW_HIDE, ewWaitUntilTerminated, Code);
    Sleep(400);
  end;
  if CurUninstallStep = usPostUninstall then
  begin
    RegDeleteKeyIncludingSubkeys(HKCU, 'Software\Classes\AppUserModelId\Whirlpool');
    if not KeepData then
    begin
      DelTree(ExpandConstant('{userappdata}\Whirlpool'), True, True, True);
      DelTree(ExpandConstant('{localappdata}\Whirlpool'), True, True, True);
    end;
  end;
end;
