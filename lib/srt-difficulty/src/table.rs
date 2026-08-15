use crate::scheme::LevelScheme;
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::sync::LazyLock;

#[derive(Debug, Clone)]
pub struct LevelTable {
    pub map: HashMap<String, u8>,
    pub max_token_len: usize,
    pub max_level: u8,
    pub scheme: LevelScheme,
}

impl LevelTable {
    pub fn from_tsv(input: &str, scheme: LevelScheme) -> Result<Self> {
        let mut map = HashMap::with_capacity(input.lines().count());
        let mut max_token_len = 0;
        let mut max_level = 0;

        for (idx, line) in input.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let (word, level_str) = if let Some((w, l)) = line.rsplit_once('\t') {
                (w, l)
            } else if let Some((w, l)) = line.rsplit_once(',') {
                (w, l)
            } else if let Some((w, l)) = line.rsplit_once(';') {
                (w, l)
            } else if let Some((w, l)) = line.rsplit_once('=') {
                (w, l)
            } else if let Some((w, l)) = line.rsplit_once(':') {
                (w, l)
            } else {
                continue;
            };

            let word = word.trim().to_lowercase();
            if word.is_empty() {
                continue;
            }

            let level: u8 = level_str
                .trim()
                .parse()
                .map_err(|e| anyhow!("Line {}: invalid level '{}': {}", idx + 1, level_str, e))?;

            let char_count = word.chars().count();
            if char_count > max_token_len {
                max_token_len = char_count;
            }

            if level > max_level {
                max_level = level;
            }

            map.insert(word, level);
        }

        Ok(Self {
            map,
            max_token_len,
            max_level,
            scheme,
        })
    }

    pub fn from_file<P: AsRef<std::path::Path>>(path: P, scheme: LevelScheme) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::from_tsv(&content, scheme)
    }

    pub fn builtin_ref(scheme: LevelScheme, lang: &str) -> &'static Self {
        let primary_lang = lang.split(['-', '_']).next().unwrap_or(lang).to_lowercase();
        match scheme {
            LevelScheme::Hsk => &HSK_TABLE,
            LevelScheme::Tocfl => &TOCFL_TABLE,
            LevelScheme::Jlpt => &JLPT_TABLE,
            LevelScheme::Topik => &TOPIK_TABLE,
            LevelScheme::Cefr => match primary_lang.as_str() {
                "it" | "ita" => &CEFR_IT_TABLE,
                "es" | "spa" => &CEFR_ES_TABLE,
                "fr" | "fra" | "fre" => &CEFR_FR_TABLE,
                "de" | "deu" | "ger" => &CEFR_DE_TABLE,
                "ru" | "rus" => &CEFR_RU_TABLE,
                "pt" | "por" => &CEFR_PT_TABLE,
                "zh" | "zho" | "chi" | "cmn" => &HSK_TABLE,
                "ja" | "jpn" | "jap" => &JLPT_TABLE,
                "ko" | "kor" => &TOPIK_TABLE,
                _ => &CEFR_EN_TABLE,
            },
            LevelScheme::Custom => &CEFR_EN_TABLE,
        }
    }

    pub fn builtin(scheme: LevelScheme, lang: &str) -> Result<Self> {
        Ok(Self::builtin_ref(scheme, lang).clone())
    }

    pub fn get(&self, word: &str) -> Option<u8> {
        self.map.get(word).copied()
    }
}

// Built-in official vocabulary databases embedded into the binary
const BUILTIN_HSK_TSV: &str = include_str!("../data/hsk.tsv");
const BUILTIN_TOCFL_TSV: &str = include_str!("../data/tocfl.tsv");
const BUILTIN_JLPT_TSV: &str = include_str!("../data/jlpt.tsv");
const BUILTIN_TOPIK_TSV: &str = include_str!("../data/topik.tsv");
const BUILTIN_CEFR_EN_TSV: &str = include_str!("../data/cefr_en.tsv");
const BUILTIN_CEFR_IT_TSV: &str = include_str!("../data/cefr_it.tsv");
const BUILTIN_CEFR_ES_TSV: &str = include_str!("../data/cefr_es.tsv");
const BUILTIN_CEFR_FR_TSV: &str = include_str!("../data/cefr_fr.tsv");
const BUILTIN_CEFR_DE_TSV: &str = include_str!("../data/cefr_de.tsv");
const BUILTIN_CEFR_RU_TSV: &str = include_str!("../data/cefr_ru.tsv");
const BUILTIN_CEFR_PT_TSV: &str = include_str!("../data/cefr_pt.tsv");

