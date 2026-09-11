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

#[test]
fn test_data_model_enums_serialization_and_invariants() {
    // 1. Band
    let bands = [
        (Band::PreA1, "\"Pre-A1\"", 0),
        (Band::A1, "\"A1\"", 1),
        (Band::A2, "\"A2\"", 2),
        (Band::B1, "\"B1\"", 3),
        (Band::B2, "\"B2\"", 4),
        (Band::C1, "\"C1\"", 5),
        (Band::C2, "\"C2\"", 6),
    ];
    for (band, expected_json, idx) in bands {
        assert_eq!(serde_json::to_string(&band).unwrap(), expected_json);
        let de: Band = serde_json::from_str(expected_json).unwrap();
        assert_eq!(de, band);
        assert_eq!(band.index(), idx);
        assert_eq!(Band::from_index(idx), Some(band));
    }
    assert_eq!(Band::PreA1.prev(), None);
    assert_eq!(Band::PreA1.next(), Some(Band::A1));
    assert_eq!(Band::C2.next(), None);
    assert_eq!(Band::C2.prev(), Some(Band::C1));

    // 2. Route
    let routes = [
        (Route::PreA1A1, "\"PreA1-A1\"", 0, Band::PreA1, Band::A1),
        (Route::A1A2, "\"A1-A2\"", 1, Band::A1, Band::A2),
        (Route::A2B1, "\"A2-B1\"", 2, Band::A2, Band::B1),
        (Route::B1B2, "\"B1-B2\"", 3, Band::B1, Band::B2),
        (Route::B2C1, "\"B2-C1\"", 4, Band::B2, Band::C1),
        (Route::C1C2, "\"C1-C2\"", 5, Band::C1, Band::C2),
    ];
    for (route, expected_json, idx, lower, upper) in routes {
        assert_eq!(serde_json::to_string(&route).unwrap(), expected_json);
        let de: Route = serde_json::from_str(expected_json).unwrap();
        assert_eq!(de, route);
        assert_eq!(route.index(), idx);
        assert_eq!(Route::from_index(idx), Some(route));
        assert_eq!(route.bands(), (lower, upper));
        assert_eq!(route.lower_band(), lower);
        assert_eq!(route.upper_band(), upper);
    }

    // 3. Module
    let modules = [
        (Module::LS, "\"LS\""),
        (Module::RD, "\"RD\""),
        (Module::LSN, "\"LSN\""),
        (Module::SPK, "\"SPK\""),
        (Module::WRT, "\"WRT\""),
    ];
    for (m, expected_json) in modules {
        assert_eq!(serde_json::to_string(&m).unwrap(), expected_json);
        let de: Module = serde_json::from_str(expected_json).unwrap();
        assert_eq!(de, m);
    }

    // 4. Domain
    let domains = [
        (Domain::Personal, "\"personal\""),
        (Domain::Public, "\"public\""),
        (Domain::Educational, "\"educational\""),
        (Domain::Occupational, "\"occupational\""),
    ];
    for (d, expected_json) in domains {
        assert_eq!(serde_json::to_string(&d).unwrap(), expected_json);
        let de: Domain = serde_json::from_str(expected_json).unwrap();
        assert_eq!(de, d);
    }

    // 5. Confidence
    assert_eq!(serde_json::to_string(&Confidence::Low).unwrap(), "\"Low\"");
    assert_eq!(serde_json::to_string(&Confidence::Moderate).unwrap(), "\"Moderate\"");

    // 6. ModuleStatus
    let statuses = [
        (ModuleStatus::Pending, "\"pending\""),
        (ModuleStatus::InProgress, "\"in_progress\""),
        (ModuleStatus::Paused, "\"paused\""),
        (ModuleStatus::Complete, "\"complete\""),
        (ModuleStatus::Abandoned, "\"abandoned\""),
        (ModuleStatus::NotMeasured, "\"not_measured\""),
    ];
    for (st, expected_json) in statuses {
        assert_eq!(serde_json::to_string(&st).unwrap(), expected_json);
        let de: ModuleStatus = serde_json::from_str(expected_json).unwrap();
        assert_eq!(de, st);
    }

    // 7. Mode
    assert_eq!(serde_json::to_string(&Mode::FreeBeta).unwrap(), "\"free_beta\"");
    assert_eq!(serde_json::to_string(&Mode::Verified).unwrap(), "\"verified\"");

    // 8. Role
    assert_eq!(serde_json::to_string(&Role::Candidate).unwrap(), "\"candidate\"");
    assert_eq!(serde_json::to_string(&Role::Reviewer).unwrap(), "\"reviewer\"");
    assert_eq!(serde_json::to_string(&Role::Admin).unwrap(), "\"admin\"");
}

