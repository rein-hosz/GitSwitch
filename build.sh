#!/bin/bash
set -e

# Default to build nothing unless specified
BUILD_DEB=0
BUILD_RPM=0
BUILD_TARBALL=0
BUILD_ALL=0
VERSION="" # Added version variable

# Function to show usage
show_usage() {
  echo "Usage: $0 [options] [--version <VERSION_STRING>]" # Updated usage
  echo "Options:"
  echo "  --all      Build all package types (deb, rpm, tarball)"
  echo "  --deb      Build Debian package"
  echo "  --rpm      Build RPM package"
  echo "  --tarball  Build tar.gz package"
  echo "  --version  Specify the version string (e.g., v0.1.0). Defaults to git describe or Cargo.toml." # Added version option
  echo "  --help     Show this help message"
  echo ""
  echo "Example: $0 --deb --rpm --version v0.2.0"
}

# Parse command line arguments
while [ "$1" != "" ]; do
  case $1 in
    --all )      BUILD_ALL=1
                 ;;
    --deb )      BUILD_DEB=1
                 ;;
    --rpm )      BUILD_RPM=1
                 ;;
    --tarball )  BUILD_TARBALL=1
                 ;;
    --version )  shift # Consume --version
                 if [ -z "$1" ]; then
                   echo "Error: --version requires an argument."
                   show_usage
                   exit 1
                 fi
                 VERSION="$1" # Set version
                 ;;
    --help )     show_usage
                 exit
                 ;;
    * )          show_usage
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

# Determine version if not provided
if [ -z "$VERSION" ]; then
  if command -v git &> /dev/null && git rev-parse --is-inside-work-tree &> /dev/null; then
    VERSION=$(git describe --tags --abbrev=0 2>/dev/null || echo "")
  fi
  if [ -z "$VERSION" ] && [ -f Cargo.toml ]; then
    VERSION="v$(grep \'^version\' Cargo.toml | sed \'s/version = \"\\(.*\\)\"/\\1/\')"
  elif [ -z "$VERSION" ]; then
    VERSION="v0.1.0" # Fallback if no other version info found
    echo "Warning: Could not determine version automatically. Using fallback: $VERSION"
  fi
fi

