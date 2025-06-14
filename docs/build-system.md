# Building GitSwitch

I've created a comprehensive build system for GitSwitch that makes it easy to create packages for all major platforms. Whether you want to build for Linux, Windows, or macOS, I've got you covered with dedicated build scripts.

## 🚀 Quick Build Commands

### Linux

```bash
./build.sh --all --version v1.0.0
```

### Windows

```powershell
.\build-windows.ps1 -BuildVersion "v1.0.0"
```

### macOS

```bash
./build-macos.sh --all --version v1.0.0
```

## 📦 What Gets Built

### Linux Packages

- **DEB packages** - For Debian/Ubuntu systems
- **RPM packages** - For RHEL/Fedora systems
- **TAR.GZ archives** - Universal Linux distribution

### Windows Packages

- **ZIP archives** - With executables and install scripts
- **Install script** - For easy system-wide installation

### macOS Packages

- **DMG images** - Native macOS disk images
- **PKG installers** - Native macOS installer packages
- **TAR.GZ archives** - For manual installation

## ⚙️ Build System Features

### Smart Version Detection

All my build scripts automatically detect the version from:

1. Git tags (if available)
2. Cargo.toml version field
3. Manual override with `--version` parameter

### Colorful Output

I've made the build process enjoyable with:

- 🎨 Color-coded output for different stages
- 📊 Progress indicators and status messages
- ✅ Clear success/failure indicators
- 🚨 Helpful error messages

### Build Options

Each script supports:

- `--clean` - Clean build (removes previous artifacts)
- `--skip-tests` - Skip running tests before building
- `--all` - Build all package types for the platform
- Specific package types (e.g., `--deb`, `--rpm`, `--dmg`)

## 🔧 Automated CI/CD

I've set up three focused CI/CD workflows:

### 1. **Quick Checks** (`ci.yml`)

Runs on every push/PR for fast feedback:

- Code formatting and linting
- Security audits
- Basic compilation checks

### 2. **Comprehensive Testing** (`rust_tests.yml`)

Runs when code changes:

- Multi-platform testing (Linux, Windows, macOS)
- Multiple Rust versions (stable, beta, nightly)
- Full test suite execution

### 3. **Release Pipeline** (`release.yml`)

Runs on version tags:

- Builds packages for all platforms
- Creates GitHub releases
- Uploads all package artifacts

## 💡 Why I Built This System

I wanted to make GitSwitch easy to install on any platform, so I created comprehensive build scripts that:

- Generate native packages for each platform
- Provide consistent commands across all operating systems
- Include helpful, colorful output that makes building enjoyable
- Automatically handle version detection and packaging

## 🛠️ Requirements

### Linux

- Rust toolchain (stable)
- `cargo-deb` for DEB packages: `cargo install cargo-deb`
- `cargo-generate-rpm` for RPM packages: `cargo install cargo-generate-rpm`

### Windows

- Rust toolchain (stable)
- PowerShell 5.0+

### macOS

- Rust toolchain (stable)
- Xcode command line tools
- `create-dmg` (optional): `brew install create-dmg`

## 🎯 Development Workflow

1. Make your changes
2. Run tests: `cargo test`
3. Test build locally: `./build.sh --tarball --skip-tests`
4. Push changes - CI will handle the rest!

For releases, just create and push a version tag:

```bash
git tag -a v1.0.0 -m "Release v1.0.0"
git push origin v1.0.0
```

The automated release pipeline will build and publish packages for all platforms! 🚀