#[test]
fn test_data_model_bank_objects_roundtrip_and_defaults() {
    // 1. OptionChoice
    let opt = OptionChoice {
        option_id: "opt_1".to_string(),
        text: "Sample option".to_string(),
    };
    let opt_json = serde_json::to_string(&opt).unwrap();
    assert_eq!(opt_json, "{\"option_id\":\"opt_1\",\"text\":\"Sample option\"}");

    // 2. ObjectiveItem defaults and full roundtrip
    let min_item_json = r#"{
        "item_id": "LS-B1-01",
        "module": "LS",
        "band": "B1",
        "stem": "Choose the correct word.",
        "options": [
            {"option_id": "o1", "text": "A"},
            {"option_id": "o2", "text": "B"},
            {"option_id": "o3", "text": "C"}
        ]
    }"#;
    let item: ObjectiveItem = serde_json::from_str(min_item_json).unwrap();
    assert_eq!(item.version, 1);
    assert_eq!(item.status, "trial");
    assert!(!item.locator_candidate);
    assert!(!item.is_anchor);
    assert!(!item.is_pretest);
    assert!(item.reviewer_provenance.is_empty());
    assert!(item.exposure.is_none());

    let full_item_json = r#"{
        "item_id": "LS-B1-02",
        "module": "LS",
        "band": "B1",
        "version": 2,
        "status": "active",
        "stem": "Complete the phrase.",
        "options": [
            {"option_id": "o1", "text": "A"},
            {"option_id": "o2", "text": "B"},
            {"option_id": "o3", "text": "C"}
        ],
        "construct": "collocation",
        "domain": "educational",
        "topic_family": "study",
        "locator_candidate": true,
        "is_anchor": true,
        "is_pretest": false,
        "reviewer_provenance": [
            {"reviewer": "Dr Smith", "date": "2026-09-01", "note": "Checked key balance"}
        ],
        "exposure": {
            "delivered": 150,
            "correct": 120,
            "medianMs": 4500.0
        }
    }"#;
    let full_item: ObjectiveItem = serde_json::from_str(full_item_json).unwrap();
    assert_eq!(full_item.reviewer_provenance.len(), 1);
    assert_eq!(full_item.reviewer_provenance[0].reviewer, "Dr Smith");
    let exp = full_item.exposure.unwrap();
    assert_eq!(exp.delivered, 150);
    assert_eq!(exp.correct, 120);
    assert_eq!(exp.median_ms, Some(4500.0));

    // 3. ReadingStimulus
    let rd_stim_json = r#"{
        "stimulus_id": "RD-B1-S1",
        "band": "B1",
        "stimulus_type": "notice",
        "domain": "public",
        "topic_family": "library",
        "text": "Library hours have changed.",
        "word_count": 45,
        "item_ids": ["RD-B1-01", "RD-B1-02"]
    }"#;
    let rd_stim: ReadingStimulus = serde_json::from_str(rd_stim_json).unwrap();
    assert_eq!(rd_stim.band, Band::B1);
    assert_eq!(rd_stim.word_count, 45);

    // 4. ListeningStimulusPublic and Media camelCase
    let lsn_pub_json = r#"{
        "stimulus_id": "LSN-B1-S1",
        "band": "B1",
        "speaker_count": 2,
        "domain": "personal",
        "topic_family": "weekend",
        "media": {
            "storagePath": "audio/stimuli/LSN-B1-S1/v1.opus",
            "durationSec": 28.5,
            "loudnessLufs": -16.0,
            "checksum": "sha256:abc",
            "codec": "opus"
        },
        "item_ids": ["LSN-B1-01", "LSN-B1-02"]
    }"#;
    let lsn_pub: ListeningStimulusPublic = serde_json::from_str(lsn_pub_json).unwrap();
    let media = lsn_pub.media.unwrap();
    assert_eq!(media.storage_path, "audio/stimuli/LSN-B1-S1/v1.opus");
    assert_eq!(media.duration_sec, 28.5);
    assert_eq!(media.loudness_lufs, -16.0);

    // 5. SpeakingTask with camelCase prepSeconds/maxSpeakSeconds/allowsRerecord
    let spk_json = r#"{
        "task_id": "SPK-B1B2-ER1",
        "route": "B1-B2",
        "route_bands": ["B1", "B2"],
        "task_type": "extended_response",
        "task_group": "group_1",
        "turn": null,
        "label": "Extended Response",
        "prompt": "Describe a project you enjoyed working on.",
        "candidate_sees_text": true,
        "delivery": "text_prompt",
        "weight": 0.3,
        "spontaneous_or_interactive": true,
        "traits_scored": ["fluency", "grammar", "vocabulary", "coherence"],
        "topic_family": "work",
        "prepSeconds": 20,
        "maxSpeakSeconds": 60,
        "allowsRerecord": false
    }"#;
    let spk: SpeakingTask = serde_json::from_str(spk_json).unwrap();
    assert_eq!(spk.prep_seconds, 20);
    assert_eq!(spk.max_speak_seconds, 60);
    assert!(!spk.allows_rerecord);

    // 6. WritingTask with camelCase timeLimitSeconds
    let wrt_json = r#"{
        "task_id": "WRT-B1B2-1",
        "route": "B1-B2",
        "route_bands": ["B1", "B2"],
        "task_type": "functional_communication",
        "label": "Email to Manager",
        "prompt": "Write an email explaining your schedule request.",
        "focus": "register and clarity",
        "word_guidance": {"min": 80, "max": 120},
        "weight": 0.35,
        "scored_in_writing_level": true,
        "topic_family": "workplace",
        "timeLimitSeconds": 480
    }"#;
    let wrt: WritingTask = serde_json::from_str(wrt_json).unwrap();
    assert_eq!(wrt.time_limit_seconds, 480);
    assert_eq!(wrt.word_guidance.as_ref().unwrap().min, 80);

    // 7. RestrictedKey
    let key_json = r#"{
        "item_id": "LS-B1-01",
        "module": "LS",
        "band": "B1",
        "key_option_id": "opt_2",
        "authoring_letter": "B",
        "answer_text": "correct phrase",
        "option_count": 4,
        "evidence_focus": "collocation",
        "rationale": "Option B is standard usage"
    }"#;
    let key: RestrictedKey = serde_json::from_str(key_json).unwrap();
    assert_eq!(key.authoring_letter, "B");
    assert_eq!(key.key_option_id, "opt_2");
}

