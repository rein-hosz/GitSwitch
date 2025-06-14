#Requires -Version 5.0

<#
.SYNOPSIS
    GitSwitch Build Script for Windows

.DESCRIPTION
    Builds GitSwitch for Windows with automatic version detection and packaging.
    Creates a ZIP file with the executable and supporting files.

.PARAMETER BuildVersion
    Override the version to use for building (e.g., "v1.0.0" or "1.0.0")

.PARAMETER SkipTests
    Skip running tests before building

.PARAMETER Clean
    Clean build (removes target directory)

.PARAMETER Help
    Show help information

.EXAMPLE
    .\build-windows.ps1
    
.EXAMPLE
    .\build-windows.ps1 -BuildVersion "v1.2.3"
    
.EXAMPLE
    .\build-windows.ps1 -Clean -SkipTests
#>

Param(
    [string]$BuildVersion = "",
    [switch]$SkipTests = $false,
    [switch]$Clean = $false,
    [switch]$Help = $false
)

# Stop on any error
$ErrorActionPreference = "Stop"

# Color functions for better output
function Write-Info { param($Message) Write-Host "ℹ️  $Message" -ForegroundColor Blue }
function Write-Success { param($Message) Write-Host "✅ $Message" -ForegroundColor Green }
function Write-Warning { param($Message) Write-Host "⚠️  $Message" -ForegroundColor Yellow }
function Write-Error { param($Message) Write-Host "❌ $Message" -ForegroundColor Red }
function Write-Header { param($Message) Write-Host "🔨 $Message" -ForegroundColor Cyan }

# Show usage information
function Show-Usage {
    Write-Host "GitSwitch Build Script for Windows" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Usage: .\build-windows.ps1 [options]" -ForegroundColor White
    Write-Host ""
    Write-Host "Options:" -ForegroundColor Yellow
    Write-Host "  -BuildVersion <version>  Specify version (e.g., 'v1.0.0' or '1.0.0')"
    Write-Host "  -SkipTests               Skip running tests before building"
    Write-Host "  -Clean                   Clean build (removes target directory)"
    Write-Host "  -Help                    Show this help message"
    Write-Host ""
    Write-Host "Examples:" -ForegroundColor Yellow
    Write-Host "  .\build-windows.ps1"
    Write-Host "  .\build-windows.ps1 -BuildVersion 'v1.2.3'"
    Write-Host "  .\build-windows.ps1 -Clean -SkipTests"
}

if ($Help) {
    Show-Usage
    exit 0
}

Write-Header "GitSwitch Windows Build Script"

# Configuration
$AppName = "git-switch"
$BinaryName = "git-switch"
$Version = ""
$OriginalCargoTomlContent = ""
$ProjectRoot = (Get-Location).Path
$CargoTomlPath = Join-Path $ProjectRoot "Cargo.toml"
$CargoTomlBackupPath = $CargoTomlPath + ".bak"


# Clean build if requested
if ($Clean) {
    Write-Info "Cleaning build directory..."
    if (Test-Path (Join-Path $ProjectRoot "target")) {
        Remove-Item -Recurse -Force (Join-Path $ProjectRoot "target")
        Write-Success "Build directory cleaned"
    }
}

# Determine version
if ($BuildVersion -ne "") {
    $Version = $BuildVersion
    Write-Info "Using provided version: $Version"
} elseif (Test-Path $CargoTomlPath) {
    $CargoContent = Get-Content -Path $CargoTomlPath -Raw
    if ($CargoContent -match 'version\s*=\s*"([^"]+)"') {
        $Version = "v" + $Matches[1]
        Write-Success "Using version from Cargo.toml: $Version"
    }
}

if ($Version -eq "") {
    try {
        if (Get-Command git -ErrorAction SilentlyContinue) {
            $GitDescribe = git describe --tags --abbrev=0 2>$null
            if ($LASTEXITCODE -eq 0 -and $GitDescribe -ne "") {
                $Version = $GitDescribe
                Write-Success "Using version from git tag: $Version"
            } else {
                Write-Warning "git describe failed or no tags found"
            }
        } else {
            Write-Warning "git command not found"
        }
    } catch {
        Write-Warning "Git command failed: $($_.Exception.Message)"
    }
}

if ($Version -eq "") {
    $Version = "v0.1.0"
    Write-Warning "Could not determine version automatically. Using fallback: $Version"
}

# Ensure version format consistency
if (-not $Version.StartsWith("v")) {
    $Version = "v" + $Version
}
$VersionNoV = $Version.TrimStart('v')

Write-Header "Building $AppName version $Version for Windows"

# Check Rust installation
try {
    $RustVersion = rustc --version
    Write-Success "Using $RustVersion"
} catch {
    Write-Error "Rust is not installed or not in your PATH!"
    Write-Error "Please install Rust from https://rustup.rs/"
    exit 1
}

