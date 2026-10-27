; Inno Setup script for rui-ColeSDK (rui-cdr). Built by CI on windows-latest.
; Variables: /DVersion=x.y.z /DSrcDir=<dir with rui-cdr.exe and rui-cdr-cli.exe>

#ifndef Version
  #define Version "0.19.6"
#endif
#ifndef SrcDir
  #define SrcDir "..\..\target\release"
#endif

[Setup]
AppId={{A8E72D1B-5F2C-4E8A-8C3D-3B8E6A2C1F22}
AppName=Rui
AppVersion={#Version}
AppPublisher=CoreSDK
AppCopyright=Copyright (c) 2026 CoreSDK. All rights reserved.
AppPublisherURL=https://github.com/rui/rui-cdr
DefaultDirName={autopf}\Rui
DefaultGroupName=Rui
UninstallDisplayIcon={app}\rui-cdr.exe
OutputDir=..\..\dist
OutputBaseFilename=Rui-{#Version}-windows-x86_64-setup
SetupIconFile=..\..\assets\icon\tracedraw.ico
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
PrivilegesRequiredOverridesAllowed=dialog
ChangesAssociations=yes
ChangesEnvironment=yes

[Languages]
Name: "chinesesimplified"; MessagesFile: "compiler:Languages\ChineseSimplified.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"
Name: "addtopath"; Description: "将 rui-cdr-cli 添加到环境变量 PATH"; GroupDescription: "命令行工具:"

[Files]
Source: "{#SrcDir}\rui-cdr.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SrcDir}\rui-cdr-cli.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\LICENSE-MIT"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\LICENSE-APACHE"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\Rui"; Filename: "{app}\rui-cdr.exe"
Name: "{group}\卸载 Rui"; Filename: "{uninstallexe}"
Name: "{autodesktop}\Rui"; Filename: "{app}\rui-cdr.exe"; Tasks: desktopicon

[Registry]
; .tdraw file association
Root: HKA; Subkey: "Software\Classes\.tdraw"; ValueType: string; ValueName: ""; ValueData: "Rui.Document"; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\Rui.Document"; ValueType: string; ValueName: ""; ValueData: "Rui Document"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\Rui.Document\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\rui-cdr.exe,0"
Root: HKA; Subkey: "Software\Classes\Rui.Document\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\rui-cdr.exe"" ""%1"""
; "Open with Rui" for .cdr
Root: HKA; Subkey: "Software\Classes\.cdr\OpenWithProgids"; ValueType: string; ValueName: "Rui.Document"; ValueData: ""; Flags: uninsdeletevalue
; PATH
Root: HKA; Subkey: "{code:PathRegKey}"; ValueType: expandsz; ValueName: "Path"; ValueData: "{olddata};{app}"; Tasks: addtopath; Check: NeedsAddPath('{app}')

[Run]
Filename: "{app}\rui-cdr.exe"; Description: "{cm:LaunchProgram,Rui}"; Flags: nowait postinstall skipifsilent shellexec runasoriginaluser

[Code]
function PathRegKey(Param: string): string;
begin
  if IsAdminInstallMode then
    Result := 'SYSTEM\CurrentControlSet\Control\Session Manager\Environment'
  else
    Result := 'Environment';
end;

function NeedsAddPath(Param: string): boolean;
var
  OrigPath: string;
  Root: Integer;
begin
  if IsAdminInstallMode then Root := HKLM else Root := HKCU;
  if not RegQueryStringValue(Root, PathRegKey(''), 'Path', OrigPath) then
  begin
    Result := True;
    exit;
  end;
  Result := Pos(';' + Uppercase(ExpandConstant(Param)) + ';', ';' + Uppercase(OrigPath) + ';') = 0;
end;
