#!/bin/bash
# ===========================================================================
# build-aur.sh
# Build locale del pacchetto Arch (.pkg.tar.zst) per test.
# 1. Compila il .deb con Tauri
# 2. Lo usa come source per il PKGBUILD.local
# 3. Opzionalmente installa il pacchetto
# ===========================================================================

set -euo pipefail
export LC_ALL=C

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$SCRIPT_DIR"

# ── Leggi versione dal PKGBUILD ──────────────────────────────
VERSION=$(grep -Po '^pkgver=\K.*' PKGBUILD)
_PKGNAME=$(grep -Po '^_pkgname=\K.*' PKGBUILD)
echo -e "${BLUE}🏗️  VESTA — Build Arch Package v${VERSION}${NC}"
echo "=================================="

# ── Allinea prima tutti i file ───────────────────────────────
echo -e "${YELLOW}🔄 Allineamento informazioni progetto...${NC}"
bash "$SCRIPT_DIR/update_project_info.sh"
echo -e "${YELLOW}🔎 Verifica coerenza versioni interne...${NC}"
bash "$SCRIPT_DIR/check_internal_crate_versions.sh"
echo ""

# ── Build del .deb con Tauri ─────────────────────────────────
TAURI_DIR="$PROJECT_ROOT/apps/srt-gui"
echo -e "${YELLOW}📦 Build Tauri (.deb)...${NC}"
cd "$TAURI_DIR"

# Usa i binding vendorizzati di whisper-rs-sys per evitare mismatch bindgen host-dependent.
export WHISPER_DONT_GENERATE_BINDINGS=1

if [ ! -d "node_modules" ]; then
    echo -e "${YELLOW}   Installazione dipendenze frontend...${NC}"
    npm install
fi

npm run tauri build -- --bundles deb

# ── Trova e copia il .deb ────────────────────────────────────
cd "$SCRIPT_DIR"
DEB_PATH=$(find "$PROJECT_ROOT/target/release/bundle/deb" -name "*.deb" | head -n 1)

if [ ! -f "$DEB_PATH" ]; then
    echo -e "${RED}❌ .deb non trovato in target/release/bundle/deb/${NC}"
    exit 1
fi

echo -e "${GREEN}✅ .deb trovato: $(basename "$DEB_PATH")${NC}"
cp "$DEB_PATH" "${_PKGNAME}_${VERSION}_amd64.deb"

# ── Build del pacchetto Arch ─────────────────────────────────
echo -e "${YELLOW}📦 Packaging per Arch con PKGBUILD.local...${NC}"
rm -f ./*.pkg.tar.zst
makepkg -p PKGBUILD.local -sfc --noconfirm

PKG_FILE=$(ls "${_PKGNAME}"*.pkg.tar.zst 2>/dev/null | head -n 1)
if [ ! -f "$PKG_FILE" ]; then
    echo -e "${RED}❌ Creazione pacchetto Arch fallita${NC}"
    exit 1
fi

echo ""
echo -e "${GREEN}════════════════════════════════════════════${NC}"
echo -e "${GREEN}  ✅ Build completato!${NC}"
echo -e "${GREEN}  📦 $PWD/$PKG_FILE${NC}"
echo -e "${GREEN}════════════════════════════════════════════${NC}"

# ── Installazione opzionale ──────────────────────────────────
read -p "Vuoi installare il pacchetto ora? [y/N] " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    PKGNAME=$(grep -Po '^pkgname=\K.*' PKGBUILD)
    if pacman -Qi "$PKGNAME" &> /dev/null; then
        echo -e "${YELLOW}Rimozione versione precedente...${NC}"
        sudo pacman -Rns "$PKGNAME" --noconfirm
    fi
    echo -e "${YELLOW}Installazione...${NC}"
    sudo pacman -U "$PKG_FILE" --noconfirm
    echo -e "${GREEN}✅ Installato!${NC}"
else
    echo -e "${YELLOW}Installazione saltata.${NC}"
fi