echo "Building with version: $VERSION"
APP_NAME="git-switch"
# Strip 'v' prefix for some packaging formats if it exists
VERSION_NO_V=${VERSION#v}


# Ensure target directory exists
mkdir -p target

# Build the release binary first
echo "Building release binary..."
cargo build --release
if [ $? -ne 0 ]; then
  echo "Cargo build failed!"
  exit 1
fi
BINARY_PATH="target/release/$APP_NAME"

# Create Debian package if requested
if [ $BUILD_DEB -eq 1 ]; then
  echo "Installing cargo-deb if needed..."
  cargo install cargo-deb || true

  echo "Creating Debian package..."
  cargo deb --target-dir target

  # If you need to rename the output based on $VERSION:
  # mv target/debian/git-switch_*_$VERSION_NO_V*.deb target/git-switch-$VERSION-amd64.deb
  echo "Debian package created in target/debian/"
fi

# Create RPM package if requested
if [ $BUILD_RPM -eq 1 ]; then
  echo "Creating RPM package manually..."

  # Get version from Cargo.toml
  VERSION=$(grep '^version =' Cargo.toml | cut -d '"' -f2 || echo "0.1.0")

  # Create RPM build directory structure
  mkdir -p target/rpm-build/{BUILD,RPMS,SOURCES,SPECS,SRPMS}

  # Create a tarball for rpmbuild
  mkdir -p target/rpm-build/SOURCES/git-switch-${VERSION}
  mkdir -p target/rpm-build/SOURCES/git-switch-${VERSION}/usr/bin
  mkdir -p target/rpm-build/SOURCES/git-switch-${VERSION}/usr/share/doc/git-switch
  cp target/release/git_switch target/rpm-build/SOURCES/git-switch-${VERSION}/usr/bin/git-switch
  cp README.md target/rpm-build/SOURCES/git-switch-${VERSION}/usr/share/doc/git-switch/ 2>/dev/null || :
  cp LICENSE target/rpm-build/SOURCES/git-switch-${VERSION}/usr/share/doc/git-switch/ 2>/dev/null || :

  # Create tarball
  (cd target/rpm-build/SOURCES && tar -czf git-switch-${VERSION}.tar.gz git-switch-${VERSION})

  # Create spec file
  cat > target/rpm-build/SPECS/git-switch.spec << EOF
%global debug_package %{nil}
%global _enable_debug_package 0
%global __os_install_post /usr/lib/rpm/brp-compress %{nil}

Name:           git-switch
Version:        ${VERSION}
Release:        1%{?dist}
Summary:        CLI tool to switch between multiple Git accounts

License:        MIT
URL:            https://github.com/rein-hosz/GitSwitch
Source0:        %{name}-%{version}.tar.gz

Requires:       git
Requires:       openssh

%description
git-switch allows users to manage and switch between multiple Git accounts.
It handles SSH key management and Git configuration updates automatically.

%prep
%setup -q

%install
mkdir -p %{buildroot}/usr/bin
mkdir -p %{buildroot}/usr/share/doc/%{name}
cp -p usr/bin/git-switch %{buildroot}/usr/bin/
cp -pr usr/share/doc/git-switch/* %{buildroot}/usr/share/doc/%{name}/ 2>/dev/null || :

%files
%attr(755, root, root) /usr/bin/git-switch
%doc /usr/share/doc/%{name}/*

%changelog
* $(date +"%a %b %d %Y") Ren Hoshizora <blackswordman@gmail.com> - ${VERSION}-1
- Initial RPM release
EOF

  # Build RPM
  if command -v rpmbuild &> /dev/null; then
    echo "Running rpmbuild..."
    (cd target/rpm-build && rpmbuild --define "_topdir $(pwd)" --define "_build_id_links none" -ba SPECS/git-switch.spec)

    # Move RPM to target directory
    mkdir -p target/rpm
    find target/rpm-build/RPMS -name "*.rpm" -exec cp {} target/rpm/ \;

    if [ -e "$(find target/rpm -name '*.rpm' 2>/dev/null)" ]; then
      echo "RPM package created: $(find target/rpm -name '*.rpm')"
    else
      echo "❌ Failed to create RPM package"
    fi
  else
    echo "❌ rpmbuild not found. Please install rpm-build package."
    echo "   On Debian/Ubuntu: sudo apt-get install rpm"
    echo "   On Fedora/RHEL: sudo dnf install rpm-build"
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
  VERSION=$(grep '^version =' Cargo.toml | cut -d '"' -f2 || echo "0.1.0")
  if [ -e "target/git-switch-${VERSION}.tar.gz" ]; then
    echo "✅ Tarball package"
  else
    echo "❌ Tarball package (build failed)"
  fi
else
  echo "❌ Tarball package (not built)"
fi

# For DEB and RPM, the versioning is typically handled by cargo-deb and cargo-generate-rpm
# Ensure Cargo.toml version is updated if these tools rely on it directly and you want the tag version to be authoritative.
# For now, we assume these tools pick up the version from Cargo.toml or their own config.
# If specific versioning for .deb/.rpm filenames is needed from THIS script's $VERSION,
# you'd need to adjust how cargo-deb/cargo-generate-rpm are called or how their output is named.

if [ $BUILD_DEB -eq 1 ]; then
  echo "Building Debian package (using cargo-deb, version from Cargo.toml)..."
  # Update Cargo.toml version to $VERSION_NO_V before building .deb if needed
  # sed -i "s/^version = .*/version = \"$VERSION_NO_V\"/" Cargo.toml
  cargo deb --target-dir target # cargo-deb typically uses version from Cargo.toml
  # If you need to rename the output based on $VERSION:
  # mv target/debian/git-switch_*_$VERSION_NO_V*.deb target/git-switch-$VERSION-amd64.deb
  echo "Debian package created in target/debian/"
fi

if [ $BUILD_RPM -eq 1 ]; then
  echo "Building RPM package (using cargo-generate-rpm, version from Cargo.toml)..."
  # Update Cargo.toml version to $VERSION_NO_V before building .rpm if needed
  # sed -i "s/^version = .*/version = \"$VERSION_NO_V\"/" Cargo.toml
  cargo generate-rpm --target-dir target # cargo-generate-rpm also uses Cargo.toml version
  # If you need to rename the output based on $VERSION:
  # mv target/rpm/git-switch-*$VERSION_NO_V*.rpm target/git-switch-$VERSION-x86_64.rpm
  echo "RPM package created in target/rpm/"
fi

echo "Build process finished."

# Restore Cargo.toml if it was changed (optional, depends on strategy)
# git checkout -- Cargo.toml
