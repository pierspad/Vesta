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
        assert_eq!(card2.level, Some(5)); // 概念 is JLPT N1 (level 5)
        assert_eq!(tag_for(LevelScheme::Jlpt, 5), "JLPT::N1");
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
    fn test_italian_cefr_analysis() {
        let table = LevelTable::builtin(LevelScheme::Cefr, "it").unwrap();
        let opts = AnalyzeOptions::default();

        let card1 = analyze("Ciao amico mio, come stai oggi?", &table, &opts);
        assert_eq!(card1.level, Some(1));
        assert_eq!(tag_for(LevelScheme::Cefr, 1), "CEFR::A1");
    }

    #[test]
    fn test_spanish_cefr_analysis() {
        let table = LevelTable::builtin(LevelScheme::Cefr, "es").unwrap();
        let opts = AnalyzeOptions::default();

        let card1 = analyze("Hola amigo, ¿cómo estás hoy?", &table, &opts);
        assert_eq!(card1.level, Some(1));
        assert_eq!(tag_for(LevelScheme::Cefr, 1), "CEFR::A1");
    }

    #[test]
    fn test_german_cefr_analysis() {
        let table = LevelTable::builtin(LevelScheme::Cefr, "de").unwrap();
        let opts = AnalyzeOptions::default();

        let card1 = analyze("Hallo Freund, wie geht es dir?", &table, &opts);
        assert_eq!(card1.level, Some(1));
        assert_eq!(tag_for(LevelScheme::Cefr, 1), "CEFR::A1");
    }

    #[test]
    fn test_korean_topik_analysis() {
        let table = LevelTable::builtin(LevelScheme::Topik, "ko").unwrap();
        let opts = AnalyzeOptions::default();

        let card1 = analyze("안녕하세요, 친구입니다.", &table, &opts);
        assert!(card1.level.is_some());
        assert_eq!(tag_for(LevelScheme::Topik, 1), "TOPIK::1");
    }

    #[test]
    fn test_chinese_tocfl_analysis() {
        let table = LevelTable::builtin(LevelScheme::Tocfl, "zh-TW").unwrap();
        let opts = AnalyzeOptions::default();

        let card1 = analyze("你好！我是學生。", &table, &opts);
        assert!(card1.level.is_some());
        assert_eq!(tag_for(LevelScheme::Tocfl, 1), "TOCFL::A1");
    }

    #[test]
    fn test_unknown_policy_highest() {
        let table = LevelTable::builtin(LevelScheme::Hsk, "zh").unwrap();
        let opts = AnalyzeOptions {
            unknown: UnknownPolicy::Highest,
            min_token_chars: 1,
        };

        let card = analyze("你好！xyz_unknown_word", &table, &opts);
        assert_eq!(card.level, Some(6)); // highest level in HSK dictionary (HSK 6)
    }

    #[test]
    fn test_unknown_policy_zero() {
        let table = LevelTable::builtin(LevelScheme::Hsk, "zh").unwrap();
        let opts = AnalyzeOptions {
            unknown: UnknownPolicy::Zero,
            min_token_chars: 1,
        };

        // Unknown text only -> Level 0
        let card = analyze("xyz_unknown_word_only", &table, &opts);
        assert_eq!(card.level, Some(0));

        // Known word (HSK 1) + Unknown word -> Level 1 (known level preserved)
        let card2 = analyze("你好！xyz_unknown_word", &table, &opts);
        assert_eq!(card2.level, Some(1));
    }

    #[test]
    fn test_level_zero_tags_all_schemes() {
        assert_eq!(tag_for(LevelScheme::Hsk, 0), "HSK::0");
        assert_eq!(tag_for(LevelScheme::Cefr, 0), "CEFR::0");
        assert_eq!(tag_for(LevelScheme::Jlpt, 0), "JLPT::0");
        assert_eq!(tag_for(LevelScheme::Topik, 0), "TOPIK::0");
        assert_eq!(tag_for(LevelScheme::Tocfl, 0), "TOCFL::0");
        assert_eq!(tag_for(LevelScheme::Custom, 0), "Level::0");
    }

    #[test]
    fn test_custom_scheme_analysis() {
        let custom_tsv = "hello\t1\nworld\t2\nquantum\t4\n";
        let table = LevelTable::from_tsv(custom_tsv, LevelScheme::Custom).unwrap();
        let opts = AnalyzeOptions::default();

        let card1 = analyze("Hello there world!", &table, &opts);
        assert_eq!(card1.level, Some(2));
        assert_eq!(tag_for(LevelScheme::Custom, 2), "Level::2");

        let card2 = analyze("Quantum computers are fast.", &table, &opts);
        assert_eq!(card2.level, Some(4));
    }
}
