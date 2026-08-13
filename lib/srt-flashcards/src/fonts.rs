use crate::types::FlashcardConfig;

pub const DEFAULT_FONT_STACK: &str =
    r#""Noto Sans", -apple-system, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif"#;

/// Return CSS `font-family` stack for given target language code.
/// Matches case-insensitively and handles sub-tags (e.g. `zh-tw` before `zh`).
pub fn font_stack_for(lang: &str) -> &'static str {
    let lower = lang.trim().to_lowercase();

    // Check specific sub-tags first
    if lower == "zh-tw" || lower == "zh-hk" || lower == "zh-hant" {
        return r#""Noto Sans CJK TC", "Noto Sans TC", "PingFang TC", "Microsoft JhengHei", sans-serif"#;
    }

    let primary = lower.split(&['-', '_'][..]).next().unwrap_or(&lower);

    match primary {
        "ja" => {
            r#""Noto Serif CJK JP", "Noto Sans CJK JP", "Noto Sans JP", "Hiragino Sans", "Yu Gothic", Meiryo, sans-serif"#
        }
        "zh" => {
            r#""Noto Sans CJK SC", "Noto Sans SC", "Source Han Sans SC", "PingFang SC", "Microsoft YaHei", sans-serif"#
        }
        "ko" => {
            r#""Noto Sans CJK KR", "Noto Sans KR", "Apple SD Gothic Neo", "Malgun Gothic", sans-serif"#
        }
        "ar" => r#""Noto Naskh Arabic", "Noto Sans Arabic", "Geeza Pro", "Segoe UI", sans-serif"#,
        "he" => r#""Noto Sans Hebrew", "Arial Hebrew", "Segoe UI", sans-serif"#,
        "th" => r#""Noto Sans Thai", Thonburi, "Leelawadee UI", sans-serif"#,
        "hi" => r#""Noto Sans Devanagari", "Kohinoor Devanagari", "Nirmala UI", sans-serif"#,
        "el" | "ru" | "uk" => r#""Noto Sans", "Segoe UI", "Helvetica Neue", sans-serif"#,
        _ => DEFAULT_FONT_STACK,
    }
}

pub fn maybe_prepend_font_vars(base_css: &str, config: &FlashcardConfig) -> String {
    if !config.auto_card_font {
        return base_css.to_string();
    }

    let stack = match config.target_language.as_deref() {
        Some(lang) if !lang.trim().is_empty() => font_stack_for(lang),
        _ => DEFAULT_FONT_STACK,
    };

    let font_block = format!(
        "/* vesta:font-start */\n:root {{\n  --vesta-target-font: {};\n}}\n.card {{\n  font-family: var(--vesta-target-font);\n}}\n/* vesta:font-end */",
        stack
    );

    if let (Some(start_idx), Some(end_idx)) = (
        base_css.find("/* vesta:font-start */"),
        base_css.find("/* vesta:font-end */"),
    ) {
        let end_block = end_idx + "/* vesta:font-end */".len();
        let mut result = String::with_capacity(base_css.len() + font_block.len());
        result.push_str(&base_css[..start_idx]);
        result.push_str(&font_block);
        result.push_str(&base_css[end_block..]);
        result
    } else {
        format!("{}\n\n{}", font_block, base_css)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_stack_matching() {
        assert!(font_stack_for("ja").contains("Noto Serif CJK JP"));
        assert!(font_stack_for("zh-TW").contains("Noto Sans CJK TC"));
        assert!(font_stack_for("ZH-cn").contains("Noto Sans CJK SC"));
        assert!(font_stack_for("ko").contains("Noto Sans CJK KR"));
        assert!(font_stack_for("ar").contains("Noto Naskh Arabic"));
        assert_eq!(font_stack_for("xyz_unknown"), DEFAULT_FONT_STACK);
    }

    #[test]
    fn test_css_injection_and_idempotency() {
        let config = FlashcardConfig {
            target_language: Some("ja".to_string()),
            auto_card_font: true,
            ..Default::default()
        };

        let base_css = ".card { color: white; }";
        let injected = maybe_prepend_font_vars(base_css, &config);
        assert!(injected.contains("/* vesta:font-start */"));
        assert!(injected.contains("Noto Serif CJK JP"));
        assert!(injected.contains(".card { color: white; }"));

        // Idempotency check: running it again on injected CSS replaces the block
        let config_zh = FlashcardConfig {
            target_language: Some("zh".to_string()),
            auto_card_font: true,
            ..Default::default()
        };
        let reinjected = maybe_prepend_font_vars(&injected, &config_zh);
        assert!(reinjected.contains("Noto Sans CJK SC"));
        assert!(!reinjected.contains("Noto Serif CJK JP"));
        assert_eq!(reinjected.matches("/* vesta:font-start */").count(), 1);

        // Disabled check
        let config_disabled = FlashcardConfig {
            auto_card_font: false,
            ..Default::default()
        };
        assert_eq!(
            maybe_prepend_font_vars(base_css, &config_disabled),
            base_css
        );
    }
}