#[test]
fn test_data_model_ruleset_form_and_session_roundtrip() {
    // 1. RoutingRuleset defaults
    let ruleset_json = r#"{
        "ruleset_id": "ruleset_v2_pilot",
        "version": "2.0",
        "confirmation": {},
        "productive": {},
        "effective_from": "2026-09-01T00:00:00Z"
    }"#;
    let ruleset: RoutingRuleset = serde_json::from_str(ruleset_json).unwrap();
    assert_eq!(ruleset.start_band, Band::B1);
    assert_eq!(ruleset.locator_pair_size, 2);
    assert_eq!(ruleset.locator_hard_cap, 8);
    assert_eq!(ruleset.confirmation.lower_block, 5);
    assert_eq!(ruleset.confirmation.upper_block, 5);
    assert_eq!(ruleset.confirmation.strong_min, 0.8);
    assert_eq!(ruleset.confirmation.borderline, 0.6);
    assert_eq!(ruleset.confirmation.weak_max, 0.4);
    assert_eq!(ruleset.confirmation.boundary_items, 4);
    assert_eq!(ruleset.confirmation.extension_items, 4);
    assert_eq!(ruleset.confirmation.descent_items, 5);
    assert_eq!(ruleset.confirmation.aberrant_gap, 3);
    assert!(ruleset.productive.lift_when_both_receptive_above_ls);
    assert_eq!(ruleset.productive.wide_window_span, 2);

    // 2. FormSpec
    let form_json = r#"{
        "form_id": "beta_form_01",
        "version": "1.0",
        "blueprint_version": "v2.0",
        "ruleset_id": "ruleset_v2_pilot",
        "item_ids_by_module": {
            "LS": ["LS-B1-01", "LS-B1-02"]
        },
        "anchors": ["LS-B1-01"],
        "key_balance_report": {
            "LS": {"A": 0.25, "B": 0.25, "C": 0.25, "D": 0.25}
        },
        "domain_coverage_report": {
            "personal": 0.25
        },
        "enemy_group_validation": {
            "conflicts": [
                {"family": "transport", "members": ["LS-B1-01", "LS-B1-02"], "severity": "warning"}
            ]
        },
        "status": "beta"
    }"#;
    let form: FormSpec = serde_json::from_str(form_json).unwrap();
    assert_eq!(form.status, "beta");
    assert_eq!(form.enemy_group_validation.conflicts.len(), 1);

    // 3. ObjectiveModuleState
    let obj_mod_json = r#"{
        "module": "LS",
        "status": "in_progress",
        "phase": "locator",
        "currentBand": "B1",
        "direction": "up",
        "locatorItemsUsed": 2,
        "locatorTrace": [
            {"band": "B1", "itemIds": ["LS-B1-01", "LS-B1-02"], "correct": 2, "kind": "pair"}
        ],
        "bracket": null,
        "confirmationTrace": [],
        "usedItemIds": ["LS-B1-01", "LS-B1-02"],
        "currentUnit": {
            "stimulusId": null,
            "itemIds": ["LS-B2-01", "LS-B2-02"],
            "deadlineAt": "2026-09-11T12:05:00Z"
        },
        "outcome": null,
        "stateVersion": 2
    }"#;
    let obj_mod: ObjectiveModuleState = serde_json::from_str(obj_mod_json).unwrap();
    assert_eq!(obj_mod.status, ModuleStatus::InProgress);
    assert_eq!(obj_mod.current_band, Band::B1);
    assert_eq!(obj_mod.locator_items_used, 2);
    assert_eq!(obj_mod.locator_trace[0].item_ids.len(), 2);

    // 4. ProductiveModuleState
    let prod_mod_json = r#"{
        "module": "SPK",
        "status": "pending",
        "route": "B1-B2",
        "taskIds": ["SPK-B1B2-OR", "SPK-B1B2-ER1"],
        "currentTaskId": "SPK-B1B2-OR",
        "deadlineAt": null,
        "attempts": {"SPK-B1B2-OR": 1},
        "micCheck": {"passed": true, "at": "2026-09-11T12:00:00Z"},
        "ratingStatus": "not_started",
        "stateVersion": 1
    }"#;
    let prod_mod: ProductiveModuleState = serde_json::from_str(prod_mod_json).unwrap();
    assert_eq!(prod_mod.module, "SPK");
    assert_eq!(prod_mod.route, Some(Route::B1B2));
    assert!(prod_mod.mic_check.as_ref().unwrap().passed);

    // 5. Session
    let session_json = r#"{
        "session_id": "sess_01J7K8M9",
        "candidate_uid": "user_123",
        "mode": "free_beta",
        "createdAt": "2026-09-11T12:00:00Z",
        "targetGoal": "OET",
        "uiLanguage": "en",
        "consent": {"privacy": true, "research": true},
        "accommodations": {
            "extendedTime": false,
            "transcriptAccess": false,
            "oralReadingAlternative": false,
            "highContrast": true,
            "textScale": 1.2,
            "spacing": "wide",
            "keyboardOnly": false
        },
        "form_id": "beta_form_01",
        "ruleset_id": "ruleset_v2_pilot",
        "modules": {
            "LS": {
                "module": "LS",
                "status": "in_progress",
                "phase": "locator",
                "currentBand": "B1",
                "direction": "none",
                "locatorItemsUsed": 0,
                "locatorTrace": [],
                "bracket": null,
                "confirmationTrace": [],
                "usedItemIds": [],
                "currentUnit": null,
                "outcome": null,
                "stateVersion": 1
            }
        },
        "productiveRoute": {
            "route": "B1-B2",
            "wideWindow": false,
            "lifted": false,
            "basis": ["LS-B1", "RD-B1", "LSN-B1"]
        },
        "device": {
            "class": "desktop",
            "ua": "Mozilla/5.0",
            "network": "broadband"
        },
        "flags": [
            {"code": "effort_rapid", "module": "LS", "detail": "Sub-second answer", "at": "2026-09-11T12:01:00Z"}
        ],
        "interruptions": [],
        "profileType": "full"
    }"#;
    let session: Session = serde_json::from_str(session_json).unwrap();
    assert_eq!(session.target_goal, "OET");
    assert_eq!(session.mode, Mode::FreeBeta);
    assert_eq!(session.accommodations.text_scale, 1.2);
    assert_eq!(session.accommodations.spacing, "wide");
    assert_eq!(session.flags.len(), 1);
    assert_eq!(session.flags[0].module, Some(Module::LS));
}

