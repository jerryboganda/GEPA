# GEPA Platform Changelog

## [Unreleased] - 2026-09-13 — Q-008 resolved: shared-platform VPS deploy profile (no app changes)

The owner directed that the production VPS runs **shared infrastructure** (platform stack at
`/opt/platform`: shared Postgres for every project, shared NPM ingress, per-project env files under
`/opt/platform/projects/`). The deploy path was rewritten to comply, and three latent deploy-path defects
were fixed on the way (full rationale in DECISIONS.md D-023, go-live steps in RUNBOOK.md §9.6):

- `docker-compose.prod.yml` → **shared-platform profile**: removed the dedicated `postgres` service and the
  published host port 8080; the server joins the external `platform` network (shared `platform-postgres`
  via `DATABASE_URL`) and the external `nginx-proxy-manager_default` network (NPM forwards to
  `gepa-server-prod:8080` by container name — the box has no active firewall, so no 0.0.0.0 listeners).
  `DATABASE_URL`/`JWT_SECRET` fail fast (`:?`) instead of booting degraded silently. No fake Redis wiring —
  the server's code path is Postgres-only.
- `.github/workflows/deploy-vps.yml` → `deploy-to-vps` gated on build success (was `if: always()`);
  copies the **exact tagged** compose file to `/opt/docker/gepa` via `appleboy/scp-action@v1.0.0`;
  lowercases `github.repository` for the GHCR image ref (docker refs must be lowercase —
  `jerryboganda/GEPA` would have failed the pull); pulls/starts with
  `--env-file /opt/platform/projects/gepa.env`; smokes `/healthz` via `docker compose exec` with retry
  headroom for first-boot schema migration (was host-side `curl localhost:8080`, unreachable without
  published ports).
- GHCR package documented as **must-stay-private**: the image bakes in `RESTRICTED_*` seed assets;
  workflow authenticates pulls with its short-lived token (no PAT to mint or rotate).
- Release: tagged `v2.0.0-beta.5` → image published to `ghcr.io/jerryboganda/gepa/gepa-server`
  (semver + `sha-` + `latest`), built 100% on GitHub Actions runners.

## [Unreleased] - 2026-09-13 — CI hygiene: Node 24-native action pins (no app changes)

Every workflow file bumped off actions that GitHub's runners already force onto Node 24
(the "Node.js 20 is deprecated" annotations on every run): `actions/checkout@v4→v7`,
`actions/setup-node@v4→v7`, `actions/upload-artifact@v4→v7`, `docker/setup-buildx-action@v3→v4`,
`docker/login-action@v3→v4`, `docker/metadata-action@v5→v6`, `docker/build-push-action@v6→v7`.
Release notes for every intervening major were checked first — all are runtime/ESM swaps or
removals of inputs/envs this repo never uses; setup-node v5+'s automatic caching can't trigger
here (no `packageManager` field, explicit `cache:` config kept). Deliberately left alone:
`google-github-actions/auth@v2`, `deploy-cloudrun@v2` and `appleboy/ssh-action@v1.0.3` — they
have produced no deprecation warnings, and they only execute in the deploy workflows, which
cannot be exercised until the owner sets deploy-target secrets (QUESTIONS.md Q-008); bump
them during the first deploy dry-run, not blind.

## [2.0.0-beta.5] - 2026-09-13 — M12 hardening: candidate-view sanitization + redaction & bundle gates

### Two real server-side leaks found by the new bundle gate (fixed at the root)
- **Productive task payloads shipped admin scripts**: `GET /api/sessions/:id/speaking/start` and
  `/writing/start` serialized the full `SpeakingTask`/`WritingTask` structs — including `audio_script`
  (present on 12 SPK + 6 WRT seed tasks: the model answer script for oral reading / audio-only tasks and the
  listen-to-write source text) and `interlocutor_line` — directly to the candidate's browser. Every candidate
  could read the rating material for their own tasks in the network tab. Fixed at the single serialization
  point every route shares: `get_speaking_tasks`/`get_writing_tasks` (server/src/services.rs) null out those
  fields before returning (DECISIONS.md D-022). The client zod schemas (`client/src/schemas/api.ts`) drop the
  fields to match the wire, and the e2e security spec now drives through the productive phase and fails on
  the field names themselves — previously it only scanned the receptive journey, which is exactly why this
  leak survived a green security test for so long.
