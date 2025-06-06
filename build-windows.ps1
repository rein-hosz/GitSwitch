#Requires -Version 5.0

# Parameter for version override
Param(
    [string]$BuildVersion = ""
)

# Stop on any error
$ErrorActionPreference = "Stop"

# Configuration
$AppName = "git-switch" # User-facing name, used for ZIP file and is the actual binary name
$BinaryName = "git-switch" # Actual name of the compiled executable, aligning with Cargo.toml package name
$Version = "" # Initialize Version variable
$OriginalCargoTomlContent = ""
$ProjectRoot = (Get-Location).Path
$CargoTomlPath = Join-Path $ProjectRoot "Cargo.toml"
$CargoTomlBackupPath = $CargoTomlPath + ".bak"


# Determine version
if ($BuildVersion -ne "") {
    $Version = $BuildVersion # Expects "vX.Y.Z" or "X.Y.Z"
    Write-Host "Using provided version: $Version" -ForegroundColor Yellow
} elseif (Test-Path $CargoTomlPath) {
    $CargoContent = Get-Content -Path $CargoTomlPath -Raw
    if ($CargoContent -match 'version\s*=\s*"([^"]+)"') {
        $Version = "v" + $Matches[1] # Add 'v' prefix if not already there
        Write-Host "Using version from Cargo.toml: $Version" -ForegroundColor Green
    }
}

if ($Version -eq "") {
    try {
        $GitPathTest = Get-Command git -ErrorAction SilentlyContinue
        if ($GitPathTest) {
            $GitDescribe = git describe --tags --abbrev=0 2>$null
            if ($LASTEXITCODE -eq 0 -and $GitDescribe -ne "") {
                $Version = $GitDescribe # Assumes git tag is like vX.Y.Z
                Write-Host "Using version from git tag: $Version" -ForegroundColor Green
            } else {
                Write-Host "git describe failed or no tags found." -ForegroundColor Yellow
            }
        } else {
            Write-Host "git command not found." -ForegroundColor Yellow
        }
    } catch {
        Write-Host "Git command failed or no tags found. Error: $($_.Exception.Message)" -ForegroundColor Yellow
    }
}

if ($Version -eq "") {
    $Version = "v0.1.0" # Fallback version
    Write-Host "Could not determine version automatically. Using fallback: $Version" -ForegroundColor Yellow
}

# Ensure $Version starts with 'v' for consistency in this script, $VersionNoV is without.
if (-not $Version.StartsWith("v")) {
    $Version = "v" + $Version
}
$VersionNoV = $Version.TrimStart('v') # Version without 'v' prefix, e.g., 0.1.0

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

