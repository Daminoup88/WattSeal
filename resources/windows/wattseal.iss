; WattSeal Windows installer script.
;
; Built with Inno Setup (https://jrsoftware.org/isinfo.php).
; To build:
;   1. cargo build --release   (produces target\release\WattSeal.exe)
;   2. ISCC resources\windows\wattseal.iss [/DMyAppVersion=1.2.3]
; The resulting setup executable is written to resources\windows\Output.

#ifndef MyAppVersion
  #define MyAppVersion "1.0.5"
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
; Always show the Select Destination Location page, even on top of a
; previously-registered install (Inno's "auto" default silently skips it then).
DisableDirPage=no
DisableProgramGroupPage=yes
; Installs per-user by default (no admin prompt); the user can still elevate
; to install into the machine-wide Program Files instead.
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
OutputDir=Output
OutputBaseFilename=WattSeal-Setup-{#MyAppVersion}
SetupIconFile=..\icon.ico
UninstallDisplayIcon={app}\{#MyAppExeName}
Compression=lzma
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "startmenuicon"; Description: "Create a Start Menu shortcut"; GroupDescription: "{cm:AdditionalIcons}"; Flags: checkedonce
Name: "autostart"; Description: "Launch WattSeal automatically when Windows starts"; GroupDescription: "Startup options:"; Flags: unchecked

[Files]
Source: "..\..\target\release\WattSeal.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: startmenuicon
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Registry]
; Mirrors common::autostart::set_enabled(true) so the app's own Settings
; toggle and the installer agree on the exact registry value.
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "WattSeal"; ValueData: """{app}\{#MyAppExeName}"" --background"; Flags: uninsdeletevalue; Tasks: autostart

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,WattSeal}"; Flags: nowait postinstall skipifsilent

[Code]
// If the chosen destination already holds a WattSeal install (our exe or an
// uninstaller left over from a previous setup), ask before overwriting it.
function NextButtonClick(CurPageID: Integer): Boolean;
var
  TargetDir: String;
begin
  Result := True;
  if CurPageID = wpSelectDir then
  begin
    TargetDir := WizardForm.DirEdit.Text;
    if FileExists(TargetDir + '\{#MyAppExeName}') or FileExists(TargetDir + '\unins000.exe') then
    begin
      if MsgBox(
        'WattSeal is already installed in:' + #13#10 + TargetDir + #13#10#13#10 +
        'Do you want to overwrite the existing installation? Click No to cancel Setup.',
        mbConfirmation, MB_YESNO) = IDNO then
      begin
        Abort;
      end;
    end;
  end;
end;

// --- Uninstall: offer to also remove WattSeal's data (database + logs) ---
// The app stores its power-monitoring database and log file alongside the exe
// (see common::DATABASE_PATH / common::LOG_FILE), so a plain uninstall - which
// only removes files it installed - leaves them behind. Ask the user up front,
// via a checkbox, whether to also delete that data.
var
  RemoveAppData: Boolean;

function InitializeUninstall(): Boolean;
var
  ConfirmForm: TSetupForm;
  InfoLabel: TNewStaticText;
  DataCheckBox: TNewCheckBox;
  ContinueButton, CancelButton: TNewButton;
begin
  RemoveAppData := False;

  ConfirmForm := TSetupForm.Create(nil);
  try
    ConfirmForm.ClientWidth := ScaleX(380);
    ConfirmForm.ClientHeight := ScaleY(150);
    ConfirmForm.Caption := 'Uninstall WattSeal';
    ConfirmForm.Position := poScreenCenter;
    ConfirmForm.BorderStyle := bsDialog;

    InfoLabel := TNewStaticText.Create(ConfirmForm);
    InfoLabel.Parent := ConfirmForm;
    InfoLabel.Left := ScaleX(16);
    InfoLabel.Top := ScaleY(16);
    InfoLabel.Width := ConfirmForm.ClientWidth - ScaleX(32);
    InfoLabel.AutoSize := False;
    InfoLabel.WordWrap := True;
    InfoLabel.Height := ScaleY(48);
    InfoLabel.Caption :=
      'WattSeal keeps its power-monitoring database and log file alongside the ' +
      'application. Choose whether to remove them too.';

    DataCheckBox := TNewCheckBox.Create(ConfirmForm);
    DataCheckBox.Parent := ConfirmForm;
    DataCheckBox.Left := ScaleX(16);
    DataCheckBox.Top := ScaleY(72);
    DataCheckBox.Width := ConfirmForm.ClientWidth - ScaleX(32);
    DataCheckBox.Height := ScaleY(17);
    DataCheckBox.Caption := 'Also remove application data (database and logs)';
    DataCheckBox.Checked := False;

    ContinueButton := TNewButton.Create(ConfirmForm);
    ContinueButton.Parent := ConfirmForm;
    ContinueButton.Width := ScaleX(80);
    ContinueButton.Height := ScaleY(23);
    ContinueButton.Left := ConfirmForm.ClientWidth - ScaleX(180);
    ContinueButton.Top := ConfirmForm.ClientHeight - ScaleY(35);
    ContinueButton.Caption := 'Continue';
    ContinueButton.ModalResult := mrOk;
    ContinueButton.Default := True;

    CancelButton := TNewButton.Create(ConfirmForm);
    CancelButton.Parent := ConfirmForm;
    CancelButton.Width := ScaleX(80);
    CancelButton.Height := ScaleY(23);
    CancelButton.Left := ConfirmForm.ClientWidth - ScaleX(90);
    CancelButton.Top := ConfirmForm.ClientHeight - ScaleY(35);
    CancelButton.Caption := 'Cancel';
    CancelButton.ModalResult := mrCancel;
    CancelButton.Cancel := True;

    Result := ConfirmForm.ShowModal() = mrOk;
    if Result then
      RemoveAppData := DataCheckBox.Checked;
  finally
    ConfirmForm.Free;
  end;
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
var
  AppDir: String;
begin
  if (CurUninstallStep = usPostUninstall) and RemoveAppData then
  begin
    AppDir := ExpandConstant('{app}');
    DeleteFile(AppDir + '\power_monitoring.db');
    DeleteFile(AppDir + '\power_monitoring.db-shm');
    DeleteFile(AppDir + '\power_monitoring.db-wal');
    DeleteFile(AppDir + '\power_monitoring.db.collector.lock');
    DeleteFile(AppDir + '\collector.log');
    RemoveDir(AppDir);
  end;
end;
