param([switch]$Installer)
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
$output = Join-Path $root 'dist/windows'
$version = (Get-Content "$output/version.txt").Trim()
$stage = Join-Path $env:RUNNER_TEMP 'Leaf prueba ñ con espacios'
if ($Installer) {
    $stage = Join-Path $env:RUNNER_TEMP 'Leaf instalada ñ'
    $setup = "$output/Leaf-$version-x86_64-setup.exe"
    $p = Start-Process $setup -ArgumentList @('/VERYSILENT','/SUPPRESSMSGBOXES','/NORESTART',"/DIR=`"$stage`"") -PassThru -Wait
    if ($p.ExitCode -ne 0) { throw "Installer exit code: $($p.ExitCode)" }
} else {
    if (Test-Path $stage) { Remove-Item $stage -Recurse -Force }
    Expand-Archive "$output/Leaf-$version-x86_64-portable.zip" $stage
    $stage = Join-Path $stage "Leaf-$version-x86_64"
}
# A clean Windows process must not find the developer's GTK, catalogs or data.
$env:PATH = "$env:SystemRoot\system32;$env:SystemRoot"
Remove-Item Env:LEAF_LOCALEDIR -ErrorAction SilentlyContinue
Remove-Item Env:CRAFT_I18N_DIR -ErrorAction SilentlyContinue
Remove-Item Env:GDK_PIXBUF_MODULE_FILE -ErrorAction SilentlyContinue
Remove-Item Env:GDK_PIXBUF_MODULEDIR -ErrorAction SilentlyContinue
$env:APPDATA = Join-Path $env:RUNNER_TEMP 'Leaf roaming ñ'
$env:LOCALAPPDATA = Join-Path $env:RUNNER_TEMP 'Leaf local ñ'
$env:GSK_RENDERER = 'cairo'
$env:LC_ALL = 'en_US.UTF-8'
$exe = Join-Path $stage 'bin/leaf.exe'
$kind = if ($Installer) { 'installed' } else { 'portable' }
function Check-Leaf([string[]]$Arguments, [string]$Name, [string]$Marker) {
    $out = "$output/$kind-$Name.stdout.log"
    $err = "$output/$kind-$Name.stderr.log"
    $p = Start-Process $exe -ArgumentList $Arguments -RedirectStandardOutput $out -RedirectStandardError $err -PassThru
    if (!$p.WaitForExit(90000)) { $p.Kill(); throw "Timed out: $Name" }
    $p.Refresh()
    Get-Content $out
    Get-Content $err
    if ($p.ExitCode -ne 0) { throw "$Name exit code: $($p.ExitCode)" }
    $text = (Get-Content $out -Raw) + (Get-Content $err -Raw)
    if (!$text.Contains($Marker)) { throw "Missing marker $Marker in $Name" }
}
foreach ($language in @('en','es')) {
    $env:LANGUAGE = $language
    Check-Leaf @('--localization-ui-test') "locale-$language" 'LOCALIZATION_UI_OK'
}
$env:LANGUAGE = 'en'
Check-Leaf @('--windows-package-test') 'resources' 'WINDOWS_PACKAGE_RESOURCES_OK'
Check-Leaf @('--multiwindow-test') 'multiwindow' 'MULTIWINDOW_OK'
if ($Installer) {
    $p = Start-Process "$stage/unins000.exe" -ArgumentList @('/VERYSILENT','/SUPPRESSMSGBOXES','/NORESTART') -PassThru -Wait
    if ($p.ExitCode -ne 0 -or (Test-Path $exe)) { throw 'Uninstall failed' }
}
Write-Output "WINDOWS_${kind}_TEST_OK"
