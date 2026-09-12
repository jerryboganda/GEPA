# QUESTIONS.md — items that need the product owner

Format: `Q-### · <title>` — Blocked? (yes/no) · What I did meanwhile · Options · Recommended · Answer (owner fills in).
Never stop the build for a non-blocking question; use the conservative default and log it here.

## Q-001 · TTS provider for beta audio
Blocked: no (Gemini TTS default works with `GEMINI_API_KEY`). Options: (a) Gemini multi-speaker TTS only; (b) ElevenLabs
adapter with an existing voice cast for British/Australian accent rotation; (c) both, ElevenLabs for listening, Gemini for
prompts. Recommended: (a) for pilot; add (b) if reviewers judge accent range insufficient. Answer:

## Q-002 · Confirm pinned Gemini model IDs
Blocked: no. The agent runs `models.list` at M1 and pins `MODEL_RATER` (Pro-class, multimodal audio, structured output),
`MODEL_FAST` (Flash-class), `MODEL_TTS`; owner confirms cost/limits are acceptable. Answer:

## Q-003 · Instruction languages for Pre-A1/A1 flows
Blocked: no (EN + AR + UR scaffolding is built). Which additional UI languages for the beta audience? Answer:

## Q-004 · Retention periods
Blocked: no (defaults: audio 90 days, text 24 months, telemetry aggregated after 12 months). Confirm or change. Answer:

## Q-005 · Approve D-001…D-010 before M3 acceptance
Blocked: no (defaults implemented). Any overrides? Answer:

## Q-006 · Google Cloud project / Cloud Run billing for deploy (M12)
Blocked: only at M12. Starter tier (2 apps, no billing) vs standard deployment with a linked billing project. Answer:

## Q-007 · Offline flite voices can't hit the slow Pre-A1/A1/A2 WPM targets
Blocked: no (audio production runs and produces real assets for all 52 items; `npm run audio:produce` no longer
fails the build on this — see DECISIONS.md D-019). ffmpeg's bundled `flite` voices speak at a fixed pace with no rate
control; a last-resort `atempo` correction (±7%, per `12_AUDIO_PRODUCTION.md §3`) closed most of the gap, but 19 of 52
assets — concentrated in the deliberately slow Pre-A1/A1/A2 listening-stimuli bands (target 105-138 WPM) plus a few
very short Speaking/Writing prompts — remain outside the ±8% WPM tolerance because closing them would need a >7%
stretch, which the spec explicitly forbids ("never beyond"). Options: (a) accept as a pilot-quality limitation and
ship (current default); (b) supply `GEMINI_API_KEY` or `ELEVENLABS_API_KEY` — `tools/audio_producer` already
auto-selects a rate-adjustable provider the moment one is configured, no code change needed; (c) manually author
slower-paced admin scripts for just the affected Pre-A1/A1/A2 stimuli. Recommended: (a) for the pilot, revisit once a
real TTS key exists. Answer:

---
<!-- Agent appends from here. Next id: Q-008 -->
