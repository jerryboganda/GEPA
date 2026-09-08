# 02 — Architecture

## 1. Stack (locked defaults — deviations go to DECISIONS.md)

| Layer | Choice | Why |
|---|---|---|
| Client | Astro.js + TypeScript strict + Tailwind CSS; interactive islands for timer, audio recorder, audio player, editor | Modern, lightweight, fast content delivery with accessible client islands |
| Server | Rust (Tokio async runtime + Axum HTTP framework) + Serde + Tower middleware | High performance, memory safety, zero-cost abstractions, robust type safety |
| Shared | Rust core domain crate (`shared/engine`) + client TypeScript/zod schemas | Pure deterministic engine, single canonical domain model, testable without I/O |
| Data | Firestore (Google Cloud SDK / Firestore REST API in Rust) | Provisioned by the agent; rules-based isolation |
| Auth | Firebase Auth: anonymous (candidates), Google (reviewer/admin with custom claims `role`) | Low friction; role gating verified via JWT validation |
| Media | Cloud Storage: `audio/stimuli/*` (served via short-lived signed URLs), `audio/responses/{session}/*` (private) | Private by default |
| AI | Gemini API via HTTP client (`reqwest` with Tokio in Rust), server-only, structured JSON output | Rating, transcription, quality checks, TTS |
| TTS | Gemini multi-speaker TTS (default) or ElevenLabs adapter (optional) | Dialogue scripts, accent rotation |
| Deploy | Cloud Run containerized deployment; secrets in Secrets panel | Scalable, containerized Rust binary + Astro build |
| Tests | `cargo test` (engine & API integration), Playwright, axe-core | Engine coverage (≥95%) + journey + a11y |

Model configuration (env): `MODEL_RATER` (Pro-class, multimodal, structured output), `MODEL_FAST` (Flash-class for
transcription/quality/consistency checks), `MODEL_TTS` (multi-speaker TTS). Discover with `models.list` at setup;
pin exact IDs in `.env.example` and `DECISIONS.md`. Never hardcode a model ID in business code.

## 2. Layer boundaries

```
client (Astro) ──HTTPS/JSON──▶ server/api (Rust / Axum) ──▶ server/services (Rust) ──▶ shared/engine (Rust pure crate)
                                                         │                          ▲
                                                         ├──▶ server/repos ─────────┘ (Firestore SDK; only layer touching restricted data)
                                                         ├──▶ server/ai   (Gemini via reqwest; prompts versioned)
                                                         └──▶ storage     (signed URLs, uploads)
```

Rules: the client never reads Firestore collections that hold items, keys or scripts directly — all item delivery is
through the API (so shuffling, exposure logging and key isolation are enforced in one place). Candidate-owned session
documents may be read by the client through Firestore rules for resume state only (or via API; choose one, log it).

## 3. Core flows

### 3.1 Session start
`POST /api/sessions` `{targetGoal?, uiLanguage, consent:{privacy:true}, accommodations?}` →
creates `sessions/{id}` with `mode:"free_beta"`, `rulesetVersion`, `formVersion`, module states `pending`, returns
`{sessionId, nextStep:"worked_example"}`. Background questions stored only if provided.

### 3.2 Objective module loop (LS, RD, LSN)
1. `POST /api/sessions/:id/modules/:module/start` → server runs `formAssembler.plan(module, session)` (enemy rules,
   domain coverage, exposure), initialises router state, returns first **delivery unit** (item, or stimulus + its items).
2. Client renders; timer deadline comes from the server (`deadlineAt`).
3. `POST /api/sessions/:id/responses` `{itemId, optionId | null, clientElapsedMs, replayCount?, events[]}` → server:
   validates the item is the current one, records permutation, scores against `restricted_keys` (server-only), appends
   to router trace, decides next unit via `engine.routing.next(state, outcome)`, persists, returns next delivery unit or
   `{moduleComplete:true}`. Timeout → `optionId:null, omitted:true`.
4. Listening: `GET /api/media/stimuli/:stimulusId/url` returns a signed URL valid 10 min; replay count is reported in the
   response payload and logged; never penalised.

