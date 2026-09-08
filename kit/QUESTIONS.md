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

---
<!-- Agent appends from here. Next id: Q-007 -->
