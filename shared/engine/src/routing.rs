use crate::models::{Band, Confidence, LocatorEvent};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    None,
    Up,
    Down,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocatorState {
    pub band: Band,
    pub direction: Direction,
    pub used: u32,
    pub pair_results: HashMap<Band, u32>,
    pub trace: Vec<LocatorEvent>,
    pub waiting_tie: bool,
}

impl Default for LocatorState {
    fn default() -> Self {
        Self::new()
    }
}

impl LocatorState {
    pub fn new() -> Self {
        Self {
            band: Band::B1,
            direction: Direction::None,
            used: 0,
            pair_results: HashMap::new(),
            trace: Vec::new(),
            waiting_tie: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BracketOutcome {
    pub bracket: (Band, Band),
    pub flags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LocatorStepResult {
    NextPair(Band),
    NextTie(Band),
    Bracket(BracketOutcome),
}

/// Advance the locator engine by one step given the score of the current unit.
/// - If waiting_tie is false, `c` is the score out of 2 (0, 1, or 2).
/// - If waiting_tie is true, `c` is the score out of 1 (0 or 1).
pub fn step_locator(state: &mut LocatorState, c: u32) -> LocatorStepResult {
    if state.waiting_tie {
        state.used += 1;
        state.waiting_tie = false;
        state.trace.push(LocatorEvent {
            band: state.band,
            item_ids: Vec::new(),
            correct: c,
            kind: "tie".to_string(),
        });

        let mut flags = Vec::new();
        if c == 1 {
            // Tie correct -> [band, band + 1]
            if state.band == Band::C1 {
                flags.push("c2_provisional".to_string());
                return LocatorStepResult::Bracket(BracketOutcome {
                    bracket: (Band::C1, Band::C2),
                    flags,
                });
            }
            let next_band = state.band.next().unwrap_or(Band::C2);
            LocatorStepResult::Bracket(BracketOutcome {
                bracket: (state.band, next_band),
                flags,
            })
        } else {
            // Tie incorrect -> [band - 1, band]
            if state.band == Band::A1 {
                flags.push("preA1_confirmation".to_string());
                return LocatorStepResult::Bracket(BracketOutcome {
                    bracket: (Band::PreA1, Band::A1),
                    flags,
                });
            }
            let prev_band = state.band.prev().unwrap_or(Band::PreA1);
            LocatorStepResult::Bracket(BracketOutcome {
                bracket: (prev_band, state.band),
                flags,
            })
        }
    } else {
        state.used += 2;
        state.pair_results.insert(state.band, c);
        state.trace.push(LocatorEvent {
            band: state.band,
            item_ids: Vec::new(),
            correct: c,
            kind: "pair".to_string(),
        });

        if c == 2 {
            if state.band == Band::C1 || state.band == Band::C2 {
                return LocatorStepResult::Bracket(BracketOutcome {
                    bracket: (Band::C1, Band::C2),
                    flags: vec!["c2_provisional".to_string()],
                });
            }
            if let Some(next_b) = state.band.next() {
                if state.pair_results.get(&next_b).copied() == Some(0) {
                    // Reversal: came down, now 2/2
                    return LocatorStepResult::Bracket(BracketOutcome {
                        bracket: (state.band, next_b),
                        flags: vec![],
                    });
                }
                state.band = next_b;
                state.direction = Direction::Up;
            }
        } else if c == 0 {
            if state.band == Band::A1 {
                return LocatorStepResult::Bracket(BracketOutcome {
                    bracket: (Band::PreA1, Band::A1),
                    flags: vec!["preA1_confirmation".to_string()],
                });
            }
            if let Some(prev_b) = state.band.prev() {
                if state.pair_results.get(&prev_b).copied() == Some(2) {
                    // Reversal: came up, now 0/2
                    return LocatorStepResult::Bracket(BracketOutcome {
                        bracket: (prev_b, state.band),
                        flags: vec![],
                    });
                }
                state.band = prev_b;
                state.direction = Direction::Down;
            }
        } else {
            // c == 1 -> tie required
            state.waiting_tie = true;
            return LocatorStepResult::NextTie(state.band);
        }

        if state.used >= 8 {
            let mut flags = vec!["boundary_confirmation".to_string()];
            let bracket = if state.direction == Direction::Up {
                (state.band.prev().unwrap_or(Band::PreA1), state.band)
            } else {
                (state.band, state.band.next().unwrap_or(Band::C2))
            };
            if bracket.1 == Band::C2 {
                flags.push("c2_provisional".to_string());
            }
            return LocatorStepResult::Bracket(BracketOutcome { bracket, flags });
        }

        LocatorStepResult::NextPair(state.band)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfirmationOutcome {
    pub band: Option<Band>,
    pub range: Option<(Band, Band)>,
    pub notes: Vec<String>,
    pub flags: Vec<String>,
    pub evidence_shortfall: bool,
    pub confidence_cap: Option<Confidence>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockEvidence {
    pub correct: u32,
    pub size: u32,
}

impl BlockEvidence {
    pub fn new(correct: u32, size: u32) -> Self {
        Self { correct, size }
    }

    pub fn proportion(&self) -> f64 {
        if self.size == 0 {
            0.0
        } else {
            self.correct as f64 / self.size as f64
        }
    }

    pub fn is_strong(&self) -> bool {
        self.proportion() >= 0.8
    }

    pub fn is_weak(&self) -> bool {
        self.proportion() <= 0.4
    }

    pub fn is_borderline(&self) -> bool {
        !self.is_strong() && !self.is_weak()
    }
}

/// Evaluate confirmation phase per specification 04 §2.
#[allow(clippy::too_many_arguments)]
pub fn evaluate_confirmation(
    bracket: (Band, Band),
    lower_evidence: &BlockEvidence,
    upper_evidence: &BlockEvidence,
    pair_results: &HashMap<Band, u32>,
    boundary_block: Option<&BlockEvidence>,
    extension_block: Option<&BlockEvidence>,
    descent_block: Option<&BlockEvidence>,
    aberrant_recheck_block: Option<&BlockEvidence>,
    evidence_shortfall: bool,
) -> ConfirmationOutcome {
    let (l, u) = bracket;
    let mut notes = Vec::new();
    let mut flags = Vec::new();
    let mut confidence_cap = None;

    let cl = lower_evidence.correct as i32;
    let cu = upper_evidence.correct as i32;
    let pl_strong = lower_evidence.is_strong();
    let pu_strong = upper_evidence.is_strong();
    let pu_weak = upper_evidence.is_weak();
    let pu_borderline = upper_evidence.is_borderline();

    // 1. Non-monotonic check: cU - cL >= 3
    if cu - cl >= 3 {
        flags.push("aberrant_pattern".to_string());
        confidence_cap = Some(Confidence::Low);

        if let Some(recheck) = aberrant_recheck_block {
            if recheck.size >= 2 && recheck.proportion() >= 0.75 {
                notes.push("upper possible; non-monotonic pattern".to_string());
                return ConfirmationOutcome {
                    band: Some(l),
                    range: None,
                    notes,
                    flags,
                    evidence_shortfall,
                    confidence_cap,
                };
            }
        }
        // 1b fallback
        return ConfirmationOutcome {
            band: None,
            range: Some((l, u)),
            notes,
            flags,
            evidence_shortfall,
            confidence_cap,
        };
    }

    // 2. strong(pL) and weak(pU)
    if pl_strong && pu_weak {
        notes.push("upper not supported".to_string());
        return ConfirmationOutcome {
            band: Some(l),
            range: None,
            notes,
            flags,
            evidence_shortfall,
            confidence_cap,
        };
    }

    // 3. strong(pL) and borderline(pU)
    if pl_strong && pu_borderline {
        if let Some(boundary) = boundary_block {
            if boundary.size < 2 {
                notes.push("upper possible".to_string());
                flags.push("boundary_unresolved".to_string());
                confidence_cap = Some(Confidence::Low);
                return ConfirmationOutcome {
                    band: Some(l),
                    range: None,
                    notes,
                    flags,
                    evidence_shortfall,
                    confidence_cap,
                };
            } else {
                let threshold = ((0.75 * boundary.size as f64).ceil()) as u32;
                if boundary.correct >= threshold {
                    return ConfirmationOutcome {
                        band: Some(u),
                        range: None,
                        notes,
                        flags,
                        evidence_shortfall,
                        confidence_cap,
                    };
                } else {
                    notes.push("upper possible".to_string());
                    return ConfirmationOutcome {
                        band: Some(l),
                        range: None,
                        notes,
                        flags,
                        evidence_shortfall,
                        confidence_cap,
                    };
                }
            }
        } else {
            notes.push("upper possible".to_string());
            flags.push("boundary_unresolved".to_string());
            confidence_cap = Some(Confidence::Low);
            return ConfirmationOutcome {
                band: Some(l),
                range: None,
                notes,
                flags,
                evidence_shortfall,
                confidence_cap,
            };
        }
    }

    // 4. strong(pL) and strong(pU)
    if pl_strong && pu_strong {
        if u == Band::C2 {
            notes.push("provisional C2-level evidence".to_string());
            return ConfirmationOutcome {
                band: Some(Band::C2),
                range: None,
                notes,
                flags,
                evidence_shortfall,
                confidence_cap,
            };
        } else if let Some(ext) = extension_block {
            let threshold = ((0.75 * ext.size as f64).ceil()) as u32;
            if ext.size >= 2 && ext.correct >= threshold {
                notes.push("upper end of band; next band possible".to_string());
                flags.push("extension_strong".to_string());
            }
        }
        return ConfirmationOutcome {
            band: Some(u),
            range: None,
            notes,
            flags,
            evidence_shortfall,
            confidence_cap,
        };
    }

    // 5. not strong(pL) (lower < 0.8)
    if l == Band::PreA1 {
        flags.push("floor_unresolved".to_string());
        confidence_cap = Some(Confidence::Low);
        return ConfirmationOutcome {
            band: Some(Band::PreA1),
            range: None,
            notes,
            flags,
            evidence_shortfall,
            confidence_cap,
        };
    }

    let prev_band = l.prev().unwrap_or(Band::PreA1);
    if pair_results.get(&prev_band).copied() == Some(2) {
        if lower_evidence.proportion() >= 0.6 {
            notes.push("lower possible".to_string());
        }
        return ConfirmationOutcome {
            band: Some(prev_band),
            range: None,
            notes,
            flags,
            evidence_shortfall,
            confidence_cap,
        };
    }

    if let Some(descent) = descent_block {
        if descent.size < 2 {
            flags.push("floor_unresolved".to_string());
            confidence_cap = Some(Confidence::Low);
        } else if descent.is_strong() {
            if lower_evidence.proportion() >= 0.6 {
                notes.push("lower possible".to_string());
            }
        } else {
            flags.push("floor_unresolved".to_string());
            confidence_cap = Some(Confidence::Low);
        }
    } else {
        flags.push("floor_unresolved".to_string());
        confidence_cap = Some(Confidence::Low);
    }

    if pu_strong {
        flags.push("inconsistent_pattern".to_string());
        confidence_cap = Some(Confidence::Low);
    }

    ConfirmationOutcome {
        band: Some(prev_band),
        range: None,
        notes,
        flags,
        evidence_shortfall,
        confidence_cap,
    }
}
