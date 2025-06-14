#!/bin/bash
set -e

# GitSwitch Build Script for Linux/Unix platforms
# Supports building DEB, RPM, and tarball packages

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Print colored output
print_info() { echo -e "${BLUE}ℹ️  $1${NC}"; }
print_success() { echo -e "${GREEN}✅ $1${NC}"; }
print_warning() { echo -e "${YELLOW}⚠️  $1${NC}"; }
print_error() { echo -e "${RED}❌ $1${NC}"; }
print_header() { echo -e "${CYAN}🔨 $1${NC}"; }

# Default to build nothing unless specified
BUILD_DEB=0
BUILD_RPM=0
BUILD_TARBALL=0
BUILD_ALL=0
VERSION_ARG="" # Store the version passed via --version
SKIP_TESTS=0
CLEAN_BUILD=0

# Function to show usage
show_usage() {
  echo -e "${CYAN}GitSwitch Build Script${NC}"
  echo ""
  echo "Usage: $0 [options] [--version <VERSION_STRING>]"
  echo ""
  echo -e "${YELLOW}Package Options:${NC}"
  echo "  --all           Build all package types (deb, rpm, tarball)"
  echo "  --deb           Build Debian package"
  echo "  --rpm           Build RPM package"
  echo "  --tarball       Build tar.gz package"
  echo ""
  echo -e "${YELLOW}Build Options:${NC}"
  echo "  --version <V>   Specify the version string (e.g., v0.1.0)"
  echo "  --skip-tests    Skip running tests before building"
  echo "  --clean         Clean build (removes target directory)"
  echo ""
  echo -e "${YELLOW}Other Options:${NC}"
  echo "  --help          Show this help message"
  echo ""
  echo -e "${GREEN}Examples:${NC}"
  echo "  $0 --deb --rpm --version v0.2.0"
  echo "  $0 --all --clean"
  echo "  $0 --tarball --skip-tests"
}

# Parse command line arguments
while [ "$1" != "" ]; do
  case $1 in
    --all )         BUILD_ALL=1
                    ;;
    --deb )         BUILD_DEB=1
                    ;;
    --rpm )         BUILD_RPM=1
                    ;;
    --tarball )     BUILD_TARBALL=1
                    ;;
    --skip-tests )  SKIP_TESTS=1
                    ;;
    --clean )       CLEAN_BUILD=1
                    ;;
    --version )     shift # Consume --version
                    if [ -z "$1" ]; then
                      print_error "--version requires an argument."
                      show_usage
                      exit 1
                    fi
                    VERSION_ARG="$1" # Set version argument
                    ;;
    --help )        show_usage
                    exit
                    ;;
    * )             print_error "Unknown option: $1"
                    show_usage
                    exit 1
  esac
  shift
done

# If --all is specified, build everything
if [ $BUILD_ALL -eq 1 ]; then
  BUILD_DEB=1
  BUILD_RPM=1
  BUILD_TARBALL=1
fi

# Check if any build option was specified
if [ $BUILD_DEB -eq 0 ] && [ $BUILD_RPM -eq 0 ] && [ $BUILD_TARBALL -eq 0 ]; then
  print_error "No build type specified. Use --all, --deb, --rpm, or --tarball"
  show_usage
  exit 1
fi

print_header "GitSwitch Build Process Starting"

# Clean build if requested
if [ $CLEAN_BUILD -eq 1 ]; then
  print_info "Cleaning previous build artifacts..."
  if [ -d "target" ]; then
    rm -rf target
    print_success "Build directory cleaned"
  fi
fi

# Determine final version
VERSION=""
if [ -n "$VERSION_ARG" ]; then
  VERSION="$VERSION_ARG"
  print_info "Using provided version: $VERSION"
