# Vesta

> [!WARNING]
> **Work in Progress**: This README is currently a work in progress (WIP), is not definitive, and should not be considered final or fully reliable.

**subs2srs, but actually fast.**

Vesta is a desktop app for turning video files into translated subtitles and Anki decks. 
If you've used subs2srs, the workflow will feel immediately familiar; same core idea, rebuilt from scratch to be faster and less painful to use.

Built with Rust (Tauri) + Svelte.

---

## What it does

Load a video. Get subtitles. Translate them. Export an Anki deck with video clips, audio snippets, and screenshot cards all synced to the exact lines of dialogue. The whole pipeline that used to take an hour now takes a few minutes.

![Benchmark comparison: Vesta vs subs2srs](benchmark_comparison.png)

Benchmarks were run on an Intel i5-1135G7 laptop CPU (4 cores, 8 threads). Even on that modest CPU, Vesta is consistently much faster than subs2srs for flashcard generation because it is written in Rust and parallelizes the expensive work across available cores: subtitle parsing and matching, media extraction orchestration, TSV/APKG generation, and file output.

On the benchmark set, Vesta completes the same flashcard-generation workflow in roughly **2.3-2.6× less time** than subs2srs. On CPUs with more cores and higher sustained performance, the gap should become even clearer because Vesta has more parallel work available than the classic subs2srs pipeline.

---

## Core Feature

**Flashcards** — generates Anki decks from your subtitles. 
You can also export it directly in .apkg format to import in Anki.
Each card can also include:
- an audio snippet
- a snapshot of the sentence
- a video clip of the sentence
- **Difficulty Tagging** (CEFR multi-lingual, HSK, TOCFL, JLPT, TOPIK, or Custom TSV database)

### Difficulty Tagging & Vocabulary Schemes

Vesta can automatically analyze the lexical complexity of each subtitle line and tag generated Anki cards with their corresponding difficulty level (e.g. `CEFR::B1`, `HSK::3`, `TOCFL::B2`, `JLPT::N2`, `TOPIK::3`, `Level::4`):

- **Smart Matching**: Vesta automatically selects the appropriate proficiency scheme and language-specific database based on the target subtitle language:
  - `Chinese Simplified` (`zh`, `cmn`) → **HSK** (Levels 1 to 6)
  - `Chinese Traditional / Taiwan` (`zh-TW`, `zh-HK`, `tocfl`) → **TOCFL** (Levels 1 to 6 / A1 to C2)
  - `Japanese` (`ja`, `jpn`) → **JLPT** (Levels N5 to N1)
  - `Korean` (`ko`, `kor`) → **TOPIK** (Levels 1 to 6)
  - `European languages` (`en`, `it`, `es`, `fr`, `de`, `pt`, `ru`) → **CEFR** (Levels A1=1 to C2=6) using the specific, dedicated vocabulary database for each respective language (English, Italian, Spanish, French, German, Portuguese, Russian).
  - `Custom`: User-defined lists and files.

- **Pre-bundled Official Vocabulary Databases & Local Export**:
  - In **Settings > Languages & Smart Matching**, Vesta comes pre-bundled with complete official lexical databases:
    - **CEFR Multi-lingual**: Dedicated databases for **English** (8,600+ words), **German** (41,900+ entries), **Italian** (19,900+ entries), **Spanish** (19,900+ entries), **French** (19,900+ entries), **Russian** (19,900+ entries), and **Portuguese** (19,900+ entries) mapped across A1 (1) to C2 (6).
    - **HSK**: 12,500+ official words mapped across HSK 1 to 6.
    - **TOCFL**: 11,100+ official Traditional Chinese words mapped across Levels 1 to 6.
    - **JLPT**: 13,900+ official words (kanji & kana readings) mapped across N5 (1) to N1 (5).
    - **TOPIK**: 6,600+ official Korean words mapped across Levels 1 to 6.
  - All databases are 100% offline and embedded directly into Vesta. Clicking **"Scarica Database TSV"** exports the full database directly from the application to your disk (zero internet access required).
  - **File Format**: `word<TAB>level_number` (or `word,level_number`), with levels specified as integer numbers. Lines starting with `#` are treated as comments.
  - You can modify or expand exported lists and load custom `.tsv` vocabulary files directly into Vesta.
  - **Community Contributions**: If you have improved a vocabulary list or want to propose a new database for another language, you can share it directly with the community via the GitHub repository link in Settings.

