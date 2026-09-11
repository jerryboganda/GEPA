use shared_engine::*;
use std::collections::HashMap;

#[test]
fn test_locator_up_up_up() {
    let mut state = LocatorState::new();
    assert_eq!(state.band, Band::B1);

    let res1 = step_locator(&mut state, 2);
    assert_eq!(res1, LocatorStepResult::NextPair(Band::B2));

    let res2 = step_locator(&mut state, 2);
    assert_eq!(res2, LocatorStepResult::NextPair(Band::C1));

    let res3 = step_locator(&mut state, 2);
    assert_eq!(
        res3,
        LocatorStepResult::Bracket(BracketOutcome {
            bracket: (Band::C1, Band::C2),
            flags: vec!["c2_provisional".to_string()],
        })
    );
    assert_eq!(state.used, 6);
}

#[test]
fn test_locator_down_down_down() {
    let mut state = LocatorState::new();
    assert_eq!(state.band, Band::B1);

    let res1 = step_locator(&mut state, 0);
    assert_eq!(res1, LocatorStepResult::NextPair(Band::A2));

    let res2 = step_locator(&mut state, 0);
    assert_eq!(res2, LocatorStepResult::NextPair(Band::A1));

    let res3 = step_locator(&mut state, 0);
    assert_eq!(
        res3,
        LocatorStepResult::Bracket(BracketOutcome {
            bracket: (Band::PreA1, Band::A1),
            flags: vec!["preA1_confirmation".to_string()],
        })
    );
    assert_eq!(state.used, 6);
}

#[test]
fn test_locator_tie_correct() {
    let mut state = LocatorState::new();
    let res1 = step_locator(&mut state, 1);
    assert_eq!(res1, LocatorStepResult::NextTie(Band::B1));

    let res2 = step_locator(&mut state, 1);
    assert_eq!(
        res2,
        LocatorStepResult::Bracket(BracketOutcome {
            bracket: (Band::B1, Band::B2),
            flags: vec![],
        })
    );
    assert_eq!(state.used, 3);
}

#[test]
fn test_locator_tie_incorrect() {
    let mut state = LocatorState::new();
    let res1 = step_locator(&mut state, 1);
    assert_eq!(res1, LocatorStepResult::NextTie(Band::B1));

    let res2 = step_locator(&mut state, 0);
    assert_eq!(
        res2,
        LocatorStepResult::Bracket(BracketOutcome {
            bracket: (Band::A2, Band::B1),
            flags: vec![],
        })
    );
    assert_eq!(state.used, 3);
}

#[test]
fn test_locator_reversal_up_then_down() {
    let mut state = LocatorState::new();
    let res1 = step_locator(&mut state, 2);
    assert_eq!(res1, LocatorStepResult::NextPair(Band::B2));

    let res2 = step_locator(&mut state, 0);
    assert_eq!(
        res2,
        LocatorStepResult::Bracket(BracketOutcome {
            bracket: (Band::B1, Band::B2),
            flags: vec![],
        })
    );
    assert_eq!(state.used, 4);
}

#[test]
fn test_locator_reversal_down_then_up() {
    let mut state = LocatorState::new();
    let res1 = step_locator(&mut state, 0);
    assert_eq!(res1, LocatorStepResult::NextPair(Band::A2));

    let res2 = step_locator(&mut state, 2);
    assert_eq!(
        res2,
        LocatorStepResult::Bracket(BracketOutcome {
            bracket: (Band::A2, Band::B1),
            flags: vec![],
        })
    );
    assert_eq!(state.used, 4);
}

#[test]
fn test_locator_up_tie_correct() {
    let mut state = LocatorState::new();
    let res1 = step_locator(&mut state, 2);
    assert_eq!(res1, LocatorStepResult::NextPair(Band::B2));

    let res2 = step_locator(&mut state, 1);
    assert_eq!(res2, LocatorStepResult::NextTie(Band::B2));

    let res3 = step_locator(&mut state, 1);
    assert_eq!(
        res3,
        LocatorStepResult::Bracket(BracketOutcome {
            bracket: (Band::B2, Band::C1),
            flags: vec![],
        })
    );
    assert_eq!(state.used, 5);
}

