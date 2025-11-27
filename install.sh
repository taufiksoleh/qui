#!/bin/bash
# Kube-TUI Installation Script
# This script automatically detects your system and installs the latest version of kube-tui

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
REPO="taufiksoleh/qui"
BINARY_NAME="kube-tui"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

# Function to print colored messages
print_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

# Detect OS
detect_os() {
    case "$(uname -s)" in
        Linux*)
            OS="linux"
            ;;
        Darwin*)
            OS="macos"
            ;;
        *)
            print_error "Unsupported operating system: $(uname -s)"
            print_info "Supported systems: Linux, macOS"
            exit 1
            ;;
    esac
}

# Detect architecture
detect_arch() {
    ARCH="$(uname -m)"
    case "$ARCH" in
        x86_64|amd64)
            ARCH="x86_64"
            ;;
        aarch64|arm64)
            ARCH="aarch64"
            ;;
        *)
            print_error "Unsupported architecture: $ARCH"
            print_info "Supported architectures: x86_64, aarch64"
            exit 1
            ;;
    esac
}

# Check if running as root for system-wide installation
check_permissions() {
    if [ "$INSTALL_DIR" = "/usr/local/bin" ] && [ "$(id -u)" -ne 0 ]; then
        print_warning "Installation to $INSTALL_DIR requires root privileges"
        print_info "Re-running with sudo..."
        exec sudo bash "$0" "$@"
    fi
}

# Get the latest release version
get_latest_version() {
    print_info "Fetching latest release version..."

    # Try to get latest release from GitHub API
    if command -v curl >/dev/null 2>&1; then
        LATEST_VERSION=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
    elif command -v wget >/dev/null 2>&1; then
        LATEST_VERSION=$(wget -qO- "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
    else
        print_error "Neither curl nor wget found. Please install one of them."
        exit 1
    fi

    if [ -z "$LATEST_VERSION" ]; then
        print_warning "Could not fetch latest version from API, using 'latest'"
        DOWNLOAD_URL="https://github.com/$REPO/releases/latest/download/$BINARY_NAME-$OS-$ARCH.tar.gz"
    else
        print_success "Latest version: $LATEST_VERSION"
        DOWNLOAD_URL="https://github.com/$REPO/releases/download/$LATEST_VERSION/$BINARY_NAME-$OS-$ARCH.tar.gz"
    fi
}

# Download the binary
download_binary() {
    print_info "Downloading $BINARY_NAME for $OS-$ARCH..."

    TEMP_DIR=$(mktemp -d)
    TEMP_FILE="$TEMP_DIR/$BINARY_NAME.tar.gz"

    if command -v curl >/dev/null 2>&1; then
        if ! curl -L -f -o "$TEMP_FILE" "$DOWNLOAD_URL"; then
            print_error "Failed to download from $DOWNLOAD_URL"
            rm -rf "$TEMP_DIR"
            exit 1
        fi
    elif command -v wget >/dev/null 2>&1; then
        if ! wget -O "$TEMP_FILE" "$DOWNLOAD_URL"; then
            print_error "Failed to download from $DOWNLOAD_URL"
            rm -rf "$TEMP_DIR"
            exit 1
        fi
    fi

    print_success "Downloaded successfully"
}

# Extract and install
install_binary() {
    print_info "Extracting and installing to $INSTALL_DIR..."

    # Extract
    tar -xzf "$TEMP_FILE" -C "$TEMP_DIR"

    # Create install directory if it doesn't exist
    mkdir -p "$INSTALL_DIR"

    # Install binary
    mv "$TEMP_DIR/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
    chmod +x "$INSTALL_DIR/$BINARY_NAME"

    # Cleanup
    rm -rf "$TEMP_DIR"

    print_success "Installed $BINARY_NAME to $INSTALL_DIR/$BINARY_NAME"
}

# Verify installation
verify_installation() {
    if command -v "$BINARY_NAME" >/dev/null 2>&1; then
        print_success "Installation verified! $BINARY_NAME is ready to use."
        print_info "Run '$BINARY_NAME' to start the Kubernetes Terminal UI"
    else
        print_warning "Installation complete, but $BINARY_NAME is not in your PATH"
        print_info "Add $INSTALL_DIR to your PATH or run: $INSTALL_DIR/$BINARY_NAME"
    fi
}

# Main installation flow
main() {
    echo ""
    echo "╔═══════════════════════════════════════════╗"
    echo "║   Kube-TUI Installation Script           ║"
    echo "║   Kubernetes Terminal UI                 ║"
    echo "╚═══════════════════════════════════════════╝"
    echo ""

    detect_os
    detect_arch

    print_info "System detected: $OS-$ARCH"
    print_info "Install directory: $INSTALL_DIR"
    echo ""

    check_permissions
    get_latest_version
    download_binary
    install_binary
    verify_installation

    echo ""
    print_success "Installation complete! 🎉"
    echo ""
    print_info "Usage:"
    echo "  $BINARY_NAME              # Start the TUI"
    echo "  $BINARY_NAME --help       # Show help (if implemented)"
    echo ""
    print_info "Documentation:"
    echo "  https://github.com/$REPO"
    echo ""
}

# Run main function
main "$@"
