#!/bin/bash
#
# sigc installer
# https://github.com/skelf-Research/sigc
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/skelf-Research/sigc/main/install.sh | bash
#   or
#   ./install.sh
#

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
DIM='\033[2m'
NC='\033[0m' # No Color

# Symbols
CHECK="${GREEN}✓${NC}"
CROSS="${RED}✗${NC}"
ARROW="${CYAN}→${NC}"
INFO="${BLUE}ℹ${NC}"

GITHUB_REPO="skelf-Research/sigc"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

# -----------------------------------------------------------------------------
# Helper functions
# -----------------------------------------------------------------------------

print_banner() {
    echo ""
    echo -e "${CYAN}${BOLD}"
    echo "  ┌─────────────────────────────────────────┐"
    echo "  │                                         │"
    echo "  │   ███████╗██╗ ██████╗  ██████╗         │"
    echo "  │   ██╔════╝██║██╔════╝ ██╔════╝         │"
    echo "  │   ███████╗██║██║  ███╗██║              │"
    echo "  │   ╚════██║██║██║   ██║██║              │"
    echo "  │   ███████║██║╚██████╔╝╚██████╗         │"
    echo "  │   ╚══════╝╚═╝ ╚═════╝  ╚═════╝         │"
    echo "  │                                         │"
    echo "  │   The Quant's Compiler                  │"
    echo "  │                                         │"
    echo "  └─────────────────────────────────────────┘"
    echo -e "${NC}"
    echo -e "  ${DIM}https://github.com/${GITHUB_REPO}${NC}"
    echo ""
}

info() {
    echo -e "${INFO} $1"
}

success() {
    echo -e "${CHECK} $1"
}

warn() {
    echo -e "${YELLOW}⚠${NC} $1"
}

error() {
    echo -e "${CROSS} $1"
}

step() {
    echo ""
    echo -e "${BOLD}${BLUE}[$1]${NC} ${BOLD}$2${NC}"
    echo -e "${DIM}─────────────────────────────────────────${NC}"
}

# -----------------------------------------------------------------------------
# Detection functions
# -----------------------------------------------------------------------------

detect_os() {
    local os
    case "$(uname -s)" in
        Linux*)  os="linux";;
        Darwin*) os="darwin";;
        MINGW*|MSYS*|CYGWIN*) os="windows";;
        *)       os="unknown";;
    esac
    echo "$os"
}

detect_arch() {
    local arch
    case "$(uname -m)" in
        x86_64|amd64)  arch="x86_64";;
        aarch64|arm64) arch="arm64";;
        armv7l)        arch="armv7";;
        *)             arch="unknown";;
    esac
    echo "$arch"
}

get_latest_release() {
    local release_url="https://api.github.com/repos/${GITHUB_REPO}/releases/latest"
    if command -v curl &> /dev/null; then
        curl -fsSL "$release_url" 2>/dev/null | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/'
    elif command -v wget &> /dev/null; then
        wget -qO- "$release_url" 2>/dev/null | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/'
    fi
}

check_command() {
    command -v "$1" &> /dev/null
}

# -----------------------------------------------------------------------------
# Installation methods
# -----------------------------------------------------------------------------

install_from_release() {
    local os="$1"
    local arch="$2"
    local version="$3"

    # Construct download URL
    local filename="sigc-${os}-${arch}.tar.gz"
    local download_url="https://github.com/${GITHUB_REPO}/releases/download/${version}/${filename}"

    info "Downloading ${filename}..."

    local tmp_dir=$(mktemp -d)
    trap "rm -rf $tmp_dir" EXIT

    if command -v curl &> /dev/null; then
        if ! curl -fsSL "$download_url" -o "$tmp_dir/sigc.tar.gz" 2>/dev/null; then
            return 1
        fi
    elif command -v wget &> /dev/null; then
        if ! wget -q "$download_url" -O "$tmp_dir/sigc.tar.gz" 2>/dev/null; then
            return 1
        fi
    else
        error "Neither curl nor wget found"
        return 1
    fi

    info "Extracting..."
    tar -xzf "$tmp_dir/sigc.tar.gz" -C "$tmp_dir"

    # Create install directory if needed
    mkdir -p "$INSTALL_DIR"

    # Install binary
    if [ -f "$tmp_dir/sigc" ]; then
        mv "$tmp_dir/sigc" "$INSTALL_DIR/sigc"
        chmod +x "$INSTALL_DIR/sigc"
        return 0
    else
        return 1
    fi
}

install_from_cargo() {
    info "Installing via cargo install sigc..."
    cargo install sigc
}

