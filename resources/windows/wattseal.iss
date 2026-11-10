; WattSeal Windows installer script.
;
; Built with Inno Setup (https://jrsoftware.org/isinfo.php).
; To build:
;   1. cargo build --release   (produces target\release\WattSeal.exe)
;   2. ISCC resources\windows\wattseal.iss [/DMyAppVersion=1.2.3]
; The resulting setup executable is written to resources\windows\Output.

#ifndef MyAppVersion
  #define MyAppVersion "1.0.6"
#endif
#define MyAppName "WattSeal"
#define MyAppPublisher "WattSeal"
#define MyAppURL "https://wattseal.com"
#define MyAppExeName "WattSeal.exe"

[Setup]
; Fixed AppId so upgrades/uninstalls target the same registry entry across versions.
AppId={{7C6C5C2B-6C7C-4E8A-9F1C-3E2B7A5C1D9E}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
; Only show the Select Destination Location page on first install
; on upgrades, use the previous install location.
DisableDirPage=auto
DisableProgramGroupPage=yes
; Installs per-user by default (no admin prompt); the user can still elevate
; to install into the machine-wide Program Files instead.
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
OutputDir=Output
OutputBaseFilename=WattSeal-windows
SetupIconFile=..\icon.ico
UninstallDisplayIcon={app}\{#MyAppExeName}
Compression=lzma
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "french"; MessagesFile: "compiler:Languages\French.isl"
Name: "chinesesimplified"; MessagesFile: "compiler:Languages\ChineseSimplified.isl"
Name: "brazilianportuguese"; MessagesFile: "compiler:Languages\BrazilianPortuguese.isl"
Name: "german"; MessagesFile: "compiler:Languages\German.isl"
; Romanian isn't supported by Inno Setup

[Files]
Source: "..\..\target\release\WattSeal.exe"; DestDir: "{app}"; DestName: "{#MyAppExeName}"; Flags: ignoreversion
Source: "..\..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,WattSeal}"; Flags: nowait postinstall skipifsilent
