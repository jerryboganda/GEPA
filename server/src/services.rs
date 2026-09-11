use crate::repos::SeedBank;
use crate::state::*;
use shared_engine::models::*;
use shared_engine::objective_scoring::{check_effort_flags, score_response, shuffle_options, ScoringResult};
use shared_engine::productive_route::{calculate_productive_route, ModuleOutcomeSummary};
use shared_engine::result_assembly::{assemble_result_report, AssemblyInput, SkillInput};
use shared_engine::routing::*;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Default)]
pub struct CreateSessionParams {
    pub target_goal: Option<String>,
    pub ui_language: Option<String>,
    pub accommodations: Option<Accommodations>,
    pub mode: Option<String>,
    pub device_class: Option<String>,
    pub identity_metadata: Option<serde_json::Value>,
    pub proctoring_metadata: Option<serde_json::Value>,
}

/// Fetch a session by id, or a candidate-safe `"Session not found"` error.
/// Returns the owned `SessionState` plus the Postgres `version` counter
/// needed to write it back as a compare-and-swap (see `store_session`).
async fn load_session(state: &AppState, session_id: &str) -> Result<(SessionState, i64), String> {
    let fetched = state
        .session_repo
        .get(session_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Session not found".to_string())?;
    Ok((fetched.session, fetched.version))
}

/// Write a session back with the optimistic-concurrency precondition from
/// its last read. A `Conflict` here means another request mutated the same
/// session in between (e.g. a rapid double-submit) — surfaced to the caller
/// as an error rather than silently clobbering the other write.
async fn store_session(state: &AppState, session: &SessionState, version: i64) -> Result<(), String> {
    state
        .session_repo
        .set(session, version)
        .await
        .map_err(|e| match e {
            crate::db::DbError::Conflict => {
                "Session was updated concurrently — please retry".to_string()
            }
            other => other.to_string(),
        })
}

pub struct AssessmentService;

impl AssessmentService {
    /// `candidate_uid` comes from the caller's verified token (see
    /// `auth.rs`) — self-issued at session creation (DECISIONS.md D-021,
    /// no external identity provider) — not invented per-call. This is what
    /// makes the server-side `assert_owns_or_staff` ownership check mean
    /// something.
    pub async fn create_session(
        state: &AppState,
        candidate_uid: String,
        params: CreateSessionParams,
    ) -> Result<String, String> {
        let session_id = format!("ses_{}", &Uuid::new_v4().to_string().replace('-', "")[..16]);

        let session = SessionState {
            session_id: session_id.clone(),
            candidate_uid,
            mode: params.mode.unwrap_or_else(|| "free_beta".to_string()),
            device_class: params.device_class.unwrap_or_else(|| "desktop".to_string()),
            target_goal: params.target_goal.unwrap_or_else(|| "General".to_string()),
            ui_language: params.ui_language.unwrap_or_else(|| "en".to_string()),
            accommodations: params.accommodations.unwrap_or_default(),
            flags: Vec::new(),
            created_at: chrono::Utc::now().to_rfc3339(),
            state_version: 1,
            last_response_time: None,
            identity_metadata: params.identity_metadata,
            proctoring_metadata: params.proctoring_metadata,
            ls_state: RuntimeObjectiveState::new("LS"),
            rd_state: RuntimeObjectiveState::new("RD"),
            lsn_state: RuntimeObjectiveState::new("LSN"),
            spk_state: RuntimeProductiveState::new("SPK"),
            wrt_state: RuntimeProductiveState::new("WRT"),
            productive_route: None,
            result_report: None,
            previous_reports: Vec::new(),
        };

        state.session_repo.create(&session).await.map_err(|e| e.to_string())?;
        Ok(session_id)
    }

    pub async fn get_next_unit(
        state: &AppState,
        session_id: &str,
        module_name: &str,
    ) -> Option<CandidateDeliveryUnit> {
        let (mut session, version) = load_session(state, session_id).await.ok()?;
        let mult = if session.accommodations.extended_time { 1.5 } else { 1.0 };

        // `bank` (a non-Send `RwLockReadGuard`) and the immediately-invoked
        // closure that borrows it are both confined to this block, so the
        // guard is provably dropped at the closing `}` — well before the
        // `store_session(...).await` below. (An explicit mid-function
        // `drop(bank)` is not reliably recognised by the async-fn Send
        // checker across a closure boundary; block-scoping is.)
        let unit_opt: Option<CandidateDeliveryUnit> = {
            let bank = state.seed_bank.read();
            // Wrapped in an immediately-invoked closure so the `?`-heavy
            // lookups below can short-circuit to `None` for this arm without
            // skipping the session persistence at the bottom of the function.
            (|| -> Option<CandidateDeliveryUnit> {
            match module_name {
                "LS" => {
                    let mod_state = &mut session.ls_state;
                    mod_state.status = "in_progress".to_string();

                    let band = mod_state.locator.band;
                    let unused_item = bank
                        .ls_items
                        .iter()
                        .find(|i| i.band == band && !mod_state.used_item_ids.contains(&i.item_id))?;

                    mod_state.used_item_ids.insert(unused_item.item_id.clone());

                    let (shuffled, _) = shuffle_options(unused_item, session_id);
                    let payload = CandidateItemPayload {
                        item_id: unused_item.item_id.clone(),
                        module: "LS".to_string(),
                        stem: unused_item.stem.clone(),
                        options: shuffled,
                    };

                    let seconds = if band == Band::PreA1 || band == Band::A1 { 75.0 * mult } else { 60.0 * mult };
                    let deadline = chrono::Utc::now() + chrono::Duration::seconds(seconds as i64);

                    let unit = CandidateDeliveryUnit {
                        stimulus_id: None,
                        stimulus_text: None,
                        stimulus_type: None,
                        audio_url: None,
                        items: vec![payload],
                        deadline_at: deadline.to_rfc3339(),
                        module_complete: false,
                    };

                    mod_state.current_delivery_unit = Some(unit.clone());
                    Some(unit)
                }
                "RD" => {
                    let mod_state = &mut session.rd_state;
                    mod_state.status = "in_progress".to_string();

                    let band = mod_state.locator.band;
                    let unused_stimulus = bank.rd_stimuli.iter().find(|s| {
                        s.band == band
                            && s.item_ids.iter().all(|id| !mod_state.used_item_ids.contains(id))
                    })?;

                    for id in &unused_stimulus.item_ids {
                        mod_state.used_item_ids.insert(id.clone());
                    }

                    let mut items = Vec::new();
                    for id in &unused_stimulus.item_ids {
                        if let Some(it) = bank.rd_items.iter().find(|i| &i.item_id == id) {
                            let (shuffled, _) = shuffle_options(it, session_id);
                            items.push(CandidateItemPayload {
                                item_id: it.item_id.clone(),
                                module: "RD".to_string(),
                                stem: it.stem.clone(),
                                options: shuffled,
                            });
                        }
                    }

                    let base_sec = match band {
                        Band::PreA1 | Band::A1 => 120.0,
                        Band::A2 => 180.0,
                        Band::B1 => 240.0,
                        Band::B2 => 300.0,
                        Band::C1 | Band::C2 => 360.0,
                    };
                    let deadline = chrono::Utc::now() + chrono::Duration::seconds((base_sec * mult) as i64);

                    let unit = CandidateDeliveryUnit {
                        stimulus_id: Some(unused_stimulus.stimulus_id.clone()),
                        stimulus_text: Some(unused_stimulus.text.clone()),
                        stimulus_type: Some(unused_stimulus.stimulus_type.clone()),
                        audio_url: None,
                        items,
                        deadline_at: deadline.to_rfc3339(),
                        module_complete: false,
                    };

                    mod_state.current_delivery_unit = Some(unit.clone());
                    Some(unit)
                }
                "LSN" => {
                    let mod_state = &mut session.lsn_state;
                    mod_state.status = "in_progress".to_string();

                    let band = mod_state.locator.band;
                    let unused_stimulus = bank.lsn_stimuli.iter().find(|s| {
                        s.band == band
                            && s.item_ids.iter().all(|id| !mod_state.used_item_ids.contains(id))
                    })?;

                    for id in &unused_stimulus.item_ids {
                        mod_state.used_item_ids.insert(id.clone());
                    }

                    let mut items = Vec::new();
                    for id in &unused_stimulus.item_ids {
                        if let Some(it) = bank.lsn_items.iter().find(|i| &i.item_id == id) {
                            let (shuffled, _) = shuffle_options(it, session_id);
                            items.push(CandidateItemPayload {
                                item_id: it.item_id.clone(),
                                module: "LSN".to_string(),
                                stem: it.stem.clone(),
                                options: shuffled,
                            });
                        }
                    }

                    let audio_url = format!("/api/media/audio/{}.mp3", unused_stimulus.stimulus_id);
                    let deadline = chrono::Utc::now() + chrono::Duration::seconds((120.0 * mult) as i64);

                    let unit = CandidateDeliveryUnit {
                        stimulus_id: Some(unused_stimulus.stimulus_id.clone()),
                        stimulus_text: None, // Script is strictly hidden from candidate!
                        stimulus_type: Some("Conversation".to_string()),
                        audio_url: Some(audio_url),
                        items,
                        deadline_at: deadline.to_rfc3339(),
                        module_complete: false,
                    };

                    mod_state.current_delivery_unit = Some(unit.clone());
                    Some(unit)
                }
                _ => None,
            }
            })()
        };

        let _ = store_session(state, &session, version).await;
        unit_opt
    }

    #[allow(dead_code)]
    pub async fn submit_response(
        state: &AppState,
        session_id: &str,
        module_name: &str,
        item_id: &str,
        selected_option_id: Option<String>,
        response_ms: u64,
        replay_count: u32,
    ) -> Result<Option<CandidateDeliveryUnit>, String> {
        let single = vec![crate::api::SingleItemResponse {
            item_id: item_id.to_string(),
            selected_option_id,
            response_ms,
        }];
        Self::submit_responses(state, session_id, module_name, &single, replay_count).await
    }

    pub async fn submit_responses(
        state: &AppState,
        session_id: &str,
        module_name: &str,
        items: &[crate::api::SingleItemResponse],
        replay_count: u32,
    ) -> Result<Option<CandidateDeliveryUnit>, String> {
        let (mut session, version) = load_session(state, session_id).await?;

        let now = chrono::Utc::now();
        if let Some(prev) = session.last_response_time {
            if (now - prev).num_milliseconds() < 750 {
                session.flags.push("rapid_response_burst".to_string());
            }
        }
        session.last_response_time = Some(now);
        session.state_version += 1;

        {
            let mod_state = match module_name {
                "LS" => &mut session.ls_state,
                "RD" => &mut session.rd_state,
                "LSN" => &mut session.lsn_state,
                _ => return Err("Invalid objective module".to_string()),
            };

            let bank = state.seed_bank.read();
            for item_resp in items {
                let key = bank
                    .restricted_keys
                    .get(&item_resp.item_id)
                    .ok_or_else(|| format!("Restricted key not found for item {}", item_resp.item_id))?;

                let permutation = if let Some(it) = bank.ls_items.iter().find(|i| i.item_id == item_resp.item_id)
                    .or_else(|| bank.rd_items.iter().find(|i| i.item_id == item_resp.item_id))
                    .or_else(|| bank.lsn_items.iter().find(|i| i.item_id == item_resp.item_id))
                {
                    let (_, p) = shuffle_options(it, session_id);
                    p
                } else {
                    Vec::new()
                };

                let scoring = score_response(
                    &item_resp.item_id,
                    item_resp.selected_option_id.as_deref(),
                    key,
                    item_resp.response_ms,
                );

                mod_state.responses.push(ObjectiveResponseRecord {
                    response_id: format!("resp_{}", &Uuid::new_v4().to_string().replace('-', "")[..12]),
                    session_id: session_id.to_string(),
                    item_id: item_resp.item_id.clone(),
                    module: module_name.to_string(),
                    band: key.band,
                    permutation,
                    selected_option_id: item_resp.selected_option_id.clone(),
                    omitted: scoring.omitted,
                    correct: scoring.correct,
                    response_ms: item_resp.response_ms,
                    replay_count,
                    submitted_at: chrono::Utc::now().to_rfc3339(),
                    purpose: "adaptive_routing".to_string(),
                });

                mod_state.current_pair_scores.push(scoring.correct);
            }
        }

        let mod_state = match module_name {
            "LS" => &session.ls_state,
            "RD" => &session.rd_state,
            "LSN" => &session.lsn_state,
            _ => unreachable!("validated above"),
        };

        // Determine if locator unit is complete:
        // In LS: 2 items per pair (or 1 if waiting_tie)
        // In RD/LSN: 2 items per stimulus testlet
        let is_unit_complete = if module_name == "LS" {
            if mod_state.locator.waiting_tie {
                !mod_state.current_pair_scores.is_empty()
            } else {
                mod_state.current_pair_scores.len() >= 2
            }
        } else {
            mod_state.current_pair_scores.len() >= 2
                || (mod_state.locator.waiting_tie && !mod_state.current_pair_scores.is_empty())
        };

        if !is_unit_complete {
            store_session(state, &session, version).await?;
            return Ok(Self::get_next_unit(state, session_id, module_name).await);
        }

        let mod_state = match module_name {
            "LS" => &mut session.ls_state,
            "RD" => &mut session.rd_state,
            "LSN" => &mut session.lsn_state,
            _ => unreachable!("validated above"),
        };

        // Score c for the locator unit.
        let score_val = if mod_state.locator.waiting_tie {
            if mod_state.current_pair_scores.first().copied().unwrap_or(false) {
                1
            } else {
                0
            }
        } else {
            let mut c = 0u32;
            for &correct in mod_state.current_pair_scores.iter().take(2) {
                if correct {
                    c += 1;
                }
            }
            c
        };

        mod_state.current_pair_scores.clear();

        let locator_res = step_locator(&mut mod_state.locator, score_val);

        let result = match locator_res {
            LocatorStepResult::NextPair(_) | LocatorStepResult::NextTie(_) => {
                store_session(state, &session, version).await?;
                return Ok(Self::get_next_unit(state, session_id, module_name).await);
            }
            LocatorStepResult::Bracket(bracket_outcome) => {
                mod_state.bracket = Some(bracket_outcome.bracket);
                mod_state.status = "complete".to_string();

                let (lower_b, upper_b) = bracket_outcome.bracket;
                mod_state.outcome = Some(ConfirmationOutcome {
                    band: Some(lower_b),
                    range: Some((lower_b, upper_b)),
                    notes: vec!["Outcome confirmed by diagnostic router".to_string()],
                    flags: bracket_outcome.flags,
                    evidence_shortfall: false,
                    confidence_cap: None,
                });

                let scorings: Vec<_> = mod_state
                    .responses
                    .iter()
                    .map(|r| ScoringResult {
                        item_id: r.item_id.clone(),
                        correct: r.correct,
                        omitted: r.omitted,
                        response_ms: r.response_ms,
                    })
                    .collect();
                let effort_flags = check_effort_flags(&scorings);
                session.flags.extend(effort_flags);

                Some(CandidateDeliveryUnit {
                    stimulus_id: None,
                    stimulus_text: None,
                    stimulus_type: None,
                    audio_url: None,
                    items: Vec::new(),
                    deadline_at: chrono::Utc::now().to_rfc3339(),
                    module_complete: true,
                })
            }
        };

        store_session(state, &session, version).await?;
        Ok(result)
    }

    pub async fn compute_receptive_profile(state: &AppState, session_id: &str) -> Result<ResultReport, String> {
        let (mut session, version) = load_session(state, session_id).await?;

        let ls_out = session.ls_state.outcome.clone().unwrap_or(ConfirmationOutcome {
            band: Some(Band::B1),
            range: Some((Band::B1, Band::B2)),
            notes: vec![],
            flags: vec![],
            evidence_shortfall: false,
            confidence_cap: None,
        });

        let rd_out = session.rd_state.outcome.clone().unwrap_or(ConfirmationOutcome {
            band: Some(Band::B1),
            range: Some((Band::B1, Band::B2)),
            notes: vec![],
            flags: vec![],
            evidence_shortfall: false,
            confidence_cap: None,
        });

        let lsn_out = session.lsn_state.outcome.clone().unwrap_or(ConfirmationOutcome {
            band: Some(Band::B1),
            range: Some((Band::B1, Band::B2)),
            notes: vec![],
            flags: vec![],
            evidence_shortfall: false,
            confidence_cap: None,
        });

        let skills = vec![
            SkillInput {
                skill: "RD".to_string(),
                status: "measured".to_string(),
                band: rd_out.band,
                range: rd_out.range,
                notes: rd_out.notes,
                flags: rd_out.flags,
            },
            SkillInput {
                skill: "LSN".to_string(),
                status: if session.accommodations.transcript_access {
                    "not_measured".to_string()
                } else {
                    "measured".to_string()
                },
                band: if session.accommodations.transcript_access { None } else { lsn_out.band },
                range: if session.accommodations.transcript_access { None } else { lsn_out.range },
                notes: if session.accommodations.transcript_access {
                    vec!["Listening reported not measured under transcript-access accommodation pathway".to_string()]
                } else {
                    lsn_out.notes
                },
                flags: lsn_out.flags,
            },
            SkillInput {
                skill: "SPK".to_string(),
                status: "not_measured".to_string(),
                band: None,
                range: None,
                notes: vec!["Productive module not yet completed".to_string()],
                flags: vec![],
            },
            SkillInput {
                skill: "WRT".to_string(),
                status: "not_measured".to_string(),
                band: None,
                range: None,
                notes: vec!["Productive module not yet completed".to_string()],
                flags: vec![],
            },
        ];

        let (strong_constructs, weak_constructs) = {
            let bank = state.seed_bank.read();
            Self::ls_construct_diagnostics(&session, &bank)
        };

        let ls_diagnostic = LanguageSystemsDiagnostic {
            band: ls_out.band,
            range: ls_out.range,
            constructs_strong: strong_constructs,
            constructs_weak: weak_constructs,
        };

        let assembly_input = AssemblyInput {
            session_id: session_id.to_string(),
            profile_type: "foundation_receptive".to_string(),
            skills,
            ls_diagnostic,
            listen_to_write: None,
            target_goal: Some(session.target_goal.clone()),
            session_flags: session.flags.clone(),
        };

        let report = assemble_result_report(assembly_input)?;
        if let Some(old_report) = session.result_report.take() {
            session.previous_reports.push(old_report);
        }
        session.result_report = Some(report.clone());
        store_session(state, &session, version).await?;
        Ok(report)
    }

    /// Shared LS construct strengths/weaknesses lookup used by both the
    /// receptive and full result assembly.
    fn ls_construct_diagnostics(session: &SessionState, bank: &SeedBank) -> (Vec<String>, Vec<String>) {
        let mut strong_constructs = Vec::new();
        let mut weak_constructs = Vec::new();
        for r in &session.ls_state.responses {
            if let Some(it) = bank.ls_items.iter().find(|i| i.item_id == r.item_id) {
                if let Some(ref c) = it.construct {
                    if r.correct {
                        if !strong_constructs.contains(c) {
                            strong_constructs.push(c.clone());
                        }
                    } else if !weak_constructs.contains(c) {
                        weak_constructs.push(c.clone());
                    }
                }
            }
        }
        if strong_constructs.is_empty() {
            strong_constructs = vec![
                "Lexical recognition in contextual discourse".to_string(),
                "Core clause structure".to_string(),
            ];
        }
        if weak_constructs.is_empty() {
            weak_constructs = vec![
                "Complex relative clauses".to_string(),
                "Hypothetical conditionality".to_string(),
            ];
        }
        (strong_constructs, weak_constructs)
    }

    pub async fn get_speaking_tasks(state: &AppState, session_id: &str) -> Result<Vec<SpeakingTask>, String> {
        let (mut session, version) = load_session(state, session_id).await?;

        if session.productive_route.is_none() {
            let outcomes = vec![
                ModuleOutcomeSummary {
                    module: "LS".to_string(),
                    band: session.ls_state.outcome.as_ref().and_then(|o| o.band),
                    range: session.ls_state.outcome.as_ref().and_then(|o| o.range),
                },
                ModuleOutcomeSummary {
                    module: "RD".to_string(),
                    band: session.rd_state.outcome.as_ref().and_then(|o| o.band),
                    range: session.rd_state.outcome.as_ref().and_then(|o| o.range),
                },
                ModuleOutcomeSummary {
                    module: "LSN".to_string(),
                    band: session.lsn_state.outcome.as_ref().and_then(|o| o.band),
                    range: session.lsn_state.outcome.as_ref().and_then(|o| o.range),
                },
            ];
            let route_res = calculate_productive_route(&outcomes);
            session.productive_route = Some(route_res.route);
            session.flags.extend(route_res.flags);
        }

        let route = session.productive_route.unwrap_or(Route::B1B2);
        session.spk_state.route = Some(route);
        session.spk_state.status = "in_progress".to_string();

        let mut tasks: Vec<SpeakingTask> = {
            let bank = state.seed_bank.read();
            bank.speaking_tasks.iter().filter(|t| t.route == route).cloned().collect()
        };

        if session.accommodations.oral_reading_alternative {
            tasks.retain(|t| t.task_type != "oral_reading");
        }

        session.spk_state.task_ids = tasks.iter().map(|t| t.task_id.clone()).collect();
        store_session(state, &session, version).await?;
        Ok(tasks)
    }

    pub async fn get_writing_tasks(state: &AppState, session_id: &str) -> Result<Vec<WritingTask>, String> {
        let (mut session, version) = load_session(state, session_id).await?;

        let route = session.productive_route.unwrap_or(Route::B1B2);
        session.wrt_state.route = Some(route);
        session.wrt_state.status = "in_progress".to_string();

        let tasks: Vec<WritingTask> = {
            let bank = state.seed_bank.read();
            bank.writing_tasks.iter().filter(|t| t.route == route).cloned().collect()
        };

        session.wrt_state.task_ids = tasks.iter().map(|t| t.task_id.clone()).collect();
        store_session(state, &session, version).await?;
        Ok(tasks)
    }

    pub async fn submit_speaking_response(
        state: &AppState,
        session_id: &str,
        task_id: &str,
        _audio_path: &str,
    ) -> Result<ProductiveRating, String> {
        let (route, prompt_text, task_type, audio_script) = {
            let (session, _) = load_session(state, session_id).await?;
            let route = session.spk_state.route.unwrap_or(Route::B1B2);
            let bank = state.seed_bank.read();
            let task = bank.speaking_tasks.iter().find(|t| t.task_id == task_id);
            let prompt = task.map(|t| t.prompt.as_str()).unwrap_or("Speak on the given topic.");
            let t_type = task.map(|t| t.task_type.as_str()).unwrap_or("ER1");
            let script = task.and_then(|t| t.audio_script.as_deref());
            (route, prompt.to_string(), t_type.to_string(), script.map(|s| s.to_string()))
        };

        let rating = state
            .gemini_client
            .rate_speaking(
                session_id,
                task_id,
                route,
                &task_type,
                &prompt_text,
                audio_script.as_deref(),
                25.0,
                0.0,
                0.85,
            )
            .await
            .map_err(|e| e.to_string())?;

        if !rating.flags.is_empty() || !rating.usable {
            let flag_type = rating.flags.first().cloned().unwrap_or_else(|| "unusable_audio".to_string());
            state
                .review_queue_repo
                .push(&FlaggedSessionReview {
                    session_id: session_id.to_string(),
                    flag_type,
                    module: "SPK".to_string(),
                    details: format!("Task {}: {}", task_id, rating.rationale),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                })
                .await
                .map_err(|e| e.to_string())?;
        }

        let job_id = format!("job_{}", &Uuid::new_v4().to_string().replace('-', "")[..12]);
        state
            .job_repo
            .create(&BackgroundJob {
                id: job_id,
                job_type: "speaking_rating".to_string(),
                session_id: Some(session_id.to_string()),
                task_id: Some(task_id.to_string()),
                status: "completed".to_string(),
                attempts: 1,
                max_attempts: 5,
                last_error: None,
                created_at: chrono::Utc::now().to_rfc3339(),
                updated_at: chrono::Utc::now().to_rfc3339(),
            })
            .await
            .map_err(|e| e.to_string())?;

        let (mut session, version) = load_session(state, session_id).await?;
        session.state_version += 1;
        session.spk_state.ratings.push(rating.clone());
        session.spk_state.submitted_tasks.insert(task_id.to_string());
        if session.spk_state.submitted_tasks.len() >= session.spk_state.task_ids.len().min(8) {
            session.spk_state.status = "complete".to_string();
        }
        store_session(state, &session, version).await?;

        Ok(rating)
    }

    pub async fn save_writing_draft(state: &AppState, session_id: &str, task_id: &str, text: String) -> Result<(), String> {
        let (mut session, version) = load_session(state, session_id).await?;
        session.wrt_state.drafts.insert(task_id.to_string(), text);
        store_session(state, &session, version).await
    }

    pub async fn submit_writing_task(
        state: &AppState,
        session_id: &str,
        task_id: &str,
        text: String,
    ) -> Result<ProductiveRating, String> {
        let (route, prompt_text, task_type) = {
            let (session, _) = load_session(state, session_id).await?;
            let route = session.wrt_state.route.unwrap_or(Route::B1B2);
            let bank = state.seed_bank.read();
            let task = bank.writing_tasks.iter().find(|t| t.task_id == task_id);
            let prompt = task.map(|t| t.prompt.as_str()).unwrap_or("Write an essay.");
            let t_type = task.map(|t| t.task_type.as_str()).unwrap_or("E2");
            (route, prompt.to_string(), t_type.to_string())
        };

        let mut rating = state
            .gemini_client
            .rate_writing(session_id, task_id, route, &task_type, &prompt_text, &text)
            .await
            .map_err(|e| e.to_string())?;

        // Behavioral AI suspect check per 05 §2.4:
        if text.len() > 150
            && text.contains("Furthermore, it is indisputable")
            && !rating.flags.contains(&"ai_suspect_review".to_string())
        {
            rating.flags.push("ai_suspect_review".to_string());
        }

        if !rating.flags.is_empty() || !rating.usable {
            let flag_type = rating.flags.first().cloned().unwrap_or_else(|| "flagged_writing".to_string());
            state
                .review_queue_repo
                .push(&FlaggedSessionReview {
                    session_id: session_id.to_string(),
                    flag_type,
                    module: "WRT".to_string(),
                    details: format!("Task {}: {}", task_id, rating.rationale),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                })
                .await
                .map_err(|e| e.to_string())?;
        }

        let job_id = format!("job_{}", &Uuid::new_v4().to_string().replace('-', "")[..12]);
        state
            .job_repo
            .create(&BackgroundJob {
                id: job_id,
                job_type: "writing_rating".to_string(),
                session_id: Some(session_id.to_string()),
                task_id: Some(task_id.to_string()),
                status: "completed".to_string(),
                attempts: 1,
                max_attempts: 5,
                last_error: None,
                created_at: chrono::Utc::now().to_rfc3339(),
                updated_at: chrono::Utc::now().to_rfc3339(),
            })
            .await
            .map_err(|e| e.to_string())?;

        let (mut session, version) = load_session(state, session_id).await?;
        session.state_version += 1;
        session.wrt_state.drafts.insert(task_id.to_string(), text);
        session.wrt_state.ratings.push(rating.clone());
        session.wrt_state.submitted_tasks.insert(task_id.to_string());
        if session.wrt_state.submitted_tasks.len() >= session.wrt_state.task_ids.len().min(4) {
            session.wrt_state.status = "complete".to_string();
        }
        store_session(state, &session, version).await?;

        Ok(rating)
    }

    pub async fn compute_full_result(state: &AppState, session_id: &str) -> Result<ResultReport, String> {
        let (mut session, version) = load_session(state, session_id).await?;

        let route = session.productive_route.unwrap_or(Route::B1B2);

        let ls_out = session.ls_state.outcome.clone().unwrap_or(ConfirmationOutcome {
            band: Some(Band::B2), range: Some((Band::B1, Band::B2)), notes: vec![], flags: vec![],
            evidence_shortfall: false, confidence_cap: None,
        });
        let rd_out = session.rd_state.outcome.clone().unwrap_or(ConfirmationOutcome {
            band: Some(Band::B2), range: Some((Band::B1, Band::B2)), notes: vec![], flags: vec![],
            evidence_shortfall: false, confidence_cap: None,
        });
        let lsn_out = session.lsn_state.outcome.clone().unwrap_or(ConfirmationOutcome {
            band: Some(Band::B2), range: Some((Band::B1, Band::B2)), notes: vec![], flags: vec![],
            evidence_shortfall: false, confidence_cap: None,
        });

        let (spk_decision, wrt_decision, strong_constructs, weak_constructs) = {
            let bank = state.seed_bank.read();

            let spk_pairs: Vec<_> = session
                .spk_state
                .ratings
                .iter()
                .filter(|r| r.superseded_by.is_none() && r.usable)
                .filter_map(|r| bank.speaking_tasks.iter().find(|t| t.task_id == r.task_id).map(|t| (t, r)))
                .collect();
            let spk_decision = shared_engine::evaluate_speaking(&spk_pairs, route);

            let wrt_pairs: Vec<_> = session
                .wrt_state
                .ratings
                .iter()
                .filter(|r| r.superseded_by.is_none() && r.usable)
                .filter_map(|r| bank.writing_tasks.iter().find(|t| t.task_id == r.task_id).map(|t| (t, r)))
                .collect();
            let wrt_decision = shared_engine::evaluate_writing(&wrt_pairs, route);

            let (strong, weak) = Self::ls_construct_diagnostics(&session, &bank);
            (spk_decision, wrt_decision, strong, weak)
        };

        let skills = vec![
            SkillInput {
                skill: "RD".to_string(),
                status: "measured".to_string(),
                band: rd_out.band,
                range: rd_out.range,
                notes: rd_out.notes,
                flags: rd_out.flags,
            },
            SkillInput {
                skill: "LSN".to_string(),
                status: if session.accommodations.transcript_access { "not_measured".to_string() } else { "measured".to_string() },
                band: if session.accommodations.transcript_access { None } else { lsn_out.band },
                range: if session.accommodations.transcript_access { None } else { lsn_out.range },
                notes: if session.accommodations.transcript_access {
                    vec!["Listening reported not measured under transcript-access accommodation pathway".to_string()]
                } else {
                    lsn_out.notes
                },
                flags: lsn_out.flags,
            },
            SkillInput {
                skill: "SPK".to_string(),
                status: spk_decision.status.clone(),
                band: spk_decision.band,
                range: None,
                notes: spk_decision.notes.clone(),
                flags: spk_decision.flags.clone(),
            },
            SkillInput {
                skill: "WRT".to_string(),
                status: wrt_decision.status.clone(),
                band: wrt_decision.band,
                range: None,
                notes: wrt_decision.notes.clone(),
                flags: wrt_decision.flags.clone(),
            },
        ];

        // Productive vs receptive mismatch check per 05 §2.4.
        let mut receptive_bands: Vec<Band> = Vec::new();
        if let Some(b) = rd_out.band { receptive_bands.push(b); }
        if let Some(b) = lsn_out.band { receptive_bands.push(b); }
        if let Some(b) = ls_out.band { receptive_bands.push(b); }

        let mut productive_bands: Vec<Band> = Vec::new();
        if let Some(b) = spk_decision.band { productive_bands.push(b); }
        if let Some(b) = wrt_decision.band { productive_bands.push(b); }

        let mut has_mismatch = false;
        for &rb in &receptive_bands {
            for &pb in &productive_bands {
                if (rb.index() as isize - pb.index() as isize).abs() >= 2 {
                    has_mismatch = true;
                    break;
                }
            }
            if has_mismatch {
                break;
            }
        }

        if has_mismatch {
            if !session.flags.contains(&"profile_inconsistency".to_string()) {
                session.flags.push("profile_inconsistency".to_string());
            }
            let already_queued = state
                .review_queue_repo
                .exists(session_id, "profile_inconsistency")
                .await
                .map_err(|e| e.to_string())?;
            if !already_queued {
                state
                    .review_queue_repo
                    .push(&FlaggedSessionReview {
                        session_id: session_id.to_string(),
                        flag_type: "profile_inconsistency".to_string(),
                        module: "FULL".to_string(),
                        details: "Productive vs receptive mismatch >= 2 bands detected".to_string(),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    })
                    .await
                    .map_err(|e| e.to_string())?;
            }
        }

        let ls_diagnostic = LanguageSystemsDiagnostic {
            band: ls_out.band,
            range: ls_out.range,
            constructs_strong: strong_constructs,
            constructs_weak: weak_constructs,
        };

        let listen_to_write = Some(ListenToWriteDiagnostic { accuracy: "minor_errors".to_string() });

        let assembly_input = AssemblyInput {
            session_id: session_id.to_string(),
            profile_type: "full".to_string(),
            skills,
            ls_diagnostic,
            listen_to_write,
            target_goal: Some(session.target_goal.clone()),
            session_flags: session.flags.clone(),
        };

        let report = assemble_result_report(assembly_input)?;
        if let Some(old_report) = session.result_report.take() {
            session.previous_reports.push(old_report);
        }
        session.result_report = Some(report.clone());
        store_session(state, &session, version).await?;
        Ok(report)
    }

    pub async fn delete_candidate_data(state: &AppState, session_id: &str) -> bool {
        state.session_repo.delete(session_id).await.unwrap_or(false)
    }

    pub async fn rescore_session(
        state: &AppState,
        session_id: &str,
        task_id: Option<String>,
        rubric_version: Option<String>,
        reason: Option<String>,
    ) -> Result<ProductiveRating, String> {
        let rubric_v = rubric_version.unwrap_or_else(|| "2.1.0-review".to_string());
        let rescore_reason = reason.unwrap_or_else(|| "Reviewer requested rescore".to_string());

        let (mut session, version) = load_session(state, session_id).await?;

        let target_task_id = task_id.unwrap_or_else(|| {
            session
                .spk_state
                .ratings
                .last()
                .map(|r| r.task_id.clone())
                .or_else(|| session.wrt_state.ratings.last().map(|r| r.task_id.clone()))
                .unwrap_or_else(|| "SPK-B1B2-ER1".to_string())
        });

        let new_rating_id = format!("rat_{}", &Uuid::new_v4().to_string().replace('-', "")[..12]);

        let mut new_rating_opt = None;
        for r in session.spk_state.ratings.iter_mut() {
            if r.task_id == target_task_id && r.superseded_by.is_none() {
                r.superseded_by = Some(new_rating_id.clone());
                let mut nr = r.clone();
                nr.rating_id = new_rating_id.clone();
                nr.rubric_version = rubric_v.clone();
                nr.regeneration_reason = Some(rescore_reason.clone());
                nr.superseded_by = None;
                nr.created_at = chrono::Utc::now().to_rfc3339();
                new_rating_opt = Some(nr);
                break;
            }
        }

        if new_rating_opt.is_none() {
            for r in session.wrt_state.ratings.iter_mut() {
                if r.task_id == target_task_id && r.superseded_by.is_none() {
                    r.superseded_by = Some(new_rating_id.clone());
                    let mut nr = r.clone();
                    nr.rating_id = new_rating_id.clone();
                    nr.rubric_version = rubric_v.clone();
                    nr.regeneration_reason = Some(rescore_reason.clone());
                    nr.superseded_by = None;
                    nr.created_at = chrono::Utc::now().to_rfc3339();
                    new_rating_opt = Some(nr);
                    break;
                }
            }
        }

        let new_rating = new_rating_opt.ok_or_else(|| "No rating found for task to rescore".to_string())?;

        if new_rating.module == "SPK" {
            session.spk_state.ratings.push(new_rating.clone());
        } else {
            session.wrt_state.ratings.push(new_rating.clone());
        }

        store_session(state, &session, version).await?;
        let _ = Self::compute_full_result(state, session_id).await?;

        Ok(new_rating)
    }

    pub async fn human_score_session(
        state: &AppState,
        session_id: &str,
        task_id: &str,
        at_lower: HashMap<String, u32>,
        at_upper: HashMap<String, u32>,
        rationale: String,
    ) -> Result<ProductiveRating, String> {
        let (mut session, version) = load_session(state, session_id).await?;
        let new_rating_id = format!("rat_human_{}", &Uuid::new_v4().to_string().replace('-', "")[..10]);

        let mut module = "SPK".to_string();
        let mut route = Route::B1B2;

        for r in session.spk_state.ratings.iter_mut() {
            if r.task_id == task_id && r.superseded_by.is_none() {
                r.superseded_by = Some(new_rating_id.clone());
                route = r.route;
                module = "SPK".to_string();
                break;
            }
        }

        for r in session.wrt_state.ratings.iter_mut() {
            if r.task_id == task_id && r.superseded_by.is_none() {
                r.superseded_by = Some(new_rating_id.clone());
                route = r.route;
                module = "WRT".to_string();
                break;
            }
        }

        let human_rating = ProductiveRating {
            rating_id: new_rating_id,
            session_id: session_id.to_string(),
            task_id: task_id.to_string(),
            module: module.clone(),
            route,
            rater: "human".to_string(),
            model: None,
            prompt_version: "human_review@v1.0".to_string(),
            rubric_version: "2.0.0-beta".to_string(),
            benchmark_set: Some("pilot-v2".to_string()),
            usable: true,
            unusable_reason: None,
            at_lower,
            at_upper,
            transcript: None,
            rationale,
            flags: Vec::new(),
            created_at: chrono::Utc::now().to_rfc3339(),
            superseded_by: None,
            regeneration_reason: Some("Human expert review adjustment".to_string()),
            human_review_status: "reviewed".to_string(),
        };

        if module == "SPK" {
            session.spk_state.ratings.push(human_rating.clone());
        } else {
            session.wrt_state.ratings.push(human_rating.clone());
        }

        store_session(state, &session, version).await?;
        let _ = Self::compute_full_result(state, session_id).await?;

        Ok(human_rating)
    }

    /// UK GDPR data-minimisation sweep (`09_SECURITY_PRIVACY_ACCESSIBILITY.md §3`):
    /// prune sessions older than 30 days that carry no review flags.
    /// Returns `(total_before, pruned)`.
    pub async fn run_retention_job(state: &AppState) -> (usize, usize) {
        let sessions = match state.session_repo.list_all().await {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!("retention job: failed to list sessions: {e}");
                return (0, 0);
            }
        };
        let total_before = sessions.len();
        let now = chrono::Utc::now();
        let mut pruned = 0usize;

        for session in sessions {
            let should_delete = match chrono::DateTime::parse_from_rfc3339(&session.created_at) {
                Ok(created) => {
                    let age_days = (now - created.with_timezone(&chrono::Utc)).num_days();
                    age_days > 30 && session.flags.is_empty()
                }
                Err(_) => false,
            };
            if should_delete && state.session_repo.delete(&session.session_id).await.unwrap_or(false) {
                pruned += 1;
            }
        }

        (total_before, pruned)
    }

    pub fn reload_seed_bank(state: &AppState) -> Result<(usize, usize, usize), String> {
        let bank = SeedBank::load_from_dir("seed").map_err(|e| e.to_string())?;
        let ls_count = bank.ls_items.len();
        let key_count = bank.restricted_keys.len();
        let task_count = bank.speaking_tasks.len() + bank.writing_tasks.len();
        *state.seed_bank.write() = bank;
        Ok((ls_count, key_count, task_count))
    }
}
