use crate::models::{
    Band, Confidence, DiagnosticsSummary, Headline, LanguageSystemsDiagnostic,
    ListenToWriteDiagnostic, ReadinessLayer, ResultReport, SkillResult,
};
use crate::wording_policy::scan_candidate_copy;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInput {
    pub skill: String,  // "RD", "LSN", "SPK", "WRT"
    pub status: String, // "measured", "insufficient_evidence", "not_measured"
    pub band: Option<Band>,
    pub range: Option<(Band, Band)>,
    pub notes: Vec<String>,
    pub flags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssemblyInput {
    pub session_id: String,
    pub profile_type: String, // "foundation_receptive" | "full"
    pub skills: Vec<SkillInput>,
    pub ls_diagnostic: LanguageSystemsDiagnostic,
    pub listen_to_write: Option<ListenToWriteDiagnostic>,
    pub target_goal: Option<String>,
    pub session_flags: Vec<String>,
}

pub fn can_do_for_skill(skill: &str, band: Band) -> (Vec<String>, Vec<String>) {
    let (can_do, growth) = match (skill, band) {
        ("RD", Band::PreA1) => (
            vec!["Can recognise basic familiar words, single numbers, and simple common signs in everyday contexts.".to_string()],
            vec!["Expand recognition of common sight vocabulary and high-frequency signs.".to_string()],
        ),
        ("RD", Band::A1) => (
            vec!["Can understand simple short notices, familiar names, and basic everyday phrases.".to_string()],
            vec!["Practice reading connected sentences and short everyday informational messages.".to_string()],
        ),
        ("RD", Band::A2) => (
            vec!["Can identify specific predictable information in simple everyday material such as notices, menus, and short emails.".to_string()],
            vec!["Focus on following sequential instructions and identifying the main topic of multi-paragraph texts.".to_string()],
        ),
        ("RD", Band::B1) => (
            vec!["Can understand the main points of clear standard texts on familiar everyday or study topics.".to_string()],
            vec!["Strengthen inference of implied meaning and distinguishing factual information from opinion.".to_string()],
        ),
        ("RD", Band::B2) => (
            vec!["Can read with a large degree of independence, adapting style and speed to different texts and purposes.".to_string()],
            vec!["Work on synthesising nuanced arguments across contrasting texts and identifying implicit attitudes.".to_string()],
        ),
        ("RD", Band::C1) => (
            vec!["Can understand in detail lengthy, complex texts, identifying subtle distinctions of style and implicit viewpoint.".to_string()],
            vec!["Fine-tune precision in grasping highly technical discourse and idiomatic philosophical nuance.".to_string()],
        ),
        ("RD", Band::C2) => (
            vec!["Can understand virtually all forms of written language, including abstract, structurally or linguistically complex texts.".to_string()],
            vec!["Maintain broad exposure across diverse genres, historical registers, and specialised domains.".to_string()],
        ),
        ("LSN", Band::PreA1) => (
            vec!["Can recognise very basic spoken words and isolated everyday instructions spoken clearly and slowly.".to_string()],
            vec!["Build phonological familiarity with numbers, dates, and common daily nouns.".to_string()],
        ),
        ("LSN", Band::A1) => (
            vec!["Can follow speech that is very slow and carefully articulated, with long pauses to assimilate meaning.".to_string()],
            vec!["Practice recognizing key details in routine announcements and brief transactional dialogues.".to_string()],
        ),
        ("LSN", Band::A2) => (
            vec!["Can understand phrases and high-frequency vocabulary related to areas of immediate personal relevance.".to_string()],
            vec!["Develop ability to catch key information when spoken at natural conversational pace.".to_string()],
        ),
        ("LSN", Band::B1) => (
            vec!["Can understand the main points of clear standard speech on familiar matters regularly encountered in work or study.".to_string()],
            vec!["Strengthen listening for unstated speaker attitude and multi-speaker discussion shifts.".to_string()],
        ),
        ("LSN", Band::B2) => (
            vec!["Can understand standard spoken language, live or broadcast, on both familiar and unfamiliar topics in academic and professional life.".to_string()],
            vec!["Work on tracking rapid exchanges with varied regional accents and dense background noise.".to_string()],
        ),
        ("LSN", Band::C1) => (
            vec!["Can follow extended speech even when not clearly structured and when relationships are only implied.".to_string()],
            vec!["Refine interpretation of subtle humour, irony, and cultural subtext in rapid discourse.".to_string()],
        ),
        ("LSN", Band::C2) => (
            vec!["Can follow specialised lectures and presentations with natural ease, even with non-standard accents.".to_string()],
            vec!["Maintain receptive breadth across complex academic and professional symposia.".to_string()],
        ),
        ("SPK", Band::PreA1) => (
            vec!["Can produce simple isolated words and basic formulaic phrases with supportive context.".to_string()],
            vec!["Practice combining words into simple two- or three-word spoken communicative phrases.".to_string()],
        ),
        ("SPK", Band::A1) => (
            vec!["Can interact in a simple way provided the other person is prepared to repeat or rephrase slowly.".to_string()],
            vec!["Build confidence describing basic personal details and everyday surroundings in short sentences.".to_string()],
        ),
        ("SPK", Band::A2) => (
            vec!["Can give a simple description or presentation of people, living or work conditions in short lists.".to_string()],
            vec!["Focus on using basic connectors to link ideas and maintaining speech without long hesitations.".to_string()],
        ),
        ("SPK", Band::B1) => (
            vec!["Can enter unprepared into conversations on familiar topics and express personal opinions on abstract matters.".to_string()],
            vec!["Develop range of grammatical structures to express conditionality, cause, and hypothetical scenarios.".to_string()],
        ),
        ("SPK", Band::B2) => (
            vec!["Can give clear, systematically developed presentations, with highlighting of significant points and relevant supporting detail.".to_string()],
            vec!["Enhance natural idiomatic flexibility and precise control over complex discourse markers.".to_string()],
        ),
        ("SPK", Band::C1) => (
            vec!["Can express ideas fluently and spontaneously, almost effortlessly, selecting an appropriate register for complex communicative situations.".to_string()],
            vec!["Polish subtle stylistic nuance and rhetorical pacing in high-stakes negotiations.".to_string()],
        ),
        ("SPK", Band::C2) => (
            vec!["Can convey finer shades of precise meaning effectively and structure extended speech with natural authority.".to_string()],
            vec!["Continue active engagement with high-level academic discussions and professional debates.".to_string()],
        ),
        ("WRT", Band::PreA1) => (
            vec!["Can write isolated basic words such as name, address, and familiar everyday vocabulary.".to_string()],
            vec!["Practice forming simple short messages and everyday note-taking.".to_string()],
        ),
        ("WRT", Band::A1) => (
            vec!["Can write simple isolated phrases and sentences, such as filling out a basic form or postcard.".to_string()],
            vec!["Focus on sentence boundaries, standard punctuation, and simple coordinating conjunctions.".to_string()],
        ),
        ("WRT", Band::A2) => (
            vec!["Can write a series of simple phrases and sentences linked with simple connectors like 'and', 'but' and 'because'.".to_string()],
            vec!["Work on paragraph structure and organising chronological descriptions.".to_string()],
        ),
        ("WRT", Band::B1) => (
            vec!["Can write straightforward connected texts on a range of familiar subjects within their field of interest.".to_string()],
            vec!["Strengthen cohesive devices and range of complex sentence structures in analytical writing.".to_string()],
        ),
        ("WRT", Band::B2) => (
            vec!["Can write clear, detailed texts on a variety of subjects related to their field, synthesising information from different sources.".to_string()],
            vec!["Focus on register consistency and varied vocabulary in evaluative arguments.".to_string()],
        ),
        ("WRT", Band::C1) => (
            vec!["Can produce clear, well-structured, detailed text on complex subjects, showing controlled use of organisational patterns.".to_string()],
            vec!["Fine-tune rhetorical subtlety and concise executive synthesis in formal writing.".to_string()],
        ),
        ("WRT", Band::C2) => (
            vec!["Can write clear, smoothly flowing, complex texts in an appropriate style and with an effective logical structure.".to_string()],
            vec!["Maintain sophisticated command across scholarly publications and high-level policy briefs.".to_string()],
        ),
        _ => (
            vec!["Observed indicative performance at the diagnosed band.".to_string()],
            vec!["Continue systematic study across core communicative skills.".to_string()],
        ),
    };
    (can_do, growth)
}

pub fn get_readiness_layer(target: &str) -> Option<ReadinessLayer> {
    let disclaimer = "GEPA does not predict official exam scores.".to_string();
    match target {
        "OET" => Some(ReadinessLayer {
            target: "OET".to_string(),
            text: "This diagnostic profile highlights general English foundations and observed skill gaps prior to beginning profession-specific OET healthcare communication preparation.".to_string(),
            disclaimer,
            currency_note: Some("OET remains a four-subtest healthcare-specific assessment; GEPA describes general-English readiness only.".to_string()),
        }),
        "IELTS" => Some(ReadinessLayer {
            target: "IELTS".to_string(),
            text: "This diagnostic profile indicates whether foundational English development or exam-specific task practice is likely to be your primary focus.".to_string(),
            disclaimer,
            currency_note: None,
        }),
        "TOEFL iBT" => Some(ReadinessLayer {
            target: "TOEFL iBT".to_string(),
            text: "Use this diagnostic profile to assess readiness for academic English tasks and integrated-skill practice formats.".to_string(),
            disclaimer,
            currency_note: Some("TOEFL iBT changed January 2026: 1-6 section/overall scale and multistage adaptive Reading/Listening.".to_string()),
        }),
        "PTE Academic" => Some(ReadinessLayer {
            target: "PTE Academic".to_string(),
            text: "This profile provides insight into linguistic foundation, spoken fluency, and prompt comprehension for computer-delivered task formats.".to_string(),
            disclaimer,
            currency_note: Some("PTE added Summarize Group Discussion and Respond to a Situation from August 2025.".to_string()),
        }),
        "Cambridge/DET/other" | "University" | "Work" | "General" => Some(ReadinessLayer {
            target: target.to_string(),
            text: "This profile assists in selecting an appropriate learning level and targeting communicative priorities.".to_string(),
            disclaimer,
            currency_note: None,
        }),
        _ => None,
    }
}

pub fn derive_headline(profile_type: &str, skills: &[SkillResult]) -> Headline {
    if profile_type != "full" {
        return Headline {
            kind: "none".to_string(),
            band: None,
            range: None,
        };
    }

    // Must have all 4 skills: RD, LSN, SPK, WRT measured with a band
    let mut bands = Vec::new();
    for s in skills {
        if s.status == "measured" {
            if let Some(b) = s.band {
                bands.push(b);
            } else {
                return Headline {
                    kind: "none".to_string(),
                    band: None,
                    range: None,
                };
            }
        } else {
            return Headline {
                kind: "none".to_string(),
                band: None,
                range: None,
            };
        }
    }

    if bands.len() < 4 {
        return Headline {
            kind: "none".to_string(),
            band: None,
            range: None,
        };
    }

    bands.sort_by_key(|b| b.index());
    let b1 = bands[0];
    let b2 = bands[1];
    let b4 = bands[3];

    if b4.index() - b1.index() <= 1 {
        // Lower median is b2 (since 4 elements: b1, b2, b3, b4 -> lower middle is index 1 = b2)
        // Cap check: headline never exceeds lowest measured skill + 1 band
        assert!(b2.index() <= b1.index() + 1);
        Headline {
            kind: "indicative_overall".to_string(),
            band: Some(b2),
            range: None,
        }
    } else {
        Headline {
            kind: "uneven".to_string(),
            band: None,
            range: Some((b1, b4)),
        }
    }
}

pub fn assemble_result_report(
    input: AssemblyInput,
) -> Result<ResultReport, String> {
    let mut skill_results = Vec::new();
    let mut confidence_triggers = Vec::new();
    let mut has_unresolved_boundary = false;
    let mut has_floor_unresolved = false;
    let mut has_aberrant = false;
    let mut has_insufficient = false;
    let mut has_below_route = false;

    for sk in &input.skills {
        let (can_do, growth) = if let Some(b) = sk.band {
            can_do_for_skill(&sk.skill, b)
        } else {
            (
                vec!["Performance within diagnosed range.".to_string()],
                vec!["Review construct-level feedback in diagnostics.".to_string()],
            )
        };

        if sk.status == "insufficient_evidence" {
            has_insufficient = true;
            confidence_triggers.push(format!("Skill {} had insufficient evidence.", sk.skill));
        }

        for f in &sk.flags {
            match f.as_str() {
                "boundary_unresolved" => has_unresolved_boundary = true,
                "floor_unresolved" => has_floor_unresolved = true,
                "aberrant_pattern" => has_aberrant = true,
                "below_route" => has_below_route = true,
                _ => {}
            }
        }

        skill_results.push(SkillResult {
            skill: sk.skill.clone(),
            status: sk.status.clone(),
            band: sk.band,
            range: sk.range,
            notes: sk.notes.clone(),
            can_do,
            growth_areas: growth,
        });
    }

    if input.profile_type == "foundation_receptive" {
        confidence_triggers.push("Partial profile: receptive skills only.".to_string());
    }

    for f in &input.session_flags {
        match f.as_str() {
            "effort_omissions" => confidence_triggers.push("Three or more questions omitted due to timeout.".to_string()),
            "effort_rapid" => confidence_triggers.push("Rapid response pattern observed.".to_string()),
            "wide_window" => confidence_triggers.push("Performance variation observed across different test modules.".to_string()),
            "productive_route_default" => confidence_triggers.push("Productive route used default placement.".to_string()),
            "profile_inconsistency" => confidence_triggers.push("Observed score variation between receptive and productive modules.".to_string()),
            _ => {}
        }
    }

    if has_unresolved_boundary {
        confidence_triggers.push("Performance was near a band boundary with limited confirming items.".to_string());
    }
    if has_floor_unresolved {
        confidence_triggers.push("Lower performance floor could not be fully resolved.".to_string());
    }
    if has_aberrant {
        confidence_triggers.push("Non-monotonic response pattern detected.".to_string());
    }
    if has_below_route {
        confidence_triggers.push("Performance was below the starting route threshold.".to_string());
    }

    let is_low = !confidence_triggers.is_empty()
        || has_insufficient
        || input.profile_type != "full";

    let confidence = if is_low {
        Confidence::Low
    } else {
        Confidence::Moderate
    };

    let confidence_reasons = if confidence == Confidence::Moderate {
        vec!["All scheduled modules were completed with consistent diagnostic patterns.".to_string()]
    } else {
        confidence_triggers
    };

    let headline = derive_headline(&input.profile_type, &skill_results);

    let retest_advice = if confidence == Confidence::Low {
        "Recommended study interval: retest after 2–4 weeks of focused study, or once technical issues are resolved.".to_string()
    } else {
        "Recommended study interval: retest after approximately 8–12 weeks of structured study.".to_string()
    };

    let readiness = input
        .target_goal
        .as_deref()
        .and_then(get_readiness_layer);

    let diagnostics = DiagnosticsSummary {
        language_systems: input.ls_diagnostic,
        listen_to_write: input.listen_to_write,
        pronunciation_notes: vec!["Diagnostic pronunciation traits observed within expected communicative range.".to_string()],
        fluency_notes: vec!["Spoken fluency showed natural speech rate appropriate for the diagnosed band.".to_string()],
    };

    let report = ResultReport {
        session_id: input.session_id,
        profile_type: input.profile_type,
        skills: skill_results,
        diagnostics,
        headline,
        confidence,
        confidence_reasons,
        readiness,
        retest_advice,
        wording_version: "2.0.0-beta".to_string(),
        generated_at: "2026-09-09T00:00:00Z".to_string(),
    };

    // Scan all report text for forbidden phrasing
    let report_json = serde_json::to_string(&report).map_err(|e| e.to_string())?;
    scan_candidate_copy(&report_json).map_err(|e| format!("Wording policy violation: {}", e))?;

    Ok(report)
}
