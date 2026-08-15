use anyhow::{Context as _, Result};
use std::path::Path;

use super::types::*;

pub(crate) fn detect_format(path: &str) -> &'static str {
    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    match ext.as_str() {
        "srt" => "srt",
        "ass" | "ssa" => "ass",
        "vtt" | "webvtt" => "vtt",
        "lrc" => "lrc",
        _ => "srt",
    }
}

/// Parse SRT file into SubEntry vec
pub(crate) fn parse_srt(content: &str) -> Result<Vec<SubEntry>> {
    let mut entries = Vec::new();
    let trimmed = content.trim_start_matches('\u{feff}');
    let mut current_block_lines: Vec<&str> = Vec::with_capacity(4);

    for line in trimmed.lines() {
        let line_trimmed = line.trim();
        if line_trimmed.is_empty() {
            if !current_block_lines.is_empty() {
                if let Some(entry) = parse_srt_block(&current_block_lines)? {
                    entries.push(entry);
                }
                current_block_lines.clear();
            }
        } else {
            current_block_lines.push(line);
        }
    }

    if !current_block_lines.is_empty()
        && let Some(entry) = parse_srt_block(&current_block_lines)?
    {
        entries.push(entry);
    }

    entries.sort_unstable_by_key(|e| e.start_ms);
    Ok(entries)
}

fn parse_srt_block(lines: &[&str]) -> Result<Option<SubEntry>> {
    if lines.len() < 2 {
        return Ok(None);
    }

    let id: u32 = match lines[0].trim().parse() {
        Ok(id) => id,
        Err(_) => return Ok(None),
    };

    let timeline = lines[1];
    let Some((start_str, end_str)) = timeline.split_once(" --> ") else {
        return Ok(None);
    };

    let start_ms = srt_parser::Timestamp::from_srt_string(start_str.trim())
        .map(|t| t.total_milliseconds() as i64)
        .map_err(|e| anyhow::anyhow!("Invalid SRT start timestamp: {}", e))?;
    let end_ms = srt_parser::Timestamp::from_srt_string(end_str.trim())
        .map(|t| t.total_milliseconds() as i64)
        .map_err(|e| anyhow::anyhow!("Invalid SRT end timestamp: {}", e))?;

    let text = if lines.len() > 2 {
        let mut buf = String::with_capacity(lines.len() * 32);
        for (i, line) in lines[2..].iter().enumerate() {
            if i > 0 {
                buf.push('\n');
            }
            buf.push_str(line);
        }
        let t = buf.trim().to_string();
        if t.is_empty() {
            return Ok(None);
        }
        t
    } else {
        return Ok(None);
    };

    Ok(Some(SubEntry {
        id,
        start_ms,
        end_ms,
        text,
        actor: None,
        style: None,
        active: true,
    }))
}

/// Parse ASS/SSA file
pub(crate) fn parse_ass(content: &str) -> Result<Vec<SubEntry>> {
    let mut entries = Vec::new();
    let mut in_events = false;
    let mut format_fields: Vec<String> = Vec::new();
    let mut start_col = None;
    let mut end_col = None;
    let mut actor_col = None;
    let mut style_col = None;
    let mut text_col = None;
    let mut id_counter: u32 = 1;

    for line in content.lines() {
        let line = line.trim();

        if line.eq_ignore_ascii_case("[Events]") {
            in_events = true;
            continue;
        }

        if line.starts_with('[') && in_events {
            break; // New section
        }

        if !in_events {
            continue;
        }

        if line.starts_with("Format:") {
            let fields_str = line.strip_prefix("Format:").unwrap_or("");
            format_fields = fields_str
                .split(',')
                .map(|f| f.trim().to_lowercase())
                .collect();
            start_col = format_fields.iter().position(|f| f == "start");
            end_col = format_fields.iter().position(|f| f == "end");
            actor_col = format_fields
                .iter()
                .position(|f| f == "name" || f == "actor");
            style_col = format_fields.iter().position(|f| f == "style");
            text_col = format_fields.iter().position(|f| f == "text");
            continue;
        }

        if line.starts_with("Dialogue:") || line.starts_with("Comment:") {
            let is_comment = line.starts_with("Comment:");
            if is_comment {
                continue;
            }

            let data = line.split_once(':').map_or("", |(_, rest)| rest).trim();
            let parts: Vec<&str> = data.splitn(format_fields.len().max(1), ',').collect();

            let get_part = |idx_opt: Option<usize>| -> Option<String> {
                idx_opt
                    .and_then(|i| parts.get(i))
                    .map(|s| s.trim().to_string())
            };

            let start_str = get_part(start_col).unwrap_or_default();
            let end_str = get_part(end_col).unwrap_or_default();
            let actor = get_part(actor_col);
            let style = get_part(style_col);

            // Text is the last field and may contain commas
            let text = if let Some(idx) = text_col {
                if idx < parts.len() {
                    // Rejoin everything from this index forward
                    parts[idx..].join(",").trim().to_string()
                } else {
                    String::new()
                }
            } else {
                parts
                    .last()
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default()
            };

            // Strip ASS formatting tags like {\b1}, {\an8}, etc.
            let text = strip_ass_tags(&text)
                .replace("\\N", "\n")
                .replace("\\n", "\n");

            if text.trim().is_empty() {
                continue;
            }

            let start_ms = parse_ass_timestamp(&start_str).unwrap_or(0);
            let end_ms = parse_ass_timestamp(&end_str).unwrap_or(0);

            entries.push(SubEntry {
                id: id_counter,
                start_ms,
                end_ms,
                text: text.trim().to_string(),
                actor,
                style,
                active: true,
            });
            id_counter += 1;
        }
    }

    entries.sort_unstable_by_key(|e| e.start_ms);
    Ok(entries)
}

