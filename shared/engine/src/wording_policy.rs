use regex::{Regex, RegexSet};
use std::sync::OnceLock;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum WordingViolation {
    #[error("Found forbidden phrase: '{0}' in text")]
    ForbiddenPhrase(String),
    #[error("Found forbidden pattern (plus/minus level): '{0}' in text")]
    ForbiddenLevelModifier(String),
    #[error("Percentage symbol forbidden inside confidence text: '{0}'")]
    PercentInConfidence(String),
    #[error("Confidence 'High' is forbidden before calibration")]
    HighConfidenceForbidden,
}

pub const FORBIDDEN_PHRASES: &[&str] = &[
    "cefr-aligned",
    "validated",
    "certified",
    "certificate",
    "your level is",
    "your cefr level",
    "official",
    "guarantee",
    "ielts band",
    "oet grade",
    "toefl score",
    "pte score",
    "predict",
    "high confidence",
    "reliability",
    "accuracy of",
];

static REGEX_SET: OnceLock<RegexSet> = OnceLock::new();
static LEVEL_RE: OnceLock<Regex> = OnceLock::new();

fn get_regex_set() -> &'static RegexSet {
    REGEX_SET.get_or_init(|| {
        let patterns: Vec<String> = FORBIDDEN_PHRASES
            .iter()
            .map(|&p| format!(r"(?i)\b{}\b", regex::escape(p)))
            .collect();
        RegexSet::new(patterns).expect("Failed to build forbidden regex set")
    })
}

fn get_level_re() -> &'static Regex {
    LEVEL_RE.get_or_init(|| {
        Regex::new(r"(?i)\b(Pre-A1|A1|A2|B1|B2|C1|C2)[+\-](?:[^\w]|$)").expect("Failed to build level regex")
    })
}

pub fn scan_candidate_copy(text: &str) -> Result<(), WordingViolation> {
    let mut clean_text = text.to_lowercase();

    // Spec 05 §3.6 specifies the exact mandatory disclaimer: "GEPA does not predict official exam scores."
    // We exempt this exact required phrase so the negative disclaimer can be displayed.
    clean_text = clean_text.replace("gepa does not predict official exam scores.", "");
    clean_text = clean_text.replace("gepa does not predict official exam scores", "");

    let set = get_regex_set();
    let matches: Vec<usize> = set.matches(&clean_text).into_iter().collect();
    if let Some(&first_idx) = matches.first() {
        return Err(WordingViolation::ForbiddenPhrase(
            FORBIDDEN_PHRASES[first_idx].to_string(),
        ));
    }

    // Check for +/- levels like B1+, B2-
    let re = get_level_re();
    if let Some(mat) = re.find(text) {
        return Err(WordingViolation::ForbiddenLevelModifier(mat.as_str().to_string()));
    }

    Ok(())
}

pub fn scan_confidence_text(text: &str) -> Result<(), WordingViolation> {
    if text.contains('%') {
        return Err(WordingViolation::PercentInConfidence(text.to_string()));
    }
    if text.to_lowercase().contains("high") {
        return Err(WordingViolation::HighConfidenceForbidden);
    }
    scan_candidate_copy(text)
}
