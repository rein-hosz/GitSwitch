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
- Rust (if building from source or for development)

### From Crates.io (Recommended for Rust developers)

```bash
cargo install git-switch
```

### From GitHub Releases

Download the appropriate binary or installer for your system from the [GitHub Releases page](https://github.com/rein-hosz/GitSwitch/releases).

#### Debian/Ubuntu and derivatives

```bash
# Replace X.Y.Z with the desired version
sudo dpkg -i git-switch_X.Y.Z_amd64.deb
```

#### Fedora/RHEL/CentOS and derivatives

```bash
# Replace X.Y.Z with the desired version
sudo rpm -i git-switch-X.Y.Z-1.x86_64.rpm
```

#### Other Linux distributions (Tarball)

```bash
# Replace X.Y.Z with the desired version
tar -xzvf git-switch-X.Y.Z.tar.gz
cd git-switch-X.Y.Z

# Run the install script (optional, you can also copy the binary to your PATH)
./install.sh
# or for a local user install:
# mkdir -p ~/.local/bin
# cp git-switch ~/.local/bin/
```

#### Windows (MSI Installer or ZIP)

1. Download the `git-switch-X.Y.Z-x86_64.msi` or `git-switch-X.Y.Z-windows.zip`.
2. Run the MSI installer, or extract the ZIP and add `git-switch.exe` to your PATH.

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

`git-switch` provides several subcommands to manage your Git identities.

Run `git-switch --help` for a full list of commands and their options.

### General Commands

- **`git-switch --version`**: Display the application version (short format).
- **`git-switch -V`**: Display the detailed application version, including Git commit hash if available.

### Account Management

#### Adding a New Account

```bash
git-switch add <ACCOUNT_NAME> <GIT_USERNAME> <GIT_EMAIL> [--ssh-key-path /path/to/existing/key]
```

- **`ACCOUNT_NAME`**: A friendly name for the account (e.g., "personal", "work").
- **`GIT_USERNAME`**: Your Git username (e.g., "johndoe").
- **`GIT_EMAIL`**: Your Git email address (e.g., "john.doe@example.com").
- **`--ssh-key-path`** (Optional): Path to an existing SSH private key. If not provided, a new key pair will be generated.

Example:

```bash
git-switch add personal "John Doe" "john.doe@example.com"
git-switch add work "Jane Doe" "jane.doe@work.com" --ssh-key-path ~/.ssh/id_rsa_work
```

This command will:

- Store the account details.
- Generate a new SSH key pair (if no existing key is provided) in `~/.ssh/git_switch_ACCOUNT_NAME`.
- Add an entry to your SSH config (`~/.ssh/config`) to use this key for a specific host alias (e.g., `github.com-personal`).
- Display the public key and the SSH host alias to use in your Git remote URLs.

#### Listing All Accounts

```bash
git-switch list
```

Displays all configured accounts, their Git usernames, emails, and associated SSH key paths.

#### Switching Global Git Identity (Not Recommended for Per-Repository Setup)

```bash
git-switch use <ACCOUNT_NAME>
```

This command updates your global Git `user.name` and `user.email` configuration.
**Note**: It's generally recommended to configure identity per repository or use Git's `includeIf` directive for more granular control, rather than frequently changing the global Git identity. `git-switch` primarily focuses on SSH key management for different accounts.

#### Removing an Account

```bash
git-switch remove <ACCOUNT_NAME> [-y | --no-prompt]
```

- **`-y`** or **`--no-prompt`**: Skip confirmation.

Removes the account from `git-switch`'s configuration. It **does not** delete the SSH key files or remove entries from the global Git config. You may need to manually clean these up if desired.

### Repository-Specific Operations

These commands should be run inside a Git repository.

#### Setting Account for Current Repository

```bash
git-switch account <ACCOUNT_NAME>
```

This command configures the **local** Git repository to use the specified account's username and email. It also helps you update the remote URL to use the account-specific SSH alias.

Example:
If you have an account named "work", running `git-switch account work` in a repository will:

1. Set `user.name` and `user.email` for that repository to the "work" account's details.
2. Guide you to update your remote URL (e.g., from `git@github.com:user/repo.git` to `git@github.com-work:user/repo.git`).

#### Modifying Remote URL Protocol

```bash
git-switch remote --https
git-switch remote --ssh
```

- **`--https`**: Switches the `origin` remote URL to use HTTPS.
- **`--ssh`**: Switches the `origin` remote URL to use SSH. It will attempt to use the account-specific SSH alias if an account is configured for the repository via `git-switch account <NAME>`.

#### Showing Current Identity and Remote Status

```bash
git-switch whoami
```

Displays:

- The Git `user.name` and `user.email` active for the current repository (local, then global).
- The SSH key that would be used for the `origin` remote, based on SSH configuration and remote URL.
- The `origin` remote URL.

### Authentication Utilities

#### Testing SSH Authentication

```bash
git-switch auth test
```

Attempts to test SSH authentication using the SSH key associated with the account configured for the current repository (if any, via `git-switch account <NAME>`). If no specific account is set for the repo, it may try a default SSH connection. This helps verify if your SSH key is correctly set up and added to your Git provider (e.g., GitHub, GitLab).

## How It Works

`git-switch` primarily helps by:

1.  **Storing Account Profiles**: Keeps a record of your different Git identities (name, email, SSH key path).
2.  **SSH Key Management**:
    - Generates new ED25519 SSH key pairs for new accounts if you don't provide an existing one.
    - Stores these keys typically in `~/.ssh/git_switch_<ACCOUNT_NAME>`.
3.  **SSH Configuration (`~/.ssh/config`)**:
    - When you add an account (e.g., "work" for `github.com`), it adds an SSH config entry like:
      ```
      Host github.com-work
          HostName github.com
          User git
          IdentityFile ~/.ssh/git_switch_work
          IdentitiesOnly yes
      ```
    - This allows you to use remotes like `git@github.com-work:your-username/your-repo.git`.
4.  **Git Configuration**:
    - `git-switch account <NAME>`: Sets `user.name` and `user.email` in the _local_ repository config.
    - `git-switch use <NAME>`: Sets `user.name` and `user.email` in the _global_ Git config (use with caution).
5.  **Remote URL Assistance**: The `account` and `remote` subcommands help you adjust your repository's remote URL to use the correct SSH alias (e.g., `github.com-work`) or switch between SSH and HTTPS protocols.

**Key Idea**: Instead of changing global SSH keys or Git configs constantly, you configure each repository (or group of repositories via SSH host aliases) to use a specific identity and its corresponding SSH key.

## Documentation

Detailed documentation for each module and function can be generated locally using:

```bash
cargo doc --no-deps --open
```

This will build the documentation and open it in your web browser.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
