#!/bin/sh
# Install or uninstall workfetch on Linux / macOS.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/KaiWesterschwiensterdt/workfetch/main/install.sh | sh
#   curl -fsSL ... | sh -s -- --version v0.2.0
#   curl -fsSL ... | sh -s -- --uninstall

set -eu

REPO="KaiWesterschwiensterdt/workfetch"
BIN_NAME="workfetch"

# --- Defaults ---
VERSION=""
UNINSTALL=0

# --- Parse args ---
while [ $# -gt 0 ]; do
    case "$1" in
        --version)  VERSION="$2"; shift 2 ;;
        --uninstall) UNINSTALL=1; shift ;;
        *)          echo "Unknown option: $1"; exit 1 ;;
    esac
done

# --- Determine install directory ---
if [ "$(id -u)" -eq 0 ]; then
    INSTALL_DIR="/usr/local/bin"
else
    INSTALL_DIR="${HOME}/.local/bin"
fi

# --- Helpers ---
info()  { printf '\033[0;36m%s\033[0m\n' "$*"; }
ok()    { printf '\033[0;32m%s\033[0m\n' "$*"; }
warn()  { printf '\033[0;33m%s\033[0m\n' "$*"; }
err()   { printf '\033[0;31m%s\033[0m\n' "$*" >&2; }

detect_target() {
    os="$(uname -s)"
    arch="$(uname -m)"

    case "$os" in
        Linux*)  os_part="unknown-linux-gnu" ;;
        Darwin*) os_part="apple-darwin" ;;
        *)       err "Unsupported OS: $os"; exit 1 ;;
    esac

    case "$arch" in
        x86_64|amd64)   arch_part="x86_64" ;;
        arm64|aarch64)  arch_part="aarch64" ;;
        *)              err "Unsupported architecture: $arch"; exit 1 ;;
    esac

    echo "${arch_part}-${os_part}"
}

get_latest_version() {
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | head -1 | sed 's/.*"tag_name": *"//;s/".*//'
    elif command -v wget >/dev/null 2>&1; then
        wget -qO- "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | head -1 | sed 's/.*"tag_name": *"//;s/".*//'
    else
        err "Neither curl nor wget found. Please install one of them."
        exit 1
    fi
}

download() {
    url="$1"
    dest="$2"
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$url" -o "$dest"
    elif command -v wget >/dev/null 2>&1; then
        wget -q "$url" -O "$dest"
    fi
}

# --- Uninstall ---
if [ "$UNINSTALL" -eq 1 ]; then
    echo ""
    info "Uninstalling workfetch..."
    if [ -f "${INSTALL_DIR}/${BIN_NAME}" ]; then
        rm -f "${INSTALL_DIR}/${BIN_NAME}"
        ok "  Removed ${INSTALL_DIR}/${BIN_NAME}"
    else
        warn "  Binary not found at ${INSTALL_DIR}/${BIN_NAME}, skipping."
    fi
    echo ""
    ok "workfetch has been uninstalled."
    exit 0
fi

# --- Install ---
echo ""
info "Installing workfetch..."

# Determine version
if [ -z "$VERSION" ]; then
    printf '  Fetching latest version... '
    VERSION="$(get_latest_version)"
    if [ -z "$VERSION" ]; then
        err "Could not determine latest version."
        exit 1
    fi
    echo "$VERSION"
fi

# Determine target
TARGET="$(detect_target)"
ASSET_NAME="workfetch-${VERSION}-${TARGET}.tar.gz"
DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${VERSION}/${ASSET_NAME}"

# Create install directory
mkdir -p "$INSTALL_DIR"

# Download
TMPDIR_DL="$(mktemp -d)"
ARCHIVE="${TMPDIR_DL}/${ASSET_NAME}"
printf '  Downloading %s... ' "$ASSET_NAME"
if ! download "$DOWNLOAD_URL" "$ARCHIVE"; then
    err "Failed to download from: $DOWNLOAD_URL"
    err "Please check that version '$VERSION' exists for target '$TARGET'."
    rm -rf "$TMPDIR_DL"
    exit 1
fi
echo "done"

# Extract
printf '  Extracting... '
tar xzf "$ARCHIVE" -C "$TMPDIR_DL"
echo "done"

# Find and install binary
BIN_PATH="$(find "$TMPDIR_DL" -name "$BIN_NAME" -type f | head -1)"
if [ -z "$BIN_PATH" ]; then
    err "Could not find $BIN_NAME in the archive."
    rm -rf "$TMPDIR_DL"
    exit 1
fi
cp "$BIN_PATH" "${INSTALL_DIR}/${BIN_NAME}"
chmod +x "${INSTALL_DIR}/${BIN_NAME}"

# Cleanup
rm -rf "$TMPDIR_DL"

echo ""
ok "workfetch ${VERSION} installed successfully!"
echo "  Location: ${INSTALL_DIR}/${BIN_NAME}"

# Check PATH
case ":${PATH}:" in
    *":${INSTALL_DIR}:"*) ;;
    *)
        echo ""
        warn "  ${INSTALL_DIR} is not on your PATH."
        echo "  Add it by running:"
        echo ""
        if [ -f "${HOME}/.zshrc" ]; then
            echo "    echo 'export PATH=\"${INSTALL_DIR}:\$PATH\"' >> ~/.zshrc && source ~/.zshrc"
        elif [ -f "${HOME}/.bashrc" ]; then
            echo "    echo 'export PATH=\"${INSTALL_DIR}:\$PATH\"' >> ~/.bashrc && source ~/.bashrc"
        else
            echo "    export PATH=\"${INSTALL_DIR}:\$PATH\""
        fi
        ;;
esac

echo ""
echo "  Then run:"
echo "    workfetch"
echo ""