#[test]
fn test_locator_down_tie_incorrect() {
    let mut state = LocatorState::new();
    let res1 = step_locator(&mut state, 0);
    assert_eq!(res1, LocatorStepResult::NextPair(Band::A2));

    let res2 = step_locator(&mut state, 1);
    assert_eq!(res2, LocatorStepResult::NextTie(Band::A2));

    let res3 = step_locator(&mut state, 0);
    assert_eq!(
        res3,
        LocatorStepResult::Bracket(BracketOutcome {
            bracket: (Band::A1, Band::A2),
            flags: vec![],
        })
    );
    assert_eq!(state.used, 5);
}

#[test]
fn test_locator_a1_tie_incorrect() {
    let mut state = LocatorState::new();
    step_locator(&mut state, 0); // at B1 -> down to A2
    step_locator(&mut state, 0); // at A2 -> down to A1
    step_locator(&mut state, 1); // at A1 -> tie
    let res = step_locator(&mut state, 0); // tie incorrect at A1
    assert_eq!(
        res,
        LocatorStepResult::Bracket(BracketOutcome {
            bracket: (Band::PreA1, Band::A1),
            flags: vec!["preA1_confirmation".to_string()],
        })
    );
}

#[test]
fn test_locator_c1_tie_correct() {
    let mut state = LocatorState::new();
    step_locator(&mut state, 2); // B1 -> B2
    step_locator(&mut state, 2); // B2 -> C1
    step_locator(&mut state, 1); // C1 -> tie
    let res = step_locator(&mut state, 1); // tie correct at C1
    assert_eq!(
        res,
        LocatorStepResult::Bracket(BracketOutcome {
            bracket: (Band::C1, Band::C2),
            flags: vec!["c2_provisional".to_string()],
        })
    );
}

#[test]
fn test_confirm_lower_strong_upper_weak() {
    let pair_results = HashMap::new();
    let res = evaluate_confirmation(
        (Band::B1, Band::B2),
        &BlockEvidence::new(5, 5),
        &BlockEvidence::new(1, 5),
        &pair_results,
        None,
        None,
        None,
        None,
        false,
    );
    assert_eq!(res.band, Some(Band::B1));
    assert!(res.notes.iter().any(|n| n.contains("upper not supported")));
}

#[test]
fn test_confirm_borderline_boundary_pass() {
    let pair_results = HashMap::new();
    let boundary = BlockEvidence::new(3, 4);
    let res = evaluate_confirmation(
        (Band::B1, Band::B2),
        &BlockEvidence::new(4, 5),
        &BlockEvidence::new(3, 5),
        &pair_results,
        Some(&boundary),
        None,
        None,
        None,
        false,
    );
    assert_eq!(res.band, Some(Band::B2));
}

#[test]
fn test_confirm_borderline_boundary_fail() {
    let pair_results = HashMap::new();
    let boundary = BlockEvidence::new(2, 4);
    let res = evaluate_confirmation(
        (Band::B1, Band::B2),
        &BlockEvidence::new(4, 5),
        &BlockEvidence::new(3, 5),
        &pair_results,
        Some(&boundary),
        None,
        None,
        None,
        false,
    );
    assert_eq!(res.band, Some(Band::B1));
    assert!(res.notes.iter().any(|n| n.contains("upper possible")));
}

#[test]
fn test_confirm_borderline_no_boundary_items() {
    let pair_results = HashMap::new();
    let boundary = BlockEvidence::new(0, 0); // < 2
    let res = evaluate_confirmation(
        (Band::B1, Band::B2),
        &BlockEvidence::new(4, 5),
        &BlockEvidence::new(3, 5),
        &pair_results,
        Some(&boundary),
        None,
        None,
        None,
        false,
    );
    assert_eq!(res.band, Some(Band::B1));
    assert!(res.flags.iter().any(|f| f == "boundary_unresolved"));
    assert_eq!(res.confidence_cap, Some(Confidence::Low));
}

#[test]
fn test_confirm_both_strong_extension_strong() {
    let pair_results = HashMap::new();
    let ext = BlockEvidence::new(3, 4);
    let res = evaluate_confirmation(
        (Band::B1, Band::B2),
        &BlockEvidence::new(5, 5),
        &BlockEvidence::new(4, 5),
        &pair_results,
        None,
        Some(&ext),
        None,
        None,
        false,
    );
    assert_eq!(res.band, Some(Band::B2));
    assert!(res.flags.iter().any(|f| f == "extension_strong"));
}