# Run tests if not skipped
if (-not $SkipTests) {
    Write-Info "Running tests..."
    cargo test --release
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Tests failed!"
        exit 1
    }
    Write-Success "All tests passed"
}

# Update Cargo.toml with the determined version
if (Test-Path $CargoTomlPath) {
    Write-Info "Updating Cargo.toml version to: $VersionNoV"
    $OriginalCargoTomlContent = Get-Content -Path $CargoTomlPath -Raw
    Copy-Item -Path $CargoTomlPath -Destination $CargoTomlBackupPath -Force
    
    $Pattern = '(?m)^(version\s*=\s*")[^"]*(")' 
    $Replacement = "`${1}$($VersionNoV)`${2}" 

    if ($OriginalCargoTomlContent -match $Pattern) {
        $UpdatedCargoTomlContent = $OriginalCargoTomlContent -replace $Pattern, $Replacement
        if ($UpdatedCargoTomlContent -ne $OriginalCargoTomlContent) {
            Set-Content -Path $CargoTomlPath -Value $UpdatedCargoTomlContent -Encoding UTF8
            Write-Success "Cargo.toml version updated to $VersionNoV"
            
            # Verify the update
            $CargoReadOutput = cargo read-manifest --manifest-path $CargoTomlPath 2>&1
            if ($LASTEXITCODE -ne 0) {
                Write-Error "Invalid Cargo.toml after update"
                Write-Error "Cargo output: $CargoReadOutput"
                Write-Info "Reverting Cargo.toml from backup..."
                Move-Item -Path $CargoTomlBackupPath -Destination $CargoTomlPath -Force -ErrorAction SilentlyContinue
                Write-Success "Cargo.toml restored"
                exit 1
            }
        } else {
            Write-Info "Cargo.toml version is already $VersionNoV"
            Remove-Item -Path $CargoTomlBackupPath -ErrorAction SilentlyContinue
        }
    } else {
        Write-Warning "Could not find version pattern in Cargo.toml"
        Remove-Item -Path $CargoTomlBackupPath -ErrorAction SilentlyContinue
    }
} else {
    Write-Error "$CargoTomlPath not found"
}

# Build the release version
Write-Info "Building release version..."
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Error "Cargo build failed!"
    if (Test-Path $CargoTomlBackupPath) {
        Write-Info "Reverting Cargo.toml from backup..."
        Move-Item -Path $CargoTomlBackupPath -Destination $CargoTomlPath -Force -ErrorAction SilentlyContinue
        Write-Success "Cargo.toml restored"
    }
    exit 1
}
Write-Success "Cargo build completed successfully"

$BinaryPath = Join-Path $ProjectRoot "target\\release\\$($BinaryName).exe"
if (-not (Test-Path $BinaryPath)) {
    Write-Error "Binary not found at $BinaryPath"
    exit 1
}
Write-Success "Binary built successfully at $BinaryPath"

# Create target directory for packages if it doesn't exist
$PackagesDir = Join-Path $ProjectRoot "target\\packages"
if (-not (Test-Path $PackagesDir)) {
    New-Item -ItemType Directory -Path $PackagesDir -Force | Out-Null
}

# Create ZIP package
$ZipFileName = "$($AppName)-$($Version)-windows-amd64.zip"
$ZipFilePath = Join-Path $PackagesDir $ZipFileName
Write-Info "Creating ZIP package: $ZipFilePath"

# Create a temporary directory for ZIP contents
$TempZipDir = Join-Path $ProjectRoot "target\\zip-temp"
if (Test-Path $TempZipDir) {
    Remove-Item -Recurse -Force $TempZipDir
}
New-Item -ItemType Directory -Path $TempZipDir -Force | Out-Null
Copy-Item -Path $BinaryPath -Destination $TempZipDir
Copy-Item -Path (Join-Path $ProjectRoot "README.md") -Destination $TempZipDir -ErrorAction SilentlyContinue
Copy-Item -Path (Join-Path $ProjectRoot "LICENSE") -Destination $TempZipDir -ErrorAction SilentlyContinue
# Add installer script if it exists
if (Test-Path (Join-Path $ProjectRoot "install.ps1")) {
    Copy-Item -Path (Join-Path $ProjectRoot "install.ps1") -Destination $TempZipDir
}

Compress-Archive -Path (Join-Path $TempZipDir "*") -DestinationPath $ZipFilePath -Force
Write-Success "ZIP package created: $ZipFilePath"
Remove-Item -Recurse -Force $TempZipDir


# Build MSI package using WiX
Write-Header "Building MSI Package with WiX Toolset"

