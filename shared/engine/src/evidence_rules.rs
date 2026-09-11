use crate::models::{Band, Confidence, ProductiveRating, Route, SpeakingTask, WritingTask};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProductiveDecision {
    pub status: String, // "measured" | "insufficient_evidence"
    pub band: Option<Band>,
    pub notes: Vec<String>,
    pub flags: Vec<String>,
    pub confidence_cap: Option<Confidence>,
}

fn calculate_trait_mean(scores: &std::collections::HashMap<String, u32>, traits: &[&str]) -> f64 {
    if traits.is_empty() {
        return 0.0;
    }
    let mut sum = 0.0;
    let mut count = 0;
    for &t in traits {
        if let Some(&score) = scores.get(t) {
            sum += score as f64;
            count += 1;
        }
    }
    if count == 0 {
        0.0
    } else {
        sum / count as f64
    }
}

pub fn evaluate_speaking(
    responses: &[(&SpeakingTask, &ProductiveRating)],
    route: Route,
) -> ProductiveDecision {
    let mut notes = Vec::new();
    let mut flags = Vec::new();
    let mut confidence_cap = None;

    // Filter spontaneous & usable
    let spont: Vec<_> = responses
        .iter()
        .filter(|(task, rating)| task.spontaneous_or_interactive && rating.usable)
        .copied()
        .collect();

    if spont.len() < 3 {
        return ProductiveDecision {
            status: "insufficient_evidence".to_string(),
            band: None,
            notes: vec!["fewer than 3 usable spontaneous responses".to_string()],
            flags: vec!["insufficient_spontaneous_speech".to_string()],
            confidence_cap: Some(Confidence::Low),
        };
    }

    let default_traits = ["intelligibility", "fluency", "grammar", "vocabulary", "communication"];
    let or_traits = ["intelligibility", "fluency"];
    let sr_traits = ["intelligibility", "fluency", "communication"];

    let mut lower_means = Vec::new();
    let mut upper_means = Vec::new();
    let mut comm_at_least_3_lower = 0;
    let mut upper_core_min_at_least_2 = true;
    let mut independent_upper_meets = HashSet::new();

    for (task, rating) in &spont {
        let traits: &[&str] = match task.task_type.as_str() {
            "oral_reading" => &or_traits,
            "sentence_reconstruction" => &sr_traits,
            _ => &default_traits,
        };

        let mean_lower = calculate_trait_mean(&rating.at_lower, traits);
        let mean_upper = calculate_trait_mean(&rating.at_upper, traits);

        lower_means.push(mean_lower);
        upper_means.push(mean_upper);

        if rating.at_lower.get("communication").copied().unwrap_or(0) >= 3 {
            comm_at_least_3_lower += 1;
        }

        let g = rating.at_upper.get("grammar").copied().unwrap_or(0);
        let v = rating.at_upper.get("vocabulary").copied().unwrap_or(0);
        let c = rating.at_upper.get("communication").copied().unwrap_or(0);
        if g < 2 || v < 2 || c < 2 {
            upper_core_min_at_least_2 = false;
        }

        if mean_upper >= 3.0 {
            // Task group for independence (INT1 and INT2 share the same task_group)
            independent_upper_meets.insert(task.task_group.clone());
        }
    }

    let mean_spont_lower: f64 = lower_means.iter().sum::<f64>() / lower_means.len() as f64;
    let mean_spont_upper: f64 = upper_means.iter().sum::<f64>() / upper_means.len() as f64;

    let lower_meets = mean_spont_lower >= 2.75 && comm_at_least_3_lower >= 2;
    let upper_meets = mean_spont_upper >= 3.0
        && upper_core_min_at_least_2
        && independent_upper_meets.len() >= 2;

    if upper_meets && lower_meets {
        ProductiveDecision {
            status: "measured".to_string(),
            band: Some(route.upper_band()),
            notes,
            flags,
            confidence_cap,
        }
    } else if lower_meets {
        ProductiveDecision {
            status: "measured".to_string(),
            band: Some(route.lower_band()),
            notes,
            flags,
            confidence_cap,
        }
    } else {
        // Below route check
        if spont.len() >= 3 && mean_spont_lower >= 1.75 {
            let prev_band = route.lower_band().prev().unwrap_or(Band::PreA1);
            notes.push("below_route".to_string());
            flags.push("below_route".to_string());
            confidence_cap = Some(Confidence::Low);
            ProductiveDecision {
                status: "measured".to_string(),
                band: Some(prev_band),
                notes,
                flags,
                confidence_cap,
            }
        } else {
            ProductiveDecision {
                status: "insufficient_evidence".to_string(),
                band: None,
                notes: vec!["evidence below route thresholds and insufficient for below-route estimate".to_string()],
                flags: vec!["insufficient_evidence".to_string()],
                confidence_cap: Some(Confidence::Low),
            }
        }
    }
}

