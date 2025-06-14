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
        Write-Info "Restoring Cargo.toml from backup..."
        Move-Item -Path $CargoTomlBackupPath -Destination $CargoTomlPath -Force
        Write-Success "Cargo.toml restored"
    }
    exit 1
}
Write-Success "$BinaryName built successfully"

# Clean up Cargo.toml backup if it exists
if (Test-Path $CargoTomlBackupPath) {
    Remove-Item -Path $CargoTomlBackupPath -ErrorAction SilentlyContinue
}

# Locate the built executable
Write-Info "Locating built executable..."

$ExePath = ""
$ProjectRoot = (Get-Location).Path

$PossibleExePaths = @(
    (Join-Path $ProjectRoot "target\release\$BinaryName.exe"),
    (Join-Path $ProjectRoot "target\x86_64-pc-windows-msvc\release\$BinaryName.exe"),
    (Join-Path $ProjectRoot "target\x86_64-pc-windows-gnu\release\$BinaryName.exe")
)

foreach ($PathAttempt in $PossibleExePaths) {
    if (Test-Path -LiteralPath $PathAttempt -PathType Leaf) {
        $ExePath = $PathAttempt
        Write-Success "Found executable at: $ExePath"
        break
    }
}

if ([string]::IsNullOrEmpty($ExePath)) {
    Write-Error "Could not find $BinaryName.exe in expected locations:"
    $PossibleExePaths | ForEach-Object { Write-Host "  - $_" }
    Write-Info "Checking target directory contents..."
    if (Test-Path (Join-Path $ProjectRoot "target\release")) {
        Get-ChildItem -Path (Join-Path $ProjectRoot "target\release") -ErrorAction SilentlyContinue
    }
    exit 1
}

# Create packaging directory
$PackageDir = Join-Path $ProjectRoot "target\release\package"
if (Test-Path $PackageDir) {
    Remove-Item -Recurse -Force $PackageDir
}
New-Item -ItemType Directory -Path $PackageDir | Out-Null

Write-Info "Packaging files..."

# Copy the executable
Copy-Item -Path $ExePath -Destination (Join-Path $PackageDir "$BinaryName.exe")
Write-Success "Copied executable to package"

# Copy supporting files
$FilesToInclude = @("README.md", "LICENSE", "install.ps1")
foreach ($File in $FilesToInclude) {
    $SourceFile = Join-Path $ProjectRoot $File
    if (Test-Path $SourceFile) {
        Copy-Item -Path $SourceFile -Destination (Join-Path $PackageDir $File)
        Write-Success "Copied $File to package"
    } else {
        Write-Warning "$File not found, skipping"
    }
}

# Create the ZIP file
$ZipFileName = "$AppName-$Version-windows-amd64.zip"
$ZipFilePath = Join-Path $ProjectRoot "target\release\$ZipFileName"
if (Test-Path $ZipFilePath) {
    Remove-Item $ZipFilePath
}

Compress-Archive -Path (Join-Path $PackageDir "*") -DestinationPath $ZipFilePath
Write-Success "$ZipFileName created at $ZipFilePath"

# Clean up temporary package directory
Remove-Item -Recurse -Force $PackageDir

Write-Header "Build completed successfully!"
Write-Success "Package: $ZipFileName"
Write-Success "Location: $ZipFilePath"