#[test]
fn test_confirm_both_strong_ceiling() {
    let pair_results = HashMap::new();
    let res = evaluate_confirmation(
        (Band::C1, Band::C2),
        &BlockEvidence::new(5, 5),
        &BlockEvidence::new(5, 5),
        &pair_results,
        None,
        None,
        None,
        None,
        false,
    );
    assert_eq!(res.band, Some(Band::C2));
    assert!(res.notes.iter().any(|n| n.contains("provisional C2-level evidence")));
}

#[test]
fn test_confirm_aberrant_recheck_high() {
    let pair_results = HashMap::new();
    let recheck = BlockEvidence::new(3, 4);
    let res = evaluate_confirmation(
        (Band::B1, Band::B2),
        &BlockEvidence::new(2, 5),
        &BlockEvidence::new(5, 5),
        &pair_results,
        None,
        None,
        None,
        Some(&recheck),
        false,
    );
    assert_eq!(res.band, Some(Band::B1));
    assert!(res.flags.iter().any(|f| f == "aberrant_pattern"));
    assert_eq!(res.confidence_cap, Some(Confidence::Low));
}

#[test]
fn test_confirm_aberrant_recheck_low() {
    let pair_results = HashMap::new();
    let recheck = BlockEvidence::new(1, 4);
    let res = evaluate_confirmation(
        (Band::B1, Band::B2),
        &BlockEvidence::new(1, 5),
        &BlockEvidence::new(4, 5),
        &pair_results,
        None,
        None,
        None,
        Some(&recheck),
        false,
    );
    assert_eq!(res.band, None);
    assert_eq!(res.range, Some((Band::B1, Band::B2)));
    assert!(res.flags.iter().any(|f| f == "aberrant_pattern"));
}

#[test]
fn test_confirm_lower_weak_descent_strong() {
    let pair_results = HashMap::new();
    let descent = BlockEvidence::new(5, 5);
    let res = evaluate_confirmation(
        (Band::A2, Band::B1),
        &BlockEvidence::new(2, 5),
        &BlockEvidence::new(0, 5),
        &pair_results,
        None,
        None,
        Some(&descent),
        None,
        false,
    );
    assert_eq!(res.band, Some(Band::A1));
}

#[test]
fn test_confirm_lower_weak_floor_from_locator() {
    let mut pair_results = HashMap::new();
    pair_results.insert(Band::A1, 2);
    let res = evaluate_confirmation(
        (Band::A2, Band::B1),
        &BlockEvidence::new(3, 5), // 0.6 -> "lower possible"
        &BlockEvidence::new(0, 5),
        &pair_results,
        None,
        None,
        None,
        None,
        false,
    );
    assert_eq!(res.band, Some(Band::A1));
    assert!(res.notes.iter().any(|n| n.contains("lower possible")));
}

#[test]
fn test_confirm_lower_weak_pre_a1_floor() {
    let pair_results = HashMap::new();
    let res = evaluate_confirmation(
        (Band::PreA1, Band::A1),
        &BlockEvidence::new(2, 4),
        &BlockEvidence::new(1, 5),
        &pair_results,
        None,
        None,
        None,
        None,
        false,
    );
    assert_eq!(res.band, Some(Band::PreA1));
    assert!(res.flags.iter().any(|f| f == "floor_unresolved"));
}

#[test]
fn test_route_median() {
    use shared_engine::productive_route::*;
    let outcomes = vec![
        ModuleOutcomeSummary {
            module: "LS".to_string(),
            band: Some(Band::B2),
            range: None,
        },
        ModuleOutcomeSummary {
            module: "RD".to_string(),
            band: Some(Band::B2),
            range: None,
        },
        ModuleOutcomeSummary {
            module: "LSN".to_string(),
            band: Some(Band::C1),
            range: None,
        },
    ];
    let res = calculate_productive_route(&outcomes);
    assert_eq!(res.route, Route::B2C1);
}

