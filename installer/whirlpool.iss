; Whirlpool Setup. Per-user, no administrator rights, one program.
; Built by CI:  iscc /DAppVersion=2.0.0 installer\whirlpool.iss
#ifndef AppVersion
  #define AppVersion "0.0.0"
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
VersionInfoVersion={#AppVersion}
VersionInfoProductName=Whirlpool
VersionInfoDescription=Whirlpool Setup
DefaultDirName={autopf}\Whirlpool
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
OutputDir=..\dist
OutputBaseFilename=Whirlpool-Setup
SetupIconFile=..\assets\brand\whirlpool.ico
UninstallDisplayIcon={app}\whirlpool.exe
UninstallDisplayName=Whirlpool
LicenseFile=..\LICENSE
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0.18362

[Tasks]
Name: "autostart"; Description: "Start Whirlpool when I sign in"; GroupDescription: "Options:"
Name: "desktopicon"; Description: "Create a desktop shortcut"; GroupDescription: "Options:"; Flags: unchecked

[Files]
Source: "..\target\release\whirlpool.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\NOTICE"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\Whirlpool"; Filename: "{app}\whirlpool.exe"; IconFilename: "{app}\whirlpool.exe"
Name: "{autoprograms}\Whirlpool Settings"; Filename: "{app}\whirlpool.exe"; Parameters: "--settings"; IconFilename: "{app}\whirlpool.exe"
Name: "{autodesktop}\Whirlpool"; Filename: "{app}\whirlpool.exe"; IconFilename: "{app}\whirlpool.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\whirlpool.exe"; Parameters: "--autostart on"; Flags: runhidden; Tasks: autostart
Filename: "{app}\whirlpool.exe"; Description: "Start Whirlpool now"; Flags: nowait postinstall skipifsilent

[UninstallRun]
Filename: "{app}\whirlpool.exe"; Parameters: "--autostart off"; Flags: runhidden; RunOnceId: "AutostartOff"
Filename: "{app}\whirlpool.exe"; Parameters: "--restore"; Flags: runhidden; RunOnceId: "RestoreTaskbar"

[Code]
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