pub(crate) fn parse_ass_timestamp(s: &str) -> Result<i64> {
    // Format: H:MM:SS.CC (centiseconds)
    let mut parts = s.split([':', '.']);
    let h: i64 = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
    let m: i64 = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
    let sec: i64 = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
    let cs: i64 = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
    Ok(h * 3_600_000 + m * 60_000 + sec * 1000 + cs * 10)
}

pub(crate) fn strip_ass_tags(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut in_tag = false;
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '{' && chars.peek() == Some(&'\\') {
            in_tag = true;
            continue;
        }
        if in_tag {
            if c == '}' {
                in_tag = false;
            }
            continue;
        }
        result.push(c);
    }
    result
}

/// Parse WebVTT file
pub(crate) fn parse_vtt(content: &str) -> Result<Vec<SubEntry>> {
    let mut entries = Vec::new();
    let mut id_counter: u32 = 1;

    let trimmed = content.trim_start_matches('\u{feff}');
    let mut current_block_lines: Vec<&str> = Vec::with_capacity(4);

    for line in trimmed.lines() {
        let line_trimmed = line.trim();
        if line_trimmed.is_empty() {
            if !current_block_lines.is_empty() {
                if let Some(entry) = parse_vtt_block(&current_block_lines, id_counter)? {
                    entries.push(entry);
                    id_counter += 1;
                }
                current_block_lines.clear();
            }
        } else {
            current_block_lines.push(line);
        }
    }

    if !current_block_lines.is_empty()
        && let Some(entry) = parse_vtt_block(&current_block_lines, id_counter)?
    {
        entries.push(entry);
    }

    entries.sort_unstable_by_key(|e| e.start_ms);
    Ok(entries)
}

fn parse_vtt_block(lines: &[&str], id_counter: u32) -> Result<Option<SubEntry>> {
    if lines.is_empty() {
        return Ok(None);
    }

    // Skip WEBVTT header and NOTE blocks
    if lines[0].starts_with("WEBVTT") || lines[0].starts_with("NOTE") {
        return Ok(None);
    }

    let mut timeline_idx = None;
    for (i, line) in lines.iter().enumerate() {
        if line.contains(" --> ") {
            timeline_idx = Some(i);
            break;
        }
    }

    let Some(t_idx) = timeline_idx else {
        return Ok(None);
    };

    let Some((start_str, end_str)) = lines[t_idx].split_once(" --> ") else {
        return Ok(None);
    };

    let start_ms = parse_vtt_timestamp(start_str.trim())?;
    let end_ms = parse_vtt_timestamp(end_str.split_whitespace().next().unwrap_or("").trim())?;

    let text = if t_idx + 1 < lines.len() {
        let mut buf = String::with_capacity((lines.len() - t_idx) * 32);
        for (i, line) in lines[t_idx + 1..].iter().enumerate() {
            if i > 0 {
                buf.push('\n');
            }
            buf.push_str(line);
        }
        buf.trim().to_string()
    } else {
        String::new()
    };

    let text = strip_vtt_tags(&text);
    if text.is_empty() {
        return Ok(None);
    }

    Ok(Some(SubEntry {
        id: id_counter,
        start_ms,
        end_ms,
        text,
        actor: None,
        style: None,
        active: true,
    }))
}

pub(crate) fn parse_vtt_timestamp(s: &str) -> Result<i64> {
    let mut parts = s.split([':', '.']);
    let first = parts
        .next()
        .ok_or_else(|| anyhow::anyhow!("Invalid VTT timestamp: {}", s))?;
    let second = parts
        .next()
        .ok_or_else(|| anyhow::anyhow!("Invalid VTT timestamp: {}", s))?;
    let third = parts
        .next()
        .ok_or_else(|| anyhow::anyhow!("Invalid VTT timestamp: {}", s))?;
    let fourth = parts.next();

    if let Some(fourth_part) = fourth {
        // HH:MM:SS.mmm
        let h: i64 = first.parse().unwrap_or(0);
        let m: i64 = second.parse().unwrap_or(0);
        let sec: i64 = third.parse().unwrap_or(0);
        let ms: i64 = fourth_part.parse().unwrap_or(0);
        Ok(h * 3_600_000 + m * 60_000 + sec * 1000 + ms)
    } else {
        // MM:SS.mmm
        let m: i64 = first.parse().unwrap_or(0);
        let sec: i64 = second.parse().unwrap_or(0);
        let ms: i64 = third.parse().unwrap_or(0);
        Ok(m * 60_000 + sec * 1000 + ms)
    }
}

pub(crate) fn strip_vtt_tags(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut in_tag = false;
    for c in text.chars() {
        if c == '<' {
            in_tag = true;
            continue;
        }
        if c == '>' {
            in_tag = false;
            continue;
        }
        if !in_tag {
            result.push(c);
        }
    }
    result.trim().to_string()
}

/// Parse any supported subtitle file
pub(crate) fn parse_subtitle_file(path: &str) -> Result<(Vec<SubEntry>, &'static str)> {
    let content = srt_parser::encoding::read_text_auto(path)
        .context(format!("Cannot read file: {}", path))?;

    let format = detect_format(path);
    let entries = match format {
        "ass" => parse_ass(&content)?,
        "vtt" => parse_vtt(&content)?,
        _ => parse_srt(&content)?,
    };

    Ok((entries, format))
}