#[test]
fn test_data_model_responses_ratings_results_roundtrip() {
    // 1. ObjectiveResponse (never sent to client)
    let resp_json = r#"{
        "response_id": "resp_01",
        "session_id": "sess_01",
        "item_id": "LS-B1-01",
        "module": "LS",
        "band": "B1",
        "permutation": ["opt_3", "opt_1", "opt_2"],
        "selected_option_id": "opt_2",
        "omitted": false,
        "correct": true,
        "responseMs": 3450,
        "replayCount": 0,
        "submittedAt": "2026-09-11T12:02:00Z",
        "purpose": "locator"
    }"#;
    let resp: ObjectiveResponse = serde_json::from_str(resp_json).unwrap();
    assert!(resp.correct);
    assert_eq!(resp.response_ms, 3450);

    // 2. ProductiveRating (dual-reference)
    let rating_json = r#"{
        "rating_id": "rate_01",
        "session_id": "sess_01",
        "task_id": "SPK-B1B2-ER1",
        "module": "SPK",
        "route": "B1-B2",
        "rater": "ai",
        "model": "gemini-1.5-pro",
        "promptVersion": "speaking_rater@v2.0",
        "rubricVersion": "2026.1",
        "benchmarkSet": null,
        "usable": true,
        "unusableReason": null,
        "atLower": {"fluency": 3, "grammar": 4},
        "atUpper": {"fluency": 2, "grammar": 3},
        "transcript": "Model transcript of speaking response",
        "rationale": "Clear communicative production",
        "flags": [],
        "createdAt": "2026-09-11T12:15:00Z",
        "supersededBy": null,
        "regenerationReason": null,
        "humanReviewStatus": "none"
    }"#;
    let rating: ProductiveRating = serde_json::from_str(rating_json).unwrap();
    assert_eq!(rating.at_lower.get("fluency"), Some(&3));
    assert_eq!(rating.at_upper.get("fluency"), Some(&2));
    assert_eq!(rating.human_review_status, "none");

    // 3. ResultReport with camelCase spec fields
    let report_json = r#"{
        "session_id": "sess_01",
        "profileType": "full",
        "skills": [
            {
                "skill": "RD",
                "status": "measured",
                "band": "B2",
                "range": null,
                "notes": ["Clear reading capability"],
                "canDo": ["Can read with large degree of independence"]
            }
        ],
        "diagnostics": {
            "languageSystems": {
                "band": "B2",
                "range": null,
                "constructsStrong": ["relative clauses", "collocations"],
                "constructsWeak": ["subjunctive"]
            },
            "listenToWrite": {
                "accuracy": "exact"
            }
        },
        "headline": {
            "kind": "indicative_overall",
            "band": "B2",
            "range": null
        },
        "confidence": "Moderate",
        "confidenceReasons": ["All modules completed with consistent profiles."],
        "readiness": {
            "target": "OET",
            "text": "Profile indicates solid preparation for medical communications.",
            "disclaimer": "GEPA does not predict official exam scores."
        },
        "retestAdvice": "Recommended study interval: retest after 8-12 weeks.",
        "wordingVersion": "2.0.0-beta",
        "generatedAt": "2026-09-11T12:30:00Z"
    }"#;
    let report: ResultReport = serde_json::from_str(report_json).unwrap();
    assert_eq!(report.profile_type, "full");
    assert_eq!(report.confidence, Confidence::Moderate);
    assert_eq!(report.confidence_reasons.len(), 1);
    assert_eq!(report.diagnostics.language_systems.constructs_strong.len(), 2);
    assert_eq!(report.readiness.as_ref().unwrap().target, "OET");
}

// =========================================================================
// SPEC 05 UNIT TESTS — OBJECTIVE SCORING, PRODUCTIVE DECISIONS, ASSEMBLY & CLAIMS
// =========================================================================

#[test]
fn test_spec05_objective_scoring_and_omissions() {
    let key = RestrictedKey {
        item_id: "LS-B1-01".to_string(),
        module: "LS".to_string(),
        band: Band::B1,
        key_option_id: "opt_correct".to_string(),
        authoring_letter: "B".to_string(),
        answer_text: "Target word".to_string(),
        option_count: 4,
        evidence_focus: None,
        rationale: None,
    };

    // 1. Correct response
    let res_correct = score_response("LS-B1-01", Some("opt_correct"), &key, 4500);
    assert!(res_correct.correct);
    assert!(!res_correct.omitted);
    assert_eq!(res_correct.response_ms, 4500);

    // 2. Incorrect response
    let res_incorrect = score_response("LS-B1-01", Some("opt_wrong"), &key, 3200);
    assert!(!res_incorrect.correct);
    assert!(!res_incorrect.omitted);

    // 3. Omission (timeout / no selection)
    let res_omitted = score_response("LS-B1-01", None, &key, 45000);
    assert!(!res_omitted.correct);
    assert!(res_omitted.omitted);
}

#[test]
fn test_spec05_shuffling_deterministic_and_permutation() {
    let item = ObjectiveItem {
        item_id: "RD-B2-01".to_string(),
        module: "RD".to_string(),
        band: Band::B2,
        version: 1,
        status: "approved".to_string(),
        stem: "What does the passage imply?".to_string(),
        options: vec![
            OptionChoice { option_id: "opt_a".to_string(), text: "Alpha".to_string() },
            OptionChoice { option_id: "opt_b".to_string(), text: "Beta".to_string() },
            OptionChoice { option_id: "opt_c".to_string(), text: "Gamma".to_string() },
            OptionChoice { option_id: "opt_d".to_string(), text: "Delta".to_string() },
        ],
        construct: Some("inferencing".to_string()),
        evidence_focus: None,
        domain: Some(Domain::Educational),
        topic_family: Some("science".to_string()),
        locator_candidate: false,
        stimulus_id: None,
        is_anchor: false,
        is_pretest: false,
        accessibility_alternative: None,
        reviewer_provenance: vec![],
        exposure: None,
    };

    let session_1 = "sess_user_alpha";
    let session_2 = "sess_user_beta";

    let (shuffled_1a, perm_1a) = shuffle_options(&item, session_1);
    let (shuffled_1b, perm_1b) = shuffle_options(&item, session_1);
    let (shuffled_2, perm_2) = shuffle_options(&item, session_2);

    // Deterministic for same session + item
    assert_eq!(perm_1a, perm_1b);
    assert_eq!(shuffled_1a, shuffled_1b);

    // Permutation matches option IDs order
    let expected_perm: Vec<String> = shuffled_1a.iter().map(|o| o.option_id.clone()).collect();
    assert_eq!(perm_1a, expected_perm);

    // Different session gets a valid permutation with all 4 items preserved
    assert_eq!(shuffled_2.len(), 4);
    assert_eq!(perm_2.len(), 4);
}

#[test]
fn test_spec05_effort_flags_rapid_clicking_and_omissions() {
    // 1. Rapid clicking: >= 5 consecutive responses under 1500ms
    let mut rapid_responses = Vec::new();
    for i in 0..6 {
        rapid_responses.push(ScoringResult {
            item_id: format!("item_{}", i),
            correct: false,
            omitted: false,
            response_ms: 800,
        });
    }
    let flags = check_effort_flags(&rapid_responses);
    assert!(flags.contains(&"effort_rapid".to_string()));

    // 2. Not rapid when streak is broken
    let mixed_responses = vec![
        ScoringResult { item_id: "i1".to_string(), correct: true, omitted: false, response_ms: 800 },
        ScoringResult { item_id: "i2".to_string(), correct: true, omitted: false, response_ms: 900 },
        ScoringResult { item_id: "i3".to_string(), correct: true, omitted: false, response_ms: 4000 },
        ScoringResult { item_id: "i4".to_string(), correct: true, omitted: false, response_ms: 1100 },
    ];
    let flags_mixed = check_effort_flags(&mixed_responses);
    assert!(!flags_mixed.contains(&"effort_rapid".to_string()));

    // 3. Omissions: >= 3 omissions triggers effort_omissions
    let omission_responses = vec![
        ScoringResult { item_id: "i1".to_string(), correct: false, omitted: true, response_ms: 30000 },
        ScoringResult { item_id: "i2".to_string(), correct: false, omitted: true, response_ms: 30000 },
        ScoringResult { item_id: "i3".to_string(), correct: false, omitted: true, response_ms: 30000 },
    ];
    let flags_omiss = check_effort_flags(&omission_responses);
    assert!(flags_omiss.contains(&"effort_omissions".to_string()));
}

