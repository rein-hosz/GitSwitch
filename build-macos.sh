#!/bin/bash
set -e

# GitSwitch Build Script for macOS
# Creates macOS app bundle and DMG package

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

# Default options
BUILD_DMG=0
BUILD_PKG=0
BUILD_TARBALL=0
BUILD_ALL=0
VERSION_ARG=""
SKIP_TESTS=0
CLEAN_BUILD=0

# Function to show usage
show_usage() {
  echo -e "${CYAN}GitSwitch Build Script for macOS${NC}"
  echo ""
  echo "Usage: $0 [options] [--version <VERSION_STRING>]"
  echo ""
  echo -e "${YELLOW}Package Options:${NC}"
  echo "  --all           Build all package types (dmg, pkg, tarball)"
  echo "  --dmg           Build DMG package"
  echo "  --pkg           Build PKG installer"
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
  echo -e "${YELLOW}Examples:${NC}"
  echo "  $0 --all --version v1.0.0"
  echo "  $0 --dmg --clean"
  echo "  $0 --tarball --skip-tests"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --all)
      BUILD_ALL=1
      BUILD_DMG=1
      BUILD_PKG=1
      BUILD_TARBALL=1
      shift
      ;;
    --dmg)
      BUILD_DMG=1
      shift
      ;;
    --pkg)
      BUILD_PKG=1
      shift
      ;;
    --tarball)
      BUILD_TARBALL=1
      shift
      ;;
    --version)
      VERSION_ARG="$2"
      shift 2
      ;;
    --skip-tests)
      SKIP_TESTS=1
      shift
      ;;
    --clean)
      CLEAN_BUILD=1
      shift
      ;;
    --help)
      show_usage
      exit 0
      ;;
    *)
      print_error "Unknown option: $1"
      show_usage
      exit 1
      ;;
  esac
done

# If no package type specified, show usage
if [[ $BUILD_DMG -eq 0 && $BUILD_PKG -eq 0 && $BUILD_TARBALL -eq 0 ]]; then
  print_warning "No package type specified"
  show_usage
  exit 1
fi

print_header "GitSwitch macOS Build Script"

# Clean build if requested
if [[ $CLEAN_BUILD -eq 1 ]]; then
  print_info "Cleaning build directory..."
  rm -rf target/
  print_success "Build directory cleaned"
fi

# Check if we're on macOS
if [[ "$(uname)" != "Darwin" ]]; then
  print_error "This script must be run on macOS"
  exit 1
fi

# Check for required tools
print_info "Checking required tools..."

# Check Rust
if ! command -v rustc &> /dev/null; then
  print_error "Rust is not installed. Please install from https://rustup.rs/"
  exit 1
fi
print_success "Rust found: $(rustc --version)"

# Check Cargo
if ! command -v cargo &> /dev/null; then
  print_error "Cargo is not installed"
  exit 1
fi

# Determine version
VERSION=""
if [[ -n "$VERSION_ARG" ]]; then
  VERSION="$VERSION_ARG"
  print_info "Using provided version: $VERSION"
elif [[ -f "Cargo.toml" ]]; then
  VERSION=$(grep '^version = ' Cargo.toml | sed 's/version = "\([^"]*\)".*/v\1/')
  print_success "Using version from Cargo.toml: $VERSION"
elif command -v git &> /dev/null; then
  VERSION=$(git describe --tags --abbrev=0 2>/dev/null || echo "")
  if [[ -n "$VERSION" ]]; then
    print_success "Using version from git tag: $VERSION"
  fi
fi

if [[ -z "$VERSION" ]]; then
  VERSION="v0.1.0"
  print_warning "Could not determine version. Using fallback: $VERSION"
fi

# Ensure version starts with 'v'
if [[ ! "$VERSION" =~ ^v ]]; then
  VERSION="v$VERSION"
fi

