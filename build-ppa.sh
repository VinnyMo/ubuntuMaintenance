#!/bin/bash
# build-ppa.sh - Helper script to build source package for PPA upload
#
# Usage:
#   ./build-ppa.sh [distribution]
#
# Examples:
#   ./build-ppa.sh              # Build for jammy (22.04)
#   ./build-ppa.sh focal        # Build for focal (20.04)
#   ./build-ppa.sh noble        # Build for noble (24.04)
#   ./build-ppa.sh all          # Build for all supported distributions

set -e

# Configuration
PACKAGE="ubuntu-maintenance"
VERSION="3.1.3"
MAINTAINER_NAME="Vincent T. Mossman"
MAINTAINER_EMAIL="vinny.mossman@gmail.com"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Supported distributions (focal removed - rustc 1.41 too old for edition 2021)
SUPPORTED_DISTS="jammy noble"
DEFAULT_DIST="jammy"

# Functions
print_header() {
    echo -e "${BLUE}=== $1 ===${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

check_requirements() {
    print_header "Checking Requirements"

    local missing=0

    # Check for required commands
    for cmd in debuild dput gpg make; do
        if ! command -v $cmd &> /dev/null; then
            print_error "Missing required command: $cmd"
            missing=1
        else
            print_success "Found: $cmd"
        fi
    done

    # Check for GPG key
    if ! gpg --list-secret-keys "$MAINTAINER_EMAIL" &> /dev/null; then
        print_error "No GPG key found for $MAINTAINER_EMAIL"
        print_warning "Generate one with: gpg --full-generate-key"
        missing=1
    else
        print_success "GPG key found"
    fi

    if [ $missing -eq 1 ]; then
        print_error "Please install missing requirements:"
        echo "  sudo apt install debhelper devscripts dh-make build-essential gnupg dput"
        exit 1
    fi

    echo ""
}

clean_build() {
    print_header "Cleaning Previous Builds"

    make clean 2>&1 | grep -v "No such file" || true
    # Remove build artifacts but keep orig.tar.xz
    rm -f ../${PACKAGE}_*.dsc 2>/dev/null || true
    rm -f ../${PACKAGE}_*.build 2>/dev/null || true
    rm -f ../${PACKAGE}_*.buildinfo 2>/dev/null || true
    rm -f ../${PACKAGE}_*.changes 2>/dev/null || true
    # Remove debian tarballs but keep upstream orig tarball
    find .. -maxdepth 1 -name "${PACKAGE}_*.tar.xz" ! -name "${PACKAGE}_${VERSION}.orig.tar.xz" -delete 2>/dev/null || true

    print_success "Build directory cleaned"
    echo ""
}

create_orig_tarball() {
    print_header "Creating Upstream Tarball"

    # Check if orig tarball already exists
    if [ -f "../${PACKAGE}_${VERSION}.orig.tar.xz" ]; then
        print_success "Upstream tarball already exists"
        return 0
    fi

    # Create orig tarball from git
    if [ -d .git ]; then
        print_success "Creating tarball from git..."
        git archive --format=tar --prefix=${PACKAGE}-${VERSION}/ HEAD | xz > ../${PACKAGE}_${VERSION}.orig.tar.xz
        print_success "Created ../${PACKAGE}_${VERSION}.orig.tar.xz"
    else
        print_error "Not a git repository and no orig tarball found"
        return 1
    fi

    echo ""
}

build_source_package() {
    local dist=$1
    local revision=$2
    local include_orig=$3  # "yes" or "no"

    print_header "Building Source Package for $dist"

    # Set environment variables for changelog
    export DEBEMAIL="$MAINTAINER_EMAIL"
    export DEBFULLNAME="$MAINTAINER_NAME"

    # Determine version string (always use ~dist1 for PPA packages)
    VERSION_STRING="${VERSION}-${revision}~${dist}1"

    echo "Version: $VERSION_STRING"
    echo "Distribution: $dist"
    echo ""

    # Update changelog distribution and version for this build
    print_success "Updating changelog for $dist..."

    # Replace both version and distribution in the first line of changelog
    sed -i "1s/([^)]*) [a-z]*;/($VERSION_STRING) $dist;/" debian/changelog

    # Verify the change
    head -1 debian/changelog

    # Build source package
    local build_opts="-S -d -k57657DC3657426DA28BC05B99D9EDCF0D2CDF97A"

    if [ "$include_orig" = "yes" ]; then
        build_opts="$build_opts -sa"
        print_success "Including orig.tar.xz in upload (first distribution)"
    else
        build_opts="$build_opts -sd"
        print_success "Using existing orig.tar.xz from PPA (subsequent distribution)"
    fi

    print_success "Running debuild..."
    if debuild $build_opts 2>&1 | tee /tmp/debuild.log | grep -E "dpkg-buildpackage|dpkg-source|error|warning"; then
        print_success "Source package built successfully"
    else
        print_error "Build failed! Check /tmp/debuild.log"
        exit 1
    fi

    # List generated files
    echo ""
    print_header "Generated Files"
    ls -lh ../${PACKAGE}_${VERSION_STRING}* 2>/dev/null || true
    echo ""
}

upload_to_ppa() {
    local dist=$1
    local revision=$2

    # Always use ~dist1 for PPA packages
    VERSION_STRING="${VERSION}-${revision}~${dist}1"

    local changes_file="../${PACKAGE}_${VERSION_STRING}_source.changes"

    if [ ! -f "$changes_file" ]; then
        print_error "Changes file not found: $changes_file"
        exit 1
    fi

    print_header "Uploading to PPA"

    echo "Changes file: $changes_file"
    echo ""

    # Show what will be uploaded
    print_success "Package contents:"
    grep -A 100 "^Files:" "$changes_file" | tail -n +2 | head -n 10

    echo ""
    read -p "Upload to PPA? (y/N) " -n 1 -r
    echo ""

    if [[ $REPLY =~ ^[Yy]$ ]]; then
        if dput ppa "$changes_file"; then
            print_success "Upload successful!"
            echo ""
            print_warning "Build status: https://launchpad.net/~YOUR_USERNAME/+archive/ubuntu/ubuntu-maintenance/+packages"
            echo ""
        else
            print_error "Upload failed!"
            exit 1
        fi
    else
        print_warning "Upload cancelled"
    fi
}

show_help() {
    cat << EOF
Ubuntu Maintenance Tool - PPA Build Script

Usage: $0 [OPTION] [DISTRIBUTION]

Build source packages for Ubuntu PPA distribution.

Options:
    -h, --help          Show this help message
    -n, --no-upload     Build only, don't upload to PPA
    -r, --revision NUM  Set Debian revision (default: 1)
    -a, --all           Build for all supported distributions

Distributions:
    jammy               Ubuntu 22.04 LTS (Jammy Jellyfish) [default]
    noble               Ubuntu 24.04 LTS (Noble Numbat)

Note: Ubuntu 20.04 (Focal) is not supported due to rustc 1.41 being too old for Rust edition 2021 (requires 1.56+)

Examples:
    $0                  # Build for jammy (default)
    $0 noble            # Build for noble
    $0 --all            # Build for all distributions (jammy, noble)
    $0 -n jammy         # Build for jammy, don't upload

Environment Variables:
    MAINTAINER_EMAIL    Override maintainer email (default: $MAINTAINER_EMAIL)
    MAINTAINER_NAME     Override maintainer name (default: $MAINTAINER_NAME)

EOF
}

build_all_distributions() {
    local revision=$1
    local upload=$2

    print_header "Building for All Distributions"
    echo "Distributions: $SUPPORTED_DISTS"
    echo ""

    # Create orig tarball once for all distributions
    create_orig_tarball

    local first_dist="yes"
    for dist in $SUPPORTED_DISTS; do
        clean_build

        # First distribution includes orig.tar.xz, others reuse it from PPA
        build_source_package "$dist" "$revision" "$first_dist"
        first_dist="no"

        if [ "$upload" = "yes" ]; then
            upload_to_ppa "$dist" "$revision"
        fi

        echo ""
        echo "---"
        echo ""
    done

    print_success "All distributions processed!"
}

# Main script
main() {
    local dist=$DEFAULT_DIST
    local revision=1
    local upload="yes"
    local build_all="no"

    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                show_help
                exit 0
                ;;
            -n|--no-upload)
                upload="no"
                shift
                ;;
            -r|--revision)
                revision="$2"
                shift 2
                ;;
            -a|--all)
                build_all="yes"
                shift
                ;;
            focal|jammy|noble)
                dist="$1"
                shift
                ;;
            *)
                print_error "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done

    # Show configuration
    print_header "Ubuntu Maintenance Tool - PPA Builder"
    echo "Package: $PACKAGE"
    echo "Version: $VERSION"
    echo "Maintainer: $MAINTAINER_NAME <$MAINTAINER_EMAIL>"
    echo ""

    # Check requirements
    check_requirements

    # Build
    if [ "$build_all" = "yes" ]; then
        build_all_distributions "$revision" "$upload"
    else
        create_orig_tarball
        clean_build
        build_source_package "$dist" "$revision" "yes"

        if [ "$upload" = "yes" ]; then
            upload_to_ppa "$dist" "$revision"
        else
            print_warning "Build complete. Run manually to upload:"
            echo "  dput ppa ../${PACKAGE}_${VERSION}-${revision}~${dist}1_source.changes"
        fi
    fi

    print_success "Done!"
}

# Run main
main "$@"