# Update Cargo.toml with the determined version (without 'v')
if (Test-Path $CargoTomlPath) {
    Write-Host "Attempting to update Cargo.toml version to: $VersionNoV"
    $OriginalCargoTomlContent = Get-Content -Path $CargoTomlPath -Raw
    Copy-Item -Path $CargoTomlPath -Destination $CargoTomlBackupPath -Force
    
    $Pattern = '(?m)^(version\s*=\s*")[^"]*(")' # Matches version = "..."
    # Corrected replacement string using ${1} and ${2} for backreferences
    $Replacement = "`${1}$($VersionNoV)`${2}" 

    if ($OriginalCargoTomlContent -match $Pattern) { # -match is used here just to confirm pattern finds something, $Matches is not used for -replace
        $UpdatedCargoTomlContent = $OriginalCargoTomlContent -replace $Pattern, $Replacement
        if ($UpdatedCargoTomlContent -ne $OriginalCargoTomlContent) {
            Set-Content -Path $CargoTomlPath -Value $UpdatedCargoTomlContent -Encoding UTF8
            Write-Host "Cargo.toml version updated to $VersionNoV."
            Write-Host "Verifying update:"
            # Attempt to parse the Cargo.toml with cargo itself to see if it's valid
            $CargoReadOutput = cargo read-manifest --manifest-path $CargoTomlPath 2>&1
            if ($LASTEXITCODE -eq 0) {
                Write-Host "Cargo.toml successfully parsed by 'cargo read-manifest'."
                # Correctly escape quotes in the pattern for Select-String
                Get-Content -Path $CargoTomlPath | Select-String -Pattern "^version\s*=\s*\`"$([regex]::Escape($VersionNoV))\`"" -Quiet
            } else {
                Write-Host "Error: 'cargo read-manifest' failed after updating Cargo.toml. Content might be invalid." -ForegroundColor Red
                Write-Host "Cargo read-manifest output: $CargoReadOutput"
                Write-Host "Reverting Cargo.toml from backup..."
                Move-Item -Path $CargoTomlBackupPath -Destination $CargoTomlPath -Force -ErrorAction SilentlyContinue
                Write-Host "Cargo.toml restored."
                exit 1 # Exit because the Cargo.toml modification failed
            }
        } else {
            Write-Host "Cargo.toml version is already $VersionNoV or pattern did not result in a change."
            Remove-Item -Path $CargoTomlBackupPath -ErrorAction SilentlyContinue
        }
    } else {
        Write-Host "Warning: Main 'version = ""...""' line not found in Cargo.toml using pattern: $Pattern" -ForegroundColor Yellow
        Remove-Item -Path $CargoTomlBackupPath -ErrorAction SilentlyContinue
    }
} else {
    Write-Host "Error: $CargoTomlPath not found. Cannot update version." -ForegroundColor Red
    # Consider exiting if this is critical: exit 1
}

# Build the release version
Write-Host "Building release version of $BinaryName (using version $VersionNoV from Cargo.toml)..." -ForegroundColor Cyan
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Host "Cargo build failed!" -ForegroundColor Red
    if (Test-Path $CargoTomlBackupPath) {
        Write-Host "Restoring Cargo.toml from backup..."
        Move-Item -Path $CargoTomlBackupPath -Destination $CargoTomlPath -Force
        Write-Host "Cargo.toml restored."
    }
    exit 1
}
Write-Host "$BinaryName built successfully." -ForegroundColor Green

# Clean up Cargo.toml backup if it exists and build was successful
if (Test-Path $CargoTomlBackupPath) {
    Remove-Item -Path $CargoTomlBackupPath -ErrorAction SilentlyContinue
}

Write-Host "Current working directory before checking for EXE: $((Get-Location).Path)" -ForegroundColor Magenta

# Determine the location of the built executable
$ExePath = ""
# $ProjectRoot is already defined above as (Get-Location).Path

$PossibleExePaths = @(
    (Join-Path $ProjectRoot "target\\\\release\\\\$BinaryName.exe"),
    (Join-Path $ProjectRoot "target\\\\x86_64-pc-windows-msvc\\\\release\\\\$BinaryName.exe"),
    (Join-Path $ProjectRoot "target\\\\x86_64-pc-windows-gnu\\\\release\\\\$BinaryName.exe")
)

foreach ($PathAttempt in $PossibleExePaths) {
    Write-Host "Testing path: $PathAttempt" -ForegroundColor Gray
    if (Test-Path -LiteralPath $PathAttempt -PathType Leaf) {
        $ExePath = $PathAttempt
        Write-Host "Found executable at: $ExePath" -ForegroundColor Green
        break
    } else {
        Write-Host "Path not found or not a file: $PathAttempt" -ForegroundColor Yellow
    }
}

if ([string]::IsNullOrEmpty($ExePath)) {
    Write-Host "Error: Could not find $BinaryName.exe in the following checked paths:" -ForegroundColor Red
    $PossibleExePaths | ForEach-Object { Write-Host "  - $_" }
    Write-Host "Listing target\\\\release contents (if exists):"
    Get-ChildItem -Path (Join-Path $ProjectRoot "target\\\\release") -ErrorAction SilentlyContinue
    Write-Host "Listing target\\\\x86_64-pc-windows-msvc\\\\release contents (if exists):"
    Get-ChildItem -Path (Join-Path $ProjectRoot "target\\\\x86_64-pc-windows-msvc\\\\release") -ErrorAction SilentlyContinue
    Write-Host "Listing target\\\\x86_64-pc-windows-gnu\\\\release contents (if exists):"
    Get-ChildItem -Path (Join-Path $ProjectRoot "target\\\\x86_64-pc-windows-gnu\\\\release") -ErrorAction SilentlyContinue
    exit 1
}

# Create a temporary directory for packaging
$PackageDir = Join-Path $ProjectRoot "target\\\\release\\\\package"
if (Test-Path $PackageDir) {
    Remove-Item -Recurse -Force $PackageDir
}
New-Item -ItemType Directory -Path $PackageDir | Out-Null

Write-Host "Packaging files..." -ForegroundColor Cyan

# Copy the executable
Write-Host "Copying $ExePath to $PackageDir"
Copy-Item -Path $ExePath -Destination (Join-Path $PackageDir "$BinaryName.exe") # Use BinaryName for the .exe

# Copy other files (README, LICENSE, install.ps1)
$FilesToInclude = @("README.md", "LICENSE", "install.ps1")
foreach ($File in $FilesToInclude) {
    $SourceFile = Join-Path $ProjectRoot $File
    if (Test-Path $SourceFile) {
        Write-Host "Copying $SourceFile to $PackageDir"
        Copy-Item -Path $SourceFile -Destination (Join-Path $PackageDir $File)
    } else {
        Write-Host "Warning: $SourceFile not found, skipping." -ForegroundColor Yellow
    }
}

# Create the ZIP file
# $ZipFileName = "$AppName-$Version.zip" # ZIP name uses AppName (git-switch)
$ZipFileName = "$AppName-$Version-windows-amd64.zip" # Align with GitHub Actions artifact naming
$ZipFilePath = Join-Path $ProjectRoot "target\\release\\$ZipFileName"
if (Test-Path $ZipFilePath) {
    Remove-Item $ZipFilePath
}
Compress-Archive -Path (Join-Path $PackageDir "*") -DestinationPath $ZipFilePath
Write-Host "$ZipFileName created successfully at $ZipFilePath" -ForegroundColor Green

# Clean up the temporary package directory
Remove-Item -Recurse -Force $PackageDir
Write-Host "Build and packaging complete for $AppName version $Version." -ForegroundColor Cyan