static HSK_TABLE: LazyLock<LevelTable> = LazyLock::new(|| {
    LevelTable::from_tsv(BUILTIN_HSK_TSV, LevelScheme::Hsk).expect("HSK TSV must parse")
});
static TOCFL_TABLE: LazyLock<LevelTable> = LazyLock::new(|| {
    LevelTable::from_tsv(BUILTIN_TOCFL_TSV, LevelScheme::Tocfl).expect("TOCFL TSV must parse")
});
static JLPT_TABLE: LazyLock<LevelTable> = LazyLock::new(|| {
    LevelTable::from_tsv(BUILTIN_JLPT_TSV, LevelScheme::Jlpt).expect("JLPT TSV must parse")
});
static TOPIK_TABLE: LazyLock<LevelTable> = LazyLock::new(|| {
    LevelTable::from_tsv(BUILTIN_TOPIK_TSV, LevelScheme::Topik).expect("TOPIK TSV must parse")
});
static CEFR_EN_TABLE: LazyLock<LevelTable> = LazyLock::new(|| {
    LevelTable::from_tsv(BUILTIN_CEFR_EN_TSV, LevelScheme::Cefr).expect("CEFR EN TSV must parse")
});
static CEFR_IT_TABLE: LazyLock<LevelTable> = LazyLock::new(|| {
    LevelTable::from_tsv(BUILTIN_CEFR_IT_TSV, LevelScheme::Cefr).expect("CEFR IT TSV must parse")
});
static CEFR_ES_TABLE: LazyLock<LevelTable> = LazyLock::new(|| {
    LevelTable::from_tsv(BUILTIN_CEFR_ES_TSV, LevelScheme::Cefr).expect("CEFR ES TSV must parse")
});
static CEFR_FR_TABLE: LazyLock<LevelTable> = LazyLock::new(|| {
    LevelTable::from_tsv(BUILTIN_CEFR_FR_TSV, LevelScheme::Cefr).expect("CEFR FR TSV must parse")
});
static CEFR_DE_TABLE: LazyLock<LevelTable> = LazyLock::new(|| {
    LevelTable::from_tsv(BUILTIN_CEFR_DE_TSV, LevelScheme::Cefr).expect("CEFR DE TSV must parse")
});
static CEFR_RU_TABLE: LazyLock<LevelTable> = LazyLock::new(|| {
    LevelTable::from_tsv(BUILTIN_CEFR_RU_TSV, LevelScheme::Cefr).expect("CEFR RU TSV must parse")
});
static CEFR_PT_TABLE: LazyLock<LevelTable> = LazyLock::new(|| {
    LevelTable::from_tsv(BUILTIN_CEFR_PT_TSV, LevelScheme::Cefr).expect("CEFR PT TSV must parse")
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_tsv_various_separators_and_comments() {
        let tsv_content = r#"
# Comment line to ignore
apple	1
banana,2
cherry;3
date=4
elderberry:5

# Another comment with spaces
"#;
        let table = LevelTable::from_tsv(tsv_content, LevelScheme::Custom).unwrap();
        assert_eq!(table.map.len(), 5);
        assert_eq!(table.get("apple"), Some(1));
        assert_eq!(table.get("banana"), Some(2));
        assert_eq!(table.get("cherry"), Some(3));
        assert_eq!(table.get("date"), Some(4));
        assert_eq!(table.get("elderberry"), Some(5));

        assert_eq!(table.max_level, 5);
        assert_eq!(table.max_token_len, "elderberry".chars().count()); // 10 chars
    }

    #[test]
    fn test_from_tsv_invalid_level_fails() {
        let bad_tsv = "word\tinvalid_number\n";
        assert!(LevelTable::from_tsv(bad_tsv, LevelScheme::Custom).is_err());
    }

    #[test]
    fn test_builtin_ref_language_aliases() {
        // French aliases: fr, fra, fre
        let t1 = LevelTable::builtin_ref(LevelScheme::Cefr, "fr");
        let t2 = LevelTable::builtin_ref(LevelScheme::Cefr, "fra");
        let t3 = LevelTable::builtin_ref(LevelScheme::Cefr, "fre");
        assert_eq!(t1.map.len(), t2.map.len());
        assert_eq!(t1.map.len(), t3.map.len());

        // German aliases: de, deu, ger
        let t_de = LevelTable::builtin_ref(LevelScheme::Cefr, "de");
        let t_ger = LevelTable::builtin_ref(LevelScheme::Cefr, "ger");
        assert_eq!(t_de.map.len(), t_ger.map.len());
    }
}