pub fn evaluate_writing(
    responses: &[(&WritingTask, &ProductiveRating)],
    route: Route,
) -> ProductiveDecision {
    let mut notes = Vec::new();
    let mut flags = Vec::new();
    let mut confidence_cap = None;

    let scored: Vec<_> = responses
        .iter()
        .filter(|(task, rating)| task.scored_in_writing_level && rating.usable)
        .copied()
        .collect();

    if scored.len() < 2 {
        return ProductiveDecision {
            status: "insufficient_evidence".to_string(),
            band: None,
            notes: vec!["fewer than 2 usable scored writing tasks".to_string()],
            flags: vec!["insufficient_writing_evidence".to_string()],
            confidence_cap: Some(Confidence::Low),
        };
    }

    let traits = [
        "task_fulfilment",
        "organisation",
        "grammar",
        "vocabulary",
        "mechanics_register",
    ];

    let mut lower_meeting_tasks = 0;
    let mut upper_meeting_tasks = 0;
    let mut upper_core_ok = true;
    let mut lower_means = Vec::new();

    for (_task, rating) in &scored {
        let mean_lower = calculate_trait_mean(&rating.at_lower, &traits);
        let mean_upper = calculate_trait_mean(&rating.at_upper, &traits);
        lower_means.push(mean_lower);

        let tf_lower = rating.at_lower.get("task_fulfilment").copied().unwrap_or(0);
        if mean_lower >= 2.75 && tf_lower >= 3 {
            lower_meeting_tasks += 1;
        }

        if mean_upper >= 3.0 {
            upper_meeting_tasks += 1;
        }

        let tf_upper = rating.at_upper.get("task_fulfilment").copied().unwrap_or(0);
        let org_upper = rating.at_upper.get("organisation").copied().unwrap_or(0);
        if tf_upper < 2 || org_upper < 2 {
            upper_core_ok = false;
        }
    }

    let lower_meets = lower_meeting_tasks >= 2;
    let upper_meets = upper_meeting_tasks >= 2 && upper_core_ok;

    if upper_meets && lower_meets {
        ProductiveDecision {
            status: "measured".to_string(),
            band: Some(route.upper_band()),
            notes,
            flags,
            confidence_cap,
        }
    } else if lower_meets {
        ProductiveDecision {
            status: "measured".to_string(),
            band: Some(route.lower_band()),
            notes,
            flags,
            confidence_cap,
        }
    } else {
        let mean_scored_lower = lower_means.iter().sum::<f64>() / lower_means.len() as f64;
        if scored.len() >= 2 && mean_scored_lower >= 1.75 {
            let prev_band = route.lower_band().prev().unwrap_or(Band::PreA1);
            notes.push("below_route".to_string());
            flags.push("below_route".to_string());
            confidence_cap = Some(Confidence::Low);
            ProductiveDecision {
                status: "measured".to_string(),
                band: Some(prev_band),
                notes,
                flags,
                confidence_cap,
            }
        } else {
            ProductiveDecision {
                status: "insufficient_evidence".to_string(),
                band: None,
                notes: vec!["evidence below route thresholds".to_string()],
                flags: vec!["insufficient_evidence".to_string()],
                confidence_cap: Some(Confidence::Low),
            }
        }
    }
}