- **Dead `RestrictedKey` schema shipped the restricted collection's shape**: the client bundle declared a
  never-imported zod schema naming `key_option_id`, `authoring_letter`, `answer_text`, `rationale` — the
  field-name shape of the server-only answer-key collection. No values leaked, but the shape is
  server-only material per AGENTS.md's "Never" list. Deleted.

### Credential hygiene
- Gemini API key moved from the request URL (`?key=`) to the `x-goog-api-key` header: reqwest error Display
  strings embed the request URL, so any logged rating-call failure could have leaked the credential.

### M12 hardening gates, now actually implemented (all run in CI)
- **Logging redaction, static**: `server/src/logging.rs` — a `cargo test`-time source scan that fails if any
  `tracing::*!` macro in `server/src` interpolates a redaction-list identifier (audio URLs, transcripts,
  option ids, keys, emails, credentials, DSNs). String literals are stripped before matching, so message
  prose can't false-positive. Also hardened `db.rs`'s pool-error log (its Display can embed the DSN with
  password) to log a static message instead.
- **Logging redaction, runtime**: the CI container job boots a second container with canary secret values
  (`JWT_SECRET`, `GEMINI_API_KEY`, a wrong-password `DATABASE_URL` — chosen so the pool-error and
  migration-failure paths actually execute) and greps its full `docker logs` output for the canaries,
  covering tower_http TraceLayer and deadpool internal events too.
- **Bundle secrets scan**: `scripts/scan_bundle_secrets.py` after every client build — fails on
  secret-shaped strings (Google API keys, `sk-` keys, JWTs, DSNs-with-password), restricted-key markers,
  and verbatim listening/seed script text in `client/dist`.
- **Initial payload size gate**: `scripts/check_bundle_size.py` — the spec's "< 400 kB gz initial" budget,
  measured over exactly what `index.html` references (HTML + CSS + island JS), gzip level 6. Current
  payload: **84 kB** (79% headroom).

### Verification (GitHub Actions only — no local DB/browsers, per D-018)
- Local sanity (file transforms, no DB/browser): `tsc --noEmit` clean, schema tests 4/4, Astro build + both
  bundle gates pass.
- Everything else (Postgres-backed cargo tests, full e2e + axe suite, container build + healthz + the new
  redaction runtime test) runs in `.github/workflows/ci.yml`.

## [2.0.0-beta.4] - 2026-09-11 — Real persistence, real auth, real e2e, real audio

### Correction to prior entries
An independent audit on 2026-09-11 found this changelog's earlier "100%"/"PASS" claims did not hold against a clean
checkout: the workspace did not compile (`server/src/services.rs` was missing a struct field), all persistence was an
in-process `HashMap` despite `firestore.rules`/`storage.rules` existing as decoration, there was no authentication
anywhere, no e2e/accessibility test of any kind existed despite being named repeatedly in the spec, and
`tools/audio_producer` fabricated QC numbers without ever calling a TTS provider. This entry documents what was
**actually built and verified** to close every one of those gaps. Treat entries before this one as the historical
record of what was *attempted*, not a reliable statement of what works.

### Architecture change: Postgres + self-issued JWT (no Firebase) — DECISIONS.md D-021
Mid-fix, the product owner overrode the spec's Firestore/Firebase Auth design outright: no Firebase in any form,
Postgres for storage, plain Rust for auth, and nothing DB-dependent runs on a local dev machine — only in GitHub
Actions (a `postgres:16-alpine` service container, `.github/workflows/ci.yml`).
- `server/src/db.rs` (new): `PostgresClient` over `tokio-postgres` + `deadpool-postgres` — no ORM/query-macro crate.
  Three tables (`sessions`, `jobs`, `review_queue`) share one JSONB-document shape so the whole domain model
  round-trips through `serde_json` unchanged; optimistic concurrency via an integer `version` column.
