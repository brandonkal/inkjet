# Build and package the Windows release archive for Inkjet.
# Produces output/inkjet-<version>-<triple>.zip.

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$RepoRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $RepoRoot

Write-Host "Building release binary with cargo build --release --frozen"
cargo build --release --frozen

$Metadata = cargo metadata --no-deps --format-version 1 | ConvertFrom-Json
$Package = $Metadata.packages | Where-Object { $_.name -eq "inkjet" } | Select-Object -First 1
if (-not $Package) {
    throw "Could not determine inkjet package version from cargo metadata."
}
$Version = $Package.version

$RustcVersion = rustc -vV
$HostLine = $RustcVersion | Where-Object { $_ -match "^host:" } | Select-Object -First 1
if (-not $HostLine) {
    throw "Could not determine Rust host triple from rustc -vV."
}
$Triple = ($HostLine -replace "^host:\s*", "").Trim()

$OutputDir = Join-Path $RepoRoot "output"
$StageDir = Join-Path $OutputDir "windows-package"
$ZipPath = Join-Path $OutputDir "inkjet-$Version-$Triple.zip"
$BinaryPath = Join-Path $RepoRoot "target\release\inkjet.exe"
$ReadmePath = Join-Path $RepoRoot "README.md"

if (-not (Test-Path $BinaryPath)) {
    throw "Release binary not found at $BinaryPath."
}
if (-not (Test-Path $ReadmePath)) {
    throw "README.md not found at $ReadmePath."
}

New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null
if (Test-Path $StageDir) {
    Remove-Item -Recurse -Force $StageDir
}
New-Item -ItemType Directory -Force -Path $StageDir | Out-Null

Copy-Item $BinaryPath (Join-Path $StageDir "inkjet.exe")
Copy-Item $ReadmePath (Join-Path $StageDir "README.md")

if (Test-Path $ZipPath) {
    Remove-Item -Force $ZipPath
}

Compress-Archive -Path (Join-Path $StageDir "*") -DestinationPath $ZipPath -CompressionLevel Optimal
Remove-Item -Recurse -Force $StageDir

Write-Host "Created $ZipPath"
