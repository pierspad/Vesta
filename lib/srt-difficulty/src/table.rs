use crate::scheme::LevelScheme;
use anyhow::{Result, anyhow};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct LevelTable {
    pub map: HashMap<String, u8>,
    pub max_token_len: usize,
    pub scheme: LevelScheme,
}

impl LevelTable {
    pub fn from_tsv(input: &str, scheme: LevelScheme) -> Result<Self> {
        let mut map = HashMap::new();
        let mut max_token_len = 0;

        for (idx, line) in input.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split(&['\t', ','][..]).collect();
            if parts.len() < 2 {
                continue;
            }

            let word = parts[0].trim().to_lowercase();
            if word.is_empty() {
                continue;
            }

            let level: u8 = parts[1]
                .trim()
                .parse()
                .map_err(|e| anyhow!("Line {}: invalid level '{}': {}", idx + 1, parts[1], e))?;

            let char_count = word.chars().count();
            if char_count > max_token_len {
                max_token_len = char_count;
            }

            map.insert(word, level);
        }

        Ok(Self {
            map,
            max_token_len,
            scheme,
        })
    }

    pub fn builtin(scheme: LevelScheme, lang: &str) -> Result<Self> {
        let primary_lang = lang.split(&['-', '_'][..]).next().unwrap_or(lang);
        let tsv_content = match scheme {
            LevelScheme::Hsk => BUILTIN_HSK_TSV,
            LevelScheme::Jlpt => BUILTIN_JLPT_TSV,
            LevelScheme::Cefr => match primary_lang {
                "zh" => BUILTIN_HSK_TSV,
                "ja" => BUILTIN_JLPT_TSV,
                _ => BUILTIN_CEFR_EN_TSV,
            },
        };

        Self::from_tsv(tsv_content, scheme)
    }

    pub fn get(&self, word: &str) -> Option<u8> {
        self.map.get(word).copied()
    }
}

// Built-in starter vocabulary lists (sample high-utility words per level)
const BUILTIN_HSK_TSV: &str = r#"
你	1
好	1
我	1
是	1
中国	1
人	1
谢谢	1
不	1
喜欢	1
水	1
吃	1
喝	1
看	1
听	1
去	1
来	1
想	1
小	1
大	1
好	1
学习	2
帮助	2
简单	2
问题	2
工作	2
开始	2
因为	2
所以	2
但是	2
已经	2
经常	3
环境	3
解决	3
提高	3
选择	3
几乎	3
关系	3
影响	3
偶尔	4
频繁	4
判断	4
趋势	4
极其	4
抽象	5
震撼	5
涵盖	5
"#;

const BUILTIN_JLPT_TSV: &str = r#"
私	1
本	1
人	1
日本	1
食べる	1
飲む	1
行く	1
来る	1
見る	1
聞く	1
大きい	1
小さい	1
勉強	2
簡単	2
問題	2
仕事	2
始まる	2
選ぶ	3
環境	3
関係	3
影響	3
複雑	4
概念	4
判断	4
顕著	5
"#;

const BUILTIN_CEFR_EN_TSV: &str = r#"
the	1
a	1
an	1
is	1
are	1
was	1
were	1
have	1
has	1
had	1
do	1
does	1
did	1
say	1
said	1
go	1
good	1
day	1
man	1
woman	1
child	1
book	1
friend	1
hello	1
house	1
water	1
food	1
work	1
time	1
year	1
journey	2
arrive	2
departure	2
receive	2
explain	2
decision	2
future	2
possible	2
analyze	3
analyse	3
hypothesis	4
paradigm	5
ubiquitous	6
"#;