fn make_spk_task(task_id: &str, task_type: &str, task_group: &str, spont: bool) -> SpeakingTask {
    SpeakingTask {
        task_id: task_id.to_string(),
        route: Route::B1B2,
        route_bands: (Band::B1, Band::B2),
        task_type: task_type.to_string(),
        task_group: task_group.to_string(),
        turn: None,
        label: "Speaking Task".to_string(),
        prompt: "Speak on topic".to_string(),
        candidate_sees_text: true,
        delivery: "audio".to_string(),
        audio_script: None,
        interlocutor_line: None,
        audio: None,
        weight: 0.15,
        spontaneous_or_interactive: spont,
        traits_scored: vec!["grammar".to_string(), "vocabulary".to_string(), "communication".to_string()],
        topic_family: "education".to_string(),
        prep_seconds: 15,
        max_speak_seconds: 45,
        allows_rerecord: true,
    }
}

fn make_prod_rating(task_id: &str, lower_scores: &[(&str, u32)], upper_scores: &[(&str, u32)], usable: bool) -> ProductiveRating {
    let at_lower: HashMap<String, u32> = lower_scores.iter().map(|(k, v)| (k.to_string(), *v)).collect();
    let at_upper: HashMap<String, u32> = upper_scores.iter().map(|(k, v)| (k.to_string(), *v)).collect();
    ProductiveRating {
        rating_id: format!("rat_{}", task_id),
        session_id: "sess_01".to_string(),
        task_id: task_id.to_string(),
        module: "SPK".to_string(),
        route: Route::B1B2,
        rater: "gemini".to_string(),
        model: Some("gemini-2.5-pro".to_string()),
        prompt_version: "v1".to_string(),
        rubric_version: "2.0.0-beta".to_string(),
        benchmark_set: None,
        usable,
        unusable_reason: if usable { None } else { Some("too_short".to_string()) },
        at_lower,
        at_upper,
        transcript: None,
        rationale: "Evidence demonstrated".to_string(),
        flags: Vec::new(),
        created_at: "2026-09-09T00:00:00Z".to_string(),
        superseded_by: None,
        regeneration_reason: None,
        human_review_status: "none".to_string(),
    }
}

#[test]
fn test_spec05_speaking_decision_rules() {
    let t_fs = make_spk_task("SPK-FS", "functional_situation", "grp_fs", true);
    let t_rt = make_spk_task("SPK-RT", "retell_summarise", "grp_rt", true);
    let t_er1 = make_spk_task("SPK-ER1", "extended_response_1", "grp_er1", true);
    let t_or = make_spk_task("SPK-OR", "oral_reading", "grp_or", false); // diagnostic only

    // 1. Fewer than 3 usable spontaneous responses -> insufficient_evidence
    let r_fs = make_prod_rating("SPK-FS", &[("communication", 3)], &[("communication", 3)], true);
    let r_rt_unusable = make_prod_rating("SPK-RT", &[], &[], false);
    let r_or = make_prod_rating("SPK-OR", &[("intelligibility", 4)], &[("intelligibility", 4)], true);

    let pairs = vec![(&t_fs, &r_fs), (&t_rt, &r_rt_unusable), (&t_or, &r_or)];
    let dec_insufficient = evaluate_speaking(&pairs, Route::B1B2);
    assert_eq!(dec_insufficient.status, "insufficient_evidence");
    assert_eq!(dec_insufficient.band, None);

    // 2. Meets both lower and upper -> upper band (B2)
    let r_fs_strong = make_prod_rating("SPK-FS",
        &[("intelligibility", 4), ("fluency", 4), ("grammar", 4), ("vocabulary", 4), ("communication", 4)],
        &[("intelligibility", 3), ("fluency", 3), ("grammar", 3), ("vocabulary", 3), ("communication", 3)],
        true
    );
    let r_rt_strong = make_prod_rating("SPK-RT",
        &[("intelligibility", 4), ("fluency", 4), ("grammar", 4), ("vocabulary", 4), ("communication", 4)],
        &[("intelligibility", 3), ("fluency", 3), ("grammar", 3), ("vocabulary", 3), ("communication", 3)],
        true
    );
    let r_er1_strong = make_prod_rating("SPK-ER1",
        &[("intelligibility", 4), ("fluency", 4), ("grammar", 4), ("vocabulary", 4), ("communication", 4)],
        &[("intelligibility", 3), ("fluency", 3), ("grammar", 3), ("vocabulary", 3), ("communication", 3)],
        true
    );

    let pairs_upper = vec![(&t_fs, &r_fs_strong), (&t_rt, &r_rt_strong), (&t_er1, &r_er1_strong)];
    let dec_upper = evaluate_speaking(&pairs_upper, Route::B1B2);
    assert_eq!(dec_upper.status, "measured");
    assert_eq!(dec_upper.band, Some(Band::B2));

    // 3. Meets lower only -> lower band (B1)
    let r_fs_mid = make_prod_rating("SPK-FS",
        &[("intelligibility", 3), ("fluency", 3), ("grammar", 3), ("vocabulary", 3), ("communication", 3)],
        &[("intelligibility", 2), ("fluency", 2), ("grammar", 2), ("vocabulary", 2), ("communication", 2)],
        true
    );
    let r_rt_mid = make_prod_rating("SPK-RT",
        &[("intelligibility", 3), ("fluency", 3), ("grammar", 3), ("vocabulary", 3), ("communication", 3)],
        &[("intelligibility", 2), ("fluency", 2), ("grammar", 2), ("vocabulary", 2), ("communication", 2)],
        true
    );
    let r_er1_mid = make_prod_rating("SPK-ER1",
        &[("intelligibility", 3), ("fluency", 3), ("grammar", 3), ("vocabulary", 3), ("communication", 3)],
        &[("intelligibility", 2), ("fluency", 2), ("grammar", 2), ("vocabulary", 2), ("communication", 2)],
        true
    );
    let pairs_lower = vec![(&t_fs, &r_fs_mid), (&t_rt, &r_rt_mid), (&t_er1, &r_er1_mid)];
    let dec_lower = evaluate_speaking(&pairs_lower, Route::B1B2);
    assert_eq!(dec_lower.status, "measured");
    assert_eq!(dec_lower.band, Some(Band::B1));

    // 4. Below route (mean >= 1.75 on >= 3 spont) -> lower - 1 (A2), confidenceCap Low
    let r_fs_low = make_prod_rating("SPK-FS",
        &[("intelligibility", 2), ("fluency", 2), ("grammar", 2), ("vocabulary", 2), ("communication", 2)],
        &[("intelligibility", 1), ("fluency", 1), ("grammar", 1), ("vocabulary", 1), ("communication", 1)],
        true
    );
    let r_rt_low = make_prod_rating("SPK-RT",
        &[("intelligibility", 2), ("fluency", 2), ("grammar", 2), ("vocabulary", 2), ("communication", 2)],
        &[("intelligibility", 1), ("fluency", 1), ("grammar", 1), ("vocabulary", 1), ("communication", 1)],
        true
    );
    let r_er1_low = make_prod_rating("SPK-ER1",
        &[("intelligibility", 2), ("fluency", 2), ("grammar", 2), ("vocabulary", 2), ("communication", 2)],
        &[("intelligibility", 1), ("fluency", 1), ("grammar", 1), ("vocabulary", 1), ("communication", 1)],
        true
    );
    let pairs_below = vec![(&t_fs, &r_fs_low), (&t_rt, &r_rt_low), (&t_er1, &r_er1_low)];
    let dec_below = evaluate_speaking(&pairs_below, Route::B1B2);
    assert_eq!(dec_below.status, "measured");
    assert_eq!(dec_below.band, Some(Band::A2)); // B1 prev is A2
    assert_eq!(dec_below.confidence_cap, Some(Confidence::Low));
    assert!(dec_below.flags.contains(&"below_route".to_string()));

    // 5. Below route with thin evidence (mean < 1.75) -> insufficient_evidence
    let r_fs_fail = make_prod_rating("SPK-FS",
        &[("intelligibility", 1), ("fluency", 1), ("grammar", 1), ("vocabulary", 1), ("communication", 1)],
        &[("intelligibility", 0), ("fluency", 0), ("grammar", 0), ("vocabulary", 0), ("communication", 0)],
        true
    );
    let r_rt_fail = make_prod_rating("SPK-RT",
        &[("intelligibility", 1), ("fluency", 1), ("grammar", 1), ("vocabulary", 1), ("communication", 1)],
        &[("intelligibility", 0), ("fluency", 0), ("grammar", 0), ("vocabulary", 0), ("communication", 0)],
        true
    );
    let r_er1_fail = make_prod_rating("SPK-ER1",
        &[("intelligibility", 1), ("fluency", 1), ("grammar", 1), ("vocabulary", 1), ("communication", 1)],
        &[("intelligibility", 0), ("fluency", 0), ("grammar", 0), ("vocabulary", 0), ("communication", 0)],
        true
    );
    let pairs_thin = vec![(&t_fs, &r_fs_fail), (&t_rt, &r_rt_fail), (&t_er1, &r_er1_fail)];
    let dec_thin = evaluate_speaking(&pairs_thin, Route::B1B2);
    assert_eq!(dec_thin.status, "insufficient_evidence");
    assert_eq!(dec_thin.band, None);
}

