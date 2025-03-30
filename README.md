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
- Rust and Cargo
- SSH (OpenSSH)

### Building from Source

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
```

For example:
```bash
git-switch use "Personal"
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

All account information is stored in `~/.git-switch-accounts`.
SSH keys are stored in `~/.ssh/` with names based on the account name (e.g., `~/.ssh/id_rsa_personal`).
SSH configuration is updated in `~/.ssh/config`.

## How It Works

GitSwitch simplifies managing multiple Git identities by:

1. Managing separate SSH keys for each account
2. Configuring SSH to use different keys for different repositories
3. Providing easy commands to switch between configurations
4. Automating Git config updates

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
