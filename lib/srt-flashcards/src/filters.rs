use std::collections::HashSet;

use super::types::*;

pub(crate) fn apply_filters(lines: &mut [MatchedLine], filters: &SubtitleFilters) {
    let include_set: Option<Vec<String>> = filters.include_words.as_ref().map(|w| {
        w.split(',')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect()
    });

    let exclude_set: Option<Vec<String>> = filters.exclude_words.as_ref().map(|w| {
        w.split(',')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect()
    });

    let mut seen_subs1: HashSet<&str> = HashSet::new();
    let mut seen_subs2: HashSet<&str> = HashSet::new();

    let actor_filter: Option<HashSet<String>> = filters.actor_filter.as_ref().map(|a| {
        a.split(',')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect()
    });

    for line in lines.iter_mut() {
        if !line.active {
            continue;
        }

        let duration = line.subs1.end_ms - line.subs1.start_ms;

        if include_set.is_some() || exclude_set.is_some() {
            let text_lower = line.subs1.text.to_lowercase();

            if let Some(ref words) = include_set
                && !words.iter().any(|w| text_lower.contains(w))
            {
                line.active = false;
                continue;
            }

            if let Some(ref words) = exclude_set
                && words.iter().any(|w| text_lower.contains(w))
            {
                line.active = false;
                continue;
            }
        }

        if filters.exclude_duplicates_subs1 {
            let normalized = line.subs1.text.trim();
            if seen_subs1.contains(normalized) {
                line.active = false;
                continue;
            }
            seen_subs1.insert(normalized);
        }

        if filters.exclude_duplicates_subs2
            && let Some(ref s2) = line.subs2
        {
            let normalized = s2.text.trim();
            if seen_subs2.contains(normalized) {
                line.active = false;
                continue;
            }
            seen_subs2.insert(normalized);
        }

        if filters.min_chars.is_some() || filters.max_chars.is_some() {
            let char_count = line.subs1.text.chars().count();
            if let Some(min) = filters.min_chars
                && char_count < min
            {
                line.active = false;
                continue;
            }
            if let Some(max) = filters.max_chars
                && char_count > max
            {
                line.active = false;
                continue;
            }
        }

        if let Some(min) = filters.min_duration_ms
            && duration < min
        {
            line.active = false;
            continue;
        }
        if let Some(max) = filters.max_duration_ms
            && duration > max
        {
            line.active = false;
            continue;
        }

        if filters.exclude_styled && line.subs1.text.starts_with('{') {
            line.active = false;
            continue;
        }

        if let Some(ref actors) = actor_filter {
            let Some(ref actor) = line.subs1.actor else {
                line.active = false;
                continue;
            };
            let actor_lower = actor.to_lowercase();
            if !actors.contains(&actor_lower) {
                line.active = false;
                continue;
            }
        }

        if filters.only_cjk {
            let has_cjk = line.subs1.text.chars().any(|c| {
                matches!(c,
                    '\u{4E00}'..='\u{9FFF}' |
                    '\u{3400}'..='\u{4DBF}' |
                    '\u{3040}'..='\u{309F}' |
                    '\u{30A0}'..='\u{30FF}'
                )
            });
            if !has_cjk {
                line.active = false;
                continue;
            }
        }

        if filters.remove_no_match && line.subs2.is_none() {
            line.active = false;
        }
    }
}

pub(crate) fn combine_sentences(lines: &mut Vec<MatchedLine>, continuation_chars: &str) {
    if continuation_chars.is_empty() || lines.is_empty() {
        return;
    }

    let cont_chars: Vec<char> = continuation_chars.chars().collect();
    let mut write_idx = 0;

    for read_idx in 1..lines.len() {
        let ends_with_cont = lines[write_idx]
            .subs1
            .text
            .trim_end()
            .chars()
            .last()
            .map(|c| cont_chars.contains(&c))
            .unwrap_or(false);

        if ends_with_cont && lines[write_idx].active && lines[read_idx].active {
            let next_end = lines[read_idx].subs1.end_ms;
            let next_text = std::mem::take(&mut lines[read_idx].subs1.text);

            lines[write_idx].subs1.text.reserve(1 + next_text.len());
            lines[write_idx].subs1.text.push(' ');
            lines[write_idx].subs1.text.push_str(&next_text);
            lines[write_idx].subs1.end_ms = next_end;

            let next_s2 = lines[read_idx].subs2.take();
            if let (Some(s2), Some(next_s2)) = (&mut lines[write_idx].subs2, next_s2) {
                s2.text.reserve(1 + next_s2.text.len());
                s2.text.push(' ');
                s2.text.push_str(&next_s2.text);
                s2.end_ms = next_s2.end_ms;
            }
        } else {
            write_idx += 1;
            if write_idx != read_idx {
                lines.swap(write_idx, read_idx);
            }
        }
    }

    lines.truncate(write_idx + 1);

    for (j, m) in lines.iter_mut().enumerate() {
        m.index = j;
    }
}

pub(crate) fn compute_context(lines: &mut [MatchedLine], ctx: &ContextConfig) {
    if ctx.leading == 0 && ctx.trailing == 0 {
        return;
    }

    let gap_ms = (ctx.max_gap_seconds * 1000.0) as i64;
    let len = lines.len();

    for i in 0..len {
        let mut leading = Vec::new();
        let mut trailing = Vec::new();

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

pub(crate) fn apply_span(
    lines: &mut [MatchedLine],
    span_start: Option<i64>,
    span_end: Option<i64>,
) {
    for line in lines.iter_mut() {
        if let Some(start) = span_start
            && line.subs1.end_ms < start
        {
            line.active = false;
        }
        if let Some(end) = span_end
            && line.subs1.start_ms > end
        {
            line.active = false;
        }
    }
}