#[test]
fn test_headline_and_wording_policy() {
    // Check forbidden words scan
    assert!(scan_candidate_copy("Your validated CEFR level is B2").is_err());
    assert!(scan_candidate_copy("We predict an IELTS band of 7").is_err());
    assert!(scan_candidate_copy("Result with B1+ score").is_err());
    assert!(scan_candidate_copy("Indicative placement estimate for general study").is_ok());

    // Check headline derivation
    let skills = vec![
        SkillResult {
            skill: "RD".to_string(),
            status: "measured".to_string(),
            band: Some(Band::B2),
            range: None,
            notes: vec![],
            can_do: vec![],
            growth_areas: vec![],
        },
        SkillResult {
            skill: "LSN".to_string(),
            status: "measured".to_string(),
            band: Some(Band::B2),
            range: None,
            notes: vec![],
            can_do: vec![],
            growth_areas: vec![],
        },
        SkillResult {
            skill: "SPK".to_string(),
            status: "measured".to_string(),
            band: Some(Band::B1),
            range: None,
            notes: vec![],
            can_do: vec![],
            growth_areas: vec![],
        },
        SkillResult {
            skill: "WRT".to_string(),
            status: "measured".to_string(),
            band: Some(Band::B2),
            range: None,
            notes: vec![],
            can_do: vec![],
            growth_areas: vec![],
        },
    ];
    let headline = derive_headline("full", &skills);
    assert_eq!(headline.kind, "indicative_overall");
    assert_eq!(headline.band, Some(Band::B2)); // lower median of [B1, B2, B2, B2] is B2
}

#[test]
fn test_confirm_lower_weak_descent_weak() {
    let pair_results = HashMap::new();
    let descent = BlockEvidence::new(3, 5);
    let res = evaluate_confirmation(
        (Band::A2, Band::B1),
        &BlockEvidence::new(2, 5),
        &BlockEvidence::new(0, 5),
        &pair_results,
        None,
        None,
        Some(&descent),
        None,
        false,
    );
    assert_eq!(res.band, Some(Band::A1));
    assert!(res.flags.iter().any(|f| f == "floor_unresolved"));
    assert_eq!(res.confidence_cap, Some(Confidence::Low));
}

#[test]
fn test_confirm_fraction_thresholds_size4() {
    let pair_results = HashMap::new();
    let descent = BlockEvidence::new(4, 5);
    let res = evaluate_confirmation(
        (Band::B1, Band::B2),
        &BlockEvidence::new(3, 4), // 0.75 -> not strong (>= 0.8 required)
        &BlockEvidence::new(2, 4),
        &pair_results,
        None,
        None,
        Some(&descent),
        None,
        false,
    );
    // Lower 0.75 is not strong, routes to descent path -> L-1 = A2
    assert_eq!(res.band, Some(Band::A2));
}

#[test]
fn test_confirm_evidence_shortfall() {
    let pair_results = HashMap::new();
    let res = evaluate_confirmation(
        (Band::B1, Band::B2),
        &BlockEvidence::new(5, 5),
        &BlockEvidence::new(1, 5),
        &pair_results,
        None,
        None,
        None,
        None,
        true, // evidence_shortfall = true
    );
    assert!(res.evidence_shortfall);
}

#[test]
fn test_route_lift() {
    use shared_engine::productive_route::*;
    let outcomes = vec![
        ModuleOutcomeSummary {
            module: "LS".to_string(),
            band: Some(Band::A2),
            range: None,
        },
        ModuleOutcomeSummary {
            module: "RD".to_string(),
            band: Some(Band::B1),
            range: None,
        },
        ModuleOutcomeSummary {
            module: "LSN".to_string(),
            band: Some(Band::B1),
            range: None,
        },
    ];
    let res = calculate_productive_route(&outcomes);
    assert_eq!(res.route, Route::B1B2);
    assert!(res.lifted);
}

#[test]
fn test_route_wide_window() {
    use shared_engine::productive_route::*;
    let outcomes = vec![
        ModuleOutcomeSummary {
            module: "LS".to_string(),
            band: Some(Band::B1),
            range: None,
        },
        ModuleOutcomeSummary {
            module: "RD".to_string(),
            band: Some(Band::B2),
            range: None,
        },
        ModuleOutcomeSummary {
            module: "LSN".to_string(),
            band: Some(Band::A2),
            range: None,
        },
    ];
    let res = calculate_productive_route(&outcomes);
    assert_eq!(res.route, Route::B1B2);
    assert!(res.wide_window);
    assert_eq!(res.confidence_cap, Some(Confidence::Low));
}

#[test]
fn test_route_two_valid_lower() {
    use shared_engine::productive_route::*;
    let outcomes = vec![
        ModuleOutcomeSummary {
            module: "RD".to_string(),
            band: Some(Band::B1),
            range: None,
        },
        ModuleOutcomeSummary {
            module: "LSN".to_string(),
            band: Some(Band::B2),
            range: None,
        },
    ];
    let res = calculate_productive_route(&outcomes);
    assert_eq!(res.route, Route::B1B2);
}

