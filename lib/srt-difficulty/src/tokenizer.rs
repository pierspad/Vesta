use crate::table::LevelTable;

#[derive(Debug, Clone)]
pub struct TokenMatch {
    pub token: String,
    pub level: Option<u8>,
}

pub fn tokenize(text: &str, table: &LevelTable, min_token_chars: usize) -> Vec<TokenMatch> {
    let clean = clean_html(text);

    // Determine if text is primarily CJK / non-space script
    let has_cjk = clean.chars().any(is_cjk_char);

    if has_cjk {
        tokenize_cjk(&clean, table, min_token_chars)
    } else {
        tokenize_alphabetic(&clean, table, min_token_chars)
    }
}

fn is_cjk_char(c: char) -> bool {
    matches!(c,
        '\u{4E00}'..='\u{9FFF}' | // CJK Unified Ideographs
        '\u{3400}'..='\u{4DBF}' | // CJK Extension A
        '\u{3040}'..='\u{309F}' | // Hiragana
        '\u{30A0}'..='\u{30FF}'   // Katakana
    )
}

fn tokenize_cjk(text: &str, table: &LevelTable, min_token_chars: usize) -> Vec<TokenMatch> {
    let chars: Vec<char> = text.chars().collect();
    let mut matches = Vec::new();
    let mut i = 0;
    let max_window = table.max_token_len.max(1);

    while i < chars.len() {
        let ch = chars[i];
        if !is_cjk_char(ch) {
            if ch.is_alphanumeric() {
                let start = i;
                while i < chars.len() && chars[i].is_alphanumeric() && !is_cjk_char(chars[i]) {
                    i += 1;
                }
                let word: String = chars[start..i].iter().collect();
                if word.chars().count() >= min_token_chars {
                    let level = table.get(&word.to_lowercase());
                    matches.push(TokenMatch { token: word, level });
                }
            } else {
                i += 1;
            }
            continue;
        }

        let mut matched = false;
        let upper_len = max_window.min(chars.len() - i);

        for len in (1..=upper_len).rev() {
            if len < min_token_chars && len != 1 {
                continue;
            }

            let candidate: String = chars[i..i + len].iter().collect();
            let lower_cand = candidate.to_lowercase();

            if let Some(level) = table.get(&lower_cand) {
                matches.push(TokenMatch {
                    token: candidate,
                    level: Some(level),
                });
                i += len;
                matched = true;
                break;
            }
        }

        if !matched {
            let single: String = chars[i..i + 1].iter().collect();
            if single.chars().count() >= min_token_chars {
                matches.push(TokenMatch {
                    token: single,
                    level: None,
                });
            }
            i += 1;
        }
    }

    matches
}

fn tokenize_alphabetic(text: &str, table: &LevelTable, min_token_chars: usize) -> Vec<TokenMatch> {
    let mut matches = Vec::new();

    for word in text.split(|c: char| !c.is_alphanumeric()) {
        let trimmed = word.trim();
        if trimmed.is_empty() || trimmed.chars().count() < min_token_chars {
            continue;
        }

        let lower = trimmed.to_lowercase();

        // 1. Direct lookup
        if let Some(level) = table.get(&lower) {
            matches.push(TokenMatch {
                token: trimmed.to_string(),
                level: Some(level),
            });
            continue;
        }

        // 2. Simple lemmatization fallbacks (English endings: s, es, ed, ing)
        let mut found_level = None;

        if lower.ends_with("ing") && lower.len() > 5 {
            let base = &lower[..lower.len() - 3];
            found_level = table.get(base);
        } else if (lower.ends_with("ed") || lower.ends_with("es")) && lower.len() > 4 {
            let base = &lower[..lower.len() - 2];
            found_level = table.get(base);
        } else if lower.ends_with('s') && lower.len() > 3 {
            let base = &lower[..lower.len() - 1];
            found_level = table.get(base);
        }

        matches.push(TokenMatch {
            token: trimmed.to_string(),
            level: found_level,
        });
    }

    matches
}

fn clean_html(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut inside_tag = false;

    for c in text.chars() {
        if c == '<' {
            inside_tag = true;
        } else if c == '>' {
            inside_tag = false;
            result.push(' ');
        } else if !inside_tag {
            result.push(c);
        }
    }

    result
}
