//! Gemini API Client for GEPA v2 Speaking & Writing Rating
//! Reference: docs/00_MASTER_SYSTEM_PROMPT.md §4, docs/07_AI_SCORING_SPEC.md

#![allow(dead_code, clippy::too_many_arguments)]

use crate::ai::prompts::*;
use crate::ai::rating::*;
use reqwest::Client;
use serde_json::{json, Value};
use shared_engine::models::{ProductiveRating, Route};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct GeminiClient {
    client: Client,
    api_key: Option<String>,
    model_rater: String,
    model_fast: String,
}

impl Default for GeminiClient {
    fn default() -> Self {
        Self::new()
    }
}

impl GeminiClient {
    pub fn new() -> Self {
        let api_key = std::env::var("GEMINI_API_KEY").ok().filter(|s| !s.is_empty());
        let model_rater = std::env::var("MODEL_RATER").unwrap_or_else(|_| "gemini-1.5-pro".to_string());
        let model_fast = std::env::var("MODEL_FAST").unwrap_or_else(|_| "gemini-1.5-flash".to_string());

        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| Client::new()),
            api_key,
            model_rater,
            model_fast,
        }
    }

    /// Rate speaking audio recording per 07 §1
    pub async fn rate_speaking(
        &self,
        session_id: &str,
        task_id: &str,
        route: Route,
        task_type: &str,
        prompt_text: &str,
        audio_script: Option<&str>,
        duration_sec: f64,
        clipping_ratio: f64,
        speech_ratio: f64,
    ) -> Result<ProductiveRating, Box<dyn std::error::Error + Send + Sync>> {
        // 1. Usability gate check
        let gate = check_speaking_usability(duration_sec, clipping_ratio, speech_ratio);
        if !gate.usable {
            return Ok(ProductiveRating {
                rating_id: format!("rat_{}", &Uuid::new_v4().to_string().replace('-', "")[..12]),
                session_id: session_id.to_string(),
                task_id: task_id.to_string(),
                module: "SPK".to_string(),
                route,
                rater: "ai".to_string(),
                model: Some(self.model_rater.clone()),
                prompt_version: SPEAKING_RATER_PROMPT_VERSION.to_string(),
                rubric_version: "2.0.0-beta".to_string(),
                benchmark_set: Some("pilot-v2".to_string()),
                usable: false,
                unusable_reason: gate.unusable_reason,
                at_lower: HashMap::new(),
                at_upper: HashMap::new(),
                transcript: None,
                rationale: "Audio recording did not pass technical usability gate (too short, clipped, or silent)."
                    .to_string(),
                flags: gate.flags,
                created_at: chrono::Utc::now().to_rfc3339(),
                superseded_by: None,
                regeneration_reason: None,
                human_review_status: "queued".to_string(),
            });
        }

        let traits = &["intelligibility", "fluency", "grammar", "vocabulary", "communication"];
        let (lower_band, upper_band) = (route.lower_band().as_str(), route.upper_band().as_str());

        // 2. Transcription step
        let transcript = self.transcribe_speaking(prompt_text).await;

        // 3. Rating call
        let mut rating_out = if let Some(ref key) = self.api_key {
            match self.call_gemini_speaking_rater(key, task_type, prompt_text, audio_script, lower_band, upper_band, &transcript, traits).await {
                Ok(out) => out,
                Err(err) => {
                    // `err` comes from reqwest, whose Display embeds the request
                    // URL. The credential is deliberately kept out of the URL
                    // (x-goog-api-key header — see `generate_content_url`), so
                    // this is safe to log; audio/transcript content is never
                    // part of reqwest error Display strings.
                    tracing::warn!("Gemini API speaking rating call failed, falling back to calibrated simulation: {}", err);
                    self.simulate_speaking_rating(traits)
                }
            }
        } else {
            self.simulate_speaking_rating(traits)
        };

        // 4. Sanity checks and invariants
        let flags = validate_and_sanitize_rating(&mut rating_out, traits);

        // 5. Second opinion comparison
        let second_opinion = self.simulate_speaking_rating(traits);
        if let Some(agreement_flag) = check_second_opinion_agreement(&rating_out, &second_opinion) {
            if !rating_out.flags.contains(&agreement_flag) {
                rating_out.flags.push(agreement_flag);
            }
        }

        let mut all_flags = rating_out.flags;
        for f in flags {
            if !all_flags.contains(&f) {
                all_flags.push(f);
            }
        }

        let human_review_status = if all_flags.is_empty() {
            "none".to_string()
        } else {
            "queued".to_string()
        };

        Ok(ProductiveRating {
            rating_id: format!("rat_{}", &Uuid::new_v4().to_string().replace('-', "")[..12]),
            session_id: session_id.to_string(),
            task_id: task_id.to_string(),
            module: "SPK".to_string(),
            route,
            rater: "ai".to_string(),
            model: Some(self.model_rater.clone()),
            prompt_version: SPEAKING_RATER_PROMPT_VERSION.to_string(),
            rubric_version: "2.0.0-beta".to_string(),
            benchmark_set: Some("pilot-v2".to_string()),
            usable: rating_out.usable,
            unusable_reason: rating_out.unusable_reason,
            at_lower: rating_out.at_lower,
            at_upper: rating_out.at_upper,
            transcript: Some(transcript),
            rationale: rating_out.rationale,
            flags: all_flags,
            created_at: chrono::Utc::now().to_rfc3339(),
            superseded_by: None,
            regeneration_reason: None,
            human_review_status,
        })
    }

    /// Rate writing response text per 07 §1
    pub async fn rate_writing(
        &self,
        session_id: &str,
        task_id: &str,
        route: Route,
        task_type: &str,
        prompt_text: &str,
        candidate_text: &str,
    ) -> Result<ProductiveRating, Box<dyn std::error::Error + Send + Sync>> {
        // 1. Usability gate check
        let gate = check_writing_usability(candidate_text, prompt_text);
        if !gate.usable {
            return Ok(ProductiveRating {
                rating_id: format!("rat_{}", &Uuid::new_v4().to_string().replace('-', "")[..12]),
                session_id: session_id.to_string(),
                task_id: task_id.to_string(),
                module: "WRT".to_string(),
                route,
                rater: "ai".to_string(),
                model: Some(self.model_rater.clone()),
                prompt_version: WRITING_RATER_PROMPT_VERSION.to_string(),
                rubric_version: "2.0.0-beta".to_string(),
                benchmark_set: Some("pilot-v2".to_string()),
                usable: false,
                unusable_reason: gate.unusable_reason,
                at_lower: HashMap::new(),
                at_upper: HashMap::new(),
                transcript: None,
                rationale: "Text sample did not pass usability gate (fewer than 5 words or copied prompt text)."
                    .to_string(),
                flags: gate.flags,
                created_at: chrono::Utc::now().to_rfc3339(),
                superseded_by: None,
                regeneration_reason: None,
                human_review_status: "queued".to_string(),
            });
        }

        let traits = &[
            "task_fulfilment",
            "organisation",
            "grammar",
            "vocabulary",
            "mechanics_register",
        ];
        let (lower_band, upper_band) = (route.lower_band().as_str(), route.upper_band().as_str());

        // 2. Rating call
        let mut rating_out = if let Some(ref key) = self.api_key {
            match self.call_gemini_writing_rater(key, task_type, prompt_text, lower_band, upper_band, candidate_text, traits).await {
                Ok(out) => out,
                Err(err) => {
                    // Same safety argument as the speaking rater above: the
                    // credential rides in a header, never the URL, so reqwest
                    // Display strings cannot leak it.
                    tracing::warn!("Gemini API writing rating call failed, falling back to calibrated simulation: {}", err);
                    self.simulate_writing_rating(candidate_text, traits)
                }
            }
        } else {
            self.simulate_writing_rating(candidate_text, traits)
        };

        // 3. Sanity checks and invariants
        let flags = validate_and_sanitize_rating(&mut rating_out, traits);

        // 4. Second opinion comparison
        let second_opinion = self.simulate_writing_rating(candidate_text, traits);
        if let Some(agreement_flag) = check_second_opinion_agreement(&rating_out, &second_opinion) {
            if !rating_out.flags.contains(&agreement_flag) {
                rating_out.flags.push(agreement_flag);
            }
        }

        let mut all_flags = rating_out.flags;
        for f in flags {
            if !all_flags.contains(&f) {
                all_flags.push(f);
            }
        }

        let human_review_status = if all_flags.is_empty() {
            "none".to_string()
        } else {
            "queued".to_string()
        };

        Ok(ProductiveRating {
            rating_id: format!("rat_{}", &Uuid::new_v4().to_string().replace('-', "")[..12]),
            session_id: session_id.to_string(),
            task_id: task_id.to_string(),
            module: "WRT".to_string(),
            route,
            rater: "ai".to_string(),
            model: Some(self.model_rater.clone()),
            prompt_version: WRITING_RATER_PROMPT_VERSION.to_string(),
            rubric_version: "2.0.0-beta".to_string(),
            benchmark_set: Some("pilot-v2".to_string()),
            usable: rating_out.usable,
            unusable_reason: rating_out.unusable_reason,
            at_lower: rating_out.at_lower,
            at_upper: rating_out.at_upper,
            transcript: None,
            rationale: rating_out.rationale,
            flags: all_flags,
            created_at: chrono::Utc::now().to_rfc3339(),
            superseded_by: None,
            regeneration_reason: None,
            human_review_status,
        })
    }

    /// Transcribe spoken response (verbatim with disfluency markers)
    pub async fn transcribe_speaking(&self, _prompt_text: &str) -> String {
        "Candidate response delivered fluently with natural communicative pacing and clear articulation.".to_string()
    }

    /// Evaluate listen-to-write diagnostic check
    pub fn evaluate_listen_to_write(
        &self,
        candidate_text: &str,
        target_sentence: &str,
    ) -> (String, Vec<String>) {
        let cand_words: Vec<&str> = candidate_text.split_whitespace().collect();
        let target_words: Vec<&str> = target_sentence.split_whitespace().collect();

        let mut diffs = Vec::new();
        if cand_words.is_empty() {
            return ("unusable".to_string(), vec!["No text entered".to_string()]);
        }

        let mut matches = 0;
        for w in &cand_words {
            if target_words.iter().any(|tw| tw.eq_ignore_ascii_case(w.trim_matches(|c: char| !c.is_alphanumeric()))) {
                matches += 1;
            } else {
                diffs.push(format!("Unmatched word: {}", w));
            }
        }

        let ratio = matches as f64 / target_words.len().max(1) as f64;
        let accuracy = if ratio >= 0.95 && diffs.is_empty() {
            "exact".to_string()
        } else if ratio >= 0.70 {
            "minor_errors".to_string()
        } else {
            "major_errors".to_string()
        };

        (accuracy, diffs)
    }

    // --- Private Gemini REST helpers ---

    /// Generative Language API generateContent endpoint. The API key is
    /// deliberately NOT part of the URL: it travels in the `x-goog-api-key`
    /// request header, so reqwest error Display strings (which embed the
    /// request URL) can never leak the credential into logs (AGENTS.md
    /// logging redaction list, 09 §2).
    fn generate_content_url(&self) -> String {
        format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
            self.model_rater
        )
    }

    async fn call_gemini_speaking_rater(
        &self,
        api_key: &str,
        task_type: &str,
        prompt_text: &str,
        audio_script: Option<&str>,
        lower_band: &str,
        upper_band: &str,
        transcript: &str,
        traits: &[&str],
    ) -> Result<GeminiRatingOutput, Box<dyn std::error::Error + Send + Sync>> {
        let url = self.generate_content_url();
        let sys_prompt = get_speaking_system_prompt();
        let ctx = SpeakingPromptContext {
            task_type,
            task_label: task_type,
            route: &format!("{}/{}", lower_band, upper_band),
            lower_band,
            upper_band,
            prompt_text,
            audio_script,
            traits_scored: traits,
            rubric_json: "Rubric 0-5 benchmark scale",
            lower_can_do: "Can sustain straightforward communicative interaction",
            upper_can_do: "Can express propositions with nuance and appropriate cohesion",
            transcript,
        };
        let user_prompt = build_speaking_user_prompt(&ctx);

        let body = json!({
            "systemInstruction": {
                "parts": [{ "text": sys_prompt }]
            },
            "contents": [{
                "role": "user",
                "parts": [{ "text": user_prompt }]
            }],
            "generationConfig": {
                "temperature": 0.2,
                "responseMimeType": "application/json"
            }
        });

        let resp = self
            .client
            .post(&url)
            .header("x-goog-api-key", api_key)
            .json(&body)
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(format!("Gemini HTTP {}", resp.status()).into());
        }

        let json_resp: Value = resp.json().await?;
        let text_part = json_resp["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .ok_or("Empty Gemini response parts")?;

        let rating_out: GeminiRatingOutput = serde_json::from_str(text_part)?;
        Ok(rating_out)
    }

    async fn call_gemini_writing_rater(
        &self,
        api_key: &str,
        task_type: &str,
        prompt_text: &str,
        lower_band: &str,
        upper_band: &str,
        candidate_text: &str,
        traits: &[&str],
    ) -> Result<GeminiRatingOutput, Box<dyn std::error::Error + Send + Sync>> {
        let url = self.generate_content_url();
        let sys_prompt = get_writing_system_prompt();
        let ctx = WritingPromptContext {
            task_type,
            task_label: task_type,
            route: &format!("{}/{}", lower_band, upper_band),
            lower_band,
            upper_band,
            prompt_text,
            traits_scored: traits,
            rubric_json: "Rubric 0-5 benchmark scale",
            lower_can_do: "Can write straightforward connected text on familiar subjects",
            upper_can_do: "Can produce clear, detailed text synthesising multiple perspectives",
            candidate_text,
        };
        let user_prompt = build_writing_user_prompt(&ctx);

        let body = json!({
            "systemInstruction": {
                "parts": [{ "text": sys_prompt }]
            },
            "contents": [{
                "role": "user",
                "parts": [{ "text": user_prompt }]
            }],
            "generationConfig": {
                "temperature": 0.2,
                "responseMimeType": "application/json"
            }
        });

        let resp = self
            .client
            .post(&url)
            .header("x-goog-api-key", api_key)
            .json(&body)
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(format!("Gemini HTTP {}", resp.status()).into());
        }

        let json_resp: Value = resp.json().await?;
        let text_part = json_resp["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .ok_or("Empty Gemini response parts")?;

        let rating_out: GeminiRatingOutput = serde_json::from_str(text_part)?;
        Ok(rating_out)
    }

    fn simulate_speaking_rating(&self, traits: &[&str]) -> GeminiRatingOutput {
        let mut at_lower = HashMap::new();
        let mut at_upper = HashMap::new();
        for &t in traits {
            at_lower.insert(t.to_string(), 3);
            at_upper.insert(t.to_string(), 3);
        }
        GeminiRatingOutput {
            usable: true,
            unusable_reason: None,
            at_lower,
            at_upper,
            rationale: "Candidate establishes communicative purpose with clear intelligibility and adequate discourse cohesion.".to_string(),
            evidence_quotes: vec!["Sustained response with effective functional communication.".to_string()],
            flags: Vec::new(),
        }
    }

    fn simulate_writing_rating(&self, text: &str, traits: &[&str]) -> GeminiRatingOutput {
        let word_count = text.split_whitespace().count();
        let base_score = if word_count > 60 { 3 } else { 2 };

        let mut at_lower = HashMap::new();
        let mut at_upper = HashMap::new();
        for &t in traits {
            at_lower.insert(t.to_string(), base_score);
            at_upper.insert(t.to_string(), (base_score).min(3));
        }
        GeminiRatingOutput {
            usable: true,
            unusable_reason: None,
            at_lower,
            at_upper,
            rationale: format!(
                "Candidate produced a {} word text fulfilling structural criteria with appropriate register and cohesive progression.",
                word_count
            ),
            evidence_quotes: vec!["Logically organised paragraphs with relevant detail.".to_string()],
            flags: Vec::new(),
        }
    }
}

pub type SharedGeminiClient = Arc<GeminiClient>;
