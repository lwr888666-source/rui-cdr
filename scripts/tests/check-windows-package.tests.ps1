$ErrorActionPreference = 'Stop'
$work = Join-Path ([System.IO.Path]::GetTempPath()) ('tracedraw-package-test-' + [guid]::NewGuid())
$source = Join-Path $work 'source'
$dist = Join-Path $work 'dist'
New-Item -ItemType Directory -Force (Join-Path $source 'rui-settings'), $dist | Out-Null
foreach ($name in @('tracedraw.exe', 'tracedraw-cli.exe', 'README.md', 'LICENSE-MIT', 'LICENSE-APACHE', 'rui-settings/README.txt')) {
    Set-Content -LiteralPath (Join-Path $source $name) -Value 'fixture'
}
$installer = Join-Path $dist 'tracedraw-1.2.3-windows-x86_64-setup.exe'
$archive = Join-Path $dist 'tracedraw-1.2.3-windows-x86_64-portable.zip'
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
Remove-Item -LiteralPath (Join-Path $source 'tracedraw-cli.exe')
Repack
Reject 'Portable package is missing: tracedraw-cli.exe'
[System.IO.File]::WriteAllBytes((Join-Path $source 'tracedraw-cli.exe'), [byte[]]@())
Repack
Reject 'Portable package has an empty file: tracedraw-cli.exe'
Remove-Item -LiteralPath $installer
Reject 'Package is missing or empty:'
Write-Output 'Package regression checks passed (valid, missing executable, empty executable, missing installer)'
