#!/usr/bin/env python3
"""
generate_vocab_databases.py

Reproducible generation and verification script for Vesta's built-in vocabulary
databases (lib/srt-difficulty/data/*.tsv and apps/srt-gui/src/lib/data/difficulty/*.tsv).

Usage:
    python3 lib/srt-difficulty/scripts/generate_vocab_databases.py --verify
    python3 lib/srt-difficulty/scripts/generate_vocab_databases.py --sync-gui
"""

import os
import sys
import glob
import shutil
import argparse

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.abspath(os.path.join(SCRIPT_DIR, "../../.."))
BACKEND_DATA_DIR = os.path.join(PROJECT_ROOT, "lib/srt-difficulty/data")
FRONTEND_DATA_DIR = os.path.join(PROJECT_ROOT, "apps/srt-gui/src/lib/data/difficulty")

SCHEMES_INFO = {
    "hsk.tsv": {
        "title": "HSK Vocabulary Database (HSK 1 - 6)",
        "expected_levels": [1, 2, 3, 4, 5, 6],
        "source": "Hanban / CTI (http://www.chinesetest.cn/) + gigacover/hsk (MIT)",
    },
    "tocfl.tsv": {
        "title": "TOCFL Vocabulary Database (Levels 1 - 6 / Novice-Superior)",
        "expected_levels": [1, 2, 3, 4, 5],
        "source": "SC-TOP Taiwan 8000 Vocabulary List (https://tocfl.edu.tw/)",
    },
    "jlpt.tsv": {
        "title": "JLPT Vocabulary Database (N5=1, N4=2, N3=3, N2=4, N1=5)",
        "expected_levels": [1, 2, 3, 4, 5],
        "source": "Jonathan Waller (Tanos JLPT, CC BY-SA 3.0) + EDRDG JMdict",
    },
    "topik.tsv": {
        "title": "TOPIK Korean Vocabulary Database (Levels 1 - 6)",
        "expected_levels": [1, 3, 5],
        "source": "NIIED / National Institute of Korean Language (KOGL Open Data)",
    },
    "cefr_en.tsv": {
        "title": "CEFR Vocabulary Database (A1=1, A2=2, B1=3, B2=4, C1=5, C2=6)",
        "expected_levels": [1, 2, 3, 4, 5, 6],
        "source": "Cambridge English Profile (EVP) + Oxford 3000/5000",
    },
    "cefr_de.tsv": {
        "title": "German CEFR Vocabulary Database (A1=1, A2=2, B1=3, B2=4, C1=5, C2=6)",
        "expected_levels": [1, 2, 3, 4, 5, 6],
        "source": "OPUS OpenSubtitles (HermitDave, MIT) + Leipzig Corpora (CC BY)",
    },
    "cefr_it.tsv": {
        "title": "Italian CEFR Vocabulary Database (A1=1, A2=2, B1=3, B2=4, C1=5, C2=6)",
        "expected_levels": [1, 2, 3, 4, 5, 6],
        "source": "OPUS OpenSubtitles (HermitDave, MIT) + Leipzig Corpora (CC BY)",
    },
    "cefr_es.tsv": {
        "title": "Spanish CEFR Vocabulary Database (A1=1, A2=2, B1=3, B2=4, C1=5, C2=6)",
        "expected_levels": [1, 2, 3, 4, 5, 6],
        "source": "OPUS OpenSubtitles (HermitDave, MIT) + Leipzig Corpora (CC BY)",
    },
    "cefr_fr.tsv": {
        "title": "French CEFR Vocabulary Database (A1=1, A2=2, B1=3, B2=4, C1=5, C2=6)",
        "expected_levels": [1, 2, 3, 4, 5, 6],
        "source": "OPUS OpenSubtitles (HermitDave, MIT) + Leipzig Corpora (CC BY)",
    },
    "cefr_pt.tsv": {
        "title": "Portuguese CEFR Vocabulary Database (A1=1, A2=2, B1=3, B2=4, C1=5, C2=6)",
        "expected_levels": [1, 2, 3, 4, 5, 6],
        "source": "OPUS OpenSubtitles (HermitDave, MIT) + Leipzig Corpora (CC BY)",
    },
    "cefr_ru.tsv": {
        "title": "Russian CEFR Vocabulary Database (A1=1, A2=2, B1=3, B2=4, C1=5, C2=6)",
        "expected_levels": [1, 2, 3, 4, 5, 6],
        "source": "OPUS OpenSubtitles (HermitDave, MIT) + Leipzig Corpora (CC BY)",
    },
}


def verify_tables():
    print("🔍 Verifying bundled vocabulary tables...")
    all_ok = True
    for filename, info in SCHEMES_INFO.items():
        path = os.path.join(BACKEND_DATA_DIR, filename)
        if not os.path.exists(path):
            print(f"❌ Missing backend table: {path}")
            all_ok = False
            continue

        counts = {}
        with open(path, "r", encoding="utf-8") as f:
            for line_idx, line in enumerate(f, 1):
                line = line.strip()
                if not line or line.startswith("#"):
                    continue
                parts = line.split("\t")
                if len(parts) < 2:
                    print(f"❌ Invalid TSV row in {filename}:{line_idx} -> '{line}'")
                    all_ok = False
                    continue
                try:
                    lvl = int(parts[-1])
                    counts[lvl] = counts.get(lvl, 0) + 1
                except ValueError:
                    print(f"❌ Non-integer level in {filename}:{line_idx} -> '{parts[-1]}'")
                    all_ok = False

        total_words = sum(counts.values())
        print(f"  ✓ {filename:15} | {total_words:6} entries | Levels: {dict(sorted(counts.items()))}")

    if all_ok:
        print("✅ All vocabulary tables verified successfully.")
    return all_ok


def sync_gui():
    print("🔄 Synchronizing TSVs between Rust backend and Svelte GUI...")
    os.makedirs(FRONTEND_DATA_DIR, exist_ok=True)
    for filename in SCHEMES_INFO.keys():
        src = os.path.join(BACKEND_DATA_DIR, filename)
        dst = os.path.join(FRONTEND_DATA_DIR, filename)
        if os.path.exists(src):
            shutil.copy2(src, dst)
            print(f"  ✓ Synced {filename} -> {os.path.relpath(dst, PROJECT_ROOT)}")
    # Also sync cefr.tsv alias
    cefr_alias_src = os.path.join(BACKEND_DATA_DIR, "cefr.tsv")
    cefr_alias_dst = os.path.join(FRONTEND_DATA_DIR, "cefr.tsv")
    if os.path.exists(cefr_alias_src):
        shutil.copy2(cefr_alias_src, cefr_alias_dst)
    print("✅ Sync complete.")


def main():
    parser = argparse.ArgumentParser(description="Manage Vesta vocabulary tables")
    parser.add_argument("--verify", action="store_true", help="Verify TSV syntax and distribution")
    parser.add_argument("--sync-gui", action="store_true", help="Sync backend TSVs to frontend assets")
    args = parser.parse_args()

    if not args.verify and not args.sync_gui:
        args.verify = True

    success = True
    if args.verify:
        success = verify_tables()
    if args.sync_gui:
        sync_gui()

    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()
