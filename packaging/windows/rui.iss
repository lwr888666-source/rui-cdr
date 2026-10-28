; Inno Setup script for Rui. Built by CI on windows-latest.
; Variables: /DVersion=x.y.z /DSrcDir=<dir with rui.exe and rui-cli.exe>

#ifndef Version
  #define Version "0.0.0"
#endif
#ifndef SrcDir
  #define SrcDir "..\..\target\release"
#endif

[Setup]
AppId={{7E6E3B5C-4C1E-4F7B-9D7A-2B9D5C1A0E11}
AppName=Rui
AppVersion={#Version}
AppPublisher=CoreSDK
AppCopyright=Copyright (c) 2026 Ighor Toth and the Rui contributors
AppPublisherURL=https://github.com/lwr888666-source/rui-cdr
DefaultDirName={autopf}\Rui
DefaultGroupName=Rui
UsePreviousAppDir=no
ShowLanguageDialog=yes
UninstallDisplayIcon={app}\rui.exe
OutputDir=..\..\dist
OutputBaseFilename=Rui-{#Version}-windows-x86_64-setup
SetupIconFile=..\..\assets\icon\rui.ico
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
PrivilegesRequiredOverridesAllowed=dialog
ChangesAssociations=yes
ChangesEnvironment=yes

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "chinesesimplified"; MessagesFile: "languages\ChineseSimplified.isl"

[CustomMessages]
english.AddToPath=Add rui-cli to PATH
english.CommandLine=Command line:
chinesesimplified.AddToPath=将 rui-cli 添加到 PATH 环境变量
chinesesimplified.CommandLine=命令行工具：

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"
Name: "addtopath"; Description: "{cm:AddToPath}"; GroupDescription: "{cm:CommandLine}"

[Files]
Source: "{#SrcDir}\rui.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SrcDir}\rui-cli.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\ATTRIBUTION.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\LICENSE-MIT"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\LICENSE-APACHE"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\Rui"; Filename: "{app}\rui.exe"
Name: "{group}\Uninstall Rui"; Filename: "{uninstallexe}"
Name: "{autodesktop}\Rui"; Filename: "{app}\rui.exe"; Tasks: desktopicon

[InstallDelete]
Type: files; Name: "{autodesktop}\TraceDraw.lnk"
Type: files; Name: "{userprograms}\TraceDraw\TraceDraw.lnk"
Type: files; Name: "{userprograms}\TraceDraw\Uninstall TraceDraw.lnk"

[Registry]
; .tdraw file association
Root: HKA; Subkey: "Software\Classes\.tdraw"; ValueType: string; ValueName: ""; ValueData: "Rui.Document"; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\Rui.Document"; ValueType: string; ValueName: ""; ValueData: "Rui Document"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\Rui.Document\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\rui.exe,0"
Root: HKA; Subkey: "Software\Classes\Rui.Document\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\rui.exe"" ""%1"""
; "Open with Rui" for .cdr, without stealing the default
Root: HKA; Subkey: "Software\Classes\.cdr\OpenWithProgids"; ValueType: string; ValueName: "Rui.Document"; ValueData: ""; Flags: uninsdeletevalue
; PATH
Root: HKA; Subkey: "{code:PathRegKey}"; ValueType: expandsz; ValueName: "Path"; ValueData: "{olddata};{app}"; Tasks: addtopath; Check: NeedsAddPath('{app}')

[Run]
Filename: "{app}\rui.exe"; Description: "{cm:LaunchProgram,Rui}"; Flags: nowait postinstall skipifsilent shellexec runasoriginaluser

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
