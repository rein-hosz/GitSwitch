#!/bin/bash
set -e

# Default to build nothing unless specified
BUILD_DEB=0
BUILD_RPM=0
BUILD_TARBALL=0
BUILD_ALL=0

# Function to show usage
show_usage() {
  echo "Usage: $0 [options]"
  echo "Options:"
  echo "  --all      Build all package types (deb, rpm, tarball)"
  echo "  --deb      Build Debian package"
  echo "  --rpm      Build RPM package"
  echo "  --tarball  Build tar.gz package"
  echo "  --help     Show this help message"
  echo ""
  echo "Example: $0 --deb --rpm"
}

# Parse command line arguments
if [ $# -eq 0 ]; then
  show_usage
  exit 1
fi

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

# Build the project in release mode (needed for all package types)
echo "Building release version..."
cargo build --release

# Create Debian package if requested
if [ $BUILD_DEB -eq 1 ]; then
  echo "Installing cargo-deb if needed..."
  cargo install cargo-deb || true

  echo "Creating Debian package..."
  cargo deb

  echo "Debian package created: $(find target/debian -name '*.deb')"
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
  echo "Creating tar.gz package..."

  # Create directory structure
  mkdir -p target/tar/git-switch/usr/bin
  mkdir -p target/tar/git-switch/usr/share/doc/git-switch

  # Copy binary and documentation
  cp target/release/git_switch target/tar/git-switch/usr/bin/git-switch
  cp README.md target/tar/git-switch/usr/share/doc/git-switch/ 2>/dev/null || :
  cp LICENSE target/tar/git-switch/usr/share/doc/git-switch/ 2>/dev/null || :

  # Create install script
  cat > target/tar/git-switch/install.sh << 'EOF'
#!/bin/bash
set -e

# Copy binary to /usr/bin
sudo cp usr/bin/git-switch /usr/bin/
sudo chmod 755 /usr/bin/git-switch

# Copy documentation
sudo mkdir -p /usr/share/doc/git-switch
sudo cp usr/share/doc/git-switch/* /usr/share/doc/git-switch/ 2>/dev/null || :

echo "git-switch has been installed successfully!"
echo "Run 'git-switch --help' to get started."
EOF

  # Make install script executable
  chmod +x target/tar/git-switch/install.sh

  # Create the tarball
  VERSION=$(grep '^version =' Cargo.toml | cut -d '"' -f2 || echo "0.1.0")
  cd target/tar
  tar -czvf ../git-switch-${VERSION}.tar.gz git-switch
  cd ../..

  echo "Tarball created: target/git-switch-${VERSION}.tar.gz"
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