VERSION_NO_V=${VERSION#v}

print_header "Building git-switch version $VERSION for macOS"

# Update Cargo.toml version
if [[ -f "Cargo.toml" ]]; then
  print_info "Updating Cargo.toml version to $VERSION_NO_V..."
  cp Cargo.toml Cargo.toml.bak
  sed -i '' "s/^version = \"[^\"]*\"/version = \"$VERSION_NO_V\"/" Cargo.toml
  print_success "Cargo.toml version updated"
fi

# Run tests if not skipped
if [[ $SKIP_TESTS -eq 0 ]]; then
  print_info "Running tests..."
  if cargo test --release; then
    print_success "All tests passed"
  else
    print_error "Tests failed!"
    # Restore Cargo.toml
    if [[ -f "Cargo.toml.bak" ]]; then
      mv Cargo.toml.bak Cargo.toml
    fi
    exit 1
  fi
fi

# Build the release binary
print_info "Building release binary..."
if cargo build --release; then
  print_success "Build completed successfully"
else
  print_error "Build failed!"
  # Restore Cargo.toml
  if [[ -f "Cargo.toml.bak" ]]; then
    mv Cargo.toml.bak Cargo.toml
  fi
  exit 1
fi

# Clean up Cargo.toml backup
if [[ -f "Cargo.toml.bak" ]]; then
  rm Cargo.toml.bak
fi

# Check if binary exists
BINARY_PATH="target/release/git-switch"
if [[ ! -f "$BINARY_PATH" ]]; then
  print_error "Binary not found at $BINARY_PATH"
  exit 1
fi

print_success "Binary built successfully at $BINARY_PATH"

# Create target directory for packages
mkdir -p target/packages

# Build tarball if requested
if [[ $BUILD_TARBALL -eq 1 ]]; then
  print_info "Creating tarball package..."
  
  TARBALL_NAME="git-switch-$VERSION_NO_V-macos-amd64.tar.gz"
  TEMP_DIR="target/git-switch-$VERSION_NO_V"
  
  # Create temporary directory structure
  mkdir -p "$TEMP_DIR"
  
  # Copy files
  cp "$BINARY_PATH" "$TEMP_DIR/"
  cp README.md "$TEMP_DIR/" 2>/dev/null || print_warning "README.md not found"
  cp LICENSE "$TEMP_DIR/" 2>/dev/null || print_warning "LICENSE not found"
  
  # Create tarball
  (cd target && tar -czf "packages/$TARBALL_NAME" "$(basename "$TEMP_DIR")")
  
  # Clean up
  rm -rf "$TEMP_DIR"
  
  print_success "Tarball created: target/packages/$TARBALL_NAME"
fi

# Build DMG if requested
if [[ $BUILD_DMG -eq 1 ]]; then
  print_info "Creating DMG package..."
  
  # Check for create-dmg (install with brew if not available)
  if ! command -v create-dmg &> /dev/null; then
    print_warning "create-dmg not found. Install with: brew install create-dmg"
    print_info "Creating simple DMG without create-dmg..."
    
    DMG_NAME="git-switch-$VERSION_NO_V-macos-amd64.dmg"
    TEMP_DMG_DIR="target/dmg-temp"
    
    # Create temporary directory
    mkdir -p "$TEMP_DMG_DIR"
    
    # Copy binary and files
    cp "$BINARY_PATH" "$TEMP_DMG_DIR/"
    cp README.md "$TEMP_DMG_DIR/" 2>/dev/null || true
    cp LICENSE "$TEMP_DMG_DIR/" 2>/dev/null || true
    
    # Create DMG
    hdiutil create -volname "GitSwitch $VERSION" -srcfolder "$TEMP_DMG_DIR" -ov -format UDZO "target/packages/$DMG_NAME"
    
    # Clean up
    rm -rf "$TEMP_DMG_DIR"
    
    print_success "DMG created: target/packages/$DMG_NAME"
  else
    # Use create-dmg for better DMG
    DMG_NAME="git-switch-$VERSION_NO_V-macos-amd64.dmg"
    FINAL_DMG_PATH="target/packages/$DMG_NAME"

    print_info "Ensuring final DMG path $FINAL_DMG_PATH is clear before running create-dmg..."
    rm -f "$FINAL_DMG_PATH"

    TEMP_DMG_DIR="target/dmg-temp"
    
    mkdir -p "$TEMP_DMG_DIR"
    cp "$BINARY_PATH" "$TEMP_DMG_DIR/"
    cp README.md "$TEMP_DMG_DIR/" 2>/dev/null || true
    cp LICENSE "$TEMP_DMG_DIR/" 2>/dev/null || true
    
    create-dmg \
      --volname "GitSwitch $VERSION" \
      --window-pos 200 120 \
      --window-size 600 400 \
      --hide-extension git-switch \
      "$FINAL_DMG_PATH" \
      "$TEMP_DMG_DIR"
    
    rm -rf "$TEMP_DMG_DIR"
    print_success "DMG created: $FINAL_DMG_PATH"
  fi
fi

# Build PKG if requested
if [[ $BUILD_PKG -eq 1 ]]; then
  print_info "Creating PKG installer..."
  
  PKG_NAME="git-switch-$VERSION_NO_V-macos-amd64.pkg"
  TEMP_PKG_DIR="target/pkg-temp"
  PKG_ROOT="$TEMP_PKG_DIR/root"
  PKG_SCRIPTS="$TEMP_PKG_DIR/scripts"
  
  # Create directory structure
  mkdir -p "$PKG_ROOT/usr/local/bin"
  mkdir -p "$PKG_SCRIPTS"
  
  # Copy binary
  cp "$BINARY_PATH" "$PKG_ROOT/usr/local/bin/"
  
  # Create postinstall script
  cat > "$PKG_SCRIPTS/postinstall" << 'EOF'
#!/bin/bash
chmod +x /usr/local/bin/git-switch
exit 0
EOF
  chmod +x "$PKG_SCRIPTS/postinstall"
  
  # Build PKG
  pkgbuild --root "$PKG_ROOT" \
           --scripts "$PKG_SCRIPTS" \
           --identifier "com.gitswitch.git-switch" \
           --version "$VERSION_NO_V" \
           "target/packages/$PKG_NAME"
  
  # Clean up
  rm -rf "$TEMP_PKG_DIR"
  
  print_success "PKG created: target/packages/$PKG_NAME"
fi

print_header "Build completed successfully!"
print_success "All packages created in target/packages/"

# List created packages
echo ""
print_info "Created packages:"
ls -la target/packages/ | grep -E '\.(tar\.gz|dmg|pkg)$' | while read line; do
  echo "  📦 $line"
done
