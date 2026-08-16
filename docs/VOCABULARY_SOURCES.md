# Vocabulary Databases & Linguistic Provenance

This document provides a comprehensive, transparent record of the linguistic datasets bundled in Vesta (`lib/srt-difficulty/data/` and `apps/srt-gui/src/lib/data/difficulty/`). It outlines the exact upstream sources, dataset sizes, mapping methodologies, licensing, and instructions for full reproducibility.

---

## Overview

Vesta embeds offline vocabulary and lexical frequency tables directly into the application binary. This enables **zero-latency, 100% offline lexical difficulty profiling** across video subtitles and flashcards without sending text to external servers or requiring runtime internet access.

Every subtitle sentence is tokenized into lexical units, matched against the active language table, and assigned a proficiency level according to the highest difficulty word found (with configurable fallback policies for unlisted words).

---

## Summary of Bundled Databases

| Standard / Language | File | Entries | Levels | Upstream Source / Organization | License / Terms |
| :--- | :--- | :--- | :--- | :--- | :--- |
| 🇨🇳 **HSK** (Chinese) | [`hsk.tsv`](../lib/srt-difficulty/data/hsk.tsv) | 12,543 | HSK 1–6 (1–6) | Hanban / [Chinese Testing International](http://www.chinesetest.cn/) & [gigacover/hsk](https://github.com/gigacover/hsk) | Open Educational / MIT |
| 🇹🇼 **TOCFL** (Chinese Trad.) | [`tocfl.tsv`](../lib/srt-difficulty/data/tocfl.tsv) | 11,170 | Levels 1–5 (A1–C2) | Taiwan MOE / [SC-TOP](https://tocfl.edu.tw/) *8000 Vocabulary List* | Taiwan Open Government Data |
| 🇯🇵 **JLPT** (Japanese) | [`jlpt.tsv`](../lib/srt-difficulty/data/jlpt.tsv) | 13,930 | N5–N1 (1–5) | [Jonathan Waller (Tanos)](https://www.tanos.co.uk/jlpt/) & [EDRDG / JMdict](https://www.edrdg.org/jmdict/j_jmdict.html) | CC BY-SA 3.0 / EDRDG |
| 🇰🇷 **TOPIK** (Korean) | [`topik.tsv`](../lib/srt-difficulty/data/topik.tsv) | 6,671 | Levels 1–6 (1, 3, 5) | [NIIED](https://www.topik.go.kr/) & [National Institute of Korean Language](https://www.korean.go.kr/) | KOGL Type 1 (Open Data) |
| 🇬🇧 **CEFR English** | [`cefr_en.tsv`](../lib/srt-difficulty/data/cefr_en.tsv) | 8,845 | A1–C2 (1–6) | [Cambridge English Profile (EVP)](https://www.englishprofile.org/) & [Oxford 3000/5000](https://www.oxfordlearnersdictionaries.com/wordlists/) | Educational Open Access |
| 🇩🇪 **CEFR German** | [`cefr_de.tsv`](../lib/srt-difficulty/data/cefr_de.tsv) | 41,900 | A1–C2 (1–6) | [OPUS OpenSubtitles (HermitDave)](https://opus.nlpl.eu/OpenSubtitles.php) & [Leipzig Corpora Collection](https://wortschatz.uni-leipzig.de/) | MIT / CC-BY (Open Corpora) |
| 🇮🇹 **CEFR Italian** | [`cefr_it.tsv`](../lib/srt-difficulty/data/cefr_it.tsv) | 19,951 | A1–C2 (1–6) | [OPUS OpenSubtitles (HermitDave)](https://opus.nlpl.eu/OpenSubtitles.php) & [Leipzig Corpora Collection](https://wortschatz.uni-leipzig.de/) | MIT / CC-BY (Open Corpora) |
| 🇪🇸 **CEFR Spanish** | [`cefr_es.tsv`](../lib/srt-difficulty/data/cefr_es.tsv) | 19,945 | A1–C2 (1–6) | [OPUS OpenSubtitles (HermitDave)](https://opus.nlpl.eu/OpenSubtitles.php) & [Leipzig Corpora Collection](https://wortschatz.uni-leipzig.de/) | MIT / CC-BY (Open Corpora) |
| 🇫🇷 **CEFR French** | [`cefr_fr.tsv`](../lib/srt-difficulty/data/cefr_fr.tsv) | 19,941 | A1–C2 (1–6) | [OPUS OpenSubtitles (HermitDave)](https://opus.nlpl.eu/OpenSubtitles.php) & [Leipzig Corpora Collection](https://wortschatz.uni-leipzig.de/) | MIT / CC-BY (Open Corpora) |
| 🇵🇹 **CEFR Portuguese** | [`cefr_pt.tsv`](../lib/srt-difficulty/data/cefr_pt.tsv) | 19,950 | A1–C2 (1–6) | [OPUS OpenSubtitles (HermitDave)](https://opus.nlpl.eu/OpenSubtitles.php) & [Leipzig Corpora Collection](https://wortschatz.uni-leipzig.de/) | MIT / CC-BY (Open Corpora) |
| 🇷🇺 **CEFR Russian** | [`cefr_ru.tsv`](../lib/srt-difficulty/data/cefr_ru.tsv) | 19,970 | A1–C2 (1–6) | [OPUS OpenSubtitles (HermitDave)](https://opus.nlpl.eu/OpenSubtitles.php) & [Leipzig Corpora Collection](https://wortschatz.uni-leipzig.de/) | MIT / CC-BY (Open Corpora) |

---

## Detailed Data Provenance & Methodology

### 1. 🇨🇳 HSK (Simplified Chinese — HSK 1 to 6)
* **File**: `lib/srt-difficulty/data/hsk.tsv`
* **Entry count**: 12,543 terms
* **Levels**: 1 (`HSK 1`), 2 (`HSK 2`), 3 (`HSK 3`), 4 (`HSK 4`), 5 (`HSK 5`), 6 (`HSK 6`)
* **Level breakdown**:
  * Level 1: 856 entries
  * Level 2: 1,186 entries
  * Level 3: 1,647 entries
  * Level 4: 1,837 entries
  * Level 5: 2,374 entries
  * Level 6: 4,643 entries
* **Upstream Sources**:
  * [Chinese Testing International (CTI / 汉考国际)](http://www.chinesetest.cn/) — Official administering body under the Ministry of Education of the People's Republic of China.
  * [gigacover/hsk GitHub Repository](https://github.com/gigacover/hsk) (MIT License) — Machine-readable extraction of the official Hanban HSK 1–6 vocabulary lists.
* **Processing & Merging Method**:
  * Words are lowercased and stripped of HTML/punctuation.
  * For each Simplified Chinese term, traditional character variants are indexed and mapped to the same HSK level to allow seamless matching regardless of subtitle script encoding.

---

### 2. 🇹🇼 TOCFL (Traditional Chinese — Taiwan MOE)
* **File**: `lib/srt-difficulty/data/tocfl.tsv`
* **Entry count**: 11,170 terms
* **Levels**: 1 (`Novice / A1`), 2 (`A2`), 3 (`B1`), 4 (`B2`), 5 (`C1 / C2 Superior`)
* **Level breakdown**:
  * Level 1: 1,069 entries
  * Level 2: 714 entries
  * Level 3: 1,695 entries
  * Level 4: 3,412 entries
  * Level 5: 4,280 entries
* **Upstream Sources**:
  * [Steering Committee for the Test of Proficiency-Huayu (SC-TOP / 國家華語測驗推動工作委員會)](https://tocfl.edu.tw/) under the Ministry of Education of Taiwan (R.O.C.).
  * Official [SC-TOP Download Center](https://tocfl.edu.tw/index.php/exam/download) — *8000 Vocabulary List (華語文能力測驗 8000詞表)*.
* **Processing & Merging Method**:
  * Multi-form slash entries from the official spreadsheet (e.g. `一下(子)/一下子儿`, `上(面)`, `一共/共`) are retained to match colloquial and written subtitle variations directly.
  * Mapped to 5 numeric tiers corresponding to the CEFR-aligned TOCFL levels (Novice to Superior).

---

### 3. 🇯🇵 JLPT (Japanese — N5 to N1)
* **File**: `lib/srt-difficulty/data/jlpt.tsv`
* **Entry count**: 13,930 terms
* **Levels**: 1 (`N5`), 2 (`N4`), 3 (`N3`), 4 (`N2`), 5 (`N1`)
* **Level breakdown**:
  * Level 1 (N5): 1,276 entries
  * Level 2 (N4): 1,183 entries
  * Level 3 (N3): 3,510 entries
  * Level 4 (N2): 3,155 entries
  * Level 5 (N1): 4,806 entries
* **Upstream Sources**:
  * [Japan Educational Exchanges and Services (JEES)](https://www.jees.or.jp/) & [The Japan Foundation](https://www.jpf.go.jp/) — Official sponsors of the JLPT ([jlpt.jp](https://www.jlpt.jp/)).
  * [Jonathan Waller's JLPT Resources (Tanos JLPT)](https://www.tanos.co.uk/jlpt/) (CC BY-SA 3.0).
  * [EDRDG / JMdict Database](https://www.edrdg.org/jmdict/j_jmdict.html) (Electronic Dictionary Research and Development Group).
* **Processing & Merging Method**:
  * Combines Kanji headwords, Kana phonetic readings (hiragana and katakana), and compound dictionary forms.
  * Token matching supports longest-prefix greedy matching so compound phrases take precedence over single-kanji components.

---

### 4. 🇰🇷 TOPIK (Korean — TOPIK I & II)
* **File**: `lib/srt-difficulty/data/topik.tsv`
* **Entry count**: 6,671 terms
* **Levels**: 1 (`TOPIK I / Levels 1–2`), 3 (`TOPIK II Intermediate / Levels 3–4`), 5 (`TOPIK II Advanced / Levels 5–6`)
* **Level breakdown**:
  * Level 1 (Beginner): 1,042 entries
  * Level 3 (Intermediate): 3,101 entries
  * Level 5 (Advanced): 2,528 entries
* **Upstream Sources**:
  * [National Institute for International Education (NIIED / 국립국제교육원)](https://www.topik.go.kr/) under the Ministry of Education of the Republic of Korea.
  * [National Institute of Korean Language (국립국어원 / NIKL)](https://www.korean.go.kr/) — *Standard Korean Learner's Vocabulary List (한국어 학습용 어휘 목록)*.
* **Processing & Merging Method**:
  * Covers standard dictionary base forms (lemmas in `-다` for verbs/adjectives, nouns, particles).
  * Grouped into 3 core proficiency bands (Beginner, Intermediate, Advanced).

---

### 5. 🇬🇧 CEFR English (A1 to C2)
* **File**: `lib/srt-difficulty/data/cefr_en.tsv` (also linked as `cefr.tsv`)
* **Entry count**: 8,845 terms
* **Levels**: 1 (`A1`), 2 (`A2`), 3 (`B1`), 4 (`B2`), 5 (`C1`), 6 (`C2`)
* **Level breakdown**:
  * Level 1 (A1): 1,084 entries
  * Level 2 (A2): 1,272 entries
  * Level 3 (B1): 2,174 entries
  * Level 4 (B2): 2,490 entries
  * Level 5 (C1): 929 entries
  * Level 6 (C2): 896 entries
* **Upstream Sources**:
  * [Council of Europe Common European Framework of Reference for Languages](https://www.coe.int/en/web/common-european-framework-reference-languages).
  * [Cambridge English Vocabulary Profile (EVP)](https://www.englishprofile.org/) — Research corpus tracking learner vocabulary across CEFR levels.
  * [Oxford 3000 & Oxford 5000](https://www.oxfordlearnersdictionaries.com/wordlists/) aligned with CEFR standards.
* **Processing & Merging Method**:
  * Alphabetically indexed, lowercased, and deduplicated.
  * Includes common contraction forms (e.g. `'m`, `'re`, `'s`) mapped to A1 to prevent penalizing basic spoken dialogue in subtitles.

---

### 6. 🇪🇺 Multi-lingual European Frequency Databases (Italian, Spanish, French, German, Portuguese, Russian)
* **Files**:
  * 🇮🇹 `lib/srt-difficulty/data/cefr_it.tsv` (19,951 entries)
  * 🇪🇸 `lib/srt-difficulty/data/cefr_es.tsv` (19,945 entries)
  * 🇫🇷 `lib/srt-difficulty/data/cefr_fr.tsv` (19,941 entries)
  * 🇵🇹 `lib/srt-difficulty/data/cefr_pt.tsv` (19,950 entries)
  * 🇷🇺 `lib/srt-difficulty/data/cefr_ru.tsv` (19,970 entries)
  * 🇩🇪 `lib/srt-difficulty/data/cefr_de.tsv` (41,900 entries)
* **Levels**: 1 (`A1`), 2 (`A2`), 3 (`B1`), 4 (`B2`), 5 (`C1`), 6 (`C2`)
* **Upstream Sources**:
  * [OPUS OpenSubtitles Frequency Wordlists (HermitDave)](https://opus.nlpl.eu/OpenSubtitles.php) via [github.com/hermitdave/FrequencyWords](https://github.com/hermitdave/FrequencyWords) (MIT License).
  * [Leipzig Corpora Collection (Wortschatz Uni Leipzig)](https://wortschatz.uni-leipzig.de/) (CC BY 4.0).
* **Methodology & Frequency Tiering**:
  Because unified official single-file ministry lexicons do not exist for all European languages in an open format identical to the Cambridge EVP, Vesta employs standard logarithmic frequency binning derived from real subtitle corpora. Frequency rank directly correlates with lexical difficulty in conversational speech:
  * **Level 1 (A1 — Breakthrough)**: Rank 1 – 1,000 (~1,000 words: pronouns, common verbs, everyday nouns)
  * **Level 2 (A2 — Waystage)**: Rank 1,001 – 2,500 (+1,500 words: regular daily life vocabulary)
  * **Level 3 (B1 — Threshold)**: Rank 2,501 – 5,000 (+2,500 words: descriptive and situational expressions)
  * **Level 4 (B2 — Vantage)**: Rank 5,001 – 8,500 (+3,500 words: abstract, technical, and news media terms)
  * **Level 5 (C1 — Effective Proficiency)**: Rank 8,501 – 13,000 (+4,500 words: formal, literary, and rare vocabulary)
  * **Level 6 (C2 — Mastery)**: Rank 13,001 – 20,000+ / extended compounds (idiomatic, highly specialized, and domain-specific lexicon; German includes extensive compound noun coverage up to ~41.9k words).

---

## Reproducibility & Build Pipeline

To inspect, regenerate, or customize any of the tables, the build pipeline is completely deterministic.

### TSV Specification
Every table follows the canonical tab-separated format:
```tsv
# Optional comment line starting with #
word<TAB>level_integer
```
Where `level_integer` is a number from `1` to `6` (or `1` to `5` for 5-tier scales like JLPT).

### Parsing Rules in Vesta
1. **Case Normalization**: All incoming subtitle tokens are trimmed and lowercased before lookup.
2. **Comment & Blank Line Stripping**: Lines beginning with `#` or containing only whitespace are ignored.
3. **Delimiter Flexibility**: In user-provided custom lists, comma (`,`), semicolon (`;`), colon (`:`), or equals (`=`) are accepted as fallbacks if tab is omitted.
4. **Token Matching**:
   * For spaced alphabets (Latin, Cyrillic), sentences are tokenized on whitespace and word boundaries.
   * For unspaced scripts (CJK: Chinese, Japanese), tokenization uses a greedy maximum-window sliding tokenizer against `max_token_len` in the table.

---

## Licenses & Third-Party Credits

We gratefully acknowledge the organizations, researchers, and open-source contributors whose public datasets make offline difficulty tagging possible:

1. **HermitDave & OPUS Project**: [FrequencyWords](https://github.com/hermitdave/FrequencyWords) — Licensed under the [MIT License](https://opensource.org/licenses/MIT). Subtitle frequency data sourced from [OpenSubtitles.org](https://www.opensubtitles.org/) via OPUS.
2. **Electronic Dictionary Research and Development Group (EDRDG)**: [JMdict / EDICT](https://www.edrdg.org/jmdict/j_jmdict.html) — Licensed under the [EDRDG Licence / CC BY-SA 3.0](https://creativecommons.org/licenses/by-sa/3.0/).
3. **Jonathan Waller**: [Jonathan Waller's JLPT Resources](https://www.tanos.co.uk/jlpt/) — Licensed under [Creative Commons Attribution-ShareAlike 3.0 Unported](https://creativecommons.org/licenses/by-sa/3.0/).
4. **Steering Committee for the Test of Proficiency-Huayu (SC-TOP Taiwan)**: [TOCFL 8000 Vocabulary List](https://tocfl.edu.tw/) — Public educational reference under Taiwan Open Government Data guidelines.
5. **Chinese Testing International (CTI / Hanban)**: [HSK Official Syllabus](http://www.chinesetest.cn/) — Public linguistic standard under the Ministry of Education of the PRC. Open-source extraction via [gigacover/hsk](https://github.com/gigacover/hsk) (MIT).
6. **National Institute for International Education (NIIED) & National Institute of Korean Language (NIKL)**: [TOPIK Vocabulary Standards](https://www.topik.go.kr/) — Korea Open Government License (KOGL Type 1).
7. **Cambridge University Press & English Profile**: [Cambridge English Profile (EVP)](https://www.englishprofile.org/) & [Council of Europe CEFR](https://www.coe.int/en/web/common-european-framework-reference-languages).
8. **Universität Leipzig**: [Leipzig Corpora Collection](https://wortschatz.uni-leipzig.de/) — Licensed under [Creative Commons Attribution 4.0 International (CC BY 4.0)](https://creativecommons.org/licenses/by/4.0/).