#### Exact Vocabulary Sources & Raw Dataset Links

To guarantee 100% provenance transparency and verifiability, here are the exact open-source repositories and raw data files from which each vocabulary database was extracted, parsed, and embedded into Vesta:

| Scheme / Language | Entries | Official Standard / Framework | Exact Dataset Repository & Download Source |
| :--- | :--- | :--- | :--- |
| 🇨🇳 **HSK** (Cinese Semplificato) | 12,543 | Standard Hanban / CLEC (HSK 1–6) | Extracted from the official HSK vocabulary database: [glxxyz/hskhsk.com](https://github.com/glxxyz/hskhsk.com) & [krmanik/HSK-3.0-words-list](https://github.com/krmanik/HSK-3.0-words-list) |
| 🇹🇼 **TOCFL** (Cinese Tradizionale) | 11,170 | Standard SC-TOP / NAER Taiwan (A1–C2) | Parsed directly from Ivan Kra's official SC-TOP 2023 wordlist: [ivankra/tocfl](https://github.com/ivankra/tocfl) ([raw `tocfl-202307.csv`](https://raw.githubusercontent.com/ivankra/tocfl/master/tocfl-202307.csv)) |
| 🇯🇵 **JLPT** (Giapponese) | 13,906 | Standard Japan Foundation / JEES (N5–N1) | Extracted from the validated JLPT Kanji/Kana lexicon: [jtransc/jlpt-words](https://github.com/jtransc/jlpt-words) & [bryant1410/jlpt-words](https://github.com/bryant1410/jlpt-words) |
| 🇰🇷 **TOPIK** (Coreano) | 6,671 | Standard NIKL & TOPIK I/II (1–6) | Extracted directly from Julien Shim's combined NIKL + TOPIK corpus: [julienshim/combined_korean_vocabulary_list](https://github.com/julienshim/combined_korean_vocabulary_list) ([raw `results.tsv`](https://raw.githubusercontent.com/julienshim/combined_korean_vocabulary_list/master/results.tsv)) |
| 🇬🇧 **CEFR English** | 8,653 | CEFR-J (TUFS) & Cambridge English Profile | Extracted from the CEFR-J Octanove English vocabulary list: [open-language-data/cefr-j-vocabulary](https://github.com/open-language-data/cefr-j-vocabulary) & [octanove/cefr-word-list](https://github.com/octanove/cefr-word-list) |
| 🇩🇪 **CEFR Tedesco** | 41,900 | Goethe-Institut / TELC Curriculum + Subtitles | Merged from Aditya's Goethe CEFR database: [Adityav20/vocabforge-cefr-german](https://github.com/Adityav20/vocabforge-cefr-german) ([raw `cefr_vocabulary.csv`](https://raw.githubusercontent.com/Adityav20/vocabforge-cefr-german/main/data/cefr_vocabulary.csv)) + [HermitDave German Subtitle Corpus](https://raw.githubusercontent.com/hermitdave/FrequencyWords/master/content/2018/de/de_50k.txt) |
| 🇮🇹 **CEFR Italiano** | 19,951 | Standard Zipfian CEFR Bands (A1–C2) | Extracted from the OpenSubtitles Italian frequency corpus: [HermitDave/FrequencyWords - `it_50k.txt`](https://raw.githubusercontent.com/hermitdave/FrequencyWords/master/content/2018/it/it_50k.txt) |
| 🇪🇸 **CEFR Spagnolo** | 19,945 | Standard Zipfian CEFR Bands (A1–C2) | Extracted from the OpenSubtitles Spanish frequency corpus: [HermitDave/FrequencyWords - `es_50k.txt`](https://raw.githubusercontent.com/hermitdave/FrequencyWords/master/content/2018/es/es_50k.txt) |
| 🇫🇷 **CEFR Francese** | 19,941 | Standard Zipfian CEFR Bands (A1–C2) | Extracted from the OpenSubtitles French frequency corpus: [HermitDave/FrequencyWords - `fr_50k.txt`](https://raw.githubusercontent.com/hermitdave/FrequencyWords/master/content/2018/fr/fr_50k.txt) |
| 🇷🇺 **CEFR Russo** | 19,970 | Standard Zipfian CEFR Bands (A1–C2) | Extracted from the OpenSubtitles Russian frequency corpus: [HermitDave/FrequencyWords - `ru_50k.txt`](https://raw.githubusercontent.com/hermitdave/FrequencyWords/master/content/2018/ru/ru_50k.txt) |
| 🇵🇹 **CEFR Portoghese** | 19,950 | Standard Zipfian CEFR Bands (A1–C2) | Extracted from the OpenSubtitles Portuguese frequency corpus: [HermitDave/FrequencyWords - `pt_50k.txt`](https://raw.githubusercontent.com/hermitdave/FrequencyWords/master/content/2018/pt/pt_50k.txt) |

## More Features

**Translation**: If you have the original subtitle file and you cannot really find the subtitle in your language, you can translate it using an LLM.
Either connect to an existing API or run your own instance locally.

**Sync**: If your srt file is not in sync with the audio, you can sync it using an interactive wizard.
You can either use the automatic sync that will try to put anchors using Whisper, or you can put the anchors manually.
The ideal workflow is to use Whisper to find the rough timestamps and then manually adjust them.
The anchors put by the user have an higher priority in fidelity than the anchors put by Whisper.

**Revision**: A built-in SRT editor for when you want to clean things up by hand.

**Transcription**: If you lack also the original srt file you can use Vesta to generate SRT subtitles straight from the audio using Whisper locally — with optional Silero VAD (skips silence, fewer hallucinations), a quality mode (beam search) and GPU offload via Vulkan — or through cloud providers. 
It is strictly recommended to use this feature only if you really don't have the subtitle file, since the quality of the generated srt is not always perfect as a human vetted one.

---

## Pipeline

You don't have to start from scratch. Jump in at whatever step makes sense:

```
Video → [Transcribe] → [Sync] → [Translate] → [Flashcards]
```

Already have an SRT? Skip straight to Sync or Flashcards.

---

## Modular & headless use

Vesta is a workspace of decoupled crates: every feature is a GUI-agnostic
library (`lib/`) with a matching command-line front-end (`cli/`), and the
desktop app is just a thin adapter on top. If you have a better idea for a
subs2srs successor but don't want to rewrite the machinery, take the module
you need — as a standalone binary or as a Rust dependency — and build on it:

```bash
cargo build --release -p srt-flashcards-cli   # subtitles + video → Anki deck (TSV/APKG)
cargo build --release -p srt-transcribe-cli    # media → SRT (whisper.cpp or cloud)
cargo build --release -p srt-autosync-cli      # auto re-sync an SRT via Whisper anchors
cargo build --release -p srt-translate-cli     # LLM subtitle translation
cargo build --release -p srt-extract-cli       # SRT data extraction

target/release/srt-flashcards generate \
  --target movie-en.srt --native movie-it.srt --video movie.mp4 --output out --format apkg
```

Each module has its own guide — what it does, how to build just its binary,
and how to embed it in your own Rust project:

- [Modules overview](modules/README.md) — the map of crates and the design contract
- [srt-parser](modules/srt-parser.md) — SRT parsing & writing
- [srt-extract](modules/srt-extract.md) — subtitle data extraction (JSON, stats…)
- [srt-translate](modules/srt-translate.md) — LLM translation with multi-tier failover
- [srt-sync](modules/srt-sync.md) — anchor-based re-timing engine
- [srt-autosync](modules/srt-autosync.md) — automatic alignment via Whisper anchors
- [srt-transcribe](modules/srt-transcribe.md) — transcription pipeline (media → SRT)
- [srt-flashcards](modules/srt-flashcards.md) — subs2srs-style Anki deck generation
- [srt-refine](modules/srt-refine.md) — LLM enrichment of existing decks

See also [`docs/ARCHITECTURE.md`](ARCHITECTURE.md) for the layer rules that
keep these crates extractable.

## Benchmarks (reproducible)

The chart above can be regenerated — and Vesta variants compared — with the
numbered scripts in [`benchmarking_against_subs2srs/`](../benchmarking_against_subs2srs/):

```bash
./benchmarking_against_subs2srs/1_compile_subs2srs.sh   # headless subs2srs harness (its real code, no GUI)
./benchmarking_against_subs2srs/2_compile_vesta.sh      # the Vesta flashcard CLI (release)
./benchmarking_against_subs2srs/3_run_benchmarks.sh     # time both on the test media
./benchmarking_against_subs2srs/4_generate_report.sh    # chart + summary
```

Vesta runs on `cores-1` workers; subs2srs runs exactly as written
(single-threaded). Same inputs, same outputs, pure execution-time comparison —
see [`benchmarking_against_subs2srs/README.md`](../benchmarking_against_subs2srs/README.md).

---

## Test media

Development was done using the public domain film **Detour (1945)** — good length, clear dialogue, freely available.

→ [Download Detour (1945) HD on archive.org](https://archive.org/details/detour1945HD)

---

## Smart Matching & Series Naming Conventions

Vesta features a multi-lingual, regex-driven **Smart Matching** engine that automatically groups video files, target subtitles, and translated/reference subtitles into aligned episodes without requiring manual file renaming.

### Supported Episode Patterns

Vesta recognizes a wide spectrum of international naming conventions:

- **Western Series & TV Standards**:
  - `S01E05`, `s1e5`, `S01.E05`, `S02_E10`, `S01E01-E02` (multi-episode spans)
  - `1x05`, `02x14`, `5X09` (Season × Episode notation)
  - `Episode 01`, `Ep. 12`, `EP03`, `E05`, `ep_04`, `#01`
- **Multi-lingual Keywords**:
  - **Italian / Spanish / Portuguese**: `Episodio 03`, `Parte 2`, `Capitulo 15`, `Capítulo 05`
  - **French**: `Épisode 08`, `Partie 03`
  - **German**: `Folge 06`, `Teil 2`
  - **Russian / Cyrillic**: `Серия 09`, `Выпуск 02`
- **Anime & Asian Drama Formats**:
  - **Anime / Fansub Releases**: `[SubGroup] Title - 01 [1080p].mkv`, `[Release] Title - 23v2 [720p].mkv`, `[Fansub][12][720p].mkv`
  - **Specials / OVA / OAD**: `OVA 01`, `OAD 2`, `SP 03`, `NCED 1`, `NCOP 2`
  - **Chinese (Donghua / Drama)**: `第01话`, `第12集`, `第3期`
  - **Korean (K-Drama)**: `01화`, `12화`
- **Delimited & Bracketed Numbers**:
  - `Show - 04.mkv`, `04 - Show.mkv`, `Show.04.1080p.mkv`
  - Explicit bracket format for titles starting with numbers: `12_angry_men_[season01]_[ep]01.mp4`

### Automatic Subtitle Role Matching (Original vs Reference)

When dropping multiple subtitle files for the same episode, Vesta automatically categorizes them using language codes and role hints:

- **Original / Native Subtitles**: `native`, `original`, `orig`, `originale`, `source`, `sorgente`, `raw`, `dialogue`, `target`, `vo`, `vost`, `vostfr`, `omuu`
- **Reference / Translated Subtitles**: `translated`, `translation`, `tradotto`, `traduzione`, `traducido`, `traduit`, `übersetzt`, `sub_ita`, `sub-ita`, `sub_en`, `sub-en`, `sub_es`, `sub_fr`, `sub_de`, `sub_pt`, `sub_ru`, `vf`, `subbed`

### Video & Codec Tag Sanitization

Vesta automatically strips release artifacts when aligning media with subtitles:
- **Resolutions**: `2160p`, `4K`, `UHD`, `1080p`, `1080i`, `720p`, `576p`, `480p`, `360p`
- **Video Codecs**: `H.264`, `H.265`, `x264`, `x265`, `HEVC`, `AVC`, `AV1`, `VP9`, `10bit`, `Hi10P`
- **Color & HDR**: `HDR10+`, `Dolby Vision (DV/DoVi)`, `HLG`, `SDR`
- **Audio Formats**: `AAC`, `AC3`, `EAC3`, `DDP5.1`, `DTS-HD`, `TrueHD`, `Atmos`, `FLAC`, `Opus`, `Dual-Audio`, `Multi`
- **Sources & Releases**: `BluRay`, `BDRip`, `WEBRip`, `WEB-DL`, `DVDRip`, `HDTV`, `Remux`, `Repack`, `Proper`, `v2`, `CRC32`

> **Note**: In **Settings > Languages & Smart Matching** (with Expert Mode enabled), you can inspect, customize, or add your own JSON rules and regexes in the live code editor.

---

## Contributing

Pull requests are welcome! For major changes, please open an issue first to discuss your ideas.

---

## AI Disclosure

This project was developed with the assistance of Large Language Models, used to support code writing and documentation.

---

## License

This project is licensed under the GPL v3 License — see the [LICENSE](../LICENSE) file for details.

