param(
    [Parameter(Mandatory = $true)][string] $Version,
    [Parameter(Mandatory = $true)][string] $Directory
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
    foreach ($required in @('tracedraw.exe', 'tracedraw-cli.exe', 'tracedraw-settings/README.txt', 'README.md', 'LICENSE-MIT', 'LICENSE-APACHE')) {
        if ($required -notin $entries) { throw "Portable package is missing: $required" }
    }
} finally {
    $zip.Dispose()
}
