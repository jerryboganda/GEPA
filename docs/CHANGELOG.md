# GEPA Platform Changelog

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

