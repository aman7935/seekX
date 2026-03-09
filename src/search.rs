#[derive(Clone, Debug)]
pub struct MatchScore {
    pub score: i64,
    pub start_idx: usize,
}

pub fn score(query: &str, haystack: &str) -> Option<MatchScore> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return Some(MatchScore {
            score: 1,
            start_idx: 0,
        });
    }

    let h = haystack.to_lowercase();

    if let Some(idx) = h.find(&q) {
        let positional_bonus = 1_000_i64.saturating_sub(idx as i64 * 4);
        let length_bonus = 400_i64.saturating_sub((h.len() as i64 - q.len() as i64).max(0));
        return Some(MatchScore {
            score: 10_000 + positional_bonus + length_bonus,
            start_idx: idx,
        });
    }

    let mut q_chars = q.chars();
    let mut current = q_chars.next()?;
    let mut first_match_idx = None;
    let mut matched_positions = Vec::new();

    for (idx, c) in h.chars().enumerate() {
        if c == current {
            first_match_idx.get_or_insert(idx);
            matched_positions.push(idx);
            if let Some(next) = q_chars.next() {
                current = next;
            } else {
                let span = idx.saturating_sub(*first_match_idx.get_or_insert(idx));
                let compact_bonus = 300_i64.saturating_sub(span as i64 * 3);
                let start_bonus = 300_i64.saturating_sub(first_match_idx.unwrap_or(0) as i64 * 2);
                let chain_bonus = (matched_positions.len() as i64) * 60;
                return Some(MatchScore {
                    score: 2_000 + compact_bonus + start_bonus + chain_bonus,
                    start_idx: first_match_idx.unwrap_or(0),
                });
            }
        }
    }

    None
}
