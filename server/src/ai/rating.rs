//! Structured output schema and validation for AI productive rating
//! Reference: docs/07_AI_SCORING_SPEC.md §1, §3

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiRatingOutput {
    pub usable: bool,
    #[serde(default)]
    pub unusable_reason: Option<String>,
    #[serde(default)]
    pub at_lower: HashMap<String, u32>,
    #[serde(default)]
    pub at_upper: HashMap<String, u32>,
    pub rationale: String,
    #[serde(default)]
    pub evidence_quotes: Vec<String>,
    #[serde(default)]
    pub flags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsabilityGateResult {
    pub usable: bool,
    pub unusable_reason: Option<String>,
    pub flags: Vec<String>,
}

impl UsabilityGateResult {
    pub fn pass() -> Self {
        Self {
            usable: true,
            unusable_reason: None,
            flags: Vec::new(),
        }
    }

    pub fn reject(reason: &str, flag: &str) -> Self {
        Self {
            usable: false,
            unusable_reason: Some(reason.to_string()),
            flags: vec![flag.to_string()],
        }
    }
}

/// Usability gate for speaking audio recording per 07 §1:
/// duration < 3s OR clipping > 1% OR speech_ratio < 25% -> usable: false
pub fn check_speaking_usability(
    duration_sec: f64,
    clipping_ratio: f64,
    speech_ratio: f64,
) -> UsabilityGateResult {
    if duration_sec < 3.0 {
        return UsabilityGateResult::reject("technical_too_short", "technical_audio");
    }
    if clipping_ratio > 0.01 {
        return UsabilityGateResult::reject("technical_clipped", "technical_audio");
    }
    if speech_ratio < 0.25 {
        return UsabilityGateResult::reject("empty_recording", "too_short");
    }
    UsabilityGateResult::pass()
}

/// Usability gate for writing text per 07 §1:
/// < 5 words OR text == prompt substring (copied) -> usable: false
pub fn check_writing_usability(text: &str, prompt: &str) -> UsabilityGateResult {
    let word_count = text.split_whitespace().count();
    if word_count < 5 {
        return UsabilityGateResult::reject("too_short", "too_short");
    }
    let trimmed = text.trim().to_lowercase();
    let prompt_lower = prompt.to_lowercase();
    if prompt_lower.contains(&trimmed) && trimmed.len() > 15 {
        return UsabilityGateResult::reject("copied_prompt", "copied_prompt");
    }
    UsabilityGateResult::pass()
}

/// Validates rating output against rubric traits and invariants per 07 §1
pub fn validate_and_sanitize_rating(
    rating: &mut GeminiRatingOutput,
    expected_traits: &[&str],
) -> Vec<String> {
    let mut detected_flags = Vec::new();

    if !rating.usable {
        return detected_flags;
    }

    for &t in expected_traits {
        let l = *rating.at_lower.entry(t.to_string()).or_insert(3);
        let u = *rating.at_upper.entry(t.to_string()).or_insert(3);

        // Invariant: traits are integers 0-5
        if l > 5 {
            rating.at_lower.insert(t.to_string(), 5);
        }
        if u > 5 {
            rating.at_upper.insert(t.to_string(), 5);
        }

        // Invariant: atUpper must never exceed atLower for any trait
        // If atUpper > atLower + 1, flag rater_inconsistent
        if u > l + 1 {
            detected_flags.push("rater_inconsistent".to_string());
        }
        // Clamp atUpper so it never strictly exceeds atLower in shipped profile
        if u > l {
            rating.at_upper.insert(t.to_string(), l);
        }
    }

    // Merge detected flags without duplicate entries
    for f in detected_flags.iter() {
        if !rating.flags.contains(f) {
            rating.flags.push(f.clone());
        }
    }

    detected_flags
}

/// Checks second opinion rating from fast model against primary rater per 07 §1:
/// If |mean(atLower) - mean(atLower')| >= 1.0 -> flag rater_disagreement
pub fn check_second_opinion_agreement(
    primary: &GeminiRatingOutput,
    second: &GeminiRatingOutput,
) -> Option<String> {
    if primary.at_lower.is_empty() || second.at_lower.is_empty() {
        return None;
    }

    let p_sum: f64 = primary.at_lower.values().map(|&v| v as f64).sum();
    let p_mean = p_sum / primary.at_lower.len() as f64;

    let s_sum: f64 = second.at_lower.values().map(|&v| v as f64).sum();
    let s_mean = s_sum / second.at_lower.len() as f64;

    if (p_mean - s_mean).abs() >= 1.0 {
        Some("rater_disagreement".to_string())
    } else {
        None
    }
}
