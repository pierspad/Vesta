use super::media::{MediaKind, media_filename, ms_to_ffmpeg_ts, video_clip_extension};
use super::types::*;

use std::fmt::Write;

pub(crate) fn render_text_with_context<'a, F>(
    main_text: &str,
    line: &MatchedLine,
    all_lines: &'a [MatchedLine],
    get_text: F,
    span_attr: &str,
    replace_tabs: bool,
) -> String
where
    F: Fn(&'a MatchedLine) -> Option<&'a str>,
{
    let mut text = String::with_capacity(main_text.len() + 128);

    for &ctx_idx in &line.leading_context {
        if let Some(ctx_line) = all_lines.get(ctx_idx)
            && let Some(ctx_text) = get_text(ctx_line)
        {
            let _ = write!(text, "<span {span_attr}>{ctx_text}</span><br>");
        }
    }

    text.push_str(main_text);

    for &ctx_idx in &line.trailing_context {
        if let Some(ctx_line) = all_lines.get(ctx_idx)
            && let Some(ctx_text) = get_text(ctx_line)
        {
            let _ = write!(text, "<br><span {span_attr}>{ctx_text}</span>");
        }
    }

    let mut result = String::with_capacity(text.len() + 16);
    for ch in text.chars() {
        match ch {
            '\n' => result.push_str("<br>"),
            '\t' if replace_tabs => result.push(' '),
            _ => result.push(ch),
        }
    }
    result
}

#[inline]
fn append_field(tsv: &mut String, first: &mut bool, val: &str) {
    if !*first {
        tsv.push('\t');
    }
    tsv.push_str(val);
    *first = false;
}

pub(crate) fn generate_tsv(
    lines: &[MatchedLine],
    config: &FlashcardConfig,
    _audio_dir: &str,
    _snapshot_dir: &str,
    _video_dir: &str,
) -> String {
    let active_count = lines.iter().filter(|l| l.active).count();
    let mut tsv = String::with_capacity(active_count * 200);

    let sanitized_deck = sanitize_filename(&config.deck_name);
    let ep = config.episode_number;
    let video_ext = video_clip_extension(&config.video_codec);
    let of = &config.output_fields;

    for (seq, line) in lines.iter().filter(|l| l.active).enumerate() {
        let seq_num = seq + 1;
        let start_ts = ms_to_ffmpeg_ts(line.subs1.start_ms);
        let mut first = true;

        if of.include_subs1 {
            let rendered = render_text_with_context(
                &line.subs1.text,
                line,
                lines,
                |m| Some(m.subs1.text.as_str()),
                "style=\"color:gray\"",
                true,
            );
            append_field(&mut tsv, &mut first, &rendered);
        }

        if of.include_subs2 {
            let rendered = match &line.subs2 {
                Some(s2) => render_text_with_context(
                    &s2.text,
                    line,
                    lines,
                    |m| m.subs2.as_ref().map(|s| s.text.as_str()),
                    "style=\"color:gray\"",
                    true,
                ),
                None => String::new(),
            };
            append_field(&mut tsv, &mut first, &rendered);
        }

        if of.include_audio {
            if config.generate_audio {
                let filename = media_filename(
                    MediaKind::Audio(config.audio_format),
                    &sanitized_deck,
                    ep,
                    seq_num,
                );
                if !first {
                    tsv.push('\t');
                }
                let _ = write!(tsv, "[sound:{filename}]");
                first = false;
            } else {
                append_field(&mut tsv, &mut first, "");
            }
        }

        if of.include_snapshot {
            if config.generate_snapshots {
                let filename = media_filename(
                    MediaKind::Snapshot(config.snapshot_format),
                    &sanitized_deck,
                    ep,
                    seq_num,
                );
                if !first {
                    tsv.push('\t');
                }
                let _ = write!(tsv, "<img src=\"{filename}\">");
                first = false;
            } else {
                append_field(&mut tsv, &mut first, "");
            }
        }

        if of.include_video {
            if config.generate_video_clips {
                let filename =
                    media_filename(MediaKind::Video(video_ext), &sanitized_deck, ep, seq_num);
                if !first {
                    tsv.push('\t');
                }
                let _ = write!(tsv, "[sound:{filename}]");
                first = false;
            } else {
                append_field(&mut tsv, &mut first, "");
            }
        }

        if of.include_tag {
            if !first {
                tsv.push('\t');
            }
            let _ = write!(tsv, "{}_{ep:03}", config.deck_name);
            first = false;
        }

        if of.include_sequence {
            if !first {
                tsv.push('\t');
            }
            let _ = write!(tsv, "{ep:03}_{seq_num:04}_{start_ts}");
            first = false;
        }

        if of.include_reading {
            append_field(&mut tsv, &mut first, "");
        }

        if of.include_notes {
            append_field(&mut tsv, &mut first, "");
        }

        tsv.push('\n');
    }

    tsv
}

pub(crate) fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}
