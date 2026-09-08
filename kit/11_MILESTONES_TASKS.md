# 11 — Milestones and Tasks (the agent's work order)

Fixed order M0 → M12. Each milestone ends with `npm run verify` green and the report shape from the master prompt.
Acceptance criteria (AC) are checked literally. Cite spec sections you implemented in the report.

## M0 — Scaffold & tooling
Tasks: full-stack app skeleton (`client/`, `server/`, `shared/`); TypeScript strict; ESLint/Prettier; Vitest; Playwright;
`package.json` scripts from AGENTS §2; `.env.example`; `/healthz`; `docs/CHANGELOG.md`, `DECISIONS.md`, `QUESTIONS.md`
wired; Firebase integration enabled (Firestore + Auth); Storage bucket.
AC: `verify` runs (unit suite may be empty), dev server serves a placeholder start page; Firebase emulator config exists.

## M1 — Shared schemas & seed validation
Tasks: all zod schemas from `03`; `seed:validate` implementing `10 §2`; `copy-lint` tool with forbidden list from
`results_and_claims.json`; models discovery script (`scripts/models_list.ts`) → pin `MODEL_*` in `.env.example` + D-entry.
AC: `seed:validate` passes on `/seed`; injecting a wrong key letter or removing an item fails it; copy-lint fixtures pass.

## M2 — Routing engine (pure)
Tasks: `routing.ts` (locator + confirmation + buildBlock with pilot-depth degradation), `productiveRoute.ts`, seeded RNG.
AC: every test in `04 §4` passes; property tests pass; branch coverage ≥95% for `shared/engine`.

## M3 — Objective scoring, evidence rules, result assembly (pure)
Tasks: `objectiveScoring.ts`, `evidenceRules.ts`, `resultAssembly.ts`, `wordingPolicy.ts`, Can-Do copy table.
AC: `10 §1` tests pass; headline cap invariant test; wording guard throws on forbidden phrase; no `High` anywhere.

## M4 — Data layer, rules, seed loader
Tasks: Firestore repos; `firestore.rules` + `storage.rules` per `09 §2`; rules emulator tests; `seed:load` (safe and
restricted steps); `seed_runs` audit; exposure counters.
AC: rules tests pass (deny cases); loader idempotent; `restricted_keys` and `stimulus_admin` populated; no script text in
`stimuli_listening`.

## M5 — Session & objective module API
Tasks: `POST /sessions`, `state`, `modules/:m/start`, `responses` (transaction, idempotency, permutation, scoring,
router step), media signed URLs (stimulus current/next only, 3rd play refused), timeouts, disconnection recovery,
abandonment, telemetry events, rate limit, form assembler (enemy rules, coverage, least-exposed).
AC: supertest suite passes; JSON bodies never contain `correct`/keys (regex test); a scripted client can complete LS→RD→LSN
against the emulator and get a receptive result.

## M6 — Client: start, worked example, LS/RD/LSN players, receptive result
Tasks: accessible primitives (RadioGroup, Timer, AudioPlayer without scrub, Dialog, LiveRegion); features per `06 §1–§6`;
Display & access panel (text size, spacing, contrast, extended time, transcript pathway); resume on reload; i18n for
instruction strings (EN + AR + UR scaffolding).
AC: e2e partial journey passes on desktop + mobile widths; axe 0 violations on these screens; keyboard-only journey passes.

## M7 — Speaking capture & rating pipeline
Tasks: mic check, Recorder component (level meter, countdown, quality pre-check), upload via signed URL, task timing table,
re-record rules, skip path; job queue + worker; Gemini client (`@google/genai`) with structured output; prompts
`speaking_rater@v2.0`, `transcribe@v1.0`; second-opinion check; rating persistence; results status polling.
AC: with mocked Gemini, 8 responses → ratings → Speaking evidence decision; with real key (manual), one real rating round
succeeds and stores model/prompt/rubric versions; quality gate rejects the silence and clipped fixtures.

## M8 — Writing editor & rating
Tasks: editor with autosave/word count/paste logging/time limits; listen-to-write diagnostic flow; `writing_rater@v2.0`,
`listen_to_write_check@v1.0`; behavioural AI-suspect flag (never affects score).
AC: draft survives reload; submit freezes; ratings produce Writing decision; diagnostic never changes band (test).

## M9 — Full results, readiness layer, claims enforcement
Tasks: results screen per `06 §9` and `05 §3.1`; readiness copy per target with disclaimer; retest advice; "How this was
scored"; "Delete my data"; runtime wording guard; `claims.spec.ts`.
AC: e2e full journey passes; DOM order/prominence assertion passes; forbidden-phrase scan of rendered results = 0.

## M10 — Reviewer & admin
Tasks: Google sign-in + custom claims; review queue and session view (audio, transcript, both trait profiles, rationale,
flags); accept/adjust/re-run/mark-unusable (originals preserved); admin: seed load, `forms:check`, exposure & telemetry
reports (CSV), feature flags, retention job.
AC: rescore creates a new rating with `supersededBy` on the old; report rebuild uses latest; CSV exports contain no PII.

## M11 — Audio production
Tasks: `audio:produce` per `12`: TTS for 21 listening scripts (multi-speaker, accent rotation), 6 SR sentences, 6 RT
messages, 12 INT lines, 6 listen-to-write sentences; loudness/peak/padding processing; duration vs target-WPM check;
QC report; upload to Storage; link assets to stimuli/tasks. ElevenLabs adapter behind an interface (optional key).
AC: all 51 assets produced, QC report shows every asset within spec or flagged; listening player plays real assets.

## M12 — Hardening & deploy
Tasks: performance pass (bundle < 400 kB gz initial, audio pre-buffer), error handling review, log redaction test,
`grep` secrets in bundle, Cloud Run deploy from AI Studio, rules deployed, seed + audio loaded in prod project,
post-deploy smoke journey, runbook (`docs/RUNBOOK.md`: rotate keys, rerun seed, rescore, export reports, rollback).
AC: deployed URL passes smoke journey; `/healthz` 200; `verify` green; `QUESTIONS.md` has no open blocking items or they
are listed for the human.

## Backlog (post-beta; do not build now)
Verified Mode (identity, proctoring, secure pool); calibrated MST/CAT engine; parallel forms and equating; DIF dashboards;
pretest seeding on by default; standard-setting tooling; certificate/report export; payments.
