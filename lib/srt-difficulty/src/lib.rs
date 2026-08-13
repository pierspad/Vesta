pub mod analyzer;
pub mod scheme;
pub mod table;
pub mod tokenizer;

pub use analyzer::{AnalyzeOptions, CardLevel, UnknownPolicy, analyze, tag_for};
pub use scheme::LevelScheme;
pub use table::LevelTable;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chinese_hsk_analysis() {
        let table = LevelTable::builtin(LevelScheme::Hsk, "zh").unwrap();
        let opts = AnalyzeOptions::default();

        let card1 = analyze("你好！我是中国人。", &table, &opts);
        assert_eq!(card1.level, Some(1));
        assert!(card1.known_tokens >= 3);

        let card2 = analyze("这完全是一个抽象的问题。", &table, &opts);
        assert_eq!(card2.level, Some(5)); // 抽象 is HSK 5
        assert_eq!(tag_for(LevelScheme::Hsk, 5), "HSK::5");
    }

    #[test]
    fn test_japanese_jlpt_analysis() {
        let table = LevelTable::builtin(LevelScheme::Jlpt, "ja").unwrap();
        let opts = AnalyzeOptions::default();

        let card1 = analyze("私は本を飲む。", &table, &opts);
        assert_eq!(card1.level, Some(1));
        assert_eq!(tag_for(LevelScheme::Jlpt, 1), "JLPT::N5");

        let card2 = analyze("これは概念が複雑です。", &table, &opts);
        assert_eq!(card2.level, Some(4)); // 複雑/概念 is level 4 (N2)
        assert_eq!(tag_for(LevelScheme::Jlpt, 4), "JLPT::N2");
    }

    #[test]
    fn test_english_cefr_analysis() {
        let table = LevelTable::builtin(LevelScheme::Cefr, "en").unwrap();
        let opts = AnalyzeOptions::default();

        let card1 = analyze("The child has a book.", &table, &opts);
        assert_eq!(card1.level, Some(1));
        assert_eq!(tag_for(LevelScheme::Cefr, 1), "CEFR::A1");

        let card2 = analyze("This hypothesis is paradigm ubiquitous.", &table, &opts);
        assert_eq!(card2.level, Some(6));
        assert_eq!(tag_for(LevelScheme::Cefr, 6), "CEFR::C2");
    }

    #[test]
    fn test_unknown_policy_highest() {
        let table = LevelTable::builtin(LevelScheme::Hsk, "zh").unwrap();
        let opts = AnalyzeOptions {
            unknown: UnknownPolicy::Highest,
            min_token_chars: 1,
        };

        let card = analyze("你好！xyz_unknown_word", &table, &opts);
        assert_eq!(card.level, Some(5)); // highest level in sample dictionary
    }
}
