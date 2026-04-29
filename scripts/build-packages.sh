
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
SRC_TAURI="$PROJECT_ROOT/src-tauri"
OUT_DIR="$PROJECT_ROOT/packages"

echo -e "${BOLD}BlurAutoClicker Linux Package Builder${NC}"
echo "========================================"

VERSION="$(grep '^version' "$SRC_TAURI/Cargo.toml" | head -1 | cut -d'"' -f2)"
ARCH="$(uname -m)"
echo -e "Version: ${BLUE}$VERSION${NC}"
echo -e "Arch:    ${BLUE}$ARCH${NC}"
echo ""


TAURI_TARGETS=()
MISSING_TOOLS=()

check_tool() {
    local cmd="$1"
    local target="$2"
    local label="$3"
    local pkg="$4"

    if command -v "$cmd" &> /dev/null; then
        TAURI_TARGETS+=("$target")
        echo -e "  ${GREEN}✓${NC} $label"
    else
        MISSING_TOOLS+=("$pkg")
        echo -e "  ${RED}✗${NC} $label ${YELLOW}(install: $pkg)${NC}"
    fi
}

echo "Checking packaging tools..."
check_tool "dpkg-deb" "deb" "Debian (.deb)" "dpkg"
check_tool "rpmbuild" "rpm" "RPM (.rpm)"     "rpm-build (Debian) / rpm-tools (Arch)"

TAURI_TARGETS+=("appimage")
echo -e "  ${GREEN}✓${NC} AppImage (Tauri bundled)"

echo ""


rm -rf "$OUT_DIR"
mkdir -p "$OUT_DIR"

FOUND=0


if [ ${
    TARGETS_STR="${TAURI_TARGETS[*]}"
    echo -e "Building Tauri bundles: ${BLUE}$TARGETS_STR${NC}"
    echo ""

    cd "$SRC_TAURI"
    export APPIMAGE_EXTRACT_AND_RUN=1
    export NO_STRIP=1
    if [ -n "${TAURI_SIGNING_PRIVATE_KEY:-}" ]; then
        cargo tauri build --bundles $TARGETS_STR
    else
        echo -e "  ${YELLOW}!${NC} Skipping updater artifacts (no TAURI_SIGNING_PRIVATE_KEY)"
        cargo tauri build --bundles $TARGETS_STR --config '{"bundle": {"createUpdaterArtifacts": false}}'
    fi

    echo ""
    echo -e "${BOLD}Collecting Tauri bundles...${NC}"

    BUNDLE_DIR="$SRC_TAURI/target/release/bundle"
    for target in "${TAURI_TARGETS[@]}"; do
        case $target in
            deb)
                for f in "$BUNDLE_DIR/deb/"*.deb; do
                    if [ -f "$f" ]; then
                        cp "$f" "$OUT_DIR/"
                        echo -e "  ${GREEN}✓${NC} $(basename "$f")"
                        FOUND=$((FOUND + 1))
                    fi
                done
                ;;
            rpm)
                for f in "$BUNDLE_DIR/rpm/"*.rpm; do
                    if [ -f "$f" ]; then
                        cp "$f" "$OUT_DIR/"
                        echo -e "  ${GREEN}✓${NC} $(basename "$f")"
                        FOUND=$((FOUND + 1))
                    fi
                done
                ;;
            appimage)
                for f in "$BUNDLE_DIR/appimage/"*.AppImage; do
                    if [ -f "$f" ]; then
                        cp "$f" "$OUT_DIR/"
                        echo -e "  ${GREEN}✓${NC} $(basename "$f")"
                        FOUND=$((FOUND + 1))
                    fi
                done
                ;;
        esac
    done
fi


echo ""
echo -e "${BOLD}Building portable archive...${NC}"

BINARY="$SRC_TAURI/target/release/BlurAutoClicker"
if [ ! -f "$BINARY" ]; then
    echo -e "  ${YELLOW}!${NC} Release binary not found, building..."
    cd "$SRC_TAURI"
    cargo build --release
fi

TMP_DIR="$(mktemp -d)"
PKG_NAME="BlurAutoClicker-${VERSION}-${ARCH}"
PKG_DIR="$TMP_DIR/$PKG_NAME"
mkdir -p "$PKG_DIR"


cp "$BINARY" "$PKG_DIR/BlurAutoClicker"
chmod +x "$PKG_DIR/BlurAutoClicker"


mkdir -p "$PKG_DIR/icons"
cp "$SRC_TAURI/icons/32x32.png" "$PKG_DIR/icons/"
cp "$SRC_TAURI/icons/128x128.png" "$PKG_DIR/icons/"
cp "$SRC_TAURI/icons/icon.png" "$PKG_DIR/icons/"


cat > "$PKG_DIR/BlurAutoClicker.desktop" <<EOF
[Desktop Entry]
Name=BlurAutoClicker
Comment=Fast and customizable auto clicker
Exec=\$(dirname \$(readlink -f \$0))/BlurAutoClicker
Icon=\$(dirname \$(readlink -f \$0))/icons/icon.png
Type=Application
Categories=Utility;
Terminal=false
EOF


cat > "$PKG_DIR/install.sh" <<'EOF'

set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
APPS_DIR="${APPS_DIR:-$HOME/.local/share/applications}"
ICON_DIR="${ICON_DIR:-$HOME/.local/share/icons/hicolor/128x128/apps}"

echo "Installing BlurAutoClicker..."
mkdir -p "$INSTALL_DIR" "$APPS_DIR" "$ICON_DIR"
cp "$SCRIPT_DIR/BlurAutoClicker" "$INSTALL_DIR/"
chmod +x "$INSTALL_DIR/BlurAutoClicker"
cp "$SCRIPT_DIR/icons/128x128.png" "$ICON_DIR/BlurAutoClicker.png"


cat > "$APPS_DIR/BlurAutoClicker.desktop" <<DESKTOP
[Desktop Entry]
Name=BlurAutoClicker
Comment=Fast and customizable auto clicker
Exec=$INSTALL_DIR/BlurAutoClicker
Icon=BlurAutoClicker
Type=Application
Categories=Utility;
Terminal=false
DESKTOP

update-desktop-database "$APPS_DIR" 2>/dev/null || true
echo "Done. Run 'BlurAutoClicker' or find it in your applications menu."
EOF
chmod +x "$PKG_DIR/install.sh"


cat > "$PKG_DIR/README.txt" <<EOF
BlurAutoClicker Linux ${VERSION}
================================

Portable archive. No package manager required.

Install:
  ./install.sh

Or run directly:
  ./BlurAutoClicker

Uninstall:
  rm ~/.local/bin/BlurAutoClicker
  rm ~/.local/share/applications/BlurAutoClicker.desktop
  rm ~/.local/share/icons/hicolor/128x128/apps/BlurAutoClicker.png
EOF


cd "$TMP_DIR"
tar czf "$OUT_DIR/${PKG_NAME}.tar.gz" "$PKG_NAME"
cd - > /dev/null
rm -rf "$TMP_DIR"

echo -e "  ${GREEN}✓${NC} ${PKG_NAME}.tar.gz"
FOUND=$((FOUND + 1))


echo ""
echo -e "${GREEN}${BOLD}Done!${NC} Packages in ${BLUE}packages/${NC}:"
ls -lh "$OUT_DIR/"

echo ""
if [ ${
    echo -e "${YELLOW}Note:${NC} Some package formats were skipped."
    echo "  Install missing tools to build them:"
    for pkg in "${MISSING_TOOLS[@]}"; do
        echo "    - $pkg"
    done
fi