$WixDir = $env:WIX # Standard environment variable for WiX Toolset
if (-not $WixDir -or -not (Test-Path (Join-Path $WixDir "bin\\candle.exe"))) {
    Write-Warning "WiX Toolset not found or WIX environment variable not set."
    Write-Warning "Attempting to find WiX in common Program Files locations..."
    $ProgramFilesPaths = @(
        "$env:ProgramFiles (x86)\\WiX Toolset v3.11", # Common for v3.x
        "$env:ProgramFiles\\WiX Toolset v3.11",
        "$env:ProgramFiles (x86)\\WiX Toolset v4.0", # Common for v4.x
        "$env:ProgramFiles\\WiX Toolset v4.0"
    )
    foreach ($PathItem in $ProgramFilesPaths) {
        if (Test-Path (Join-Path $PathItem "bin\\candle.exe")) {
            $WixDir = $PathItem
            Write-Info "Found WiX Toolset at: $WixDir"
            break
        }
    }
}

if (-not $WixDir -or -not (Test-Path (Join-Path $WixDir "bin\\candle.exe"))) {
    Write-Error "WiX Toolset (candle.exe, light.exe) not found. Please install it and ensure WIX environment variable is set or it's in a standard Program Files location."
    Write-Error "Download from: https://wixtoolset.org/releases/"
    # Optionally, restore Cargo.toml here if you want to halt completely
    # if (Test-Path $CargoTomlBackupPath) { Move-Item -Path $CargoTomlBackupPath -Destination $CargoTomlPath -Force }
    exit 1 # Or continue without MSI
}

$CandleExe = Join-Path $WixDir "bin\\candle.exe"
$LightExe = Join-Path $WixDir "bin\\light.exe"
$WxsFile = Join-Path $ProjectRoot "wix\\git-switch.wxs"
$WixObjDir = Join-Path $ProjectRoot "target\\wixobj"
$MsiOutDir = $PackagesDir # Place MSI alongside ZIP

if (-not (Test-Path $WxsFile)) {
    Write-Error "WiX source file not found: $WxsFile"
    exit 1
}

# Ensure WiX object directory is clean and accessible
if (Test-Path $WixObjDir) {
    Write-Info "Removing existing WiX object directory: $WixObjDir"
    Remove-Item -Recurse -Force $WixObjDir -ErrorAction SilentlyContinue # Allow to continue if it fails (e.g. dir not empty by another process)
}
Write-Info "Creating WiX object directory: $WixObjDir"
New-Item -ItemType Directory -Path $WixObjDir -Force | Out-Null

# Ensure MSI output directory exists
New-Item -ItemType Directory -Path $MsiOutDir -Force -ErrorAction SilentlyContinue | Out-Null

$MsiFileName = "$($AppName)-$($VersionNoV)-windows-amd64.msi" # Use VersionNoV for MSI
$MsiFilePath = Join-Path $MsiOutDir $MsiFileName

Write-Info "Compiling WiX source file: $WxsFile"
$WixProductVersion = $VersionNoV 
$BinarySourceDir = Join-Path $ProjectRoot "target\\release"

# Define arguments for candle.exe
$CandleArgs = @(
    $WxsFile,
    "-out",
    $WixObjDir,
    "-dProductVersion_WIX=$WixProductVersion",
    "-dBinarySourceDir=$BinarySourceDir"
)

Write-Host "Running: $CandleExe $($CandleArgs -join ' ')" # Log the command for readability
& $CandleExe @CandleArgs # Execute using call operator and splatting

if ($LASTEXITCODE -ne 0) {
    Write-Error "WiX Candle compilation failed."
    exit 1
}
Write-Success "WiX source compiled successfully."

Write-Info "Linking WiX object files to create MSI: $MsiFilePath"

# Define arguments for light.exe
$LightArgs = @(
    (Join-Path $WixObjDir "git-switch.wixobj"), # Path to .wixobj file
    "-out",
    $MsiFilePath,
    "-ext",
    "WixUIExtension",
    "-ext",
    "WixUtilExtension"
)

Write-Host "Running: $LightExe $($LightArgs -join ' ')" # Log the command for readability
& $LightExe @LightArgs # Execute using call operator and splatting

if ($LASTEXITCODE -ne 0) {
    Write-Error "WiX Light linking failed."
    exit 1
}
Write-Success "MSI package created successfully: $MsiFilePath"

# Clean up WiX object directory
Remove-Item -Recurse -Force $WixObjDir -ErrorAction SilentlyContinue

# Restore Cargo.toml if it was backed up
if (Test-Path $CargoTomlBackupPath) {
    Write-Info "Restoring original Cargo.toml..."
    Move-Item -Path $CargoTomlBackupPath -Destination $CargoTomlPath -Force -ErrorAction SilentlyContinue
    Write-Success "Cargo.toml restored"
}

Write-Header "Windows Build and Packaging Complete!"
Write-Success "Artifacts available in: $PackagesDir"
Write-Host "  - ZIP: $ZipFilePath"
Write-Host "  - MSI: $MsiFilePath"
