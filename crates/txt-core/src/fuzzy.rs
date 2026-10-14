use serde::{Deserialize, Serialize};

/// Result of a fuzzy match against a target string.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FuzzyMatch {
    pub score: i64,
    pub matched_indices: Vec<usize>,
}

/// High-performance fuzzy matcher reproducing Sublime Text's scoring heuristics.
pub struct FuzzyMatcher;

impl FuzzyMatcher {
    /// Evaluates `query` against `target`. Returns `Some(FuzzyMatch)` if matched, or `None`.
    pub fn match_str(query: &str, target: &str) -> Option<FuzzyMatch> {
        if query.is_empty() {
            return Some(FuzzyMatch {
                score: 0,
                matched_indices: Vec::new(),
            });
        }

        let query_chars: Vec<char> = query.chars().collect();
        let target_chars: Vec<char> = target.chars().collect();

        if query_chars.len() > target_chars.len() {
            return None;
        }

        // Fast check: verify all query chars exist in target in order (case-insensitive)
        let mut q_idx = 0;
        for &t_char in &target_chars {
            if t_char.to_ascii_lowercase() == query_chars[q_idx].to_ascii_lowercase() {
                q_idx += 1;
                if q_idx == query_chars.len() {
                    break;
                }
            }
        }
        if q_idx < query_chars.len() {
            return None;
        }

        // Full score computation with dynamic programming / recursive path search
        Self::compute_optimal_match(&query_chars, &target_chars)
    }

    fn compute_optimal_match(query: &[char], target: &[char]) -> Option<FuzzyMatch> {
        let q_len = query.len();
        let t_len = target.len();

        // Greedily find matched indices with boundary scoring
        let mut matched_indices = Vec::with_capacity(q_len);
        let mut score: i64 = 0;
        let mut t_idx = 0;
        let mut prev_matched_idx: Option<usize> = None;

        for (_qi, &q_char) in query.iter().enumerate() {
            let q_lower = q_char.to_ascii_lowercase();
            let mut found = false;

            // Search for best candidate starting from current t_idx
            let mut best_idx = None;
            let mut best_char_score = i64::MIN;

            for ti in t_idx..t_len {
                let t_char = target[ti];
                if t_char.to_ascii_lowercase() == q_lower {
                    let char_score = Self::score_char_match(
                        ti,
                        target,
                        prev_matched_idx,
                        q_char == t_char, // exact case match
                    );
                    if char_score > best_char_score {
                        best_char_score = char_score;
                        best_idx = Some(ti);
                        // If it's a consecutive match or word boundary, take it eagerly
                        if prev_matched_idx.map_or(false, |p| ti == p + 1) || Self::is_word_boundary(ti, target) {
                            break;
                        }
                    }
                }
            }

            if let Some(idx) = best_idx {
                score += best_char_score;
                prev_matched_idx = Some(idx);
                matched_indices.push(idx);
                t_idx = idx + 1;
                found = true;
            }

            if !found {
                return None;
            }
        }

        // Penalty for total target length to favor concise matches
        let length_penalty = (t_len.saturating_sub(q_len)) as i64 * 2;
        score = score.saturating_sub(length_penalty);

        // Exact match bonus
        if q_len == t_len {
            score += 100;
        }

        Some(FuzzyMatch {
            score,
            matched_indices,
        })
    }

    fn score_char_match(
        ti: usize,
        target: &[char],
        prev_matched: Option<usize>,
        is_exact_case: bool,
    ) -> i64 {
        let mut score: i64 = 10; // Base score for char match

        if is_exact_case {
            score += 15; // Case match bonus
        }

        // Start of string bonus
        if ti == 0 {
            score += 60;
            return score;
        }

        // Word boundary bonus (Sublime Text style)
        if Self::is_word_boundary(ti, target) {
            score += 45;
        }

        // CamelCase bonus: lowercase followed by uppercase
        if ti > 0 && target[ti - 1].is_lowercase() && target[ti].is_uppercase() {
            score += 35;
        }

        // Consecutive match bonus: huge bonus if directly following previous match
        if let Some(prev) = prev_matched {
            if ti == prev + 1 {
                score += 50;
            } else {
                // Gap penalty
                let gap = (ti - prev - 1) as i64;
                score -= gap.min(20);
            }
        }

        score
    }

    fn is_word_boundary(idx: usize, target: &[char]) -> bool {
        if idx == 0 {
            return true;
        }
        let prev = target[idx - 1];
        prev == '/' || prev == '\\' || prev == '_' || prev == '-' || prev == '.' || prev == ':' || prev == ' '
    }
}
