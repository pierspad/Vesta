//! Comandi Tauri per la generazione di flashcard Anki da sottotitoli.
//!
//! Implementazione completa ispirata a subs2srs: parsing doppi sottotitoli,
//! matching temporale, estrazione audio/snapshot/video via FFmpeg,
//! generazione TSV per Anki, filtri avanzati, context lines.

use anyhow::{Context as _, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter, State};
use tokio_util::sync::CancellationToken;

use crate::state::AppFlashcardState;

// ─── Data Types ──────────────────────────────────────────────────────────────

/// A parsed subtitle entry (supports SRT, ASS, VTT)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubEntry {
    pub id: u32,
    pub start_ms: i64,
    pub end_ms: i64,
    pub text: String,
    /// Actor name (ASS/SSA only)
    pub actor: Option<String>,
    /// Style name (ASS/SSA only)
    pub style: Option<String>,
    /// Whether this line is active (passes filters)
    pub active: bool,
}

/// Matched pair of subs1 + subs2 lines
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchedLine {
    pub index: usize,
    pub subs1: SubEntry,
    pub subs2: Option<SubEntry>,
    pub active: bool,
    /// Context: indices of leading lines
    pub leading_context: Vec<usize>,
    /// Context: indices of trailing lines
    pub trailing_context: Vec<usize>,
}

/// Info returned after loading a subtitle file
#[derive(Debug, Clone, Serialize)]
pub struct SubFileInfo {
    pub path: String,
    pub format: String,
    pub count: usize,
    pub first_text: String,
    pub last_text: String,
    /// Unique actors found (ASS only)
    pub actors: Vec<String>,
    pub duration_ms: i64,
}

/// Full flashcard generation configuration from the frontend
#[derive(Debug, Clone, Deserialize)]
pub struct FlashcardConfig {
    // Files
    pub target_subs_path: String,
    pub native_subs_path: Option<String>,
    pub video_path: Option<String>,
    pub audio_path: Option<String>,
    pub output_dir: String,

    // Subtitle options
    #[allow(dead_code)]
    pub use_timings_from: String, // "target" or "native"
    pub span_start_ms: Option<i64>,
    pub span_end_ms: Option<i64>,
    pub time_shift_target_ms: i64,
    pub time_shift_native_ms: i64,

    // Filters
    pub filters: SubtitleFilters,

    // Context lines
    pub context: ContextConfig,

    // Sentence combining
    pub combine_sentences: bool,
    pub continuation_chars: String,

    // Audio
    pub generate_audio: bool,
    pub audio_bitrate: u32,
    pub normalize_audio: bool,
    pub audio_pad_start_ms: i64,
    pub audio_pad_end_ms: i64,

    // Snapshots
    pub generate_snapshots: bool,
    pub snapshot_width: u32,
    pub snapshot_height: u32,
    pub crop_bottom: u32,

    // Video clips
    pub generate_video_clips: bool,
    pub video_codec: String,     // "h264" or "mpeg4"
    pub h264_preset: String,     // ultrafast..placebo
    pub video_bitrate: u32,
    pub video_audio_bitrate: u32,
    pub video_pad_start_ms: i64,
    pub video_pad_end_ms: i64,

    // Naming
    pub deck_name: String,
    pub episode_number: u32,

    // Export format: "tsv" or "apkg"
    pub export_format: Option<String>,

    // Note type name for Anki
    pub note_type_name: Option<String>,

    // Output fields
    pub output_fields: OutputFields,

    // Performance: CPU cores to use (optional, defaults to 3/4 of available)
    pub cpu_cores: Option<usize>,