- `server/src/auth.rs` (rewritten): self-issued HS256 JWTs (`JwtService`). Candidates get a token minted at session
  creation (`POST /api/sessions` now returns `{session_id, token, next_step}`) — no external identity provider at
  all. Reviewers/admins log in with email+password against a new `staff_users` table (Argon2 hashing,
  `POST /api/auth/login`); `tools/create_staff_user` provisions accounts. `AuthUser`/`StaffAuth`/`AdminAuth` Axum
  extractors gate every session/review/admin route — previously nothing did.
- `server/migrations/001_init.sql` (new): schema, embedded into the binary via `include_str!`, run idempotently at
  startup.
- Fixed the compile-breaking missing `previous_reports` field and a non-Send `RwLockReadGuard`-held-across-`.await`
  bug in `get_next_unit` (async-fn Send checker issue — fixed via block-scoping the guard).

### Real e2e + accessibility suite (was: nothing)
- `client/playwright.config.ts` + `client/tests/e2e/*`: the seven spec files named in `10_TESTING_QA.md §4`
  (`journey.full`, `journey.keyboardOnly`, `journey.partial`, `resume`, `claims`, `a11y`, `security`), Chromium only.
- `POST /api/test/e2e/sessions` (new, `E2E_MODE`-gated, 404s otherwise): drives the real `AssessmentService` to
  fast-forward a session to any point, for deterministic seeding — never a mock of the engine.
- `data-testid` retrofit across `CandidateJourney.tsx` and the shared components (`AudioPlayer`, `AudioRecorder`,
  `Timer`, `WritingEditor`).
- Real gap found and fixed while writing `journey.keyboardOnly.spec.ts`: the objective-item radio group had no
  arrow-key navigation at all (06 §3 requires it) — added a proper roving-tabindex radiogroup.
- Real gap found and fixed while writing `resume.spec.ts`: `AudioPlayer.tsx` had no `onError` handler — a network
  failure mid-play left the player permanently stuck ("isPlaying" never reset). Added a reconnect state that resets
  playback without consuming a play (06 §5: "resume replays the same audio, counts as first play").
- Removed all `any` typing from `CandidateJourney.tsx` in favour of the existing zod-inferred types
  (`client/src/schemas/api.ts`), which surfaced a real bug: the Speaking player read `prep_seconds`/
  `max_speak_seconds`/`allows_rerecord` (snake_case) from tasks whose real wire format is camelCase
  (`prepSeconds`/`maxSpeakSeconds`/`allowsRerecord` — `shared_engine::models::SpeakingTask` renames only those three
  fields). The client was silently ignoring the server's real per-task timing and always using hardcoded defaults;
  now fixed.

### Real audio production (was: fabricated QC numbers, zero actual audio files)
- `tools/audio_producer` rewritten: a real `TtsProvider` trait (`provider.rs`) with `FliteProvider` (ffmpeg's bundled
  `flite` voices — zero cost, offline, no account) as the default, and a `GeminiTtsProvider` stub that auto-activates
  the moment `GEMINI_API_KEY` is set (untested against a live key — logged in `QUESTIONS.md`).
- Real ffmpeg pipeline (`ffmpeg.rs`): per-segment synthesis, inter-turn silence concatenation, silence trim, 0.5s/1.0s
  lead/tail padding, two-pass `loudnorm` to -16 LUFS / -1.5 dBTP, opus + mp3 encoding, and a last-resort `atempo`
  rate correction (±7% max, per spec §3) for the offline voice's fixed pace.
