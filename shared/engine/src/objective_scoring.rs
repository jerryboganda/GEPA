use crate::models::{ObjectiveItem, OptionChoice, RestrictedKey};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScoringResult {
    pub item_id: String,
    pub correct: bool,
    pub omitted: bool,
    pub response_ms: u64,
}

pub fn score_response(
    item_id: &str,
    selected_option_id: Option<&str>,
    key: &RestrictedKey,
    response_ms: u64,
) -> ScoringResult {
    if let Some(opt_id) = selected_option_id {
        let correct = opt_id == key.key_option_id;
        ScoringResult {
            item_id: item_id.to_string(),
            correct,
            omitted: false,
            response_ms,
        }
    } else {
        ScoringResult {
            item_id: item_id.to_string(),
            correct: false,
            omitted: true,
            response_ms,
        }
    }
}

/// Simple deterministic hash to seed RNG from two strings
fn hash_strings(s1: &str, s2: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s1.bytes().chain(s2.bytes()) {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

/// Deterministic Fisher-Yates shuffle using xorshift64
pub fn shuffle_options(
    item: &ObjectiveItem,
    session_id: &str,
) -> (Vec<OptionChoice>, Vec<String>) {
    let mut rng_state = hash_strings(session_id, &item.item_id);
    if rng_state == 0 {
        rng_state = 0x853c49e6748fea9b;
    }

    let mut options = item.options.clone();
    let n = options.len();
    for i in (1..n).rev() {
        // xorshift64
        rng_state ^= rng_state << 13;
        rng_state ^= rng_state >> 7;
        rng_state ^= rng_state << 17;

        let j = (rng_state as usize) % (i + 1);
        options.swap(i, j);
    }

    let permutation = options.iter().map(|o| o.option_id.clone()).collect();
    (options, permutation)
}

/// Check for effort flags based on response sequence
pub fn check_effort_flags(
    responses: &[ScoringResult],
) -> Vec<String> {
    let mut flags = Vec::new();

    // 1. Omissions: >= 3 omissions in a module -> effort_omissions
    let omissions = responses.iter().filter(|r| r.omitted).count();
    if omissions >= 3 {
        flags.push("effort_omissions".to_string());
    }

    // 2. Rapid clicking: >= 5 consecutive responses < 1500 ms -> effort_rapid
    let mut rapid_streak = 0;
    let mut has_rapid = false;
    for r in responses {
        if r.response_ms < 1500 {
            rapid_streak += 1;
            if rapid_streak >= 5 {
                has_rapid = true;
                break;
            }
        } else {
            rapid_streak = 0;
        }
    }

    if has_rapid {
        flags.push("effort_rapid".to_string());
    }

    flags
}