#[test]
fn test_route_ceiling() {
    use shared_engine::productive_route::*;
    let outcomes = vec![
        ModuleOutcomeSummary {
            module: "LS".to_string(),
            band: Some(Band::C2),
            range: None,
        },
        ModuleOutcomeSummary {
            module: "RD".to_string(),
            band: Some(Band::C2),
            range: None,
        },
        ModuleOutcomeSummary {
            module: "LSN".to_string(),
            band: Some(Band::C2),
            range: None,
        },
    ];
    let res = calculate_productive_route(&outcomes);
    assert_eq!(res.route, Route::C1C2);
}

#[test]
fn test_route_none_valid() {
    use shared_engine::productive_route::*;
    let outcomes: Vec<ModuleOutcomeSummary> = vec![];
    let res = calculate_productive_route(&outcomes);
    assert_eq!(res.route, Route::B1B2);
    assert!(res.flags.contains(&"productive_route_default".to_string()));
}

#[test]
fn test_route_range_uses_lower() {
    use shared_engine::productive_route::*;
    let outcomes = vec![
        ModuleOutcomeSummary {
            module: "LS".to_string(),
            band: None,
            range: Some((Band::B1, Band::B2)),
        },
        ModuleOutcomeSummary {
            module: "RD".to_string(),
            band: Some(Band::B2),
            range: None,
        },
        ModuleOutcomeSummary {
            module: "LSN".to_string(),
            band: Some(Band::B2),
            range: None,
        },
    ];
    let res = calculate_productive_route(&outcomes);
    assert_eq!(res.route, Route::B2C1);
}

#[test]
fn test_locator_hard_cap_never_exceeds_8() {
    let mut state = LocatorState::new();
    let mut finished = false;
    let mut steps = 0;
    while !finished && steps < 20 {
        steps += 1;
        let score = if steps % 2 == 1 { 2 } else { 0 };
        let res = step_locator(&mut state, score);
        if let LocatorStepResult::Bracket(_) = res {
            finished = true;
        }
    }
    assert!(finished);
    assert!(state.used <= 8, "Locator used count {} exceeded hard cap 8", state.used);
}

#[test]
fn test_property_no_numeric_level_or_plus_minus_suffix() {
    for b in Band::ALL {
        let s = b.to_string();
        if s != "Pre-A1" {
            assert!(!s.contains('+'), "Band string {} contains forbidden '+'", s);
            assert!(!s.contains('-'), "Band string {} contains forbidden '-'", s);
        }
    }
}

#[test]
fn test_candidate_payload_has_no_restricted_fields() {
    let payload = CandidateDeliveryUnit {
        stimulus_id: Some("RD-B1-S1".to_string()),
        stimulus_text: Some("Public notice text".to_string()),
        stimulus_type: Some("Notice".to_string()),
        audio_url: None,
        items: vec![CandidateItemPayload {
            item_id: "RD-B1-01".to_string(),
            module: "RD".to_string(),
            stem: "What is announced?".to_string(),
            options: vec![
                OptionChoice {
                    option_id: "opt_1".to_string(),
                    text: "Delay".to_string(),
                },
                OptionChoice {
                    option_id: "opt_2".to_string(),
                    text: "Cancellation".to_string(),
                },
            ],
        }],
        deadline_at: "2026-09-09T12:00:00Z".to_string(),
        module_complete: false,
    };

    let json_str = serde_json::to_string(&payload).unwrap();
    let val: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    let forbidden_keys = ["key", "correct", "authoring_letter", "rationale", "script"];
    fn check_no_forbidden(v: &serde_json::Value, forbidden: &[&str]) {
        match v {
            serde_json::Value::Object(map) => {
                for k in map.keys() {
                    for &f in forbidden {
                        assert_ne!(
                            k.to_lowercase(),
                            f,
                            "Security audit failure: forbidden field '{}' found in candidate payload!",
                            k
                        );
                    }
                }
                for child in map.values() {
                    check_no_forbidden(child, forbidden);
                }
            }
            serde_json::Value::Array(arr) => {
                for item in arr {
                    check_no_forbidden(item, forbidden);
                }
            }
            _ => {}
        }
    }
    check_no_forbidden(&val, &forbidden_keys);
}
