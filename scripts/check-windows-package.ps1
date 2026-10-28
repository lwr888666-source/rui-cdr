param(
    [Parameter(Mandatory = $true)][string] $Version,
    [Parameter(Mandatory = $true)][string] $Directory,
    [switch] $Install
)

$ErrorActionPreference = 'Stop'
$installer = Join-Path $Directory "tracedraw-$Version-windows-x86_64-setup.exe"
$archive = Join-Path $Directory "tracedraw-$Version-windows-x86_64-portable.zip"
foreach ($file in @($installer, $archive)) {
    if (-not (Test-Path -LiteralPath $file -PathType Leaf) -or (Get-Item -LiteralPath $file).Length -eq 0) {
        throw "Package is missing or empty: $file"
    }
}

Add-Type -AssemblyName System.IO.Compression.FileSystem
$zip = [System.IO.Compression.ZipFile]::OpenRead((Resolve-Path -LiteralPath $archive).Path)
try {
    $entries = @($zip.Entries | ForEach-Object { $_.FullName.Replace('\', '/') })
    foreach ($required in @('tracedraw.exe', 'tracedraw-cli.exe', 'rui-settings/README.txt', 'README.md', 'LICENSE-MIT', 'LICENSE-APACHE')) {
        if ($required -notin $entries) { throw "Portable package is missing: $required" }
        $entry = $zip.GetEntry($required)
        if (-not $entry) { $entry = $zip.Entries | Where-Object { $_.FullName.Replace('\', '/') -eq $required } | Select-Object -First 1 }
        if ($entry.Length -eq 0) { throw "Portable package has an empty file: $required" }
    }
} finally {
    $zip.Dispose()
}

if ($Install) {
    $work = Join-Path ([System.IO.Path]::GetTempPath()) ('tracedraw-install-check-' + [guid]::NewGuid())
    $portable = Join-Path $work 'portable'
    $installed = Join-Path $work 'installed'
    Expand-Archive -LiteralPath $archive -DestinationPath $portable
    $setup = Start-Process -FilePath (Resolve-Path -LiteralPath $installer).Path -ArgumentList @(
        '/VERYSILENT', '/SUPPRESSMSGBOXES', '/SP-', '/NORESTART', '/CURRENTUSER', '/TASKS=', "/DIR=`"$installed`""
    ) -WindowStyle Hidden -PassThru
    if (-not $setup.WaitForExit(120000)) { $setup.Kill(); throw 'Installer timed out after two minutes' }
    if ($setup.ExitCode -ne 0) { throw "Installer failed: $($setup.ExitCode)" }
    try {
        foreach ($exe in @('tracedraw.exe', 'tracedraw-cli.exe')) {
            $a = Get-FileHash -LiteralPath (Join-Path $portable $exe) -Algorithm SHA256
            $b = Get-FileHash -LiteralPath (Join-Path $installed $exe) -Algorithm SHA256
            if ($a.Hash -ne $b.Hash) { throw "Installed executable differs from portable copy: $exe" }
        }
        $inputFile = Join-Path $work 'sample.svg'
        Set-Content -LiteralPath $inputFile -Value '<svg xmlns="http://www.w3.org/2000/svg" width="10mm" height="10mm"><rect width="10" height="10" fill="red"/></svg>'
        foreach ($programDir in @($portable, $installed)) {
            $outputFile = Join-Path $programDir 'sample.pdf'
            & (Join-Path $programDir 'tracedraw-cli.exe') convert $inputFile $outputFile
            if ($LASTEXITCODE -ne 0) { throw "Installed or portable CLI failed: $programDir" }
            $bytes = [System.IO.File]::ReadAllBytes($outputFile)
            if ($bytes.Length -lt 5 -or [System.Text.Encoding]::ASCII.GetString($bytes, 0, 5) -ne '%PDF-') {
                throw "CLI did not write a PDF: $programDir"
            }
        }
        Write-Output 'Installer and portable conversion checks passed'
    } finally {
        $uninstaller = Join-Path $installed 'unins000.exe'
        if (Test-Path -LiteralPath $uninstaller) {
            $remove = Start-Process -FilePath $uninstaller -ArgumentList '/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART' -WindowStyle Hidden -PassThru
            if (-not $remove.WaitForExit(120000)) { $remove.Kill(); throw 'Uninstaller timed out' }
            if ($remove.ExitCode -ne 0) { throw "Uninstaller failed: $($remove.ExitCode)" }
        }
    }
}