    // Custom Anki card templates (optional, overrides built-in defaults)
    pub card_front_html: Option<String>,
    pub card_back_html: Option<String>,
    pub card_css: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SubtitleFilters {
    pub include_words: Option<String>,
    pub exclude_words: Option<String>,
    pub exclude_duplicates_subs1: bool,
    pub exclude_duplicates_subs2: bool,
    pub min_chars: Option<usize>,
    pub max_chars: Option<usize>,
    pub min_duration_ms: Option<i64>,
    pub max_duration_ms: Option<i64>,
    pub exclude_styled: bool,
    pub actor_filter: Option<String>,
    pub only_cjk: bool,
    pub remove_no_match: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ContextConfig {
    pub leading: usize,
    pub trailing: usize,
    pub max_gap_seconds: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OutputFields {
    pub include_tag: bool,
    pub include_sequence: bool,
    pub include_audio: bool,
    pub include_snapshot: bool,
    pub include_video: bool,
    pub include_subs1: bool,
    pub include_subs2: bool,
}

/// Progress event emitted to frontend
#[derive(Debug, Clone, Serialize)]
pub struct FlashcardProgressEvent {
    pub stage: String,
    pub message: String,
    pub current: usize,
    pub total: usize,
    pub percentage: f64,
    pub params: HashMap<String, String>,
}

/// Final result
#[derive(Debug, Clone, Serialize)]
pub struct FlashcardResult {
    pub success: bool,
    pub message: String,
    pub cards_generated: usize,
    pub audio_clips: usize,
    pub snapshots: usize,
    pub video_clips: usize,
    pub tsv_path: Option<String>,
    pub apkg_path: Option<String>,
}

/// Preview data for a single line
#[derive(Debug, Clone, Serialize)]
pub struct PreviewLine {
    pub index: usize,
    pub subs1_text: String,
    pub subs2_text: Option<String>,
    pub start_ms: i64,
    pub end_ms: i64,
    pub duration_ms: i64,
    pub active: bool,
    pub actor: Option<String>,
    pub leading_context: Vec<usize>,
    pub trailing_context: Vec<usize>,
}

// ─── Subtitle Parsing ────────────────────────────────────────────────────────

/// Detect format from extension
fn detect_format(path: &str) -> &'static str {
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
fn parse_srt(content: &str) -> Result<Vec<SubEntry>> {
    let mut entries = Vec::new();
    // Normalize CRLF → LF so that block splitting works for Windows-encoded files
    let normalized = content.replace("\r\n", "\n");
    let blocks: Vec<&str> = normalized.split("\n\n").collect();

    for block in blocks {
        let block = block.trim();
        if block.is_empty() {
            continue;
        }

        let lines: Vec<&str> = block.lines().collect();
        if lines.len() < 2 {
            continue;
        }

        let id: u32 = match lines[0].trim().parse() {
            Ok(id) => id,
            Err(_) => continue,
        };

        let timeline = lines[1];
        let parts: Vec<&str> = timeline.split(" --> ").collect();
        if parts.len() != 2 {
            continue;
        }

        let start_ms = parse_srt_timestamp(parts[0].trim())?;
        let end_ms = parse_srt_timestamp(parts[1].trim())?;

        let text = if lines.len() > 2 {
            lines[2..].join("\n").trim().to_string()
        } else {
            String::new()
        };

        if text.is_empty() {
            continue;
        }

        entries.push(SubEntry {
            id,
            start_ms,
            end_ms,
            text,
            actor: None,
            style: None,
            active: true,
        });
    }

    entries.sort_by_key(|e| e.start_ms);
    Ok(entries)
}

fn parse_srt_timestamp(s: &str) -> Result<i64> {
    let parts: Vec<&str> = s.split(&[':', ','][..]).collect();
    if parts.len() != 4 {
        anyhow::bail!("Invalid SRT timestamp: {}", s);
    }
    let h: i64 = parts[0].parse().context("Invalid hours")?;
    let m: i64 = parts[1].parse().context("Invalid minutes")?;
    let sec: i64 = parts[2].parse().context("Invalid seconds")?;
    let ms: i64 = parts[3].parse().context("Invalid milliseconds")?;
    Ok(h * 3600_000 + m * 60_000 + sec * 1000 + ms)
}

/// Parse ASS/SSA file
fn parse_ass(content: &str) -> Result<Vec<SubEntry>> {
    let mut entries = Vec::new();
    let mut in_events = false;
    let mut format_fields: Vec<String> = Vec::new();
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
            format_fields = fields_str.split(',').map(|f| f.trim().to_lowercase()).collect();
            continue;
        }

        if line.starts_with("Dialogue:") || line.starts_with("Comment:") {
            let is_comment = line.starts_with("Comment:");
            if is_comment {
                continue;
            }

            let data = line.splitn(2, ':').nth(1).unwrap_or("").trim();
            let parts: Vec<&str> = data.splitn(format_fields.len().max(1), ',').collect();

            let get_field = |name: &str| -> Option<String> {
                format_fields
                    .iter()
                    .position(|f| f == name)
                    .and_then(|i| parts.get(i))
                    .map(|s| s.trim().to_string())
            };

            let start_str = get_field("start").unwrap_or_default();
            let end_str = get_field("end").unwrap_or_default();
            let actor = get_field("name").or_else(|| get_field("actor"));
            let style = get_field("style");

            // Text is the last field and may contain commas
            let text_field_idx = format_fields.iter().position(|f| f == "text");
            let text = if let Some(idx) = text_field_idx {
                if idx < parts.len() {
                    // Rejoin everything from this index forward
                    parts[idx..].join(",").trim().to_string()
                } else {
                    String::new()
                }
            } else {
                parts.last().map(|s| s.trim().to_string()).unwrap_or_default()
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

    entries.sort_by_key(|e| e.start_ms);
    Ok(entries)
}

fn parse_ass_timestamp(s: &str) -> Result<i64> {
    // Format: H:MM:SS.CC (centiseconds)
    let parts: Vec<&str> = s.split(&[':', '.'][..]).collect();
    if parts.len() != 4 {
        anyhow::bail!("Invalid ASS timestamp: {}", s);
    }
    let h: i64 = parts[0].parse().unwrap_or(0);
    let m: i64 = parts[1].parse().unwrap_or(0);
    let sec: i64 = parts[2].parse().unwrap_or(0);
    let cs: i64 = parts[3].parse().unwrap_or(0);
    Ok(h * 3600_000 + m * 60_000 + sec * 1000 + cs * 10)
}

fn strip_ass_tags(text: &str) -> String {
    let mut result = String::new();
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
fn parse_vtt(content: &str) -> Result<Vec<SubEntry>> {
    let mut entries = Vec::new();
    let mut id_counter: u32 = 1;

    // Skip WEBVTT header
    let content = content.trim_start_matches('\u{feff}'); // BOM
    let blocks: Vec<&str> = content.split("\n\n").collect();

    for block in blocks {
        let block = block.trim();
        if block.is_empty() || block.starts_with("WEBVTT") || block.starts_with("NOTE") {
            continue;
        }

        let lines: Vec<&str> = block.lines().collect();
        let mut timeline_idx = 0;

        // Find timeline (contains " --> ")
        for (i, line) in lines.iter().enumerate() {
            if line.contains(" --> ") {
                timeline_idx = i;
                break;
            }
        }

        let parts: Vec<&str> = lines[timeline_idx].split(" --> ").collect();
        if parts.len() != 2 {
            continue;
        }

        // VTT timestamps can be MM:SS.mmm or HH:MM:SS.mmm
        let start_ms = parse_vtt_timestamp(parts[0].trim())?;
        let end_ms = parse_vtt_timestamp(parts[1].split_whitespace().next().unwrap_or("").trim())?;

        let text = if timeline_idx + 1 < lines.len() {
            lines[timeline_idx + 1..]
                .join("\n")
                .trim()
                .to_string()
        } else {
            String::new()
        };

        // Strip VTT tags like <b>, <i>, <c.colorname>, etc.
        let text = strip_vtt_tags(&text);

        if text.is_empty() {
            continue;
        }

        entries.push(SubEntry {
            id: id_counter,
            start_ms,
            end_ms,
            text,
            actor: None,
            style: None,
            active: true,
        });
        id_counter += 1;
    }

    entries.sort_by_key(|e| e.start_ms);
    Ok(entries)
}

fn parse_vtt_timestamp(s: &str) -> Result<i64> {
    let parts: Vec<&str> = s.split(&[':', '.'][..]).collect();
    match parts.len() {
        // MM:SS.mmm
        3 => {
            let m: i64 = parts[0].parse().unwrap_or(0);
            let sec: i64 = parts[1].parse().unwrap_or(0);
            let ms: i64 = parts[2].parse().unwrap_or(0);
            Ok(m * 60_000 + sec * 1000 + ms)
        }
        // HH:MM:SS.mmm
        4 => {
            let h: i64 = parts[0].parse().unwrap_or(0);
            let m: i64 = parts[1].parse().unwrap_or(0);
            let sec: i64 = parts[2].parse().unwrap_or(0);
            let ms: i64 = parts[3].parse().unwrap_or(0);
            Ok(h * 3600_000 + m * 60_000 + sec * 1000 + ms)
        }
        _ => anyhow::bail!("Invalid VTT timestamp: {}", s),
    }
}

fn strip_vtt_tags(text: &str) -> String {
    let mut result = String::new();
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
fn parse_subtitle_file(path: &str) -> Result<(Vec<SubEntry>, &'static str)> {
    let content = std::fs::read_to_string(path)
        .or_else(|_| -> Result<String> {
            // Try common encodings
            let bytes = std::fs::read(path)?;
            // Try UTF-8 with BOM
            if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
                Ok(String::from_utf8_lossy(&bytes[3..]).to_string())
            } else {
                // Try Latin-1 as fallback
                Ok(bytes.iter().map(|&b| b as char).collect())
            }
        })
        .context(format!("Cannot read file: {}", path))?;

    let format = detect_format(path);
    let entries = match format {
        "ass" => parse_ass(&content)?,
        "vtt" => parse_vtt(&content)?,
        _ => parse_srt(&content)?,
    };

    Ok((entries, format))
}

// ─── Dual Subtitle Matching (subs2srs algorithm) ────────────────────────────

/// Calculate temporal overlap between two subtitle entries (in ms)
fn get_overlap(a_start: i64, a_end: i64, b_start: i64, b_end: i64) -> i64 {
    let overlap_start = a_start.max(b_start);
    let overlap_end = a_end.min(b_end);
    (overlap_end - overlap_start).max(0)
}

/// Match subs1 entries with subs2 entries based on temporal overlap.
/// Returns a Vec of MatchedLine with the best subs2 match for each subs1 entry.
fn match_subtitles(subs1: &[SubEntry], subs2: &[SubEntry]) -> Vec<MatchedLine> {
    let mut matched: Vec<MatchedLine> = Vec::with_capacity(subs1.len());

    for (i, s1) in subs1.iter().enumerate() {
        // Find best matching subs2 entry by overlap
        let mut best_match: Option<&SubEntry> = None;
        let mut best_overlap: i64 = 0;

        for s2 in subs2.iter() {
            let overlap = get_overlap(s1.start_ms, s1.end_ms, s2.start_ms, s2.end_ms);
            if overlap > best_overlap {
                best_overlap = overlap;
                best_match = Some(s2);
            }
            // Optimization: if subs2 starts after subs1 ends + 5s margin, stop
            if s2.start_ms > s1.end_ms + 5000 {
                break;
            }
        }

        matched.push(MatchedLine {
            index: i,
            subs1: s1.clone(),
            subs2: best_match.cloned(),
            active: s1.active,
            leading_context: Vec::new(),
            trailing_context: Vec::new(),
        });
    }

    // Pass 2: Combine consecutive lines that map to the same subs2
    combine_consecutive_repeats(&mut matched);

    matched
}

/// If multiple subs1 lines map to the same subs2 line, combine them
fn combine_consecutive_repeats(matched: &mut Vec<MatchedLine>) {
    let mut i = 0;
    while i + 1 < matched.len() {
        let same_s2 = match (&matched[i].subs2, &matched[i + 1].subs2) {
            (Some(a), Some(b)) => a.id == b.id,
            _ => false,
        };

        if same_s2 {
            // Merge i+1 into i
            let next_text = matched[i + 1].subs1.text.clone();
            let next_end = matched[i + 1].subs1.end_ms;
            matched[i].subs1.text = format!("{} {}", matched[i].subs1.text, next_text);
            matched[i].subs1.end_ms = next_end;
            matched.remove(i + 1);
            // Reindex
            for (j, m) in matched.iter_mut().enumerate() {
                m.index = j;
            }
        } else {
            i += 1;
        }
    }
}

// ─── Filters ─────────────────────────────────────────────────────────────────

fn apply_filters(lines: &mut [MatchedLine], filters: &SubtitleFilters) {
    let include_set: Option<Vec<String>> = filters.include_words.as_ref().map(|w| {
        w.split(',').map(|s| s.trim().to_lowercase()).filter(|s| !s.is_empty()).collect()
    });

    let exclude_set: Option<Vec<String>> = filters.exclude_words.as_ref().map(|w| {
        w.split(',').map(|s| s.trim().to_lowercase()).filter(|s| !s.is_empty()).collect()
    });

    // Collect texts for duplicate detection
    let mut seen_subs1: HashSet<String> = HashSet::new();
    let mut seen_subs2: HashSet<String> = HashSet::new();

    // Actor filter
    let actor_filter: Option<Vec<String>> = filters.actor_filter.as_ref().map(|a| {
        a.split(',').map(|s| s.trim().to_lowercase()).filter(|s| !s.is_empty()).collect()
    });

    for line in lines.iter_mut() {
        if !line.active {
            continue;
        }

        let text_lower = line.subs1.text.to_lowercase();
        let duration = line.subs1.end_ms - line.subs1.start_ms;

        // Include words filter
        if let Some(ref words) = include_set {
            if !words.iter().any(|w| text_lower.contains(w)) {
                line.active = false;
                continue;
            }
        }

        // Exclude words filter
        if let Some(ref words) = exclude_set {
            if words.iter().any(|w| text_lower.contains(w)) {
                line.active = false;
                continue;
            }
        }

        // Exclude duplicates subs1
        if filters.exclude_duplicates_subs1 {
            let normalized = line.subs1.text.trim().to_string();
            if seen_subs1.contains(&normalized) {
                line.active = false;
                continue;
            }
            seen_subs1.insert(normalized);
        }

        // Exclude duplicates subs2
        if filters.exclude_duplicates_subs2 {
            if let Some(ref s2) = line.subs2 {
                let normalized = s2.text.trim().to_string();
                if seen_subs2.contains(&normalized) {
                    line.active = false;
                    continue;
                }
                seen_subs2.insert(normalized);
            }
        }

        // Min/max character length
        if let Some(min) = filters.min_chars {
            if line.subs1.text.chars().count() < min {
                line.active = false;
                continue;
            }
        }
        if let Some(max) = filters.max_chars {
            if line.subs1.text.chars().count() > max {
                line.active = false;
                continue;
            }
        }

        // Min/max duration
        if let Some(min) = filters.min_duration_ms {
            if duration < min {
                line.active = false;
                continue;
            }
        }
        if let Some(max) = filters.max_duration_ms {
            if duration > max {
                line.active = false;
                continue;
            }
        }

        // Exclude styled lines (ASS)
        if filters.exclude_styled {
            if line.subs1.text.starts_with('{') {
                line.active = false;
                continue;
            }
        }

        // Actor filter (ASS only)
        if let Some(ref actors) = actor_filter {
            if let Some(ref actor) = line.subs1.actor {
                if !actors.iter().any(|a| a == &actor.to_lowercase()) {
                    line.active = false;
                    continue;
                }
            } else {
                // No actor info, filter out if actor filter is set
                line.active = false;
                continue;
            }
        }

        // Only CJK characters
        if filters.only_cjk {
            let has_cjk = line.subs1.text.chars().any(|c| {
                matches!(c,
                    '\u{4E00}'..='\u{9FFF}' |  // CJK Unified Ideographs
                    '\u{3400}'..='\u{4DBF}' |  // CJK Extension A
                    '\u{3040}'..='\u{309F}' |  // Hiragana
                    '\u{30A0}'..='\u{30FF}'    // Katakana
                )
            });
            if !has_cjk {
                line.active = false;
                continue;
            }
        }

        // Remove lines with no subs2 match
        if filters.remove_no_match && line.subs2.is_none() {
            line.active = false;
        }
    }
}

// ─── Sentence Combining ─────────────────────────────────────────────────────

fn combine_sentences(lines: &mut Vec<MatchedLine>, continuation_chars: &str) {
    if continuation_chars.is_empty() {
        return;
    }

    let cont_chars: Vec<char> = continuation_chars.chars().collect();
    let mut i = 0;

    while i + 1 < lines.len() {
        let ends_with_cont = lines[i]
            .subs1
            .text
            .trim_end()
            .chars()
            .last()
            .map(|c| cont_chars.contains(&c))
            .unwrap_or(false);

        if ends_with_cont && lines[i].active && lines[i + 1].active {
            let next_text = lines[i + 1].subs1.text.clone();
            let next_end = lines[i + 1].subs1.end_ms;
            let next_s2 = lines[i + 1].subs2.clone();

            lines[i].subs1.text = format!("{} {}", lines[i].subs1.text, next_text);
            lines[i].subs1.end_ms = next_end;

            // Combine subs2 too if both present
            if let (Some(ref mut s2), Some(next_s2)) = (&mut lines[i].subs2, next_s2) {
                s2.text = format!("{} {}", s2.text, next_s2.text);
                s2.end_ms = next_s2.end_ms;
            }

            lines.remove(i + 1);
            // Reindex
            for (j, m) in lines.iter_mut().enumerate() {
                m.index = j;
            }
            // Don't advance i - might need to combine more
        } else {
            i += 1;
        }
    }
}

// ─── Context Lines ───────────────────────────────────────────────────────────

fn compute_context(lines: &mut Vec<MatchedLine>, ctx: &ContextConfig) {
    if ctx.leading == 0 && ctx.trailing == 0 {
        return;
    }

    let gap_ms = (ctx.max_gap_seconds * 1000.0) as i64;
    let len = lines.len();

    for i in 0..len {
        let mut leading = Vec::new();
        let mut trailing = Vec::new();

        // Leading context
        for j in 1..=ctx.leading {
            if i < j {
                break;
            }
            let prev_idx = i - j;
            let gap = lines[i].subs1.start_ms - lines[prev_idx].subs1.end_ms;
            if gap_ms > 0 && gap > gap_ms {
                break;
            }
            leading.push(prev_idx);
        }
        leading.reverse();

        // Trailing context
        for j in 1..=ctx.trailing {
            let next_idx = i + j;
            if next_idx >= len {
                break;
            }
            let gap = lines[next_idx].subs1.start_ms - lines[i].subs1.end_ms;
            if gap_ms > 0 && gap > gap_ms {
                break;
            }
            trailing.push(next_idx);
        }

        lines[i].leading_context = leading;
        lines[i].trailing_context = trailing;
    }
}

// ─── Span Filter ─────────────────────────────────────────────────────────────

fn apply_span(lines: &mut [MatchedLine], span_start: Option<i64>, span_end: Option<i64>) {
    for line in lines.iter_mut() {
        if let Some(start) = span_start {
            if line.subs1.end_ms < start {
                line.active = false;
            }
        }
        if let Some(end) = span_end {
            if line.subs1.start_ms > end {
                line.active = false;
            }
        }
    }
}

// ─── FFmpeg Media Extraction ─────────────────────────────────────────────────

/// Check if ffmpeg is available
async fn check_ffmpeg() -> Result<bool> {
    let output = tokio::process::Command::new("ffmpeg")
        .arg("-version")
        .output()
        .await;
    Ok(output.is_ok())
}

/// Format milliseconds as ffmpeg timestamp HH:MM:SS.mmm
fn ms_to_ffmpeg_ts(ms: i64) -> String {
    let ms = ms.max(0);
    let total_secs = ms / 1000;
    let millis = ms % 1000;
    let secs = total_secs % 60;
    let mins = (total_secs / 60) % 60;
    let hours = total_secs / 3600;
    format!("{:02}:{:02}:{:02}.{:03}", hours, mins, secs, millis)
}

/// Extract audio clip for a single subtitle line
async fn extract_audio_clip(
    source_path: &str,
    output_path: &Path,
    start_ms: i64,
    end_ms: i64,
    pad_start_ms: i64,
    pad_end_ms: i64,
    bitrate: u32,
) -> Result<()> {
    let actual_start = (start_ms - pad_start_ms).max(0);
    let duration_ms = (end_ms + pad_end_ms) - actual_start;

    let mut cmd = tokio::process::Command::new("ffmpeg");
    cmd.args([
        "-nostdin",
        "-loglevel", "error",
        "-y",
        "-ss", &ms_to_ffmpeg_ts(actual_start),
        "-t", &ms_to_ffmpeg_ts(duration_ms),
        "-i", source_path,
        "-vn", "-sn", "-dn",
        "-ac", "2",
        "-ab", &format!("{}k", bitrate),
        "-ar", "44100",
        "-f", "mp3",
    ]);
    cmd.arg(output_path.as_os_str());

    let output = cmd.output().await.context("Failed to run ffmpeg for audio")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("ffmpeg audio error: {}", stderr);
    }
    Ok(())
}

/// Extract snapshot at midpoint of subtitle
async fn extract_snapshot(
    video_path: &str,
    output_path: &Path,
    start_ms: i64,
    end_ms: i64,
    width: u32,
    height: u32,
    crop_bottom: u32,
) -> Result<()> {
    let midpoint_ms = start_ms + (end_ms - start_ms) / 2;

    let mut vf_filters = Vec::new();
    if crop_bottom > 0 {
        vf_filters.push(format!("crop=in_w:in_h-{}:0:0", crop_bottom));
    }
    vf_filters.push(format!("scale={}:{}:flags=bicubic", width, height));
    let vf = vf_filters.join(",");

    let mut cmd = tokio::process::Command::new("ffmpeg");
    cmd.args([
        "-nostdin",
        "-loglevel", "error",
        "-y",
        "-ss", &ms_to_ffmpeg_ts(midpoint_ms),
        "-i", video_path,
        "-an", "-sn", "-dn",
        "-vframes", "1",
        "-vf", &vf,
        "-pix_fmt", "yuvj420p",
        "-q:v", "2",
    ]);
    cmd.arg(output_path.as_os_str());

    let output = cmd.output().await.context("Failed to run ffmpeg for snapshot")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("ffmpeg snapshot error: {}", stderr);
    }
    Ok(())
}

/// Extract video clip for a single subtitle line
async fn extract_video_clip(
    video_path: &str,
    output_path: &Path,
    start_ms: i64,
    end_ms: i64,
    pad_start_ms: i64,
    pad_end_ms: i64,
    codec: &str,
    preset: &str,
    video_bitrate: u32,
    audio_bitrate: u32,
) -> Result<()> {
    let actual_start = (start_ms - pad_start_ms).max(0);
    let duration_ms = (end_ms + pad_end_ms) - actual_start;

    let mut cmd = tokio::process::Command::new("ffmpeg");
    cmd.args([
        "-nostdin",
        "-loglevel", "error",
        "-y",
        "-ss", &ms_to_ffmpeg_ts(actual_start),
        "-t", &ms_to_ffmpeg_ts(duration_ms),
        "-i", video_path,
    ]);

    match codec {
        "h264" => {
            cmd.args([
                "-c:v", "libx264",
                "-preset", preset,
                "-b:v", &format!("{}k", video_bitrate),
                "-c:a", "aac",
                "-b:a", &format!("{}k", audio_bitrate),
            ]);
        }
        _ => {
            // mpeg4
            cmd.args([
                "-c:v", "mpeg4",
                "-b:v", &format!("{}k", video_bitrate),
                "-c:a", "mp3",
                "-b:a", &format!("{}k", audio_bitrate),
            ]);
        }
    }

    cmd.arg(output_path.as_os_str());

    let output = cmd.output().await.context("Failed to run ffmpeg for video clip")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("ffmpeg video error: {}", stderr);
    }
    Ok(())
}

/// Normalize audio volume using ffmpeg loudnorm
async fn normalize_audio(file_path: &Path) -> Result<()> {
    let temp_path = file_path.with_extension("normalized.mp3");

    let mut cmd = tokio::process::Command::new("ffmpeg");
    cmd.args([
        "-y",
        "-i",
    ]);
    cmd.arg(file_path.as_os_str());
    cmd.args([
        "-af", "loudnorm=I=-16:TP=-1.5:LRA=11",
        "-ar", "44100",
        "-ac", "2",
    ]);
    cmd.arg(temp_path.as_os_str());

    let output = cmd.output().await.context("Failed to normalize audio")?;
    if output.status.success() {
        std::fs::rename(&temp_path, file_path)?;
    } else {
        let _ = std::fs::remove_file(&temp_path);
    }
    Ok(())
}

// ─── TSV Generation ──────────────────────────────────────────────────────────

fn generate_tsv(
    lines: &[MatchedLine],
    config: &FlashcardConfig,
    _audio_dir: &str,
    _snapshot_dir: &str,
    _video_dir: &str,
) -> String {
    let active_lines: Vec<&MatchedLine> = lines.iter().filter(|l| l.active).collect();
    let mut tsv = String::with_capacity(active_lines.len() * 200);

    // Pre-calculate loop-invariant values
    let sanitized_deck = sanitize_filename(&config.deck_name);
    let ep = config.episode_number;
    let video_ext = if config.video_codec == "h264" { "mp4" } else { "avi" };

    for (seq, line) in active_lines.iter().enumerate() {
        let mut fields: Vec<String> = Vec::new();

        let seq_num = seq + 1;
        let start_ts = ms_to_ffmpeg_ts(line.subs1.start_ms);

        // Tag
        if config.output_fields.include_tag {
            fields.push(format!("{}_{:03}", config.deck_name, ep));
        }

        // Sequence marker
        if config.output_fields.include_sequence {
            fields.push(format!("{:03}_{:04}_{}", ep, seq_num, start_ts));
        }

        // Audio
        if config.output_fields.include_audio && config.generate_audio {
            let filename = format!(
                "{}_{:03}_{:04}.mp3",
                sanitized_deck,
                ep,
                seq_num
            );
            fields.push(format!("[sound:{}]", filename));
        }

        // Snapshot
        if config.output_fields.include_snapshot && config.generate_snapshots {
            let filename = format!(
                "{}_{:03}_{:04}.jpg",
                sanitized_deck,
                ep,
                seq_num
            );
            fields.push(format!("<img src=\"{}\">", filename));
        }

        // Video clip
        if config.output_fields.include_video && config.generate_video_clips {
            let filename = format!(
                "{}_{:03}_{:04}.{}",
                sanitized_deck,
                ep,
                seq_num,
                video_ext
            );
            fields.push(format!("[sound:{}]", filename));
        }

        // Subs1 text (with context)
        if config.output_fields.include_subs1 {
            let mut text = String::new();
            // Leading context
            for &ctx_idx in &line.leading_context {
                if let Some(ctx_line) = lines.get(ctx_idx) {
                    text.push_str(&format!("<span style=\"color:gray\">{}</span><br>", ctx_line.subs1.text));
                }
            }
            text.push_str(&line.subs1.text);
            // Trailing context
            for &ctx_idx in &line.trailing_context {
                if let Some(ctx_line) = lines.get(ctx_idx) {
                    text.push_str(&format!("<br><span style=\"color:gray\">{}</span>", ctx_line.subs1.text));
                }
            }
            fields.push(text.replace('\t', " ").replace('\n', "<br>"));
        }

        // Subs2 text (with context)
        if config.output_fields.include_subs2 {
            if let Some(ref s2) = line.subs2 {
                let mut text = String::new();
                for &ctx_idx in &line.leading_context {
                    if let Some(ctx_line) = lines.get(ctx_idx) {
                        if let Some(ref ctx_s2) = ctx_line.subs2 {
                            text.push_str(&format!("<span style=\"color:gray\">{}</span><br>", ctx_s2.text));
                        }
                    }
                }
                text.push_str(&s2.text);
                for &ctx_idx in &line.trailing_context {
                    if let Some(ctx_line) = lines.get(ctx_idx) {
                        if let Some(ref ctx_s2) = ctx_line.subs2 {
                            text.push_str(&format!("<br><span style=\"color:gray\">{}</span>", ctx_s2.text));
                        }
                    }
                }
                fields.push(text.replace('\t', " ").replace('\n', "<br>"));
            } else {
                fields.push(String::new());
            }
        }

        tsv.push_str(&fields.join("\t"));
        tsv.push('\n');
    }

    tsv
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

// ─── APKG Generation ─────────────────────────────────────────────────────────

/// Generate an APKG file (Anki package) from matched lines.
/// Builds the SQLite database (collection.anki2) and packages it into a ZIP
/// along with media files. This approach mirrors the Anki internal format.
fn generate_apkg(
    lines: &[MatchedLine],
    config: &FlashcardConfig,
    media_dir: &Path,
    output_path: &Path,
) -> Result<(), String> {
    use std::collections::HashMap;

    let active_lines: Vec<&MatchedLine> = lines.iter().filter(|l| l.active).collect();
    if active_lines.is_empty() {
        return Err("No active lines to export".to_string());
    }

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let deck_id: i64 = {
        // Use a deterministic deck ID from the deck name
        let mut hash: u64 = 0;
        for b in config.deck_name.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(b as u64);
        }
        // Ensure positive and not 1 (reserved for "Default" deck)
        ((hash % 1_000_000_000) + 1_000_000) as i64
    };

    let model_id: i64 = deck_id + 1;
    let deck_sanitized = sanitize_filename(&config.deck_name);
    let ep = config.episode_number;

    // Create a temp directory for the SQLite DB
    let tmp_dir = tempfile::tempdir().map_err(|e| format!("Cannot create temp dir: {e}"))?;
    let db_path = tmp_dir.path().join("collection.anki2");

    {
        // Open SQLite connection
        // We use rusqlite-like approach but since we don't have rusqlite,
        // we'll write the SQL file using the system sqlite3 command.
        // Actually, let's build it with raw bytes / a simple SQLite writer.
        // Better: use std::process::Command to call sqlite3.
        
        // Check if sqlite3 is available
        let sqlite_check = std::process::Command::new("sqlite3")
            .arg("--version")
            .output();
        
        if sqlite_check.is_err() || !sqlite_check.unwrap().status.success() {
            return Err("sqlite3 not found. Install sqlite3 to export APKG files.".to_string());
        }

        // Build SQL commands
        let mut sql = String::with_capacity(active_lines.len() * 512);

        // Wrap all operations in a transaction for performance
        sql.push_str("BEGIN TRANSACTION;\n");

        // Create tables
        sql.push_str("CREATE TABLE col (
            id INTEGER PRIMARY KEY,
            crt INTEGER NOT NULL,
            mod INTEGER NOT NULL,
            scm INTEGER NOT NULL,
            ver INTEGER NOT NULL,
            dty INTEGER NOT NULL,
            usn INTEGER NOT NULL,
            ls INTEGER NOT NULL,
            conf TEXT NOT NULL,
            models TEXT NOT NULL,
            decks TEXT NOT NULL,
            dconf TEXT NOT NULL,
            tags TEXT NOT NULL
        );\n");

        sql.push_str("CREATE TABLE notes (
            id INTEGER PRIMARY KEY,
            guid TEXT NOT NULL,
            mid INTEGER NOT NULL,
            mod INTEGER NOT NULL,
            usn INTEGER NOT NULL,
            tags TEXT NOT NULL,
            flds TEXT NOT NULL,
            sfld TEXT NOT NULL,
            csum INTEGER NOT NULL,
            flags INTEGER NOT NULL,
            data TEXT NOT NULL
        );\n");

        sql.push_str("CREATE TABLE cards (
            id INTEGER PRIMARY KEY,
            nid INTEGER NOT NULL,
            did INTEGER NOT NULL,
            ord INTEGER NOT NULL,
            mod INTEGER NOT NULL,
            usn INTEGER NOT NULL,
            type INTEGER NOT NULL,
            queue INTEGER NOT NULL,
            due INTEGER NOT NULL,
            ivl INTEGER NOT NULL,
            factor INTEGER NOT NULL,
            reps INTEGER NOT NULL,
            lapses INTEGER NOT NULL,
            left INTEGER NOT NULL,
            odue INTEGER NOT NULL,
            odid INTEGER NOT NULL,
            flags INTEGER NOT NULL,
            data TEXT NOT NULL
        );\n");

        sql.push_str("CREATE TABLE graves (
            usn INTEGER NOT NULL,
            oid INTEGER NOT NULL,
            type INTEGER NOT NULL
        );\n");

        sql.push_str("CREATE TABLE revlog (
            id INTEGER PRIMARY KEY,
            cid INTEGER NOT NULL,
            usn INTEGER NOT NULL,
            ease INTEGER NOT NULL,
            ivl INTEGER NOT NULL,
            lastIvl INTEGER NOT NULL,
            factor INTEGER NOT NULL,
            time INTEGER NOT NULL,
            type INTEGER NOT NULL
        );\n");

        // Build model fields based on what the user selected
        let mut field_defs = Vec::new();
        let mut ord = 0;

        if config.output_fields.include_subs1 {
            field_defs.push(format!(
                r#"{{"name":"Expression","ord":{},"sticky":false,"rtl":false,"font":"Arial","size":20,"description":"","plainText":false,"collapsed":false,"excludeFromSearch":false}}"#,
                ord
            ));
            ord += 1;
        }
        if config.output_fields.include_subs2 {
            field_defs.push(format!(
                r#"{{"name":"Meaning","ord":{},"sticky":false,"rtl":false,"font":"Arial","size":20,"description":"","plainText":false,"collapsed":false,"excludeFromSearch":false}}"#,
                ord
            ));
            ord += 1;
        }
        if config.output_fields.include_audio {
            field_defs.push(format!(
                r#"{{"name":"Audio","ord":{},"sticky":false,"rtl":false,"font":"Arial","size":20,"description":"","plainText":false,"collapsed":false,"excludeFromSearch":false}}"#,
                ord
            ));
            ord += 1;
        }
        if config.output_fields.include_snapshot {
            field_defs.push(format!(
                r#"{{"name":"Snapshot","ord":{},"sticky":false,"rtl":false,"font":"Arial","size":20,"description":"","plainText":false,"collapsed":false,"excludeFromSearch":false}}"#,
                ord
            ));
            ord += 1;
        }
        if config.output_fields.include_video {
            field_defs.push(format!(
                r#"{{"name":"Video","ord":{},"sticky":false,"rtl":false,"font":"Arial","size":20,"description":"","plainText":false,"collapsed":false,"excludeFromSearch":false}}"#,
                ord
            ));
            ord += 1;
        }
        if config.output_fields.include_tag {
            field_defs.push(format!(
                r#"{{"name":"Tag","ord":{},"sticky":false,"rtl":false,"font":"Arial","size":20,"description":"","plainText":false,"collapsed":false,"excludeFromSearch":false}}"#,
                ord
            ));
            ord += 1;
        }
        if config.output_fields.include_sequence {
            field_defs.push(format!(
                r#"{{"name":"SequenceMarker","ord":{},"sticky":false,"rtl":false,"font":"Arial","size":20,"description":"","plainText":false,"collapsed":false,"excludeFromSearch":false}}"#,
                ord
            ));
            ord += 1;
        }
        // Reading field (always included, user fills manually)
        field_defs.push(format!(
            r#"{{"name":"Reading","ord":{},"sticky":false,"rtl":false,"font":"Arial","size":20,"description":"","plainText":false,"collapsed":false,"excludeFromSearch":false}}"#,
            ord
        ));
        let _ = ord;

        // If no fields, add defaults
        if field_defs.is_empty() {
            field_defs.push(r#"{"name":"Front","ord":0,"sticky":false,"rtl":false,"font":"Arial","size":20,"description":"","plainText":false,"collapsed":false,"excludeFromSearch":false}"#.to_string());
            field_defs.push(r#"{"name":"Back","ord":1,"sticky":false,"rtl":false,"font":"Arial","size":20,"description":"","plainText":false,"collapsed":false,"excludeFromSearch":false}"#.to_string());
        }

        // Use custom templates if provided, otherwise use defaults
        let qfmt = config.card_front_html.as_deref().unwrap_or(ANKI_FRONT_TEMPLATE);
        let afmt = config.card_back_html.as_deref().unwrap_or(ANKI_BACK_TEMPLATE);
        let css = config.card_css.as_deref().unwrap_or(ANKI_CARD_STYLING);

        let note_type_name = config.note_type_name.as_deref().unwrap_or("subs2srs");

        let models_json = format!(
            r#"{{"{mid}":{{"id":{mid},"name":"{note_type}","type":0,"mod":{ts},"usn":-1,"sortf":0,"did":{did},"tmpls":[{{"name":"Card 1","ord":0,"qfmt":"{qfmt}","afmt":"{afmt}","did":null,"bqfmt":"","bafmt":""}}],"flds":[{flds}],"css":"{css}","latexPre":"\\\\documentclass[12pt]{{article}}\\\\special{{papersize=3in,5in}}\\\\usepackage[utf8]{{inputenc}}\\\\usepackage{{amssymb,amsmath}}\\\\pagestyle{{empty}}\\\\setlength{{\\\\parindent}}{{0in}}\\\\begin{{document}}\\n","latexPost":"\\\\end{{document}}","latexsvg":false,"req":[[0,"all",[0]]]}}}}"#,
            mid = model_id,
            note_type = note_type_name.replace('"', r#"\""#),
            ts = timestamp,
            did = deck_id,
            qfmt = qfmt.replace('"', r#"\""#).replace('\n', "\\n"),
            afmt = afmt.replace('"', r#"\""#).replace('\n', "\\n"),
            flds = field_defs.join(","),
            css = css.replace('"', r#"\""#).replace('\n', "\\n"),
        );

        let decks_json = format!(
            r#"{{"{did}":{{"id":{did},"name":"{name}","mod":{ts},"usn":-1,"lrnToday":[0,0],"revToday":[0,0],"newToday":[0,0],"timeToday":[0,0],"collapsed":false,"browserCollapsed":false,"desc":"","dyn":0,"conf":1,"extendNew":10,"extendRev":50}}}}"#,
            did = deck_id,
            name = config.deck_name.replace('"', r#"\""#),
            ts = timestamp,
        );

        let dconf_json = r#"{"1":{"id":1,"name":"Default","replayq":true,"lapse":{"delays":[10],"mult":0,"minInt":1,"leechFails":8,"leechAction":0},"rev":{"perDay":200,"ease4":1.3,"fuzz":0.05,"minSpace":1,"ivlFct":1,"maxIvl":36500,"buried":false,"hardFactor":1.2},"timer":0,"maxTaken":60,"usn":0,"new":{"delays":[1,10],"ints":[1,4,0],"initialFactor":2500,"order":1,"perDay":20,"buried":false},"mod":0,"autoplay":true}}"#;

        let conf_json = r#"{"activeDecks":[1],"curDeck":1,"newSpread":0,"collapseTime":1200,"timeLim":0,"estTimes":true,"dueCounts":true,"curModel":null,"nextPos":1,"sortType":"noteFld","sortBackwards":false,"addToCur":true}"#;

        // Escape for SQL
        let models_sql = models_json.replace('\'', "''");
        let decks_sql = decks_json.replace('\'', "''");
        let dconf_sql = dconf_json.replace('\'', "''");
        let conf_sql = conf_json.replace('\'', "''");

        sql.push_str(&format!(
            "INSERT INTO col VALUES (1, {ts}, {ts}, 0, 11, 0, 0, 0, '{conf}', '{models}', '{decks}', '{dconf}', '{{}}');\n",
            ts = timestamp,
            conf = conf_sql,
            models = models_sql,
            decks = decks_sql,
            dconf = dconf_sql,
        ));

        // Insert notes and cards
        for (seq, line) in active_lines.iter().enumerate() {
            let note_id = timestamp * 1000 + seq as i64;
            let card_id = note_id + 1_000_000;
            let seq_num = seq + 1;
            let start_ts = ms_to_ffmpeg_ts(line.subs1.start_ms);

            // Build fields (separated by \x1f)
            let mut fields: Vec<String> = Vec::new();

            // Expression (subs1)
            if config.output_fields.include_subs1 {
                let mut text = String::new();
                for &ctx_idx in &line.leading_context {
                    if let Some(ctx_line) = lines.get(ctx_idx) {
                        text.push_str(&format!("<span class=\"context\">{}</span><br>", ctx_line.subs1.text));
                    }
                }
                text.push_str(&line.subs1.text);
                for &ctx_idx in &line.trailing_context {
                    if let Some(ctx_line) = lines.get(ctx_idx) {
                        text.push_str(&format!("<br><span class=\"context\">{}</span>", ctx_line.subs1.text));
                    }
                }
                fields.push(text.replace('\n', "<br>"));
            }

            // Meaning (subs2)
            if config.output_fields.include_subs2 {
                if let Some(ref s2) = line.subs2 {
                    let mut text = String::new();
                    for &ctx_idx in &line.leading_context {
                        if let Some(ctx_line) = lines.get(ctx_idx) {
                            if let Some(ref ctx_s2) = ctx_line.subs2 {
                                text.push_str(&format!("<span class=\"context\">{}</span><br>", ctx_s2.text));
                            }
                        }
                    }
                    text.push_str(&s2.text);
                    for &ctx_idx in &line.trailing_context {
                        if let Some(ctx_line) = lines.get(ctx_idx) {
                            if let Some(ref ctx_s2) = ctx_line.subs2 {
                                text.push_str(&format!("<br><span class=\"context\">{}</span>", ctx_s2.text));
                            }
                        }
                    }
                    fields.push(text.replace('\n', "<br>"));
                } else {
                    fields.push(String::new());
                }
            }

            // Audio — only reference if the file actually exists
            if config.output_fields.include_audio {
                let filename = format!("{}_{:03}_{:04}.mp3", deck_sanitized, ep, seq_num);
                let file_path = media_dir.join(&filename);
                if file_path.exists() {
                    fields.push(format!("[sound:{}]", filename));
                } else {
                    fields.push(String::new());
                }
            }

            // Snapshot — only reference if the file actually exists
            if config.output_fields.include_snapshot {
                let filename = format!("{}_{:03}_{:04}.jpg", deck_sanitized, ep, seq_num);
                let file_path = media_dir.join(&filename);
                if file_path.exists() {
                    fields.push(format!("<img src=\"{}\">", filename));
                } else {
                    fields.push(String::new());
                }
            }

            // Video — only reference if the file actually exists
            if config.output_fields.include_video {
                let ext = if config.video_codec == "h264" { "mp4" } else { "avi" };
                let filename = format!("{}_{:03}_{:04}.{}", deck_sanitized, ep, seq_num, ext);
                let file_path = media_dir.join(&filename);
                if file_path.exists() {
                    fields.push(format!("[sound:{}]", filename));
                } else {
                    fields.push(String::new());
                }
            }

            // Tag
            if config.output_fields.include_tag {
                fields.push(format!("{}_{:03}", config.deck_name, ep));
            }

            // SequenceMarker
            if config.output_fields.include_sequence {
                fields.push(format!("{:03}_{:04}_{}", ep, seq_num, start_ts));
            }

            // Reading (empty — user fills manually in Anki)
            fields.push(String::new());

            let flds = fields.join("\x1f");
            let sfld = if !fields.is_empty() { &fields[0] } else { "" };

            // Compute checksum: Anki uses first 4 bytes of SHA-1(sfld) as u32
            // We approximate with a simple FNV-1a hash truncated to u32
            let csum = {
                let mut hash: u32 = 2166136261;
                for byte in sfld.as_bytes() {
                    hash ^= *byte as u32;
                    hash = hash.wrapping_mul(16777619);
                }
                hash as i64
            };

            // GUID
            let guid = format!("{:010x}", note_id as u64);

            let flds_sql = flds.replace('\'', "''");
            let sfld_sql = sfld.replace('\'', "''");
            let guid_sql = guid.replace('\'', "''");

            sql.push_str(&format!(
                "INSERT INTO notes VALUES ({nid}, '{guid}', {mid}, {ts}, 0, '', '{flds}', '{sfld}', {csum}, 0, '');\n",
                nid = note_id,
                guid = guid_sql,
                mid = model_id,
                ts = timestamp,
                flds = flds_sql,
                sfld = sfld_sql,
                csum = csum,
            ));

            sql.push_str(&format!(
                "INSERT INTO cards VALUES ({cid}, {nid}, {did}, 0, {ts}, 0, 0, 0, {due}, 0, 2500, 0, 0, 0, 0, 0, 0, '');\n",
                cid = card_id,
                nid = note_id,
                did = deck_id,
                ts = timestamp,
                due = seq + 1,
            ));
        }

        sql.push_str("COMMIT;\n");

        // Write SQL to temp file
        let sql_path = tmp_dir.path().join("create.sql");
        let mut sql_file = std::fs::File::create(&sql_path)
            .map_err(|e| format!("Cannot create SQL file: {e}"))?;
        sql_file.write_all(sql.as_bytes())
            .map_err(|e| format!("Cannot write SQL: {e}"))?;
        drop(sql_file);

        // Execute with sqlite3
        let output = std::process::Command::new("sqlite3")
            .arg(db_path.to_str().unwrap_or(""))
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .arg(format!(".read {}", sql_path.to_str().unwrap_or("")))
            .output()
            .map_err(|e| format!("Failed to execute sqlite3: {e}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("sqlite3 error: {stderr}"));
        }
    }

    // Build media map: { "0": "filename.mp3", "1": "filename.jpg", ... }
    let mut media_map: HashMap<String, String> = HashMap::new();
    let mut media_files: Vec<(String, PathBuf)> = Vec::new();
    let mut media_idx = 0u64;

    for (seq, line) in active_lines.iter().enumerate() {
        let seq_num = seq + 1;

        // Audio
        if config.generate_audio {
            let filename = format!("{}_{:03}_{:04}.mp3", deck_sanitized, ep, seq_num);
            let file_path = media_dir.join(&filename);
            if file_path.exists() {
                media_map.insert(media_idx.to_string(), filename.clone());
                media_files.push((media_idx.to_string(), file_path));
                media_idx += 1;
            }
        }

        // Snapshot
        if config.generate_snapshots {
            let filename = format!("{}_{:03}_{:04}.jpg", deck_sanitized, ep, seq_num);
            let file_path = media_dir.join(&filename);
            if file_path.exists() {
                media_map.insert(media_idx.to_string(), filename.clone());
                media_files.push((media_idx.to_string(), file_path));
                media_idx += 1;
            }
        }

        // Video
        if config.generate_video_clips {
            let ext = if config.video_codec == "h264" { "mp4" } else { "avi" };
            let filename = format!("{}_{:03}_{:04}.{}", deck_sanitized, ep, seq_num, ext);
            let file_path = media_dir.join(&filename);
            if file_path.exists() {
                media_map.insert(media_idx.to_string(), filename.clone());
                media_files.push((media_idx.to_string(), file_path));
                media_idx += 1;
            }
        }

        // Suppress unused variable warning
        let _ = line;
    }

    // Write media JSON to temp
    let media_json_path = tmp_dir.path().join("media");
    std::fs::write(&media_json_path, serde_json::to_string(&media_map).unwrap_or_else(|_| "{}".to_string()))
        .map_err(|e| format!("Cannot write media JSON: {e}"))?;

    // Create the APKG ZIP file
    let apkg_file = std::fs::File::create(output_path)
        .map_err(|e| format!("Cannot create APKG: {e}"))?;
    let mut zip = zip::ZipWriter::new(apkg_file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    // Add collection.anki2
    zip.start_file("collection.anki2", options)
        .map_err(|e| format!("ZIP error: {e}"))?;
    let db_bytes = std::fs::read(&db_path)
        .map_err(|e| format!("Cannot read DB: {e}"))?;
    zip.write_all(&db_bytes)
        .map_err(|e| format!("ZIP write error: {e}"))?;

    // Add media JSON
    zip.start_file("media", options)
        .map_err(|e| format!("ZIP error: {e}"))?;
    let media_json_bytes = std::fs::read(&media_json_path)
        .map_err(|e| format!("Cannot read media JSON: {e}"))?;
    zip.write_all(&media_json_bytes)
        .map_err(|e| format!("ZIP write error: {e}"))?;

    // Add actual media files (indexed by number)
    for (idx_str, file_path) in &media_files {
        zip.start_file(idx_str, options)
            .map_err(|e| format!("ZIP error adding media: {e}"))?;
        let file_bytes = std::fs::read(file_path)
            .map_err(|e| format!("Cannot read media file: {e}"))?;
        zip.write_all(&file_bytes)
            .map_err(|e| format!("ZIP write error: {e}"))?;
    }

    zip.finish().map_err(|e| format!("ZIP finish error: {e}"))?;

    Ok(())
}

// ─── Anki Card Templates ─────────────────────────────────────────────────────
//
// These constants define the note type used for APKG export.
// Edit them to customise how cards look in Anki.

const ANKI_FRONT_TEMPLATE: &str = r#"
<div id="tags-container"></div>
<div id="tags-source" style="display: none;">{{Tags}}</div>
<div id="timestamp-source" style="display: none;">{{SequenceMarker}}</div>
<div class='expression'>{{Expression}}</div>
<hr>
<script>
try {
    var container = document.getElementById('tags-container');
    container.innerHTML = '';
    try {
        var rawString = document.getElementById('timestamp-source').innerText;
        if (rawString && rawString.includes('_') && rawString.includes('.')) {
            var fullTimestamp = rawString.split('_').pop();
            var parts = fullTimestamp.split('.');
            var formattedTimestamp = parts.slice(0, 3).join(':');
            var ts_pill = document.createElement('span');
            ts_pill.className = 'tag-pill';
            ts_pill.textContent = formattedTimestamp;
            container.appendChild(ts_pill);
        }
    } catch (e_ts) {}
    try {
        var rawTags = document.getElementById('tags-source').innerText;
        var tags = rawTags.trim().split(' ').filter(tag => tag.length > 0);
        tags.forEach(function(tag) {
            var pill = document.createElement('span');
            pill.className = 'tag-pill';
            pill.textContent = tag;
            container.appendChild(pill);
        });
    } catch (e_tags) {}
} catch (e) {}
</script>
"#;

const ANKI_BACK_TEMPLATE: &str = r#"
<div id="tags-container"></div>
<div id="tags-source" style="display: none;">{{Tags}}</div>
<div id="timestamp-source" style="display: none;">{{SequenceMarker}}</div>
<span class='media'>{{Audio}}</span>
<div class="expression">{{Expression}}</div>
<hr>
<br>
<div class='reading'>{{Reading}}</div>
<div class='meaning'>{{Meaning}}</div>
<br>
<div class='media'>{{Snapshot}}</div>
<span class='media'>{{Video}}</span>
<br />
<script>
try {
    var container = document.getElementById('tags-container');
    container.innerHTML = '';
    try {
        var rawString = document.getElementById('timestamp-source').innerText;
        if (rawString && rawString.includes('_') && rawString.includes('.')) {
            var fullTimestamp = rawString.split('_').pop();
            var parts = fullTimestamp.split('.');
            var formattedTimestamp = parts.slice(0, 3).join(':');
            var ts_pill = document.createElement('span');
            ts_pill.className = 'tag-pill';
            ts_pill.textContent = formattedTimestamp;
            container.appendChild(ts_pill);
        }
    } catch (e_ts) {}
    try {
        var rawTags = document.getElementById('tags-source').innerText;
        var tags = rawTags.trim().split(' ').filter(tag => tag.length > 0);
        tags.forEach(function(tag) {
            var pill = document.createElement('span');
            pill.className = 'tag-pill';
            pill.textContent = tag;
            container.appendChild(pill);
        });
    } catch (e_tags) {}
} catch (e) {}
</script>
"#;

const ANKI_CARD_STYLING: &str = r#"
#tags-container {
  text-align: left;
  margin-bottom: 8px;
  min-height: 20px;
}
.tag-pill {
  display: inline-block;
  font-size: 11px;
  font-family: arial, sans-serif;
  font-weight: 600;
  color: #333;
  background-color: #f0f0f0;
  padding: 4px 8px;
  border-radius: 8px;
  margin-right: 4px;
  margin-bottom: 4px;
  border: 1px solid #ddd;
  box-shadow: 0 1px 1px rgba(0,0,0,0.05);
}
.card video,
.card iframe {
  width: 600px;
  height: 400px;
  max-width: 100%;
  display: block;
  margin: 10px auto;
  border: 1px solid #eee;
}
.card {
  font-family: arial;
  font-size: 20px;
  text-align: center;
  color: black;
  background-color: white;
}
hr.solid {
  border-top: 3px solid #bbb;
}
.expression {
  font-size: 36px;
}
.reading {
  font-family: arial;
  font-size: 36px;
  color: #AA0000;
}
.meaning {
  font-family: arial;
  font-size: 36px;
}
.sequence_marker {
  font-size: 9px;
}
.media {
  font-size: 8px;
  color: #000000;
}
"#;

// ─── Tauri Commands ──────────────────────────────────────────────────────────

/// Load a subtitle file and return info
#[tauri::command]
pub async fn flashcard_load_subs(path: String) -> Result<SubFileInfo, String> {
    let (entries, format) = parse_subtitle_file(&path).map_err(|e| e.to_string())?;

    let mut actors: Vec<String> = entries
        .iter()
        .filter_map(|e| e.actor.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    actors.sort();

    let first_text = entries.first().map(|e| e.text.clone()).unwrap_or_default();
    let last_text = entries.last().map(|e| e.text.clone()).unwrap_or_default();
    let duration_ms = entries.last().map(|e| e.end_ms).unwrap_or(0);

    Ok(SubFileInfo {
        path,
        format: format.to_string(),
        count: entries.len(),
        first_text,
        last_text,
        actors,
        duration_ms,
    })
}

/// Generate preview data: parse, match, filter, and return all lines
#[tauri::command]
pub async fn flashcard_preview(config: FlashcardConfig) -> Result<Vec<PreviewLine>, String> {
    let (mut subs1, _) = parse_subtitle_file(&config.target_subs_path).map_err(|e| e.to_string())?;

    let subs2 = if let Some(ref path) = config.native_subs_path {
        let (entries, _) = parse_subtitle_file(path).map_err(|e| e.to_string())?;
        Some(entries)
    } else {
        None
    };

    // Apply time shifts to raw subs
    for s in subs1.iter_mut() {
        s.start_ms += config.time_shift_target_ms;
        s.end_ms += config.time_shift_target_ms;
    }

    // Match dual subtitles
    let mut matched = if let Some(ref s2) = subs2 {
        let mut s2_shifted = s2.clone();
        for s in s2_shifted.iter_mut() {
            s.start_ms += config.time_shift_native_ms;
            s.end_ms += config.time_shift_native_ms;
        }
        match_subtitles(&subs1, &s2_shifted)
    } else {
        subs1
            .iter()
            .enumerate()
            .map(|(i, s)| MatchedLine {
                index: i,
                subs1: s.clone(),
                subs2: None,
                active: true,
                leading_context: Vec::new(),
                trailing_context: Vec::new(),
            })
            .collect()
    };

    // Apply span filter
    apply_span(&mut matched, config.span_start_ms, config.span_end_ms);

    // Apply filters
    apply_filters(&mut matched, &config.filters);

    // Sentence combining
    if config.combine_sentences {
        combine_sentences(&mut matched, &config.continuation_chars);
    }

    // Context lines
    compute_context(&mut matched, &config.context);

    // Convert to preview lines
    let preview: Vec<PreviewLine> = matched
        .iter()
        .map(|m| PreviewLine {
            index: m.index,
            subs1_text: m.subs1.text.clone(),
            subs2_text: m.subs2.as_ref().map(|s| s.text.clone()),
            start_ms: m.subs1.start_ms,
            end_ms: m.subs1.end_ms,
            duration_ms: m.subs1.end_ms - m.subs1.start_ms,
            active: m.active,
            actor: m.subs1.actor.clone(),
            leading_context: m.leading_context.clone(),
            trailing_context: m.trailing_context.clone(),
        })
        .collect();

    Ok(preview)
}

/// Main generation command - processes everything with parallel FFmpeg calls
#[tauri::command]
pub async fn flashcard_generate(
    app: AppHandle,
    state: State<'_, AppFlashcardState>,
    config: FlashcardConfig,
) -> Result<FlashcardResult, String> {
    // Check if already processing
    {
        let mut fc_state = state.lock().map_err(|e| e.to_string())?;
        if fc_state.is_processing {
            return Err("Already processing flashcards".to_string());
        }
        fc_state.is_processing = true;
        fc_state.cancellation_token = Some(CancellationToken::new());
    }

    let cancel_token = {
        let fc_state = state.lock().map_err(|e| e.to_string())?;
        fc_state.cancellation_token.clone().unwrap()
    };

    let result = perform_generation(app.clone(), config, cancel_token).await;

    // Reset state
    {
        if let Ok(mut fc_state) = state.lock() {
            fc_state.is_processing = false;
            fc_state.cancellation_token = None;
        }
    }

    result
}

async fn perform_generation(
    app: AppHandle,
    config: FlashcardConfig,
    cancel_token: CancellationToken,
) -> Result<FlashcardResult, String> {
    // --- Stage 1: Parse subtitles ---
    emit_progress(&app, "parsing", "flashcards.progress.parsing", 0, 100, 0.0, HashMap::new());

    let (mut subs1, _) = parse_subtitle_file(&config.target_subs_path).map_err(|e| e.to_string())?;

    let subs2 = if let Some(ref path) = config.native_subs_path {
        let (entries, _) = parse_subtitle_file(path).map_err(|e| e.to_string())?;
        Some(entries)
    } else {
        None
    };

    if cancel_token.is_cancelled() {
        return Ok(FlashcardResult {
            success: false,
            message: "Cancelled".to_string(),
            cards_generated: 0,
            audio_clips: 0,
            snapshots: 0,
            video_clips: 0,
            tsv_path: None,
            apkg_path: None,
        });
    }

    // Apply time shifts
    for s in subs1.iter_mut() {
        s.start_ms += config.time_shift_target_ms;
        s.end_ms += config.time_shift_target_ms;
    }

    // --- Stage 2: Match subtitles ---
    emit_progress(&app, "matching", "flashcards.progress.matching", 5, 100, 5.0, HashMap::new());

    let mut matched = if let Some(ref s2) = subs2 {
        let mut s2_shifted = s2.clone();
        for s in s2_shifted.iter_mut() {
            s.start_ms += config.time_shift_native_ms;
            s.end_ms += config.time_shift_native_ms;
        }
        match_subtitles(&subs1, &s2_shifted)
    } else {
        subs1
            .iter()
            .enumerate()
            .map(|(i, s)| MatchedLine {
                index: i,
                subs1: s.clone(),
                subs2: None,
                active: true,
                leading_context: Vec::new(),
                trailing_context: Vec::new(),
            })
            .collect()
    };

    // --- Stage 3: Filter ---
    emit_progress(&app, "filtering", "flashcards.progress.filtering", 10, 100, 10.0, HashMap::new());

    apply_span(&mut matched, config.span_start_ms, config.span_end_ms);
    apply_filters(&mut matched, &config.filters);

    if config.combine_sentences {
        combine_sentences(&mut matched, &config.continuation_chars);
    }

    compute_context(&mut matched, &config.context);

    let active_count = matched.iter().filter(|m| m.active).count();
    let total_active = active_count;

    if active_count == 0 {
        return Ok(FlashcardResult {
            success: false,
            message: "No active subtitle lines after filtering".to_string(),
            cards_generated: 0,
            audio_clips: 0,
            snapshots: 0,
            video_clips: 0,
            tsv_path: None,
            apkg_path: None,
        });
    }

    // Create output directories
    let output_dir = PathBuf::from(&config.output_dir);
    let export_format = config.export_format.as_deref().unwrap_or("tsv");

    // For APKG: use a temp directory for media (will be cleaned up after packaging)
    // For TSV: use the standard collection.media in output_dir
    let apkg_temp_dir = if export_format == "apkg" {
        Some(tempfile::tempdir().map_err(|e| format!("Cannot create temp dir for media: {}", e))?)
    } else {
        None
    };

    let media_dir = if let Some(ref tmp) = apkg_temp_dir {
        tmp.path().join("collection.media")
    } else {
        output_dir.join("collection.media")
    };
    // Clean existing media directory to prevent stale files from prior runs
    if media_dir.exists() {
        let _ = std::fs::remove_dir_all(&media_dir);
    }
    std::fs::create_dir_all(&media_dir).map_err(|e| format!("Cannot create output dir: {}", e))?;

    // Determine media source
    let media_source = config.audio_path.as_deref()
        .or(config.video_path.as_deref());

    let video_source = config.video_path.as_deref();

    // --- Stage 4: Generate media (parallelized) ---
    let mut audio_count = 0usize;
    let mut snapshot_count = 0usize;
    let mut video_count = 0usize;

    let active_lines: Vec<(usize, &MatchedLine)> = matched
        .iter()
        .filter(|m| m.active)
        .enumerate()
        .collect();

    let needs_audio = config.generate_audio && media_source.is_some();
    let needs_snapshots = config.generate_snapshots && video_source.is_some();
    let needs_video = config.generate_video_clips && video_source.is_some();

    // Check ffmpeg availability
    if needs_audio || needs_snapshots || needs_video {
        if !check_ffmpeg().await.unwrap_or(false) {
            return Err("ffmpeg not found. Install ffmpeg to extract media.".to_string());
        }
    }

    let deck_sanitized = sanitize_filename(&config.deck_name);
    let ep = config.episode_number;

    // Pre-calculate media source strings (avoid allocating per-line)
    let media_source_str = media_source.map(|s| s.to_string());
    let video_source_str = video_source.map(|s| s.to_string());
    let video_codec = config.video_codec.clone();
    let h264_preset = config.h264_preset.clone();

    // Use configured CPU cores, or default to 3/4 of available cores
    let num_cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    let batch_size = if let Some(user_cores) = config.cpu_cores {
        // Ensure user value is within safe bounds: min 2, max num_cores - 1 (leave at least 1 free)
        user_cores.max(2).min(num_cores.saturating_sub(1).max(2))
    } else {
        (num_cores / 4 * 3).max(2)
    };
    let total_media_ops = active_lines.len()
        * (needs_audio as usize + needs_snapshots as usize + needs_video as usize);
    let mut completed_ops = 0usize;

    for chunk in active_lines.chunks(batch_size) {
        if cancel_token.is_cancelled() {
            return Ok(FlashcardResult {
                success: false,
                message: "Cancelled by user".to_string(),
                cards_generated: 0,
                audio_clips: audio_count,
                snapshots: snapshot_count,
                video_clips: video_count,
                tsv_path: None,
                apkg_path: None,
            });
        }

        let mut handles = Vec::new();

        for &(seq, line) in chunk {
            let seq_num = seq + 1;
            let start_ms = line.subs1.start_ms;
            let end_ms = line.subs1.end_ms;
            let line_seq = seq_num; // capture for error reporting

            // Audio extraction
            if needs_audio {
                let source = media_source_str.clone().unwrap();
                let output_path = media_dir.join(format!(
                    "{}_{:03}_{:04}.mp3",
                    deck_sanitized, ep, seq_num
                ));
                let bitrate = config.audio_bitrate;
                let pad_s = config.audio_pad_start_ms;
                let pad_e = config.audio_pad_end_ms;
                let normalize = config.normalize_audio;

                handles.push(tokio::spawn(async move {
                    let result = extract_audio_clip(
                        &source, &output_path, start_ms, end_ms, pad_s, pad_e, bitrate,
                    )
                    .await;
                    if result.is_ok() && normalize {
                        let _ = normalize_audio(&output_path).await;
                    }
                    ("audio", result, line_seq)
                }));
            }

            // Snapshot extraction
            if needs_snapshots {
                let source = video_source_str.clone().unwrap();
                let output_path = media_dir.join(format!(
                    "{}_{:03}_{:04}.jpg",
                    deck_sanitized, ep, seq_num
                ));
                let w = config.snapshot_width;
                let h = config.snapshot_height;
                let crop = config.crop_bottom;

                handles.push(tokio::spawn(async move {
                    let result = extract_snapshot(&source, &output_path, start_ms, end_ms, w, h, crop).await;
                    ("snapshot", result, line_seq)
                }));
            }

            // Video clip extraction
            if needs_video {
                let source = video_source_str.clone().unwrap();
                let ext = if video_codec == "h264" { "mp4" } else { "avi" };
                let output_path = media_dir.join(format!(
                    "{}_{:03}_{:04}.{}",
                    deck_sanitized, ep, seq_num, ext
                ));
                let codec = video_codec.clone();
                let preset = h264_preset.clone();
                let vbr = config.video_bitrate;
                let abr = config.video_audio_bitrate;
                let pad_s = config.video_pad_start_ms;
                let pad_e = config.video_pad_end_ms;

                handles.push(tokio::spawn(async move {
                    let result = extract_video_clip(
                        &source, &output_path, start_ms, end_ms, pad_s, pad_e, &codec, &preset,
                        vbr, abr,
                    )
                    .await;
                    ("video", result, line_seq)
                }));
            }
        }

        // Await all handles in this batch
        for handle in handles {
            if let Ok((kind, result, seq_num)) = handle.await {
                completed_ops += 1;
                let pct = 15.0 + (completed_ops as f64 / total_media_ops.max(1) as f64) * 75.0;

                match kind {
                    "audio" => {
                        if let Err(ref e) = result {
                            eprintln!("Audio extraction failed for line {}: {}", seq_num, e);
                            emit_progress(
                                &app,
                                "media",
                                "flashcards.progress.audioFailed",
                                completed_ops,
                                total_media_ops,
                                pct,
                                HashMap::from([("line".to_string(), seq_num.to_string()), ("error".to_string(), e.to_string())]),
                            );
                        } else {
                            audio_count += 1;
                        }
                    }
                    "snapshot" => {
                        if let Err(ref e) = result {
                            eprintln!("Snapshot extraction failed for line {}: {}", seq_num, e);
                            emit_progress(
                                &app,
                                "media",
                                "flashcards.progress.snapshotFailed",
                                completed_ops,
                                total_media_ops,
                                pct,
                                HashMap::from([("line".to_string(), seq_num.to_string()), ("error".to_string(), e.to_string())]),
                            );
                        } else {
                            snapshot_count += 1;
                        }
                    }
                    "video" => {
                        if let Err(ref e) = result {
                            eprintln!("Video clip extraction failed for line {}: {}", seq_num, e);
                            emit_progress(
                                &app,
                                "media",
                                "flashcards.progress.videoFailed",
                                completed_ops,
                                total_media_ops,
                                pct,
                                HashMap::from([("line".to_string(), seq_num.to_string()), ("error".to_string(), e.to_string())]),
                            );
                        } else {
                            video_count += 1;
                        }
                    }
                    _ => {}
                }

                if result.is_ok() {
                    emit_progress(
                        &app,
                        "media",
                        "flashcards.progress.extractingMedia",
                        completed_ops,
                        total_media_ops,
                        pct,
                        HashMap::from([("current".to_string(), completed_ops.to_string()), ("total".to_string(), total_media_ops.to_string())]),
                    );
                }
            }
        }
    }

    // Report media extraction failures
    if needs_audio && audio_count < total_active {
        let failed = total_active - audio_count;
        emit_progress(&app, "media", "flashcards.progress.audioExtractionsFailed", completed_ops, total_media_ops, 90.0, HashMap::from([("count".to_string(), failed.to_string())]));
    }
    if needs_snapshots && snapshot_count < total_active {
        let failed = total_active - snapshot_count;
        emit_progress(&app, "media", "flashcards.progress.snapshotExtractionsFailed", completed_ops, total_media_ops, 90.0, HashMap::from([("count".to_string(), failed.to_string())]));
    }
    if needs_video && video_count < total_active {
        let failed = total_active - video_count;
        emit_progress(&app, "media", "flashcards.progress.videoExtractionsFailed", completed_ops, total_media_ops, 90.0, HashMap::from([("count".to_string(), failed.to_string())]));
    }

    // --- Stage 5: Generate export file ---
    let mut tsv_path_result: Option<String> = None;
    let mut apkg_path_result: Option<String> = None;

    if export_format == "apkg" {
        emit_progress(&app, "tsv", "flashcards.progress.generatingApkg", 90, 100, 90.0, HashMap::new());

        let apkg_filename = format!("{}.apkg", sanitize_filename(&config.deck_name));
        let apkg_path = output_dir.join(&apkg_filename);

        generate_apkg(&matched, &config, &media_dir, &apkg_path)?;

        apkg_path_result = Some(apkg_path.to_string_lossy().to_string());
    } else {
        emit_progress(&app, "tsv", "flashcards.progress.generatingTsv", 90, 100, 90.0, HashMap::new());

        let tsv_content = generate_tsv(
            &matched,
            &config,
            media_dir.to_str().unwrap_or(""),
            media_dir.to_str().unwrap_or(""),
            media_dir.to_str().unwrap_or(""),
        );

        let tsv_filename = format!("{}.tsv", sanitize_filename(&config.deck_name));
        let tsv_path = output_dir.join(&tsv_filename);
        std::fs::write(&tsv_path, tsv_content.as_bytes())
            .map_err(|e| format!("Cannot write TSV: {}", e))?;

        tsv_path_result = Some(tsv_path.to_string_lossy().to_string());
    }

    // --- Done ---
    emit_progress(&app, "done", "flashcards.progress.complete", 100, 100, 100.0, HashMap::new());

    Ok(FlashcardResult {
        success: true,
        message: format!(
            "Generated {} cards ({} audio, {} snapshots, {} video clips)",
            total_active, audio_count, snapshot_count, video_count
        ),
        cards_generated: total_active,
        audio_clips: audio_count,
        snapshots: snapshot_count,
        video_clips: video_count,
        tsv_path: tsv_path_result,
        apkg_path: apkg_path_result,
    })
}

/// Cancel flashcard generation
#[tauri::command]
pub async fn flashcard_cancel(state: State<'_, AppFlashcardState>) -> Result<bool, String> {
    let mut fc_state = state.lock().map_err(|e| e.to_string())?;
    if let Some(ref token) = fc_state.cancellation_token {
        token.cancel();
    }
    fc_state.is_processing = false;
    fc_state.cancellation_token = None;
    Ok(true)
}

/// Check if ffmpeg is available
#[tauri::command]
pub async fn flashcard_check_deps() -> Result<bool, String> {
    check_ffmpeg().await.map_err(|e| e.to_string())
}

/// Check if a directory exists
#[tauri::command]
pub async fn flashcard_check_dir_exists(path: String) -> Result<bool, String> {
    Ok(std::path::Path::new(&path).is_dir())
}

/// Get the number of available CPU cores
#[tauri::command]
pub async fn flashcard_get_cpu_count() -> Result<usize, String> {
    Ok(std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4))
}

fn emit_progress(app: &AppHandle, stage: &str, message: &str, current: usize, total: usize, pct: f64, params: HashMap<String, String>) {
    let _ = app.emit(
        "flashcard-progress",
        FlashcardProgressEvent {
            stage: stage.to_string(),
            message: message.to_string(),
            current,
            total,
            percentage: pct,
            params,
        },
    );
}