#[test]
fn test_spec05_speaking_int_task_two_turns_count_once_for_independence() {
    // D-004: Two turns of simulated interaction share grp_int and count once for independent responses
    let t_int1 = make_spk_task("SPK-INT1", "simulated_interaction_two_turn", "grp_int", true);
    let t_int2 = make_spk_task("SPK-INT2", "simulated_interaction_two_turn", "grp_int", true);
    let t_fs = make_spk_task("SPK-FS", "functional_situation", "grp_fs", true);

    let r_int1 = make_prod_rating("SPK-INT1",
        &[("intelligibility", 4), ("fluency", 4), ("grammar", 4), ("vocabulary", 4), ("communication", 4)],
        &[("intelligibility", 3), ("fluency", 3), ("grammar", 3), ("vocabulary", 3), ("communication", 3)],
        true
    );
    let r_int2 = make_prod_rating("SPK-INT2",
        &[("intelligibility", 4), ("fluency", 4), ("grammar", 4), ("vocabulary", 4), ("communication", 4)],
        &[("intelligibility", 3), ("fluency", 3), ("grammar", 3), ("vocabulary", 3), ("communication", 3)],
        true
    );
    let r_fs_low_upper = make_prod_rating("SPK-FS",
        &[("intelligibility", 3), ("fluency", 3), ("grammar", 3), ("vocabulary", 3), ("communication", 3)],
        &[("intelligibility", 2), ("fluency", 2), ("grammar", 2), ("vocabulary", 2), ("communication", 2)],
        true
    );

    // INT1 and INT2 meet upper, but both belong to grp_int -> only 1 independent task group meets upper!
    // Therefore, upperMeets requirement (independent_upper_meets.len() >= 2) fails!
    let pairs = vec![(&t_int1, &r_int1), (&t_int2, &r_int2), (&t_fs, &r_fs_low_upper)];
    let dec = evaluate_speaking(&pairs, Route::B1B2);
    assert_eq!(dec.status, "measured");
    assert_eq!(dec.band, Some(Band::B1)); // lower meets, but upper does NOT meet due to independence rule!

    // Now if FS also meets upper, we have 2 distinct task groups (grp_int and grp_fs) -> upper meets!
    let r_fs_high_upper = make_prod_rating("SPK-FS",
        &[("intelligibility", 4), ("fluency", 4), ("grammar", 4), ("vocabulary", 4), ("communication", 4)],
        &[("intelligibility", 3), ("fluency", 3), ("grammar", 3), ("vocabulary", 3), ("communication", 3)],
        true
    );
    let pairs_both = vec![(&t_int1, &r_int1), (&t_int2, &r_int2), (&t_fs, &r_fs_high_upper)];
    let dec_upper = evaluate_speaking(&pairs_both, Route::B1B2);
    assert_eq!(dec_upper.status, "measured");
    assert_eq!(dec_upper.band, Some(Band::B2));
}

