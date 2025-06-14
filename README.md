# 🔄 GitSwitch

**The ultimate Git identity management tool for developers who juggle multiple accounts.**

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey.svg)]()

> 🚀 **Tired of wrestling with Git identities?** GitSwitch makes managing multiple Git accounts (work, personal, client projects) as simple as one command. No more SSH key confusion, no more accidentally committing with the wrong email!

---

## 🎯 **Why GitSwitch?**

**The Problem:** You're a developer with multiple Git accounts - work, personal, freelance clients. You constantly switch between them, manually updating configurations, managing SSH keys, and inevitably making mistakes.

**The Solution:** GitSwitch automates everything. One command switches your entire Git identity, complete with SSH keys, configurations, and even remote URLs.

```bash
# Switch your entire Git identity in one command
git-switch use work     # Now you're "John Doe <john@company.com>" with work SSH keys
git-switch use personal # Now you're "John <john@gmail.com>" with personal SSH keys
```

---

## ✨ **What Makes GitSwitch Special**

### 🔥 **Core Features**

- **🎭 Multiple Identity Management**: Seamlessly switch between work, personal, and client Git identities
- **🔐 Automatic SSH Key Management**: Generate, organize, and manage SSH keys for each account
- **⚡ One-Command Switching**: Change your entire Git setup (name, email, SSH keys) instantly
- **📁 Smart Repository Detection**: Automatically suggest the right account for each repository
- **🔄 Profile Workflows**: Group accounts into profiles for different work contexts
- **🛠️ Bulk Operations**: Configure multiple repositories at once
- **🌐 Cross-Platform**: Works perfectly on Windows, macOS, and Linux
- **🎨 Beautiful Output**: Colorful, clear terminal output that makes sense

### 🚀 **Advanced Capabilities**

- **Repository Discovery**: Find all Git repos and apply appropriate identities automatically
- **Template System**: Quick account setup for GitHub, GitLab, Bitbucket, and more
- **Backup & Restore**: Export/import your entire configuration
- **Analytics Dashboard**: Track your Git usage patterns
- **Shell Completions**: Tab completion for Bash, Zsh, Fish, PowerShell
- **Authentication Testing**: Verify your SSH setup with one command
- **Remote URL Management**: Switch between HTTPS and SSH seamlessly

---

## 🚀 **Quick Start**

### **Installation**

#### Option 1: Install from Crates.io (Recommended for Rust users)

```bash
cargo install git-switch
```

#### Option 2: Download Binary (Universal)

Download the latest release for your platform from [GitHub Releases](https://github.com/rein-hosz/GitSwitch/releases).

#### Option 3: Build from Source

```bash
git clone https://github.com/rein-hosz/GitSwitch.git
cd GitSwitch
cargo build --release
cargo install --path .
```

### **Basic Setup (2 minutes)**

```bash
# Add your work account
git-switch add work "John Doe" "john@company.com"

# Add your personal account
git-switch add personal "John" "john@gmail.com"

# Switch to work mode
git-switch use work

# Switch to personal mode
git-switch use personal

# See what's currently active
git-switch whoami
```

**That's it!** GitSwitch handles SSH key generation, configuration, and everything else automatically.

---

## 💡 **Real-World Usage Examples**

### **Scenario 1: Daily Work Switching**

```bash
# Morning: Switch to work identity
git-switch use work
git clone git@github.com-work:company/secret-project.git

# Evening: Switch to personal projects
git-switch use personal
git clone git@github.com-personal:myusername/side-project.git
```

### **Scenario 2: Project-Specific Configuration**

```bash
# Inside a specific repository
cd my-work-project/
git-switch account work  # Configure just this repo for work

cd ../my-blog/
git-switch account personal  # Configure just this repo for personal
```

### **Scenario 3: Bulk Repository Management**

```bash
# Discover all Git repositories in your workspace
git-switch repo discover ~/Projects

# Automatically configure them with suggested accounts
git-switch repo apply
```

---

## 📚 **Core Commands**

### **Account Management**

- `git-switch add <name> <username> <email>` - Add a new Git identity
- `git-switch list` - Show all configured accounts
- `git-switch use <name>` - Switch global Git identity
- `git-switch remove <name>` - Remove an account

### **Repository Operations**

- `git-switch account <name>` - Configure current repo for specific account
- `git-switch whoami` - Show current Git identity and SSH key status
- `git-switch remote --ssh/--https` - Switch remote URL protocol

### **Advanced Features**

- `git-switch profile create <name>` - Create account profiles for workflows
- `git-switch repo discover <path>` - Find and configure repositories automatically
- `git-switch template use github <name>` - Create account from provider template
- `git-switch auth test` - Test SSH authentication
- `git-switch backup create` - Export your configuration

**💡 Tip**: Run `git-switch --help` or `git-switch <command> --help` for detailed usage information.

---

## 🎯 **How It Works**

1. **🗂️ Managing Account Profiles**: Securely stores your Git identities (name, email, SSH keys)
2. **🔐 SSH Key Automation**: Generates and manages unique SSH keys for each account
3. **⚙️ Smart SSH Configuration**: Automatically configures `~/.ssh/config` with host aliases
4. **🎯 Repository Configuration**: Sets local Git config per repository or globally
5. **🔗 Intelligent URL Management**: Helps switch between SSH/HTTPS and account-specific remotes

**Example SSH Configuration** (auto-generated):

```ssh
Host github.com-work
    HostName github.com
    User git
    IdentityFile ~/.ssh/git_switch_work
    IdentitiesOnly yes
```

This enables repository URLs like: `git@github.com-work:company/project.git`

---

## 📖 **Documentation**

- **[📋 Complete Feature Overview](docs/project-overview.md)** - Detailed implementation and capabilities
- **[🧪 Testing Guide](docs/testing-guide.md)** - Comprehensive testing framework and cross-platform support
- **[🎨 Development Guide](docs/development-guide.md)** - Beautiful development tools and colorful output

---

## 🏗️ **Building from Source**

### **Prerequisites**

- Rust 1.70+ and Cargo
- Git and SSH (OpenSSH)

### **Quick Build**

```bash
git clone https://github.com/rein-hosz/GitSwitch.git
cd GitSwitch
cargo build --release
cargo install --path .
```

### **Package Building**

```bash
# Linux packages
./build.sh --all          # Build all package types
./build.sh --deb          # Debian/Ubuntu package
./build.sh --rpm          # Fedora/RHEL package

# Windows package
.\build-windows.ps1       # Windows installer/ZIP
```

---

## 🤝 **Contributing**

GitSwitch is built with ❤️ and welcomes contributions! Whether you're:

- 🐛 **Reporting bugs** - Help us improve
- 💡 **Suggesting features** - Share your ideas
- 📝 **Writing documentation** - Make it better for everyone
- 🔧 **Contributing code** - Join the development

**Development Setup:**

```bash
git clone https://github.com/rein-hosz/GitSwitch.git
cd GitSwitch
cargo test --test comprehensive_platform_tests -- --nocapture  # See beautiful test output!
```

---

## 📄 **License**

MIT License - Use GitSwitch freely in personal and commercial projects.

---

## 🌟 **Why Developers Love GitSwitch**

> _"Finally, no more SSH key confusion! GitSwitch just works."_ - Developer testimonial

> _"Switching between 5 different client accounts used to be a nightmare. Now it's one command."_ - Freelance developer

> _"The colorful test output alone makes this tool worth using for development."_ - Open source contributor

**Ready to simplify your Git workflow?** Install GitSwitch and never worry about Git identity management again! 🚀
