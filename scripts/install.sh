#!/bin/bash

# Automatic installation script for TermPlay
# Usage: curl -sSL https://raw.githubusercontent.com/MedCy1/TermPlay/main/scripts/install.sh | bash

set -e

# Configuration
REPO="MedCy1/TermPlay"  # Replace with your GitHub repo
BINARY_NAME="termplay"
INSTALL_DIR="$HOME/.local/bin"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${BLUE}ℹ️  $1${NC}"; }
log_success() { echo -e "${GREEN}✅ $1${NC}"; }
log_warning() { echo -e "${YELLOW}⚠️  $1${NC}"; }
log_error() { echo -e "${RED}❌ $1${NC}"; }

# Detect OS and architecture
detect_platform() {
    local os=$(uname -s | tr '[:upper:]' '[:lower:]')
    local arch=$(uname -m)
    
    case $os in
        linux)
            OS="linux"
            ;;
        darwin)
            OS="macos"
            ;;
        *)
            log_error "Unsupported OS: $os"
            exit 1
            ;;
    esac
    
    case $arch in
        x86_64|amd64)
            ARCH="x86_64"
            ;;
        aarch64|arm64)
            ARCH="aarch64"
            ;;
        *)
            log_error "Unsupported architecture: $arch"
            exit 1
            ;;
    esac
    
    PLATFORM="${OS}-${ARCH}"
    log_info "Detected platform: $PLATFORM"
}

# Get latest version
get_latest_version() {
    log_info "Fetching latest version..."
    
    if command -v curl >/dev/null 2>&1; then
        LATEST_VERSION=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/' | sed 's/v//')
    elif command -v wget >/dev/null 2>&1; then
        LATEST_VERSION=$(wget -qO- "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/' | sed 's/v//')
    else
        log_error "curl or wget required for installation"
        exit 1
    fi
    
    if [ -z "$LATEST_VERSION" ]; then
        log_error "Unable to fetch latest version"
        exit 1
    fi
    
    log_info "Latest version: $LATEST_VERSION"
}

# Build download URL
build_download_url() {
    case $OS in
        linux|macos)
            ARCHIVE_NAME="${BINARY_NAME}-${PLATFORM}.tar.gz"
            ;;
        *)
            log_error "Unsupported OS for download"
            exit 1
            ;;
    esac
    
    DOWNLOAD_URL="https://github.com/$REPO/releases/download/v$LATEST_VERSION/$ARCHIVE_NAME"
    log_info "Download URL: $DOWNLOAD_URL"
}

# Download and install
download_and_install() {
    local temp_dir=$(mktemp -d)
    local archive_path="$temp_dir/$ARCHIVE_NAME"
    
    log_info "Downloading..."
    
    if command -v curl >/dev/null 2>&1; then
        curl -sL "$DOWNLOAD_URL" -o "$archive_path"
    elif command -v wget >/dev/null 2>&1; then
        wget -q "$DOWNLOAD_URL" -O "$archive_path"
    fi
    
    if [ ! -f "$archive_path" ]; then
        log_error "Download failed"
        exit 1
    fi
    
    log_info "Extracting archive..."
    tar -xzf "$archive_path" -C "$temp_dir"
    
    # Create install directory if needed
    mkdir -p "$INSTALL_DIR"
    
    # Install binary
    local binary_path="$temp_dir/${BINARY_NAME}-${PLATFORM}"
    if [ -f "$binary_path" ]; then
        cp "$binary_path" "$INSTALL_DIR/$BINARY_NAME"
        chmod +x "$INSTALL_DIR/$BINARY_NAME"
        log_success "TermPlay installed to $INSTALL_DIR/$BINARY_NAME"
    else
        log_error "Binary not found in archive"
        exit 1
    fi
    
    # Clean up
    rm -rf "$temp_dir"
}

# Verify installation
verify_installation() {
    if [ -x "$INSTALL_DIR/$BINARY_NAME" ]; then
        local version=$("$INSTALL_DIR/$BINARY_NAME" --version 2>/dev/null || echo "unknown")
        log_success "Installation successful! Version: $version"
    else
        log_error "Installation failed"
        exit 1
    fi
}

# Add to PATH if needed
setup_path() {
    if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
        log_warning "$INSTALL_DIR is not in your PATH"
        
        # Determine which profile file to modify
        local profile_file=""
        if [ -f "$HOME/.zshrc" ]; then
            profile_file="$HOME/.zshrc"
        elif [ -f "$HOME/.bashrc" ]; then
            profile_file="$HOME/.bashrc"
        elif [ -f "$HOME/.bash_profile" ]; then
            profile_file="$HOME/.bash_profile"
        elif [ -f "$HOME/.profile" ]; then
            profile_file="$HOME/.profile"
        fi
        
        if [ -n "$profile_file" ]; then
            echo "" >> "$profile_file"
            echo "# Added by TermPlay install script" >> "$profile_file"
            echo "export PATH=\"\$PATH:$INSTALL_DIR\"" >> "$profile_file"
            log_info "PATH updated in $profile_file"
            log_warning "Restart your terminal or run: source $profile_file"
        else
            log_warning "Unable to detect shell profile file"
            log_info "Please manually add this line to your shell profile:"
            echo "export PATH=\"\$PATH:$INSTALL_DIR\""
        fi
    fi
}

# Show final instructions
show_completion_message() {
    echo ""
    log_success "🎉 Installation complete!"
    echo ""
    echo "To start playing:"
    echo "  $INSTALL_DIR/$BINARY_NAME"
    echo ""
    echo "Or if $INSTALL_DIR is in your PATH:"
    echo "  $BINARY_NAME"
    echo ""
    echo "Available games:"
    echo "  $BINARY_NAME game snake"
    echo "  $BINARY_NAME game tetris"
    echo "  $BINARY_NAME list"
    echo ""
    echo "Help:"
    echo "  $BINARY_NAME --help"
    echo ""
}

# Main script
main() {
    echo "🎮 Installing TermPlay"
    echo "========================="
    
    detect_platform
    get_latest_version
    build_download_url
    download_and_install
    verify_installation
    setup_path
    show_completion_message
}

# Pre-checks
if [ "$EUID" -eq 0 ]; then
    log_error "Do not run this script as root"
    exit 1
fi

# Run installation
main "$@"