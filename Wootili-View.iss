; Script generated by the Inno Setup Script Wizard.
; Modifications by Mr.Ender

#define MyAppName "Wootili-View"
#define MyAppVersion "0.7.4"
#define MyAppPublisher "Mr.Ender"
#define MyAppURL "https://github.com/MrEnder0/wootili-view"
#define MyAppExeName "Wootili-View.exe"

[Setup]
; NOTE: The value of AppId uniquely identifies this application. Do not use the same AppId value in installers for other applications.
; (To generate a new GUID, click Tools | Generate GUID inside the IDE.)
AppId={{EA2B1670-154D-497E-B229-284F07E869F1}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
;AppVerName={#MyAppName} {#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
DisableProgramGroupPage=yes
PrivilegesRequiredOverridesAllowed=dialog
OutputDir=target\innosetup
OutputBaseFilename={#MyAppName} Setup v{#MyAppVersion}
Compression=lzma
SolidCompression=yes
WizardStyle=modern

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "githubicon"; Description: "Create a shortcut to the project Github repository (recommended)"; GroupDescription: "{cm:AdditionalIcons}"; Components: baseinstall
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Components: baseinstall; Flags: unchecked
Name: "startonstartup"; Description: "Create a system startup shortcut"; GroupDescription: "{cm:AdditionalIcons}"; Components: baseinstall; Flags: unchecked

[Types]
Name: "custom"; Description: "Custom installation"; Flags: iscustom

[Components]
Name: "baseinstall"; Description: "Includes necessary base files"; Flags: exclusive
Name: "baseinstall\updatecheck"; Description: "Lastest version info checker (recommended)"

[Files]
Source: "wootili-view.exe"; DestDir: "{app}"; Flags: ignoreversion; Components: baseinstall
Source: "update_check.dll"; DestDir: "{app}"; Flags: ignoreversion; Components: baseinstall\updatecheck
; NOTE: Don't use "Flags: ignoreversion" on any shared system files

[Icons]
Name: "{autoprograms}\{#MyAppName}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{autoprograms}\{#MyAppName}\{#MyAppName} Github"; Filename: "{#MyAppURL}"; Tasks: githubicon

Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon
Name: "{commonstartup}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: startonstartup;

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent

