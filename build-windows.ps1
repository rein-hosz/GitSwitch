#Requires -Version 5.0

# Stop on any error
$ErrorActionPreference = "Stop"

# Configuration
$AppName = "git-switch"
$Version = ""

# Get version from Cargo.toml
if (Test-Path "Cargo.toml") {
    $CargoContent = Get-Content -Path "Cargo.toml" -Raw
    if ($CargoContent -match 'version\s*=\s*"([^"]+)"') {
        $Version = $Matches[1]
    }
}
if ($Version -eq "") {
    $Version = "0.1.0"
}


Write-Host "Building $AppName version $Version for Windows..." -ForegroundColor Cyan

# Ensure Rust is installed
try {
    $RustVersion = rustc --version
    Write-Host "Using $RustVersion" -ForegroundColor Green
} catch {
    Write-Host "Rust is not installed or not in your PATH!" -ForegroundColor Red
    Write-Host "Please install Rust from https://rustup.rs/" -ForegroundColor Red
    exit 1
}

# Build the release version
Write-Host "Building release version..." -ForegroundColor Cyan
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Host "Build failed!" -ForegroundColor Red
    exit 1
}

# Create output directory for packaging
$OutputDir = "target\windows-package"
if (Test-Path $OutputDir) {
    Remove-Item $OutputDir -Recurse -Force
}
New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null

# Copy binary and documentation
Write-Host "Copying files to package directory..." -ForegroundColor Cyan
Copy-Item "target\release\git_switch.exe" $OutputDir
if (Test-Path "README.md") {
    Copy-Item "README.md" $OutputDir
}
if (Test-Path "LICENSE") {
    Copy-Item "LICENSE" $OutputDir
}

# Create ZIP package
$ZipFile = "target\$AppName-$Version-windows.zip"
Write-Host "Creating ZIP package: $ZipFile" -ForegroundColor Cyan
Compress-Archive -Path "$OutputDir\*" -DestinationPath $ZipFile -Force

Write-Host "Package created successfully: $ZipFile" -ForegroundColor Green
Write-Host "You can distribute this ZIP file to Windows users."
Write-Host "To install, they can extract the ZIP and add the folder to their system PATH."