install_from_source() {
    local repo_dir="$1"

    info "Building from source..."

    if [ -z "$repo_dir" ] || [ ! -d "$repo_dir" ]; then
        local tmp_dir=$(mktemp -d)
        repo_dir="$tmp_dir/sigc"

        info "Cloning repository..."
        git clone --depth 1 "https://github.com/${GITHUB_REPO}.git" "$repo_dir"
    fi

    cd "$repo_dir"
    info "Building release binary (this may take a few minutes)..."
    cargo build --release

    mkdir -p "$INSTALL_DIR"
    cp target/release/sigc "$INSTALL_DIR/sigc"
    chmod +x "$INSTALL_DIR/sigc"
}

# -----------------------------------------------------------------------------
# Main installation flow
# -----------------------------------------------------------------------------

main() {
    print_banner

    # Step 1: Detect system
    step "1/4" "Detecting system"

    local os=$(detect_os)
    local arch=$(detect_arch)

    info "Operating System: ${BOLD}${os}${NC}"
    info "Architecture: ${BOLD}${arch}${NC}"

    if [ "$os" = "unknown" ] || [ "$arch" = "unknown" ]; then
        error "Unsupported system: ${os}-${arch}"
        exit 1
    fi

    success "System detected"

    # Step 2: Check for existing installation
    step "2/4" "Checking existing installation"

    if check_command sigc; then
        local current_version=$(sigc --version 2>/dev/null | head -1 || echo "unknown")
        warn "sigc is already installed: ${current_version}"
        echo ""
        read -p "Do you want to reinstall/upgrade? [y/N] " -n 1 -r
        echo ""
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            info "Installation cancelled"
            exit 0
        fi
    else
        info "No existing installation found"
    fi

    # Step 3: Install
    step "3/4" "Installing sigc"

    local installed=false

    # Method 1: Try GitHub release
    info "Checking for pre-built binary..."
    local latest_version=$(get_latest_release)

    if [ -n "$latest_version" ]; then
        info "Latest release: ${BOLD}${latest_version}${NC}"

        if install_from_release "$os" "$arch" "$latest_version"; then
            success "Installed from GitHub release"
            installed=true
        else
            warn "Pre-built binary not available for ${os}-${arch}"
        fi
    else
        warn "Could not fetch latest release info"
    fi

    # Method 2: Try cargo install
    if [ "$installed" = false ]; then
        if check_command cargo; then
            info "Falling back to cargo install..."
            if install_from_cargo; then
                success "Installed via cargo"
                installed=true
            else
                warn "cargo install failed"
            fi
        else
            warn "Cargo not found"
        fi
    fi

    # Method 3: Build from source
    if [ "$installed" = false ]; then
        info "Falling back to building from source..."

        if ! check_command cargo; then
            error "Rust toolchain required. Install from https://rustup.rs"
            echo ""
            echo -e "${ARROW} Run: ${CYAN}curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh${NC}"
            exit 1
        fi

        if ! check_command git; then
            error "Git is required to build from source"
            exit 1
        fi

        # Check if we're in the repo
        if [ -f "Cargo.toml" ] && grep -q 'name = "sigc"' Cargo.toml 2>/dev/null; then
            install_from_source "$(pwd)"
        else
            install_from_source ""
        fi

        success "Built from source"
        installed=true
    fi

    # Step 4: Verify installation
    step "4/4" "Verifying installation"

    # Check PATH
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        warn "$INSTALL_DIR is not in your PATH"
        echo ""
        echo -e "${ARROW} Add this to your shell profile (.bashrc, .zshrc, etc.):"
        echo ""
        echo -e "    ${CYAN}export PATH=\"\$PATH:$INSTALL_DIR\"${NC}"
        echo ""

        # Try to use full path for verification
        SIGC_BIN="$INSTALL_DIR/sigc"
    else
        SIGC_BIN="sigc"
    fi

    if [ -x "$INSTALL_DIR/sigc" ]; then
        local version=$("$INSTALL_DIR/sigc" --version 2>/dev/null | head -1 || echo "installed")
        success "sigc ${version}"
    else
        error "Installation verification failed"
        exit 1
    fi

    # Success message
    echo ""
    echo -e "${GREEN}${BOLD}════════════════════════════════════════════${NC}"
    echo -e "${GREEN}${BOLD}  Installation complete!${NC}"
    echo -e "${GREEN}${BOLD}════════════════════════════════════════════${NC}"
    echo ""
    echo -e "${BOLD}Next steps:${NC}"
    echo ""
    echo -e "  ${ARROW} Run the demo:        ${CYAN}./demo.sh${NC}"
    echo -e "  ${ARROW} Quick start:         ${CYAN}sigc run examples/momentum.sig${NC}"
    echo -e "  ${ARROW} Read the docs:       ${CYAN}https://docs.skelfresearch.com/sigc${NC}"
    echo ""
}

main "$@"
