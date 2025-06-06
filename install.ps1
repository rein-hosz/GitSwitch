# install.ps1 - Installation helper for git-switch on Windows

# Get the directory where this script and git-switch.exe are located.
$ScriptPath = $PSScriptRoot
$ExecutablePath = Join-Path -Path $ScriptPath -ChildPath "git-switch.exe"

Write-Host "git-switch Installation Helper" -ForegroundColor Yellow
Write-Host "--------------------------------"

if (-not (Test-Path $ExecutablePath)) {
    Write-Host "Error: git-switch.exe not found in the same directory as this script: $ScriptPath" -ForegroundColor Red
    Write-Host "Please ensure git-switch.exe is in the same folder as install.ps1 before running." -ForegroundColor Red
    exit 1
}

Write-Host "git-switch.exe found at: $ExecutablePath"
Write-Host "This script will help you add git-switch to your user PATH."
Write-Host ""

# Check if the path is already in the user's PATH
$UserPath = [System.Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -like "*$ScriptPath*") {
    Write-Host "The directory '$ScriptPath' seems to be already in your user PATH." -ForegroundColor Green
    Write-Host "You should be able to use 'git-switch' from a new terminal session." -ForegroundColor Green
    exit 0
}

Write-Host "To make 'git-switch' accessible from any terminal, its directory needs to be added to your PATH environment variable."
Write-Host "Directory to add: $ScriptPath"
Write-Host ""

$Choice = Read-Host -Prompt "Do you want this script to attempt to add '$ScriptPath' to your USER PATH? (Requires a new terminal to take effect) [Y/N]"

if ($Choice -eq 'Y' -or $Choice -eq 'y') {
    try {
        Write-Host "Attempting to add to USER PATH..."
        $CurrentUserPath = [System.Environment]::GetEnvironmentVariable("Path", "User")
        if ([string]::IsNullOrEmpty($CurrentUserPath)) {
            # Path is empty or doesn't exist, create it
            [System.Environment]::SetEnvironmentVariable("Path", $ScriptPath, "User")
            Write-Host "'$ScriptPath' has been added to your USER PATH." -ForegroundColor Green
        } elseif ($CurrentUserPath -notlike "*$ScriptPath*") {
            # Path exists and doesn't contain our script path
            $NewUserPath = $CurrentUserPath + ";" + $ScriptPath
            [System.Environment]::SetEnvironmentVariable("Path", $NewUserPath, "User")
            Write-Host "'$ScriptPath' has been appended to your USER PATH." -ForegroundColor Green
        } else {
            Write-Host "'$ScriptPath' is already in your USER PATH. No changes made." -ForegroundColor Yellow
        }
        Write-Host "Important: You will need to open a NEW terminal window or restart your current one for this change to take effect." -ForegroundColor Cyan
    } catch {
        Write-Host "Error modifying USER PATH: $($_.Exception.Message)" -ForegroundColor Red
        Write-Host "You may need to run this script with administrator privileges or add the path manually."
        Write-Host "Manual instructions:"
        Write-Host "1. Search for 'environment variables' in the Start Menu."
        Write-Host "2. Click 'Edit the system environment variables'."
        Write-Host "3. Click the 'Environment Variables...' button."
        Write-Host "4. Under 'User variables', select 'Path' and click 'Edit...'."
        Write-Host "5. Click 'New' and add: $ScriptPath"
        Write-Host "6. Click OK on all dialogs."
    }
} else {
    Write-Host "No changes made to your PATH."
    Write-Host "To use git-switch, you can either:"
    Write-Host "  1. Navigate to '$ScriptPath' in your terminal and run '.\git-switch.exe'"
    Write-Host "  2. Add '$ScriptPath' to your PATH manually (see instructions above if you change your mind)."
}

Write-Host ""
Write-Host "Setup helper finished."
