# srt-difficulty

Lexical difficulty analysis, vocabulary profiling, and proficiency tagging engine.

## What it does

`srt-difficulty` analyzes sentences from subtitles and cards to determine their lexical complexity according to international language proficiency standards.

### Supported Language Schemes & Built-in Databases

- 🇨🇳 **HSK** (Simplified Chinese): HSK 1–6 (12,500+ words) — [Official CTI](http://www.chinesetest.cn/) / [Ministry of Education PRC](http://www.moe.gov.cn/) · [HSK Open Datasets](https://github.com/gigacover/hsk)
- 🇹🇼 **TOCFL** (Traditional Chinese): Levels 1–6 (11,100+ words) — [SC-TOP Official Portal & 8000 Vocab Downloads](https://tocfl.edu.tw/)
- 🇯🇵 **JLPT** (Japanese): N5–N1 (13,900+ words across kanji/kana forms) — [Official JLPT](https://www.jlpt.jp/) · [Jonathan Waller's JLPT](https://www.tanos.co.uk/jlpt/) · [EDRDG / JMdict](https://www.edrdg.org/jmdict/j_jmdict.html)
- 🇰🇷 **TOPIK** (Korean): TOPIK 1–6 (6,600+ words) — [Official TOPIK Portal (NIIED)](https://www.topik.go.kr/) · [National Institute of Korean Language](https://www.korean.go.kr/)
- 🇪🇺 **CEFR Multi-lingual**: CEFR A1–C2 across English ([Cambridge English Profile](https://www.englishprofile.org/)), German ([Goethe-Institut](https://www.goethe.de/)), Italian ([CLIQ / UniStraPg](https://www.unistrapg.it/)), Spanish ([Instituto Cervantes](https://cvc.cervantes.es/)), French ([France Éducation International](https://www.france-education-international.fr/)), Russian ([TORFL / SPbU](https://testingcenter.spbu.ru/)), and Portuguese ([CAPLE](https://caple.letras.ulisboa.pt/))

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
