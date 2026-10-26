; AniBeam's Windows installer, compiled by bundle.sh with Inno Setup 6. A per-user install:
; no administrator prompt, the app under %LOCALAPPDATA%\Programs\AniBeam, a Start menu entry
; and an uninstaller. The library, settings and downloads live in the user's own AppData and
; are left alone on uninstall.
; bundle.sh passes AppVersion, FullVersion, SourceDir, OutputDir and OutputName.

[Setup]
AppId={{6F3B2A51-5E0C-4C7B-9A0E-2B7D3E4A1C90}
AppName=AniBeam
AppVersion={#AppVersion}
AppVerName=AniBeam {#AppVersion}
VersionInfoVersion={#AppVersion}
VersionInfoProductTextVersion={#FullVersion}
AppPublisher=Marcus Rosado
AppPublisherURL=https://github.com/marcusbandit/AniBeam
DefaultDirName={autopf}\AniBeam
DefaultGroupName=AniBeam
DisableProgramGroupPage=yes
DisableDirPage=yes
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0
OutputDir={#OutputDir}
OutputBaseFilename={#OutputName}
SetupIconFile={#SourceDir}\anibeam.ico
UninstallDisplayIcon={app}\anibeam.exe
UninstallDisplayName=AniBeam
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
; AniBeam keeps running the qBittorrent it started; an update replaces its files too.
CloseApplications=yes
RestartApplications=no

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"

[Files]
Source: "{#SourceDir}\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[InstallDelete]
; An update starts from a clean folder, so a file the new version no longer ships is not left
; behind to be loaded by mistake.
Type: filesandordirs; Name: "{app}\qml"
Type: filesandordirs; Name: "{app}\anirss"

[Icons]
Name: "{autoprograms}\AniBeam"; Filename: "{app}\anibeam.exe"; IconFilename: "{app}\anibeam.ico"
Name: "{autodesktop}\AniBeam"; Filename: "{app}\anibeam.exe"; IconFilename: "{app}\anibeam.ico"; Tasks: desktopicon

[Run]
Filename: "{app}\anibeam.exe"; Description: "{cm:LaunchProgram,AniBeam}"; Flags: nowait postinstall skipifsilent

[UninstallRun]
; The managed qBittorrent outlives the app on purpose; it has to stop before its files go.
Filename: "{sys}\taskkill.exe"; Parameters: "/F /IM qbittorrent-nox.exe"; Flags: runhidden; RunOnceId: "StopQbittorrent"
