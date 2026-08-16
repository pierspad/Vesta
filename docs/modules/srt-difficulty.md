# srt-difficulty

Lexical difficulty analysis, vocabulary profiling, and proficiency tagging engine.

## What it does

`srt-difficulty` analyzes sentences from subtitles and cards to determine their lexical complexity according to international language proficiency standards.

### Supported Language Schemes, Built-in Databases & Provenance

- 🇨🇳 **HSK (Simplified Chinese)** — `12,543 words` (HSK 1–6): Official Hanban / [CTI](http://www.chinesetest.cn/) syllabus & [Ministry of Education PRC](http://www.moe.gov.cn/) word lists with Simplified/Traditional cross-character variants.
- 🇹🇼 **TOCFL (Traditional Chinese)** — `11,170 words` (Levels 1–6 / Novice–Superior): Official Taiwan Ministry of Education [SC-TOP](https://tocfl.edu.tw/) *8000 Vocabulary List (華語文能力測驗 8000詞表)* ([Downloads](https://tocfl.edu.tw/index.php/exam/download)).
- 🇯🇵 **JLPT (Japanese)** — `13,930 words` (Levels N5–N1): [Jonathan Waller's JLPT (Tanos)](https://www.tanos.co.uk/jlpt/) & [EDRDG / JMdict](https://www.edrdg.org/jmdict/j_jmdict.html) Japanese lexicon including kanji expressions, kana readings, and dictionary forms.
- 🇰🇷 **TOPIK (Korean)** — `6,671 words` (Levels 1–6): Official [NIIED (국립국제교육원)](https://www.topik.go.kr/) & [National Institute of Korean Language (국립국어원)](https://www.korean.go.kr/) standard vocabulary syllabus for TOPIK.
- 🇬🇧 **CEFR English** — `8,845 words` (Levels A1–C2): [Cambridge English Profile (EVP)](https://www.englishprofile.org/) & [Oxford 3000/5000](https://www.oxfordlearnersdictionaries.com/wordlists/) aligned with [Council of Europe CEFR](https://www.coe.int/en/web/common-european-framework-reference-languages).
- 🇪🇺 **CEFR Multi-lingual European Databases** (🇩🇪 German `41,900`, 🇮🇹 Italian `19,951`, 🇪🇸 Spanish `19,945`, 🇫🇷 French `19,941`, 🇵🇹 Portuguese `19,950`, 🇷🇺 Russian `19,970 words`): Frequency-based corpora from [OPUS OpenSubtitles (HermitDave)](https://opus.nlpl.eu/OpenSubtitles.php) & [Leipzig Corpora](https://wortschatz.uni-leipzig.de/) partitioned into 6 CEFR difficulty brackets (A1: Top 1K, A2: +1.5K, B1: +2.5K, B2: +3.5K, C1: +4.5K, C2: 13K–20K+).

All official databases are embedded directly into the crate binary (zero external file or internet lookup needed).

### Custom Vocabulary Databases
Users can load custom TSV vocabulary lists (`word\tlevel`) with custom prefix tags (e.g. `Frequency::Top1000`).

### Unknown Word Policy
Configurable behavior for unlisted words:
- `Ignore`: Evaluate card level based only on recognized vocabulary.
- `Highest`: Treat cards with unknown words as the maximum level.
- `Level 0`: Assign cards with unrecognized vocabulary to Level 0 (e.g. `HSK::0`, `CEFR::0`, `JLPT::0`, `Level::0`).

## Crate Info

- **Path**: `lib/srt-difficulty`
- **Dependencies**: `serde`, `serde_json`, `anyhow`

## Rust API Example

```rust
use srt_difficulty::{LevelScheme, LevelTable, AnalyzeOptions, analyze, tag_for};

// Load pre-bundled database for Japanese JLPT
let table = LevelTable::builtin(LevelScheme::Jlpt, "ja").expect("Built-in JLPT database exists");
let opts = AnalyzeOptions::default();

// Analyze sentence
let result = analyze("これは概念が複雑です。", &table, &opts);
println!("Level: {:?}", result.level); // Some(5) (N1)

// Generate hierarchical Anki tag
if let Some(level) = result.level {
    println!("Tag: {}", tag_for(LevelScheme::Jlpt, level)); // "JLPT::N1"
}
```