fn make_wrt_task(task_id: &str, task_type: &str, scored: bool) -> WritingTask {
    WritingTask {
        task_id: task_id.to_string(),
        route: Route::B1B2,
        route_bands: (Band::B1, Band::B2),
        task_type: task_type.to_string(),
        label: "Writing Task".to_string(),
        prompt: "Write essay".to_string(),
        target: None,
        focus: "academic".to_string(),
        word_guidance: None,
        weight: 0.3,
        scored_in_writing_level: scored,
        topic_family: "work".to_string(),
        time_limit_seconds: 300,
        audio_script: None,
        audio: None,
    }
}

#[test]
fn test_spec05_writing_decision_rules() {
    let t_fc = make_wrt_task("WRT-FC", "functional_communication", true);
    let t_ms = make_wrt_task("WRT-MS", "mediation_synthesis", true);
    let t_diag = make_wrt_task("WRT-L2W", "integrated_accuracy_diagnostic", false); // listen-to-write

    // 1. Fewer than 2 scored tasks -> insufficient_evidence
    let r_fc = make_prod_rating("WRT-FC", &[("task_fulfilment", 3)], &[("task_fulfilment", 2)], true);
    let r_diag = make_prod_rating("WRT-L2W", &[("task_fulfilment", 4)], &[("task_fulfilment", 4)], true);
    let pairs_one = vec![(&t_fc, &r_fc), (&t_diag, &r_diag)];
    let dec_insuf = evaluate_writing(&pairs_one, Route::B1B2);
    assert_eq!(dec_insuf.status, "insufficient_evidence");
    assert_eq!(dec_insuf.band, None);

    // 2. Meets upper and lower -> upper band (B2)
    let r_fc_upper = make_prod_rating("WRT-FC",
        &[("task_fulfilment", 4), ("organisation", 4), ("grammar", 4), ("vocabulary", 4), ("mechanics_register", 4)],
        &[("task_fulfilment", 3), ("organisation", 3), ("grammar", 3), ("vocabulary", 3), ("mechanics_register", 3)],
        true
    );
    let r_ms_upper = make_prod_rating("WRT-MS",
        &[("task_fulfilment", 4), ("organisation", 4), ("grammar", 4), ("vocabulary", 4), ("mechanics_register", 4)],
        &[("task_fulfilment", 3), ("organisation", 3), ("grammar", 3), ("vocabulary", 3), ("mechanics_register", 3)],
        true
    );
    let pairs_upper = vec![(&t_fc, &r_fc_upper), (&t_ms, &r_ms_upper)];
    let dec_upper = evaluate_writing(&pairs_upper, Route::B1B2);
    assert_eq!(dec_upper.status, "measured");
    assert_eq!(dec_upper.band, Some(Band::B2));

    // 3. Meets lower only -> lower band (B1)
    let r_fc_lower = make_prod_rating("WRT-FC",
        &[("task_fulfilment", 3), ("organisation", 3), ("grammar", 3), ("vocabulary", 3), ("mechanics_register", 3)],
        &[("task_fulfilment", 2), ("organisation", 2), ("grammar", 2), ("vocabulary", 2), ("mechanics_register", 2)],
        true
    );
    let r_ms_lower = make_prod_rating("WRT-MS",
        &[("task_fulfilment", 3), ("organisation", 3), ("grammar", 3), ("vocabulary", 3), ("mechanics_register", 3)],
        &[("task_fulfilment", 2), ("organisation", 2), ("grammar", 2), ("vocabulary", 2), ("mechanics_register", 2)],
        true
    );
    let pairs_lower = vec![(&t_fc, &r_fc_lower), (&t_ms, &r_ms_lower)];
    let dec_lower = evaluate_writing(&pairs_lower, Route::B1B2);
    assert_eq!(dec_lower.status, "measured");
    assert_eq!(dec_lower.band, Some(Band::B1));

    // 4. Below route (mean >= 1.75 on >= 2 tasks) -> lower - 1 (A2)
    let r_fc_low = make_prod_rating("WRT-FC",
        &[("task_fulfilment", 2), ("organisation", 2), ("grammar", 2), ("vocabulary", 2), ("mechanics_register", 2)],
        &[("task_fulfilment", 1), ("organisation", 1), ("grammar", 1), ("vocabulary", 1), ("mechanics_register", 1)],
        true
    );
    let r_ms_low = make_prod_rating("WRT-MS",
        &[("task_fulfilment", 2), ("organisation", 2), ("grammar", 2), ("vocabulary", 2), ("mechanics_register", 2)],
        &[("task_fulfilment", 1), ("organisation", 1), ("grammar", 1), ("vocabulary", 1), ("mechanics_register", 1)],
        true
    );
    let pairs_below = vec![(&t_fc, &r_fc_low), (&t_ms, &r_ms_low)];
    let dec_below = evaluate_writing(&pairs_below, Route::B1B2);
    assert_eq!(dec_below.status, "measured");
    assert_eq!(dec_below.band, Some(Band::A2));
    assert_eq!(dec_below.confidence_cap, Some(Confidence::Low));
    assert!(dec_below.flags.contains(&"below_route".to_string()));
}

