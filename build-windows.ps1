#Requires -Version 5.0

<#
.SYNOPSIS
    Builds git-switch for Windows
.DESCRIPTION
    This script builds git-switch and creates an installable package for Windows
.EXAMPLE
    .\build-windows.ps1
#>

# Stop on any error
$ErrorActionPreference = "Stop"

# Configuration
$AppName = "git-switch"
$Version = ""

# Get version from Cargo.toml
if ($Version -eq "") {
    $CargoContent = Get-Content -Path "Cargo.toml" -Raw
    if ($CargoContent -match 'version\s*=\s*"([^"]+)"') {
        $Version = $Matches[1]
    } else {
        $Version = "0.1.0"
    }
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

# Create output directory
$OutputDir = "target\windows-package\$AppName-$Version"
New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null
New-Item -ItemType Directory -Path "$OutputDir\bin" -Force | Out-Null

# Copy binary
Write-Host "Copying files to package directory..." -ForegroundColor Cyan
Copy-Item "target\release\git_switch.exe" "$OutputDir\bin\git-switch.exe"

# Copy documentation
if (Test-Path "README.md") {
    Copy-Item "README.md" "$OutputDir\"
}
if (Test-Path "LICENSE") {
    Copy-Item "LICENSE" "$OutputDir\"
}

# Create installation script
Write-Host "Creating installation script..." -ForegroundColor Cyan
$InstallScript = @'
@echo off
echo Installing git-switch...

:: Check for admin privileges
net session >nul 2>&1
if %errorLevel% neq 0 (
    echo This installation requires administrator privileges.
    echo Please run this script as administrator.
    pause
    exit /b 1
)

:: Create program files directory if it doesn't exist
if not exist "%ProgramFiles%\git-switch" mkdir "%ProgramFiles%\git-switch"

:: Copy files
echo Copying files...
copy /Y "bin\git-switch.exe" "%ProgramFiles%\git-switch\"
if exist "README.md" copy /Y "README.md" "%ProgramFiles%\git-switch\"
if exist "LICENSE" copy /Y "LICENSE" "%ProgramFiles%\git-switch\"

:: Add to PATH
echo Adding to PATH...
setx PATH "%PATH%;%ProgramFiles%\git-switch" /M

echo.
echo Installation complete!
echo You may need to restart your command prompt to use git-switch.
echo.
pause
'@

$InstallScript | Out-File -FilePath "$OutputDir\install.bat" -Encoding ascii

# Create uninstallation script
Write-Host "Creating uninstallation script..." -ForegroundColor Cyan
$UninstallScript = @'
@echo off
echo Uninstalling git-switch...

:: Check for admin privileges
net session >nul 2>&1
if %errorLevel% neq 0 (
    echo This uninstallation requires administrator privileges.
    echo Please run this script as administrator.
    pause
    exit /b 1
)

:: Remove from PATH (this is a bit complex)
echo Removing from PATH...
for /f "tokens=2*" %%a in ('reg query "HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Environment" /v PATH') do (
    set "oldPath=%%b"
)
set "newPath=%oldPath:;%ProgramFiles%\git-switch=%"
reg add "HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Environment" /v PATH /t REG_EXPAND_SZ /d "%newPath%" /f

:: Remove files
echo Removing files...
if exist "%ProgramFiles%\git-switch" (
    rmdir /s /q "%ProgramFiles%\git-switch"
)

echo.
echo Uninstallation complete!
echo You may need to restart your command prompt for the PATH changes to take effect.
echo.
pause
'@

$UninstallScript | Out-File -FilePath "$OutputDir\uninstall.bat" -Encoding ascii

# Create README for installation
$InstallReadme = @"
# Git-Switch for Windows

## Installation Instructions

1. Right-click on `install.bat` and select "Run as administrator"
2. Follow the prompts to complete installation
3. Restart your command prompt or PowerShell

After installation, you can use git-switch from any command prompt or PowerShell window:

```
git-switch --help
git-switch add "Personal" "username" "email@example.com"
git-switch use "Personal"
git-switch list
```

## Uninstallation

To uninstall, right-click on `uninstall.bat` and select "Run as administrator".

## Manual Installation

If you prefer to install manually:

1. Copy `bin\git-switch.exe` to a location of your choice
2. Add that location to your PATH environment variable
"@

$InstallReadme | Out-File -FilePath "$OutputDir\INSTALL.md" -Encoding utf8

# Create ZIP package
Write-Host "Creating ZIP package..." -ForegroundColor Cyan
$ZipFile = "target\$AppName-$Version-windows.zip"

# Remove existing ZIP if it exists
if (Test-Path $ZipFile) {
    Remove-Item $ZipFile -Force
}

# Create ZIP file
Add-Type -AssemblyName System.IO.Compression.FileSystem
[System.IO.Compression.ZipFile]::CreateFromDirectory((Resolve-Path $OutputDir), (Resolve-Path "target").Path + "\$AppName-$Version-windows.zip")

Write-Host "Package created successfully: $ZipFile" -ForegroundColor Green
Write-Host "You can distribute this ZIP file to Windows users."
