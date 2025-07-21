; SPDX-FileCopyrightText: 2022  Emmanuele Bassi
; SPDX-License-Identifier: GPL-3.0-or-later

; Amberol Windows Installer Script
; This script creates a Windows installer using Inno Setup

[Setup]
AppId={{B7E2F1A4-3C5D-4E6F-8A9B-1C2D3E4F5A6B}
AppName=Amberol
AppVersion=2024.2
AppPublisher=Emmanuele Bassi
AppPublisherURL=https://apps.gnome.org/Amberol/
AppSupportURL=https://gitlab.gnome.org/World/amberol/-/issues
AppUpdatesURL=https://apps.gnome.org/Amberol/
DefaultDirName={autopf}\Amberol
DefaultGroupName=Amberol
AllowNoIcons=yes
LicenseFile=LICENSES\GPL-3.0-or-later.txt
InfoBeforeFile=README.md
OutputDir=dist
OutputBaseFilename=amberol-windows-installer
SetupIconFile=data\icons\hicolor\scalable\apps\io.bassi.Amberol.svg
Compression=lzma
SolidCompression=yes
WizardStyle=modern
PrivilegesRequired=admin
ArchitecturesAllowed=x64
ArchitecturesInstallIn64BitMode=x64

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "german"; MessagesFile: "compiler:Languages\German.isl"
Name: "french"; MessagesFile: "compiler:Languages\French.isl"
Name: "spanish"; MessagesFile: "compiler:Languages\Spanish.isl"
Name: "italian"; MessagesFile: "compiler:Languages\Italian.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "quicklaunchicon"; Description: "{cm:CreateQuickLaunchIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked; OnlyBelowVersion: 6.1
Name: "associatefiles"; Description: "Associate audio files with Amberol"; GroupDescription: "File associations"; Flags: checkedonce

[Files]
; Main executable
Source: "amberol-windows-portable\bin\amberol.exe"; DestDir: "{app}\bin"; Flags: ignoreversion

; Required DLLs
Source: "amberol-windows-portable\bin\*.dll"; DestDir: "{app}\bin"; Flags: ignoreversion

; GStreamer plugins
Source: "amberol-windows-portable\lib\gstreamer-1.0\*"; DestDir: "{app}\lib\gstreamer-1.0"; Flags: ignoreversion recursesubdirs createallsubdirs

; Application resources
Source: "amberol-windows-portable\share\*"; DestDir: "{app}\share"; Flags: ignoreversion recursesubdirs createallsubdirs

; Documentation
Source: "README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "LICENSES\GPL-3.0-or-later.txt"; DestDir: "{app}"; DestName: "LICENSE.txt"; Flags: ignoreversion
Source: "CHANGES.md"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\Amberol"; Filename: "{app}\bin\amberol.exe"; WorkingDir: "{app}\bin"
Name: "{group}\{cm:UninstallProgram,Amberol}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\Amberol"; Filename: "{app}\bin\amberol.exe"; WorkingDir: "{app}\bin"; Tasks: desktopicon
Name: "{userappdata}\Microsoft\Internet Explorer\Quick Launch\Amberol"; Filename: "{app}\bin\amberol.exe"; WorkingDir: "{app}\bin"; Tasks: quicklaunchicon

[Run]
Filename: "{app}\bin\amberol.exe"; Description: "{cm:LaunchProgram,Amberol}"; Flags: nowait postinstall skipifsilent

[Registry]
; File associations (if user selected the task)
Root: HKCR; Subkey: ".mp3"; ValueType: string; ValueName: ""; ValueData: "AmberolAudioFile"; Tasks: associatefiles
Root: HKCR; Subkey: ".mp4"; ValueType: string; ValueName: ""; ValueData: "AmberolAudioFile"; Tasks: associatefiles
Root: HKCR; Subkey: ".m4a"; ValueType: string; ValueName: ""; ValueData: "AmberolAudioFile"; Tasks: associatefiles
Root: HKCR; Subkey: ".aac"; ValueType: string; ValueName: ""; ValueData: "AmberolAudioFile"; Tasks: associatefiles
Root: HKCR; Subkey: ".ogg"; ValueType: string; ValueName: ""; ValueData: "AmberolAudioFile"; Tasks: associatefiles
Root: HKCR; Subkey: ".oga"; ValueType: string; ValueName: ""; ValueData: "AmberolAudioFile"; Tasks: associatefiles
Root: HKCR; Subkey: ".flac"; ValueType: string; ValueName: ""; ValueData: "AmberolAudioFile"; Tasks: associatefiles
Root: HKCR; Subkey: ".wav"; ValueType: string; ValueName: ""; ValueData: "AmberolAudioFile"; Tasks: associatefiles
Root: HKCR; Subkey: ".wma"; ValueType: string; ValueName: ""; ValueData: "AmberolAudioFile"; Tasks: associatefiles

Root: HKCR; Subkey: "AmberolAudioFile"; ValueType: string; ValueName: ""; ValueData: "Audio File"; Tasks: associatefiles
Root: HKCR; Subkey: "AmberolAudioFile\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\bin\amberol.exe,0"; Tasks: associatefiles
Root: HKCR; Subkey: "AmberolAudioFile\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\bin\amberol.exe"" ""%1"""; Tasks: associatefiles

[Code]
function InitializeSetup(): Boolean;
var
  Version: TWindowsVersion;
begin
  GetWindowsVersionEx(Version);
  if Version.Major < 10 then begin
    MsgBox('Amberol requires Windows 10 or later.', mbError, MB_OK);
    Result := False;
  end else
    Result := True;
end;

procedure InitializeWizard();
begin
  WizardForm.LicenseAcceptedRadio.Checked := True;
end;

[UninstallDelete]
Type: filesandordirs; Name: "{app}"