$ErrorActionPreference = 'Stop'
$work = Join-Path ([System.IO.Path]::GetTempPath()) ('rui-package-test-' + [guid]::NewGuid())
$source = Join-Path $work 'source'
$dist = Join-Path $work 'dist'
New-Item -ItemType Directory -Force (Join-Path $source 'rui-settings'), $dist | Out-Null
foreach ($name in @('rui.exe', 'rui-cli.exe', 'README.md', 'ATTRIBUTION.md', 'LICENSE-MIT', 'LICENSE-APACHE', 'rui-settings/README.txt')) {
    Set-Content -LiteralPath (Join-Path $source $name) -Value 'fixture'
}
$installer = Join-Path $dist 'Rui-1.2.3-windows-x86_64-setup.exe'
$archive = Join-Path $dist 'Rui-1.2.3-windows-x86_64-portable.zip'
Set-Content -LiteralPath $installer -Value 'fixture'
$check = Join-Path $PSScriptRoot '../check-windows-package.ps1'

function Repack {
    Compress-Archive -Path "$source/*" -DestinationPath $archive -Force
}
function Reject([string] $reason) {
    try {
        & $check -Version '1.2.3' -Directory $dist
    } catch {
        if ($_.Exception.Message -like "*$reason*") { return }
        throw
    }
    throw "Invalid package was accepted: $reason"
}

Repack
& $check -Version '1.2.3' -Directory $dist
Remove-Item -LiteralPath (Join-Path $source 'rui-cli.exe')
Repack
Reject 'Portable package is missing: rui-cli.exe'
[System.IO.File]::WriteAllBytes((Join-Path $source 'rui-cli.exe'), [byte[]]@())
Repack
Reject 'Portable package has an empty file: rui-cli.exe'
Remove-Item -LiteralPath $installer
Reject 'Package is missing or empty:'
Write-Output 'Package regression checks passed (valid, missing executable, empty executable, missing installer)'