### 3.3 Receptive result
`GET /api/sessions/:id/results/receptive` → engine computes per-module band (or range/flags) + Grammar/Vocab
diagnostics + confidence; response labelled `profileType:"foundation_receptive"`.

### 3.4 Speaking
`start` → route computed by `engine.productiveRoute` from objective brackets → server returns task list (8 responses)
with prompt text and signed URLs for any audio prompts (SR sentence, RT message, INT interlocutor line).
Client: mic check (only here), record via MediaRecorder (webm/opus, 48 kHz), client-side quality pre-check
(duration, RMS, clipping, silence ratio), `POST /api/media/responses/:taskId/upload-url` → PUT to signed URL →
`POST /api/sessions/:id/speaking/:taskId/submit {storagePath, clientMetrics}`. Server queues rating (see `07`).
One re-record allowed per task where `allowsRerecord` (all except SR turn timing rules — see `06`).

### 3.5 Writing
`start` → tasks by route with word guidance + per-task time limits; editor autosaves every 5 s or 30 chars
(`PUT /api/sessions/:id/writing/:taskId/draft`); tools telemetry (paste events, length jumps, focus loss) logged;
`submit` freezes text. Diagnostic (listen-to-write) uses the same audio flow as Listening with one play + one replay.

### 3.6 Full result
Ratings complete (poll `GET /api/sessions/:id/results/status`) → `GET /api/sessions/:id/results/full` →
engine applies evidence rules → bands/insufficient → headline rule → confidence → readiness layer text.

### 3.7 Review and admin
`GET /api/review/queue`, `GET /api/review/sessions/:id`, `POST /api/review/sessions/:id/rescore {rubricVersion, reason}`
(new scoring doc; original preserved), `POST /api/review/sessions/:id/human-score`.
Admin: `POST /api/admin/seed/load`, `POST /api/admin/forms/check`, `POST /api/admin/audio/produce`,
`GET /api/admin/reports/exposure`, `GET /api/admin/reports/telemetry`. Role checked via custom claims.

## 4. API contract rules
- All request/response bodies validated with Serde schemas in the Rust server and mirrored by zod schemas in the Astro client (`client/src/schemas/api.ts`).
- Idempotency: `responses` and `submit` endpoints accept an `Idempotency-Key` header; duplicate = same result.
- Concurrency: session documents updated in Firestore transactions; router state versioned (`stateVersion`).
- Rate limits: per-session token bucket on `responses` (max 1 per 750 ms) — rapid clicking still accepted but flagged.
- Time: server issues `deadlineAt`; enforces with +3 s grace; client shows countdown only.

## 5. Background work
Rating jobs run in-process with a persistent queue collection (`jobs/{id}`: `type, status, attempts, lastError`) and a
worker loop with exponential backoff (max 5 attempts) — no external queue in beta. Audio production is an admin-triggered
job that writes assets + QC report. Cloud Run min instances 0; the worker resumes pending jobs on startup.

## 6. Configuration and secrets
`GEMINI_API_KEY` (auto), optional `ELEVENLABS_API_KEY`, `FIREBASE_*` (from integration), `MODEL_RATER`, `MODEL_FAST`,
`MODEL_TTS`, `SIGNED_URL_TTL_SECONDS=600`, `FEATURE_PRETEST_ITEMS=false`, `FEATURE_AI_DETECTOR_FLAGS=false`,
`BETA_CONFIDENCE_CAP=Moderate`. Provide `.env.example`; never commit real values.

## 7. Observability
Structured logs with `sessionId, module, itemId(hash), latencyMs`; redaction list in AGENTS.md. Metrics counters:
responses, timeouts, disconnections, recordings failed QC, rating jobs (ok/failed), forbidden-wording lint failures
(should be zero at runtime). Health endpoint `/healthz` checks Firestore + model reachability (cached 60 s).

## 8. Deployment
`Deploy → Cloud Run` from AI Studio. Post-deploy checklist (M12): secrets present; Firestore + Storage rules deployed and
tested with the emulator or rules unit tests; seed loaded; audio assets produced; `verify` green; `/healthz` 200;
smoke journey passes against the deployed URL.