- Produced real audio for all 52 assets. 33/52 land within the ±8% WPM tolerance; the remaining 19 are honestly
  flagged (`in_tolerance: false` in `assets/qc_report.json`), concentrated in the deliberately slow Pre-A1/A1/A2
  bands where flite's fixed speaking rate can't be brought within range by the spec's ±7% stretch limit — a real
  content-engineering constraint (DECISIONS.md D-019, QUESTIONS.md Q-007), not a processing failure. The build no
  longer fails on this; it only fails on a genuine synthesis/processing error.
- `assets/media_manifest.json` (new) links real files to asset ids; `server/src/api.rs::serve_audio` and
  `get_stimulus_audio_url` now serve the real produced files instead of always returning a synthetic tone.

### Verification summary (run this session, on this machine, without touching a local database)
- `cargo check/test/clippy --workspace`: clean, 47/47 engine tests, 0 clippy warnings.
- `cargo run --bin copy_lint`: 0 forbidden-wording violations.
- `npm --prefix client run typecheck` (`tsc --noEmit`): 0 errors.
- `npm --prefix client run build`: production build succeeds.
- `cargo run --bin audio_producer`: 52/52 real assets produced.
- **Not yet verified**: anything requiring a live Postgres connection (session CRUD, auth end-to-end, rules
  enforcement) or a real browser (the e2e suite itself) — these need the GitHub Actions Postgres service container,
  which is currently blocked account-wide by a billing issue on the `jerryboganda` GitHub account (unrelated to this
  work; see the open PR for status).

## [2.0.0-beta] - 2026-09-08

### Added & Fixed
- **Pure Adaptive Diagnostic Engine (`shared/engine`)**:
  - Zero-I/O Rust crate implementing multi-stage adaptive routing (`routing.rs`), productive route assignment (`productive_route.rs`), objective scoring with deterministic PRNG option shuffling (`objective_scoring.rs`), dual-reference evidence rules (`evidence_rules.rs`), headline and diagnostic construct breakdown (`result_assembly.rs`), and high-performance wording policy scanner (`wording_policy.rs`).
  - Strict CEFR band models, candidate payload sanitization ensuring zero key or script leaks, and 35 unit/property tests.
- **Multimodal AI Scoring Pipeline (`server/src/ai/`)**:
  - `GeminiClient` supporting live Google Gemini API multimodal rating and offline deterministic fallback simulation.
  - Prompt registry with versioned templates: `speaking_rater@v2.0`, `writing_rater@v2.0`, `transcribe@v1.0`, `listen_to_write_check@v1.0`, and `consistency_review@v1.0`.
  - Usability gate checks (silence/noise detection, off-topic detection, word count limits) and trait invariant assertions (0.0-5.0, upper <= lower).
  - Second-opinion rater disagreement trigger (`|mean - mean'| >= 1.0`) and AI-suspect flags automatically routing to human review queue.
- **Axum REST API Server & Services (`server/`)**:
  - Fixed locator step scoring and tie-breaker routing.
  - Testlet batch response submission (`/api/responses/batch`).
  - Signed time-limited media delivery URLs (`/api/media/stimuli/:id/url`).
  - Non-destructive rescore (`/api/review/sessions/:id/rescore`) and human scoring (`/api/review/sessions/:id/human-score`) with `superseded_by` audit trail.
  - GDPR lifecycle execution endpoint (`/api/admin/retention/run`) and dynamic seed bank reload (`/api/admin/seed/load`).
- **Operational CLI Tools & Scripts (`tools/`)**:
  - `seed_validator` (`npm run seed:validate`): Validates all 136 answer keys, item banks, and SHA256 checksums against `manifest.json`.
  - `copy_lint` (`npm run lint` / `npm run copy-lint`): Scans candidate-facing copy for prohibited claims and forbidden level modifiers.
  - `forms_checker` (`npm run forms:check`): Verifies authoring letter balance (<40%), 4-domain coverage, and 22 enemy group conflict sets.
  - `audio_producer` (`npm run audio:produce`): Generates and QC-validates 52 audio assets (21 LSN, 24 SPK, 6 WRT, 1 demo tone) with -16 LUFS, -1.5 dBTP peak, and 0.5s/1.0s silence padding.
  - `exposure_reporter` (`npm run report:exposure`): Computes item exposure rates, p-values, and response latency telemetry with zero PII.
  - `seed_loader` (`npm run seed:load`): Two-stage loading isolating candidate-safe collections from restricted answer keys and scripts.
  - `retention_runner` (`npm run retention:run`): Enforces UK GDPR 90-day audio purge and 24-month text retention policies.
