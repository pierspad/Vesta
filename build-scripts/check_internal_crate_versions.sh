#!/bin/bash

set -euo pipefail
export LC_ALL=C

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
PKGBUILD="$SCRIPT_DIR/PKGBUILD"
WORKSPACE_CARGO="$PROJECT_ROOT/Cargo.toml"

if [ ! -f "$PKGBUILD" ]; then
    echo -e "${RED}❌ PKGBUILD non trovato in $SCRIPT_DIR${NC}"
    exit 1
fi

if [ ! -f "$WORKSPACE_CARGO" ]; then
    echo -e "${RED}❌ Cargo.toml workspace non trovato in $PROJECT_ROOT${NC}"
    exit 1
fi

# Estrae pkgver in modo portabile (compatibile anche con ambienti Bash su Windows).
VERSION=$(awk -F'=' '/^pkgver[[:space:]]*=/{print $2; exit}' "$PKGBUILD" | tr -d '\r' | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//' -e 's/^"//' -e 's/"$//' || true)
if [ -z "$VERSION" ]; then
    echo -e "${RED}❌ Impossibile leggere pkgver da PKGBUILD${NC}"
    exit 1
fi

ERRORS=0

check_package_version() {
    local file="$1"
    local relative="${file#$PROJECT_ROOT/}"
    local current

    current=$(sed -n 's/^version = "\([^"]*\)"/\1/p' "$file" | tr -d '\r' | head -n 1)
    if [ -z "$current" ]; then
        echo -e "  ${RED}✗${NC} $relative — versione package non trovata"
        ((ERRORS++))
        return
    fi

    if [ "$current" != "$VERSION" ]; then
        echo -e "  ${RED}✗${NC} $relative — version=$current (atteso: $VERSION)"
        ((ERRORS++))
    else
        echo -e "  ${GREEN}✓${NC} $relative — version=$current"
    fi
}

echo -e "${YELLOW}Controllo versioni package core/lib (atteso: $VERSION)...${NC}"
while IFS= read -r crate_toml; do
    check_package_version "$crate_toml"
done < <(find "$PROJECT_ROOT/core" "$PROJECT_ROOT/lib" -mindepth 2 -maxdepth 2 -name Cargo.toml | sort)

echo -e "${YELLOW}Controllo [workspace.dependencies] internal crates...${NC}"
for crate in srt-parser srt-extract srt-sync srt-translate; do
    line=$(grep -E "^${crate}\s*=\s*\{[^}]*path\s*=\s*\"" "$WORKSPACE_CARGO" || true)
    if [ -z "$line" ]; then
        echo -e "  ${RED}✗${NC} Cargo.toml — dipendenza $crate con path non trovata"
        ((ERRORS++))
        continue
    fi

    declared=$(echo "$line" | tr -d '\r' | sed -n 's/.*version = "\([^"]*\)".*/\1/p')
    if [ -z "$declared" ]; then
        echo -e "  ${RED}✗${NC} Cargo.toml — dipendenza $crate senza versione"
        ((ERRORS++))
        continue
    fi

    if [ "$declared" != "$VERSION" ]; then
        echo -e "  ${RED}✗${NC} Cargo.toml — $crate version=$declared (atteso: $VERSION)"
        ((ERRORS++))
    else
        echo -e "  ${GREEN}✓${NC} Cargo.toml — $crate version=$declared"
    fi
done

echo ""
if [ "$ERRORS" -gt 0 ]; then
    echo -e "${RED}❌ Coerenza versioni fallita (${ERRORS} errori)${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Coerenza versioni crate interni verificata (${VERSION})${NC}"
