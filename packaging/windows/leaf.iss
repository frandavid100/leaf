#ifndef LeafVersion
  #error LeafVersion must identify the tested package
#endif
[Setup]
AppId=io.github.frandavid100.leaf.windows
AppName=Leaf
AppVersion={#LeafVersion}
AppPublisher=David Prieto
AppPublisherURL=https://github.com/frandavid100/leaf
DefaultDirName={localappdata}\Programs\Leaf
DefaultGroupName=Leaf
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0
OutputDir=..\..\dist\windows
OutputBaseFilename=Leaf-{#LeafVersion}-x86_64-setup
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
UninstallDisplayIcon={app}\bin\leaf.exe
CloseApplications=yes
LicenseFile=..\..\LICENSE
[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "spanish"; MessagesFile: "compiler:Languages\Spanish.isl"
[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
[Files]
Source: "..\..\dist\windows\Leaf-{#LeafVersion}-x86_64\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs
[Icons]
Name: "{group}\Leaf"; Filename: "{app}\bin\leaf.exe"
Name: "{autodesktop}\Leaf"; Filename: "{app}\bin\leaf.exe"; Tasks: desktopicon
[Run]
Filename: "{app}\bin\leaf.exe"; Description: "{cm:LaunchProgram,Leaf}"; Flags: nowait postinstall skipifsilent
