//! `TtsProvider` interface (docs/12_AUDIO_PRODUCTION.md §2) — one real,
//! zero-cost implementation (`FliteProvider`, via ffmpeg's bundled `flite`
//! voices) that runs today with nothing but what's already on this machine,
//! and one auto-activating upgrade path (`GeminiTtsProvider`) for when a
//! real `GEMINI_API_KEY` exists. See docs/DECISIONS.md D-019 for why the
//! default voice cast covers only American + Scottish accents.

use crate::ffmpeg;
use serde_json::json;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Segment {
    pub speaker: String,
    pub text: String,
    pub voice: String,
}

pub struct SynthesisRequest<'a> {
    pub asset_id: &'a str,
    pub segments: &'a [Segment],
}

pub trait TtsProvider {
    fn name(&self) -> &'static str;
    /// Synthesize into `workdir/out_wav` (48kHz mono WAV, unprocessed — the
    /// shared pipeline in `main.rs` does silence trim/pad/loudnorm/encode
    /// afterward regardless of which provider produced the raw audio).
    fn synthesize(&self, workdir: &Path, req: &SynthesisRequest, out_wav: &str) -> Result<(), String>;
}

/// Default provider: ffmpeg's bundled `flite` voices. Zero cost, zero
/// account, works offline. Four voices available on this build: `kal16`/
/// `rms` (US English, male), `slt` (US English, female), `awb` (Scottish
/// English, male) — see `main.rs::voice_for` for the rotation plan.
pub struct FliteProvider;

impl TtsProvider for FliteProvider {
    fn name(&self) -> &'static str {
        "flite"
    }

    fn synthesize(&self, workdir: &Path, req: &SynthesisRequest, out_wav: &str) -> Result<(), String> {
        let mut parts = Vec::new();
        for (i, seg) in req.segments.iter().enumerate() {
            let seg_txt = format!("seg_{i}.txt");
            let seg_wav = format!("seg_{i}.wav");
            ffmpeg::synthesize_segment(workdir, &seg.text, &seg.voice, &seg_txt, &seg_wav)?;
            parts.push(seg_wav);
            if i + 1 < req.segments.len() {
                let pause_wav = format!("pause_{i}.wav");
                ffmpeg::make_silence(workdir, 0.7, &pause_wav)?;
                parts.push(pause_wav);
            }
        }
        ffmpeg::concat_wavs(workdir, &parts, out_wav)
    }
}

/// Auto-selected instead of `FliteProvider` when `GEMINI_API_KEY` is set
/// (see `select_provider`). Written to the documented Gemini multi-speaker
/// TTS request/response shape, in the same `reqwest`-based style as
/// `server/src/ai/client.rs` — but this session has no key, so the exact
/// wire format is **unverified against a live call** (logged in
/// docs/QUESTIONS.md rather than claimed as tested).
pub struct GeminiTtsProvider {
    api_key: String,
    model: String,
    client: reqwest::blocking::Client,
}

impl GeminiTtsProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            api_key,
            model,
            client: reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(60))
                .build()
                .unwrap_or_else(|_| reqwest::blocking::Client::new()),
        }
    }
}

impl TtsProvider for GeminiTtsProvider {
    fn name(&self) -> &'static str {
        "gemini"
    }

    fn synthesize(&self, workdir: &Path, req: &SynthesisRequest, out_wav: &str) -> Result<(), String> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let full_text = req
            .segments
            .iter()
            .map(|s| format!("{}: {}", s.speaker, s.text))
            .collect::<Vec<_>>()
            .join("\n");

        let speaker_voice_configs: Vec<_> = {
            let mut seen = std::collections::HashSet::new();
            req.segments
                .iter()
                .filter(|s| seen.insert(s.speaker.clone()))
                .map(|s| {
                    json!({
                        "speaker": s.speaker,
                        "voiceConfig": { "prebuiltVoiceConfig": { "voiceName": gemini_voice_name(&s.voice) } }
                    })
                })
                .collect()
        };

        let body = json!({
            "contents": [{ "role": "user", "parts": [{ "text": full_text }] }],
            "generationConfig": {
                "responseModalities": ["AUDIO"],
                "speechConfig": { "multiSpeakerVoiceConfig": { "speakerVoiceConfigs": speaker_voice_configs } }
            }
        });

        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .map_err(|e| format!("Gemini TTS request failed: {e}"))?;
        if !resp.status().is_success() {
            return Err(format!("Gemini TTS HTTP {}", resp.status()));
        }
        let json_resp: serde_json::Value = resp.json().map_err(|e| format!("Gemini TTS response decode failed: {e}"))?;
        let inline = json_resp["candidates"][0]["content"]["parts"][0]["inlineData"]["data"]
            .as_str()
            .ok_or("Gemini TTS response missing inlineData.data")?;

        use base64::Engine;
        let pcm = base64::engine::general_purpose::STANDARD
            .decode(inline)
            .map_err(|e| format!("Gemini TTS base64 decode failed: {e}"))?;
        std::fs::write(workdir.join(format!("{}.pcm", req.asset_id)), &pcm).map_err(|e| e.to_string())?;

        // Gemini TTS returns raw 24kHz 16-bit mono PCM (no WAV header) —
        // wrap it with ffmpeg into a proper WAV for the shared pipeline.
        std::process::Command::new("ffmpeg")
            .current_dir(workdir)
            .args([
                "-y", "-f", "s16le", "-ar", "24000", "-ac", "1",
                "-i", &format!("{}.pcm", req.asset_id),
                "-ar", "48000", "-ac", "1", out_wav,
            ])
            .output()
            .map_err(|e| format!("failed to wrap Gemini PCM into WAV: {e}"))?;
        Ok(())
    }
}

/// Maps this pipeline's internal flite voice ids onto Gemini's prebuilt
/// voice names (documentation-only mapping — never exercised live).
fn gemini_voice_name(local_voice: &str) -> &'static str {
    match local_voice {
        "kal16" | "rms" => "Charon",
        "slt" => "Kore",
        "awb" => "Puck",
        _ => "Charon",
    }
}

/// Chooses the real Gemini provider when a key is configured, otherwise the
/// offline default. This is the only place provider selection happens.
pub fn select_provider() -> Box<dyn TtsProvider> {
    let key = std::env::var("GEMINI_API_KEY").unwrap_or_default();
    if !key.is_empty() {
        let model = std::env::var("MODEL_TTS").unwrap_or_else(|_| "gemini-2.5-flash".to_string());
        return Box::new(GeminiTtsProvider::new(key, model));
    }
    Box::new(FliteProvider)
}
