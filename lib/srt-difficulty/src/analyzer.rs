use crate::scheme::LevelScheme;
use crate::table::LevelTable;
use crate::tokenizer::tokenize;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnknownPolicy {
    Ignore,
    Highest,
    Zero,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeOptions {
    pub unknown: UnknownPolicy,
    pub min_token_chars: usize,
}

impl Default for AnalyzeOptions {
    fn default() -> Self {
        Self {
            unknown: UnknownPolicy::Ignore,
            min_token_chars: 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CardLevel {
    pub level: Option<u8>,
    pub known_tokens: usize,
    pub unknown_tokens: usize,
}

pub fn analyze(text: &str, table: &LevelTable, opts: &AnalyzeOptions) -> CardLevel {
    let tokens = tokenize(text, table, opts.min_token_chars);
    let mut max_level: Option<u8> = None;
    let mut known_count = 0;
    let mut unknown_count = 0;

    for tok in tokens {
        match tok.level {
            Some(lvl) => {
                known_count += 1;
                max_level = Some(max_level.map_or(lvl, |m| m.max(lvl)));
            }
            None => {
                unknown_count += 1;
            }
        }
    }

    if opts.unknown == UnknownPolicy::Highest && unknown_count > 0 {
        let table_max = if table.max_level > 0 {
            table.max_level
        } else {
            6
        };
        max_level = Some(max_level.map_or(table_max, |m| m.max(table_max)));
    } else if opts.unknown == UnknownPolicy::Zero
        && (max_level.is_none() || (known_count == 0 && unknown_count > 0))
    {
        max_level = Some(0);
    }

    CardLevel {
        level: max_level,
        known_tokens: known_count,
        unknown_tokens: unknown_count,
    }
}

pub fn tag_for(scheme: LevelScheme, level: u8) -> String {
    match scheme {
        LevelScheme::Hsk => format!("HSK::{}", level),
        LevelScheme::Tocfl => match level {
            0 => "TOCFL::0".to_string(),
            1 => "TOCFL::A1".to_string(),
            2 => "TOCFL::A2".to_string(),
            3 => "TOCFL::B1".to_string(),
            4 => "TOCFL::B2".to_string(),
            5 => "TOCFL::C1".to_string(),
            6 => "TOCFL::C2".to_string(),
            _ => format!("TOCFL::{}", level),
        },
        LevelScheme::Jlpt => match level {
            0 => "JLPT::0".to_string(),
            1 => "JLPT::N5".to_string(),
            2 => "JLPT::N4".to_string(),
            3 => "JLPT::N3".to_string(),
            4 => "JLPT::N2".to_string(),
            5 => "JLPT::N1".to_string(),
            _ => format!("JLPT::N{}", level),
        },
        LevelScheme::Topik => format!("TOPIK::{}", level),
        LevelScheme::Cefr => match level {
            0 => "CEFR::0".to_string(),
            1 => "CEFR::A1".to_string(),
            2 => "CEFR::A2".to_string(),
            3 => "CEFR::B1".to_string(),
            4 => "CEFR::B2".to_string(),
            5 => "CEFR::C1".to_string(),
            6 => "CEFR::C2".to_string(),
            _ => format!("CEFR::L{}", level),
        },
        LevelScheme::Custom => format!("Level::{}", level),
    }
}