#[test]
fn test_spec05_result_assembly_and_confidence_triggers() {
    let make_skill = |skill: &str, status: &str, band: Option<Band>, flags: Vec<String>| SkillInput {
        skill: skill.to_string(),
        status: status.to_string(),
        band,
        range: None,
        notes: vec![],
        flags,
    };

    let ls_diag = LanguageSystemsDiagnostic {
        band: Some(Band::B2),
        range: None,
        constructs_strong: vec!["clause structure".to_string()],
        constructs_weak: vec!["conditionals".to_string()],
    };

    // 1. Full clean profile -> Moderate confidence, headline indicative_overall (lower median)
    let skills_clean = vec![
        make_skill("RD", "measured", Some(Band::B1), vec![]),
        make_skill("LSN", "measured", Some(Band::B2), vec![]),
        make_skill("SPK", "measured", Some(Band::B2), vec![]),
        make_skill("WRT", "measured", Some(Band::B2), vec![]),
    ];
    let input_clean = AssemblyInput {
        session_id: "sess_clean".to_string(),
        profile_type: "full".to_string(),
        skills: skills_clean,
        ls_diagnostic: ls_diag.clone(),
        listen_to_write: None,
        target_goal: Some("OET".to_string()),
        session_flags: vec![],
    };
    let report_clean = assemble_result_report(input_clean).unwrap();
    assert_eq!(report_clean.confidence, Confidence::Moderate);
    assert_eq!(report_clean.headline.kind, "indicative_overall");
    assert_eq!(report_clean.headline.band, Some(Band::B2)); // lower median of [B1, B2, B2, B2] is B2
    assert!(report_clean.readiness.is_some());
    assert_eq!(report_clean.readiness.unwrap().disclaimer, "GEPA does not predict official exam scores.");

    // 2. Uneven profile (spread >= 2 bands) -> kind "uneven", range [b1, b4]
    let skills_uneven = vec![
        make_skill("RD", "measured", Some(Band::A2), vec![]),
        make_skill("LSN", "measured", Some(Band::B1), vec![]),
        make_skill("SPK", "measured", Some(Band::B2), vec![]),
        make_skill("WRT", "measured", Some(Band::C1), vec![]),
    ];
    let input_uneven = AssemblyInput {
        session_id: "sess_uneven".to_string(),
        profile_type: "full".to_string(),
        skills: skills_uneven,
        ls_diagnostic: ls_diag.clone(),
        listen_to_write: None,
        target_goal: None,
        session_flags: vec![],
    };
    let report_uneven = assemble_result_report(input_uneven).unwrap();
    assert_eq!(report_uneven.headline.kind, "uneven");
    assert_eq!(report_uneven.headline.range, Some((Band::A2, Band::C1)));

    // 3. Partial profile (foundation_receptive) -> headline "none", confidence Low
    let skills_receptive = vec![
        make_skill("RD", "measured", Some(Band::B2), vec![]),
        make_skill("LSN", "measured", Some(Band::B2), vec![]),
        make_skill("SPK", "not_measured", None, vec![]),
        make_skill("WRT", "not_measured", None, vec![]),
    ];
    let input_receptive = AssemblyInput {
        session_id: "sess_rec".to_string(),
        profile_type: "foundation_receptive".to_string(),
        skills: skills_receptive,
        ls_diagnostic: ls_diag.clone(),
        listen_to_write: None,
        target_goal: None,
        session_flags: vec![],
    };
    let report_rec = assemble_result_report(input_receptive).unwrap();
    assert_eq!(report_rec.confidence, Confidence::Low);
    assert_eq!(report_rec.headline.kind, "none");

    // 4. Test each confidence drop trigger from 05 §3.4:
    let test_flags = [
        "boundary_unresolved",
        "floor_unresolved",
        "aberrant_pattern",
        "inconsistent_pattern",
        "evidenceShortfall",
        "evidence_shortfall",
        "wideWindow",
        "wide_window",
        "productive_route_default",
        "effort_omissions",
        "effort_rapid",
        "profile_inconsistency",
        "below_route",
    ];
    for flag in test_flags {
        let skills_test = vec![
            make_skill("RD", "measured", Some(Band::B2), vec![]),
            make_skill("LSN", "measured", Some(Band::B2), vec![]),
            make_skill("SPK", "measured", Some(Band::B2), vec![]),
            make_skill("WRT", "measured", Some(Band::B2), vec![]),
        ];
        let input_flag = AssemblyInput {
            session_id: format!("sess_{}", flag),
            profile_type: "full".to_string(),
            skills: skills_test,
            ls_diagnostic: ls_diag.clone(),
            listen_to_write: None,
            target_goal: None,
            session_flags: vec![flag.to_string()],
        };
        let rep = assemble_result_report(input_flag).unwrap();
        assert_eq!(rep.confidence, Confidence::Low, "Flag '{}' must drop confidence to Low", flag);
    }

    // 5. Multiple technical replacements (>= 2) drops confidence to Low
    let input_tech = AssemblyInput {
        session_id: "sess_tech".to_string(),
        profile_type: "full".to_string(),
        skills: vec![
            make_skill("RD", "measured", Some(Band::B2), vec![]),
            make_skill("LSN", "measured", Some(Band::B2), vec![]),
            make_skill("SPK", "measured", Some(Band::B2), vec!["technical_replaced".to_string(), "technical_replaced".to_string()]),
            make_skill("WRT", "measured", Some(Band::B2), vec![]),
        ],
        ls_diagnostic: ls_diag.clone(),
        listen_to_write: None,
        target_goal: None,
        session_flags: vec![],
    };
    let rep_tech = assemble_result_report(input_tech).unwrap();
    assert_eq!(rep_tech.confidence, Confidence::Low);

    // 6. >= 2 integrity flags drops confidence to Low
    let input_integ = AssemblyInput {
        session_id: "sess_integ".to_string(),
        profile_type: "full".to_string(),
        skills: vec![
            make_skill("RD", "measured", Some(Band::B2), vec![]),
            make_skill("LSN", "measured", Some(Band::B2), vec![]),
            make_skill("SPK", "measured", Some(Band::B2), vec![]),
            make_skill("WRT", "measured", Some(Band::B2), vec![]),
        ],
        ls_diagnostic: ls_diag,
        listen_to_write: None,
        target_goal: None,
        session_flags: vec!["effort_rapid".to_string(), "ai_suspect_review".to_string()],
    };
    let rep_integ = assemble_result_report(input_integ).unwrap();
    assert_eq!(rep_integ.confidence, Confidence::Low);
}

#[test]
fn test_spec05_wording_policy_comprehensive() {
    // 1. Forbidden phrases (case-insensitive)
    let forbidden_samples = [
        "This certificate is official",
        "Your validated score is ready",
        "Your CEFR level is B2",
        "We predict an IELTS band score",
        "Guaranteed TOEFL score increase",
        "PTE score equivalency",
        "High confidence in placement",
        "Reliability of 98%",
        "Accuracy of the diagnostic algorithm",
    ];
    for sample in forbidden_samples {
        assert!(scan_candidate_copy(sample).is_err(), "Must reject forbidden wording: {}", sample);
    }

    // 2. Plus/minus levels forbidden case-insensitively
    assert!(scan_candidate_copy("Diagnosed as B1+ level").is_err());
    assert!(scan_candidate_copy("Diagnosed as b1+ level").is_err());
    assert!(scan_candidate_copy("Observed B2- performance").is_err());
    assert!(scan_candidate_copy("Observed a2+ performance").is_err());

    // 3. Percentage in confidence text forbidden
    assert!(scan_confidence_text("Confidence 95%").is_err());
    assert!(scan_confidence_text("High confidence").is_err());

    // 4. Exact mandatory negative disclaimer MUST be permitted
    assert!(scan_candidate_copy("GEPA does not predict official exam scores.").is_ok());
    assert!(scan_candidate_copy("GEPA does not predict official exam scores").is_ok());
}


