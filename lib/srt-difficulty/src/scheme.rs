use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LevelScheme {
    Cefr,
    Hsk,
    Jlpt,
    Custom,
}

impl LevelScheme {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Cefr => "CEFR",
            Self::Hsk => "HSK",
            Self::Jlpt => "JLPT",
            Self::Custom => "CUSTOM",
        }
    }
}
