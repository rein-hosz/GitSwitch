# GitSwitch

A command-line tool to easily manage and switch between multiple Git accounts (personal, work, etc.) with automatic SSH key management.

## Features

- Manage multiple Git accounts with different usernames and emails
- Automatically generate and manage SSH keys for each account
- Easily switch between accounts with a single command
- Update SSH configuration automatically
- Update Git repository remote URLs
- Display public SSH keys for adding to GitHub or other services

## Installation

### Prerequisites

- Git
- SSH (OpenSSH)

### Install from Packages

#### Debian/Ubuntu and derivatives
```bash
sudo dpkg -i git-switch_0.1.0_amd64.deb
```

#### Fedora/RHEL/CentOS and derivatives
```bash
sudo rpm -i git-switch-0.1.0-1.x86_64.rpm
```

#### Other Linux distributions
```bash
# Extract the archive
tar -xzvf git-switch-0.1.0.tar.gz
cd git-switch

# Run the install script
./install.sh
```

#### Windows
1. Extract the ZIP file `git-switch-0.1.0-windows.zip`
2. Right-click on `install.bat` and select "Run as administrator"
3. Restart your command prompt or PowerShell

For manual installation on Windows:
1. Copy `bin\git-switch.exe` to a location of your choice
2. Add that location to your PATH environment variable

### Building from Source

#### Prerequisites for building
- Rust and Cargo
- Build essentials package (for Linux)
- Visual Studio Build Tools (for Windows)

#### Build and install on Linux
1. Clone the repository:
   ```bash
   git clone https://github.com/rein-hosz/GitSwitch.git git-switch
   cd git-switch
   ```

2. Build with Cargo:
   ```bash
   cargo build --release
   ```

3. Install the binary:
   ```bash
   cargo install --path .
   ```

#### Build and install on Windows
1. Clone the repository:
   ```powershell
   git clone https://github.com/rein-hosz/GitSwitch.git git-switch
   cd git-switch
   ```

2. Build with Cargo:
   ```powershell
   cargo build --release
   ```

3. Copy the binary to a location in your PATH:
   ```powershell
   copy target\release\git-switch.exe C:\path\to\bin\git-switch.exe
   ```

#### Building packages

##### Linux Packages
You can build installation packages for Linux using the included build script:

```bash
# Build all package types
./build.sh --all

# Or build specific package types
./build.sh --deb       # Build only Debian package
./build.sh --rpm       # Build only RPM package
./build.sh --tarball   # Build only tarball
```

##### Windows Package
Build a Windows installable ZIP package:

```powershell
# Build the Windows package
.\build-windows.ps1
```

The packages will be created in the `target` directory.

## Usage

### Adding a New Account

```bash
git-switch add "Account Name" "Git Username" "email@example.com"
```

For example:
```bash
git-switch add "Personal" "johndoe" "personal@example.com"
git-switch add "Work" "jdoe-company" "john.doe@company.com"
```

This will:
- Generate a new SSH key for this account
- Save your account details
- Update your SSH config
- Display the public key to add to GitHub or another Git service

### Switching Between Accounts

```bash
git-switch use "Account Name"
# OR
git-switch use "Git Username"
```

For example:
```bash
git-switch use "Personal"
# OR
git-switch use "johndoe"
```

This will:
- Update your global Git configuration with the account's username and email
- Load the appropriate SSH key into your SSH agent
- Optionally update the remote URL for the current repository

### Listing All Accounts

```bash
git-switch list
```

This will display all saved accounts with their details.

## Configuration

- **Linux/macOS**:
  - Configuration stored in `~/.git-switch-accounts`
  - SSH keys stored in `~/.ssh/` with names based on account names
  - SSH configuration updated in `~/.ssh/config`

- **Windows**:
  - Configuration stored in `%USERPROFILE%\.git-switch-accounts`
  - SSH keys stored in `%USERPROFILE%\.ssh\`
  - SSH configuration updated in `%USERPROFILE%\.ssh\config`

## How It Works

GitSwitch simplifies managing multiple Git identities by:

1. Managing separate SSH keys for each account
2. Configuring SSH to use different keys for different repositories
3. Providing easy commands to switch between configurations
4. Automating Git config updates

## Platform Support

- **Linux**: Full support for Debian, Ubuntu, Fedora, RHEL, CentOS, and other distributions
- **Windows**: Full support for Windows 10 and later
- **macOS**: Support through the tarball package

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