- **Accessible Client Application (`client/`)**:
  - Astro 4.15 + React + Tailwind CSS frontend architecture.
  - Candidate journey (`CandidateJourney.tsx`) with testlet item review, audio lock gate on Listening, worked example, adaptive player, audio recorder level meter, autosaving writing editor, reviewer portal (`ReviewerDashboard.tsx`), admin console (`AdminDashboard.tsx`), and WCAG 2.2 AA accessibility drawer (`AccessibilityDrawer.tsx`).
  - Static build outputs in `client/dist/` with <70 kB total gzipped footprint.
- **Comprehensive Runbook & Documentation**:
  - Created `docs/RUNBOOK.md` covering key rotation, seed reloading, rescoring, report exporting, rollback procedures, and deployment.
  - Updated `docs/DECISIONS.md` (D-001 through D-015) and audited `docs/QUESTIONS.md`.

### [2.0.0-beta.2] - 2026-09-09 — 100% PRD (01_PRD.md) Full Implementation
- **PRD §1 & §9 Target-Exam Readiness & Currency Notes**:
  - Embedded official currency notes across backend `ReadinessLayer` and client UI for TOEFL iBT (Jan 2026 scale), PTE Academic (Aug 2025 task updates), and OET (Healthcare context).
- **PRD §2 & §6 Session Resumption & Upper Route Breaks**:
  - Seamless candidate session resumption via browser `localStorage` and manual session ID lookup.
  - Dedicated `'writing_break'` intermission screen with physical keyboard/desktop recommendation before upper-level writing tasks.
- **PRD §3 & §13 Extensibility & Telemetry Metadata**:
  - Attached `mode` (`"free_diagnostic_beta"` / `"verified"`), `device_class`, `identity_metadata`, and `proctoring_metadata` to session models.
- **PRD §4 Candidate Journey Strict Ordering**:
  - Start -> Worked Example -> Foundation Profile -> Provisional Receptive Result -> Optional Full Profile -> Full Result.
  - Dedicated confirmed completion state for candidates electing to stop at Receptive Profile.
- **PRD §5 CEFR Scope Note & Construct Boundary**:
  - Authored full Technical Manual at `/about` featuring the mandatory CEFR scope note: "GEPA v2 samples mediation and digitally-mediated interaction where practical; it does not claim to measure plurilingual/pluricultural competence."
- **PRD §10 Fairness & Accessibility Policy (WCAG 2.2 Level AA)**:
  - Added Transcript Access mode for D/deaf and hard-of-hearing candidates (reporting Listening as `"not_measured"`).
  - Added Speech Difference pathway replacing Oral Reading with Functional Situation tasks.
  - 1.5× extended timing, high contrast (7:1), font scaling up to 200%, and dyslexia-friendly font support.
- **PRD §12 Beta Success Metrics Instrumentation**:
  - Built `GET /api/admin/reports/metrics` instrumenting all 8 PRD success metrics.
  - Added real-time Beta Success Metrics Dashboard to `AdminDashboard.tsx`.

