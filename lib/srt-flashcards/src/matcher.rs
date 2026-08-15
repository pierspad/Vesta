use super::types::*;

pub(crate) fn get_overlap(a_start: i64, a_end: i64, b_start: i64, b_end: i64) -> i64 {
    let overlap_start = a_start.max(b_start);
    let overlap_end = a_end.min(b_end);
    (overlap_end - overlap_start).max(0)
}

pub(crate) fn match_subtitles(subs1: &[SubEntry], subs2: &[SubEntry]) -> Vec<MatchedLine> {
    let mut matched: Vec<MatchedLine> = Vec::with_capacity(subs1.len());

    for (i, s1) in subs1.iter().enumerate() {
        let mut best_match: Option<&SubEntry> = None;
        let mut best_overlap: i64 = 0;

        let search_start = subs2.partition_point(|s2| s2.end_ms < s1.start_ms.saturating_sub(5000));

        for s2 in &subs2[search_start..] {
            let overlap = get_overlap(s1.start_ms, s1.end_ms, s2.start_ms, s2.end_ms);
            if overlap > best_overlap {
                best_overlap = overlap;
                best_match = Some(s2);
            }

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

    combine_consecutive_repeats(&mut matched);

    matched
}

pub(crate) fn combine_consecutive_repeats(matched: &mut Vec<MatchedLine>) {
    if matched.is_empty() {
        return;
    }

    let mut write_idx = 0;
    for read_idx in 1..matched.len() {
        let same_s2 = match (&matched[write_idx].subs2, &matched[read_idx].subs2) {
            (Some(a), Some(b)) => a.id == b.id,
            _ => false,
        };

        if same_s2 {
            let next_end = matched[read_idx].subs1.end_ms;
            let next_text = std::mem::take(&mut matched[read_idx].subs1.text);
            matched[write_idx].subs1.text.reserve(1 + next_text.len());
            matched[write_idx].subs1.text.push(' ');
            matched[write_idx].subs1.text.push_str(&next_text);
            matched[write_idx].subs1.end_ms = next_end;
        } else {
            write_idx += 1;
            if write_idx != read_idx {
                matched.swap(write_idx, read_idx);
            }
        }
    }

    matched.truncate(write_idx + 1);

    for (j, m) in matched.iter_mut().enumerate() {
        m.index = j;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sub(id: u32, start_ms: i64, end_ms: i64, text: &str) -> SubEntry {
        SubEntry {
            id,
            start_ms,
            end_ms,
            text: text.to_string(),
            actor: None,
            style: None,
            active: true,
        }
    }

    #[test]
    fn test_get_overlap() {
        // Disjoint
        assert_eq!(get_overlap(1000, 2000, 3000, 4000), 0);
        assert_eq!(get_overlap(3000, 4000, 1000, 2000), 0);
        // Partial overlap
        assert_eq!(get_overlap(1000, 3000, 2000, 4000), 1000);
        assert_eq!(get_overlap(2000, 4000, 1000, 3000), 1000);
        // Fully contained
        assert_eq!(get_overlap(1000, 4000, 2000, 3000), 1000);
        assert_eq!(get_overlap(2000, 3000, 1000, 4000), 1000);
        // Touching boundaries (0ms overlap)
        assert_eq!(get_overlap(1000, 2000, 2000, 3000), 0);
    }

    #[test]
    fn test_match_subtitles_basic() {
        let s1 = vec![sub(1, 1000, 3000, "Hello"), sub(2, 4000, 6000, "World")];
        let s2 = vec![sub(10, 1050, 2950, "Ciao"), sub(20, 3950, 6050, "Mondo")];

        let matched = match_subtitles(&s1, &s2);
        assert_eq!(matched.len(), 2);
        assert_eq!(matched[0].subs1.text, "Hello");
        assert_eq!(matched[0].subs2.as_ref().unwrap().text, "Ciao");
        assert_eq!(matched[1].subs1.text, "World");
        assert_eq!(matched[1].subs2.as_ref().unwrap().text, "Mondo");
    }

    #[test]
    fn test_combine_consecutive_repeats() {
        // Two consecutive s1 matching the same s2 (id 10)
        let s1 = vec![
            sub(1, 1000, 2000, "How"),
            sub(2, 2100, 3000, "are you?"),
            sub(3, 4000, 5000, "I am fine"),
        ];
        let s2 = vec![
            sub(10, 1000, 3000, "Come stai?"),
            sub(20, 4000, 5000, "Sto bene"),
        ];

        let matched = match_subtitles(&s1, &s2);
        assert_eq!(matched.len(), 2);
        assert_eq!(matched[0].subs1.text, "How are you?");
        assert_eq!(matched[0].subs1.start_ms, 1000);
        assert_eq!(matched[0].subs1.end_ms, 3000);
        assert_eq!(matched[0].subs2.as_ref().unwrap().text, "Come stai?");

        assert_eq!(matched[1].subs1.text, "I am fine");
        assert_eq!(matched[1].subs2.as_ref().unwrap().text, "Sto bene");
    }
}
