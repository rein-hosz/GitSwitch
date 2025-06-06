#Requires -Version 5.0

# Stop on any error
$ErrorActionPreference = "Stop"

# Configuration
$AppName = "git-switch"
$Version = "" # Initialize Version variable

# Parameter for version override
Param(
    [string]$BuildVersion = ""
)

# Determine version
if ($BuildVersion -ne "") {
    $Version = $BuildVersion
    Write-Host "Using provided version: $Version" -ForegroundColor Yellow
} elseif (Test-Path "Cargo.toml") {
    $CargoContent = Get-Content -Path "Cargo.toml" -Raw
    if ($CargoContent -match 'version\s*=\s*"([^"]+)"') {
        $Version = "v" + $Matches[1] # Add 'v' prefix to match tag format
        Write-Host "Using version from Cargo.toml: $Version" -ForegroundColor Green
    }
}

if ($Version -eq "") {
    # Try to get version from git tag
    try {
        # Ensure git command is available and we are in a git repo
        $GitPathTest = Get-Command git -ErrorAction SilentlyContinue
        if ($GitPathTest) {
            $GitDescribe = git describe --tags --abbrev=0 2>$null
            if ($LASTEXITCODE -eq 0 -and $GitDescribe -ne "") {
                $Version = $GitDescribe
                Write-Host "Using version from git tag: $Version" -ForegroundColor Green
            } else {
                Write-Host "git describe failed or no tags found." -ForegroundColor Yellow
            }
        } else {
            Write-Host "git command not found." -ForegroundColor Yellow
        }
    } catch {
        Write-Host "Git command failed or no tags found, cannot determine version from git. Error: $($_.Exception.Message)" -ForegroundColor Yellow
    }
}

if ($Version -eq "") {
    $Version = "v0.1.0" # Fallback version
    Write-Host "Could not determine version automatically. Using fallback: $Version" -ForegroundColor Yellow
}

$VersionNoV = $Version.TrimStart('v') # Version without 'v' prefix

Write-Host "Building $AppName version $Version (package version $VersionNoV) for Windows..." -ForegroundColor Cyan

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
Write-Host "Building release version of $AppName..." -ForegroundColor Cyan
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Host "Cargo build failed!" -ForegroundColor Red
    exit 1
}
Write-Host "$AppName built successfully." -ForegroundColor Green

# Determine the location of the built executable
$ExePath = ""
$PossibleExePaths = @(
    "target\\release\\$AppName.exe",
    "target\\x86_64-pc-windows-msvc\\release\\$AppName.exe", # Common for MSVC toolchain on GitHub runners
    "target\\x86_64-pc-windows-gnu\\release\\$AppName.exe"  # Common for GNU toolchain
)

foreach ($PathAttempt in $PossibleExePaths) {
    if (Test-Path $PathAttempt) {
        $ExePath = $PathAttempt
        break
    }
}

if ([string]::IsNullOrEmpty($ExePath)) {
    Write-Host "Error: Could not find $AppName.exe in the following checked paths:" -ForegroundColor Red
    $PossibleExePaths | ForEach-Object { Write-Host "  - $_" }
    Write-Host "Listing target\\release contents (if exists):"
    Get-ChildItem -Path "target\\release" -ErrorAction SilentlyContinue
    Write-Host "Listing target\\x86_64-pc-windows-msvc\\release contents (if exists):"
    Get-ChildItem -Path "target\\x86_64-pc-windows-msvc\\release" -ErrorAction SilentlyContinue
    Write-Host "Listing target\\x86_64-pc-windows-gnu\\release contents (if exists):"
    Get-ChildItem -Path "target\\x86_64-pc-windows-gnu\\release" -ErrorAction SilentlyContinue
    exit 1
}
Write-Host "Found executable at: $ExePath" -ForegroundColor Green


# Create output directory for packaging
$PackageDirName = "$AppName-$Version-windows-pkg"
$PackagePath = Join-Path -Path "target" -ChildPath $PackageDirName

if (Test-Path $PackagePath) {
    Write-Host "Removing existing package directory: $PackagePath" -ForegroundColor Yellow
    Remove-Item $PackagePath -Recurse -Force
}
New-Item -ItemType Directory -Path $PackagePath -Force | Out-Null
Write-Host "Created package directory: $PackagePath" -ForegroundColor Green

# Files to include in the ZIP
$SourceExe = $ExePath # Use the dynamically found path
$SourceInstallScript = "install.ps1"
$SourceReadme = "README.md"
$SourceLicense = "LICENSE"

# Copy files to package directory
Write-Host "Copying files to package directory..." -ForegroundColor Cyan
Copy-Item $SourceExe -Destination $PackagePath
Copy-Item $SourceInstallScript -Destination $PackagePath
Copy-Item $SourceReadme -Destination $PackagePath
Copy-Item $SourceLicense -Destination $PackagePath

Write-Host "Files copied:"
Get-ChildItem -Path $PackagePath | ForEach-Object { Write-Host "  $($_.Name)" }

# Create a ZIP archive
$ZipFileName = "$AppName-$Version-windows-amd64.zip"
$ZipPath = Join-Path -Path "target" -ChildPath $ZipFileName

if (Test-Path $ZipPath) {
    Write-Host "Removing existing ZIP file: $ZipPath" -ForegroundColor Yellow
    Remove-Item $ZipPath -Force
}

Write-Host "Creating ZIP archive: $ZipPath ..." -ForegroundColor Cyan
Compress-Archive -Path "$PackagePath\\*" -DestinationPath $ZipPath -Force

if (Test-Path $ZipPath) {
    Write-Host "Windows ZIP package created successfully: $ZipPath" -ForegroundColor Green
} else {
    Write-Host "Error: ZIP package not created at $ZipPath" -ForegroundColor Red
    exit 1
}

# Clean up intermediate package directory
Write-Host "Cleaning up intermediate package directory: $PackagePath" -ForegroundColor Cyan
Remove-Item $PackagePath -Recurse -Force

Write-Host "Build process finished for Windows."