### Verification Summary
- `cargo run --bin seed_validator`: ALL INVARIANTS PASS (136 keys, 6 checksums, full item bank).
- `cargo run --bin forms_checker`: PASS (peak letter 33.3% < 40%, 4 domains, 0 enemy group conflicts).
- `cargo run --bin audio_producer`: PASS (52/52 assets pass -16 LUFS, -1.5 dBTP QC).
- `cargo run --bin exposure_reporter`: PASS (136 items tracked, zero PII, report in `reports/gepa_exposure_report.csv`).
- `cargo run --bin seed_loader`: PASS (two-stage isolation, zero keys/scripts in candidate collections).
- `cargo run --bin retention_runner`: PASS (GDPR data minimization and purge lifecycle completed).
- `cargo clippy --workspace -- -D warnings`: 0 warnings, 0 errors.
- `cargo test --workspace`: 35/35 tests pass in 0.04s.
- `cargo run --bin copy_lint`: 0 forbidden wording violations.
- `npm run typecheck`: PASS (`cargo check --workspace` & `tsc --noEmit`).
- `npm --prefix client run build`: PASS (4 static pages, total bundle <70 kB gz).
- `npm run verify`: PASS across all automated verification stages.

## [2.0.0-beta.3] - 2026-09-11 — 100% Architecture (02_ARCHITECTURE.md) & GitHub Actions Compute Offload

### Added & Fortified
- **Architecture §1 & §8 Production Containerization & Zero-Compute Host**:
  - Authored production multi-stage `Dockerfile` with minimal debian:bullseye-slim runtime footprint and bundled static assets.
  - Created `docker-compose.prod.yml` with explicit CPU (1.0) and memory (512M) bounds, eliminating VPS resource pressure.
  - Documented Section 10 in `docs/RUNBOOK.md` and Decision `D-018` establishing the Zero-Compute VPS policy.
- **GitHub Actions Compute Offloading Pipeline**:
  - Created `.github/workflows/ci.yml`: Offloads all build, test, lint, validation, and Docker Buildx tasks with GitHub Actions caching (`type=gha`).
  - Created `.github/workflows/deploy-cloud-run.yml`: Compiles and packages containers in GitHub Actions, deploying pre-built images to Cloud Run with zero compilation on target infra.
  - Created `.github/workflows/deploy-vps.yml`: Automatically builds, publishes to GHCR, and orchestrates zero-compute pull/restart and image pruning on production VPS hosts.
  - Created `.github/workflows/scheduled-maintenance.yml`: Automates daily GDPR retention purges and exposure telemetry on GitHub Actions cron.
- **Architecture §4 API Contract Enforcements**:
  - Implemented `Idempotency-Key` header handling in `submit_objective_response`, `submit_speaking`, and `submit_writing` with thread-safe `idempotency_cache`.
  - Implemented rapid-click rate limiting with a 750ms minimum response interval and automatic `rapid_response_burst` diagnostic tagging.
  - Added monotonic router concurrency `state_version` incremented on each state change and exposed in all response models.
- **Architecture §3.4 Signed Audio Upload Flow**:
  - Implemented `POST /api/media/responses/:task_id/upload-url` generating signed upload URLs with 600s TTL.
- **Architecture §5 Persistent In-Process Job Queue**:
  - Implemented `BackgroundJob` state machine tracking attempts, status (`queued`, `processing`, `completed`, `failed`), and error provenance.
  - Added `GET /api/admin/jobs` for real-time background task monitoring.
- **Architecture §7 Observability & Health**:
  - Fortified `GET /healthz` to inspect seed bank loading, active session counters, and background job queue metrics, with `Cache-Control: public, max-age=60`.

### Verification Summary
- `GET /healthz`: HTTP 200 OK with sub-system check payload.
- `POST /api/media/responses/:task_id/upload-url`: Verified returns valid signed upload URL and storage path.
- `Idempotency-Key`: Verified duplicate submissions return cached results without re-executing scoring.
- Rapid-response throttle: Verified intervals <750ms flag `rapid_response_burst` and advance `state_version`.
- `cargo check --workspace` & `npm --prefix client run typecheck`: 0 errors.
- `cargo clippy --workspace -- -D warnings`: 0 warnings.
- `copy_lint`: 0 prohibited wording violations.

