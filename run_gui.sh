#!/bin/bash

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

if [ ! -d "apps/srt-gui" ]; then
    echo -e "${RED}❌ Errore: Esegui questo script dalla root del progetto (dove c'è la cartella apps/srt-gui).${NC}"
    echo -e "   Percorso attuale: $(pwd)"
    exit 1
fi

echo -e "${BLUE}🚀 Avvio SRT Tools GUI...${NC}"
echo ""

cd apps/srt-gui

if ! command -v npm &> /dev/null; then
    echo -e "${RED}❌ Errore: npm non trovato. Installa Node.js per continuare.${NC}"
    exit 1
fi

if [ ! -d "node_modules" ]; then
    echo -e "${YELLOW}📦 Installazione dipendenze frontend...${NC}"
    npm install
    if [ $? -ne 0 ]; then
        echo -e "${RED}❌ Errore durante l'installazione delle dipendenze.${NC}"
        exit 1
    fi
    echo ""
fi

if [ ! -f "node_modules/.bin/tauri" ]; then
    echo -e "${YELLOW}📦 Installazione Tauri CLI...${NC}"
    npm install @tauri-apps/cli
    echo ""
fi

echo -e "${GREEN}✅ Dipendenze OK${NC}"
echo -e "${BLUE}🖥️  Avvio Tauri in modalità sviluppo...${NC}"
echo -e "${YELLOW}   (Premi Ctrl+C per fermare)${NC}"
echo ""


export WEBKIT_DISABLE_COMPOSITING_MODE=1

export WHISPER_DONT_GENERATE_BINDINGS=1

npm run tauri dev