else
  print_info "Attempting to determine version automatically..."
  if command -v git &> /dev/null && git rev-parse --is-inside-work-tree &> /dev/null; then
    VERSION=$(git describe --tags --abbrev=0 2>/dev/null || echo "")
    if [ -n "$VERSION" ]; then
      print_success "Using version from git tag: $VERSION"
    fi
  fi
  if [ -z "$VERSION" ] && [ -f Cargo.toml ]; then
    # Try to get version from Cargo.toml, ensure it starts with 'v'
    CARGO_VERSION_RAW=$(grep '^version' Cargo.toml | head -1 | sed 's/version *= *"\([^"]*\)".*/\1/')
    if [ -n "$CARGO_VERSION_RAW" ]; then
      if [[ "$CARGO_VERSION_RAW" == v* ]]; then
        VERSION="$CARGO_VERSION_RAW"
      else
        VERSION="v$CARGO_VERSION_RAW"
      fi
      print_success "Using version from Cargo.toml: $VERSION (raw: $CARGO_VERSION_RAW)"
    fi
  fi
  if [ -z "$VERSION" ]; then
    VERSION="v0.1.0" # Fallback if no other version info found
    echo "Warning: Could not determine version automatically. Using fallback: $VERSION"
  fi
fi

echo "Final version for build: $VERSION"
APP_NAME="git-switch"
VERSION_NO_V=${VERSION#v} # Version without 'v' prefix, e.g., 0.1.0

# Update Cargo.toml with the determined version (without 'v')
if [ -f Cargo.toml ]; then
  echo "Updating Cargo.toml to version: $VERSION_NO_V"
  # Create a backup, then replace.
  cp Cargo.toml Cargo.toml.bak
  # sed pattern: match 'version = "..."', capture content, replace with new version.
  # Works for version = "0.1.0" with or without comments
  sed -E "s/^(version\s*=\s*\")[^\"]*(\".*)$/\\1$VERSION_NO_V\\2/" Cargo.toml.bak > Cargo.toml
  if [ $? -ne 0 ]; then
    echo "Error: Failed to update Cargo.toml version."
    mv Cargo.toml.bak Cargo.toml # Restore backup
    exit 1
  fi
  rm Cargo.toml.bak
  echo "Cargo.toml updated successfully. Verifying:"
  grep "^version = " Cargo.toml
else
  echo "Error: Cargo.toml not found. Cannot update version."
  exit 1
fi

# Ensure target directory exists
mkdir -p target

# Build the release binary first (will use updated Cargo.toml)
echo "Building release binary for $APP_NAME version $VERSION_NO_V..."
cargo build --release
if [ $? -ne 0 ]; then
  echo "Cargo build failed!"
  exit 1
fi
BINARY_PATH="target/release/$APP_NAME"
echo "$APP_NAME binary built successfully at $BINARY_PATH"

# Create Debian package if requested
if [ $BUILD_DEB -eq 1 ]; then
  echo "Installing cargo-deb if needed..."
  cargo install cargo-deb --locked || true # Added --locked for potentially faster/more reliable installs in CI

  echo "Creating Debian package..."
  cargo deb # Removed --target-dir target

  # The output is typically in target/debian/
  # Example: target/debian/git-switch_0.1.1-1_amd64.deb (adjust versioning as needed for exact name)
  echo "Debian package should be in target/debian/"
  # Find the .deb file (name might vary slightly with architecture or revision)
  DEB_FILE=$(find target/debian -name "${APP_NAME}_*${VERSION_NO_V}*.deb" -print -quit)
  if [ -n "$DEB_FILE" ]; then
    echo "Debian package created: $DEB_FILE"
  else
    echo "Warning: Could not find the created .deb file in target/debian/ for version $VERSION_NO_V. Listing contents:"
    ls -R target/debian/
  fi
fi

# Create RPM package if requested
if [ $BUILD_RPM -eq 1 ]; then
  print_info "Checking for cargo-generate-rpm..."
  if ! command -v cargo-generate-rpm &> /dev/null; then
    print_warning "cargo-generate-rpm not found. Installing..."
    cargo install cargo-generate-rpm
    if ! command -v cargo-generate-rpm &> /dev/null; then
      print_error "Failed to install cargo-generate-rpm. Please install it manually."
      print_error "Skipping RPM build."
      BUILD_RPM=0 # Skip RPM build if installation fails
    else
      print_success "cargo-generate-rpm installed successfully."
    fi
  else
    print_success "cargo-generate-rpm found."
  fi

  if [ $BUILD_RPM -eq 1 ]; then # Check again in case it was disabled
    print_info "Creating RPM package using cargo-generate-rpm..."
    # Ensure Cargo.toml has necessary metadata for RPM generation
    # Example: [package.metadata.rpm]
    #          release = "1%{?dist}"
    #          requires = ["git", "openssh"]
    #          [package.metadata.rpm.changelog]
    #          "* Mon Jan 01 2024 Your Name <your.email@example.com> - {version}-1" = ["Initial RPM release."]
    # cargo-generate-rpm will use these, or defaults.

    # The tool typically outputs to target/rpm/ directory by default.
    # We might need to specify the output directory or move the file if needed.
    if cargo generate-rpm; then
      RPM_FILE=$(find target/rpm -name "${APP_NAME}-${VERSION_NO_V}*.rpm" -print -quit)
      if [ -n "$RPM_FILE" ]; then
        print_success "RPM package created: $RPM_FILE"
      else
        print_warning "Could not find the created .rpm file in target/rpm/ for version $VERSION_NO_V. Listing contents:"
        ls -R target/rpm/
      fi
    else
      print_error "cargo-generate-rpm failed!"
    fi
  fi
fi

# Create tar.gz package if requested
if [ $BUILD_TARBALL -eq 1 ]; then
  echo "Building tarball..."
  TARBALL_DIR="target/tar/$APP_NAME-$VERSION"
  TARBALL_NAME="$APP_NAME-$VERSION.tar.gz"
  mkdir -p "$TARBALL_DIR"
  cp "$BINARY_PATH" "$TARBALL_DIR/"
  cp README.md "$TARBALL_DIR/"
  cp LICENSE "$TARBALL_DIR/"
  # Add other files like completions if they exist
  # cp completions/git-switch.bash "$TARBALL_DIR/"
  
  echo "Creating $TARBALL_NAME in target/ ..."
  tar -czf "target/$TARBALL_NAME" -C "target/tar" "$APP_NAME-$VERSION"
  rm -rf "target/tar" # Clean up intermediate tar directory
  echo "Tarball created: target/$TARBALL_NAME"
fi

# Summary of what was built
echo ""
echo "Build Summary:"
if [ $BUILD_DEB -eq 1 ]; then
  if [ -e "$(find target/debian -name '*.deb' 2>/dev/null)" ]; then
    echo "✅ Debian package"
  else
    echo "❌ Debian package (build failed)"
  fi
else
  echo "❌ Debian package (not built)"
fi

if [ $BUILD_RPM -eq 1 ]; then
  if [ -e "$(find target/rpm -name '*.rpm' 2>/dev/null)" ]; then
    echo "✅ RPM package"
  else
    echo "❌ RPM package (build failed)"
  fi
else
  echo "❌ RPM package (not built)"
fi

if [ $BUILD_TARBALL -eq 1 ]; then
  # Use VERSION_NO_V for consistency, as Cargo.toml was updated to this
  if [ -e "target/$APP_NAME-v${VERSION_NO_V}.tar.gz" ] || [ -e "target/$APP_NAME-${VERSION_NO_V}.tar.gz" ]; then # Check with and without 'v'
    echo "✅ Tarball package"
  else
    echo "❌ Tarball package (build failed - expected target/$APP_NAME-v${VERSION_NO_V}.tar.gz or target/$APP_NAME-${VERSION_NO_V}.tar.gz)"
  fi
else
  echo "❌ Tarball package (not built)"
fi

echo "Build process finished for version $VERSION."

# Restore Cargo.toml if it was changed (optional, depends on strategy)
# If Cargo.toml.bak exists, it means we modified Cargo.toml
if [ -f Cargo.toml.bak ]; then
  echo "Restoring original Cargo.toml..."
  mv Cargo.toml.bak Cargo.toml
fi
