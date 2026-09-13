# DECISIONS.md — interpretation and engineering decisions

Format: `D-### · <title>` — Context · Decision · Alternatives · Spec ref · Reversible? · Status (proposed | approved | superseded).
Entries D-001…D-009 were made while building this kit from the review PDF; the product owner should approve or override
them before M3 is accepted. The agent appends new entries below.

## D-001 · Locator reversal establishes the bracket
Context: blueprint §4.1 says 2/2 → move up, 0/2 → move down, but not what happens when the direction reverses.
Decision: a reversal (2/2 at band X after 0/2 at X+1, or 0/2 at X after 2/2 at X−1) brackets [X, X+1] / [X−1, X] immediately.
Alternatives: keep presenting pairs until a 1/2 tie (could oscillate); count reversal as tie trigger. Spec: 04 §1. Reversible: yes. Status: proposed.

## D-002 · Strong evidence at C1 brackets [C1, C2] without a C2 locator pair
Context: "At C1, strong evidence opens the C1/C2 route." Decision: 2/2 at C1 → bracket [C1, C2]; C2 evidence comes from the
upper confirmation block; C2 reported as "provisional C2-level evidence". Alternative: present the C2 locator candidates as a
pair first. Spec: 04 §1. Reversible: yes. Status: proposed.

## D-003 · Dual-reference rating of productive responses
Context: the evidence rules speak of responses "at the lower target" and "at the upper target" but every route has one task
set. Decision: each usable response is rated twice — against the lower band expectation and the upper band expectation
(3 = meets that band's typical performance) — and the thresholds (2.75 / 3.0) apply to the respective profile.
Alternative: tag half of the tasks as lower-target and half as upper-target. Spec: 05 §2, 07 §2. Reversible: yes (needs new
fixtures). Status: proposed.

## D-004 · Interaction turns count as one independent response
Context: "at least 2 independent responses meet the upper threshold". Decision: INT1+INT2 contribute two rated responses to
means but count once toward the independence count. Spec: 05 §2.2. Reversible: yes. Status: proposed.

## D-005 · "Below route" handling
Context: rules define lower/upper/insufficient only. Decision: when lower evidence is not met but ≥3 spontaneous (Speaking)
or ≥2 scored (Writing) responses are usable with mean-at-lower ≥ 1.75, report route.lower − 1 with note "below route" and
confidence Low; otherwise "insufficient evidence". Alternative: always "insufficient". Spec: 05 §2.2–2.3. Status: proposed.

## D-006 · Productive upper-extension task is a post-result optional extra (feature flag off in pilot)
Context: §4.3 "strong upper-level productive performance triggers one higher extension task". Ratings are asynchronous, so
an inline extension would stall the flow. Decision: `FEATURE_PRODUCTIVE_EXTENSION` off; when on, offer one ER from the next
route on the results screen and rebuild the report if completed. Spec: 04 §3. Status: proposed.

## D-007 · Oral Reading alternative for speech-difference accommodation
Decision: replace OR with an additional Functional Situation from the same route's topic pool (pilot: reuse FS prompt with a
different scenario line from a small `accommodation_alternatives.json` the agent authors); weights re-normalised so the
diagnostic cap (≤20%) and spontaneous floor (≥60%) still hold. Spec: 06 §7, 09 §5. Status: proposed.

## D-008 · Pilot-depth degradation of confirmation blocks
Context: pilot bank has 6 RD/LSN items per band and 8 LS items; blueprint blocks (5+5, +4 boundary, +4 extension, +5 descent)
cannot always be filled. Decision: blocks count already-answered locator responses at that band; boundary/extension/descent
blocks with <2 available items are skipped with flags (`boundary_unresolved`, `floor_unresolved`) and confidence Low;
thresholds are fractions so even block sizes (RD/LSN) work. Spec: 04 §2.1. Status: proposed.

## D-009 · Client never receives correctness during the test; drafts and resume state flow through the API
Context: §13 key security; AI Studio Firestore client SDK availability. Decision: all item/response traffic via API;
Firestore client reads limited to own `sessions/{id}` and `results/{id}`. Status: proposed.

## D-010 · Kit-assigned topic families and enemy groups
Context: the PDF tags topic_family for objective items only. Decision: the kit assigns topic families to Speaking/Writing
tasks and lists 22 enemy groups in `seed/enemy_groups.json`; reviewers may rename/merge. Status: proposed.

## D-011 · Architecture stack update: Astro.js frontend and Rust backend & APIs
Context: The product owner specified that the frontend stack will be Astro.js, the backend stack will be Rust, and all APIs will also be written in Rust.
Decision: Standardize on Astro.js (TypeScript strict + Tailwind CSS + accessible interactive client islands for timers, audio recording, audio playback without scrub, and writing editor) for the client application (`client/`), and Rust (Tokio async runtime + Axum HTTP framework + Serde typed data models + Tower middleware) for the backend and APIs (`server/`). The pure diagnostic engine (`shared/engine`) is implemented as a pure Rust crate with zero I/O and ≥95% branch coverage. Canonical data models are defined in Rust with Serde and mirrored as zod schemas in the Astro client for form and request validation.
Spec: 00, 02, 03, 07, 10, 11, AGENTS.md, GEMINI.md. Status: approved.

## D-012 · Wording Policy Word-Boundary and Mandatory Disclaimer Exemption
Context: Spec 05 §3.6 specifies the mandatory negative disclaimer "GEPA does not predict official exam scores." Because "predict" and "official" are prohibited from positive high-stakes claims, the claims policy scanner must explicitly exempt the exact mandatory disclaimer so it can be prominently presented to the candidate. In addition, single prohibited tokens match on word boundaries so standard descriptors (such as "predictable routine information") do not trigger false positives.
Decision: Exclude the exact negative disclaimer string in `shared_engine::wording_policy` and enforce word boundaries via compiled `RegexSet`.
Spec: 05 §3.6, 05 §4. Status: approved.

## D-013 · Gemini Multimodal AI Rating Architecture & Versioned Prompts
Context: Spec 07 requires automated AI scoring of Speaking audio and Writing prose using multimodal Gemini models, with strict rubric gates, disfluency-preserving transcripts, trait invariant enforcement (scores in [0.0, 5.0], upper <= lower), and dual-rater adjudication when differences exceed 1.0.
Decision: Implement `server/src/ai` module with `GeminiClient` supporting both live multimodal calls and offline deterministic simulation fallbacks. Implement versioned prompt registry (`speaking_rater@v2.0`, `writing_rater@v2.0`, `transcribe@v1.0`, `listen_to_write_check@v1.0`, `consistency_review@v1.0`). Any rater disagreement (mean score delta >= 1.0) or AI-suspect flag automatically enqueues the session into the human reviewer queue.
Spec: 05 §2, 07 §1-§5, 11 (M7, M8). Status: approved.

## D-014 · Non-Destructive Audit Trail for Rescoring and Human Scoring
Context: Spec 02 §6 and Spec 10 require that when an assessment session is rescored (e.g. following a rubric recalibration or expert examiner review), previous ratings must never be overwritten or deleted from the datastore.
Decision: When `rescore_session` or `human_score_session` is executed, the previous rating has its `superseded_by` field set to the identifier of the new rating (`Some(new_rating_id)`). The new rating records the reason, adjudicator ID, and timestamp. The result assembly engine strictly queries ratings where `superseded_by.is_none() && usable == true`, guaranteeing full historical provenance and reproducibility.
Spec: 02 §6, 03 §4, 10 §2. Status: approved.

## D-015 · GDPR Data Retention & Deletion Lifecycle Implementation
Context: Spec 09 §3 and UK GDPR mandate strict data minimization: audio recordings must be pruned after 90 days unless explicit research consent was given, writing submissions anonymized and retained for 24 months, and candidates provided with a "Delete my data" self-service right to erasure.
Decision: Implement dedicated operational runner (`retention:run` / `tools/retention_runner`) and API endpoint (`POST /api/admin/retention/run`) enforcing these retention limits. Implement self-service candidate erasure which hard-deletes audio blobs and draft buffers while maintaining only anonymized exposure counts.
Spec: 09 §3, 11 (M10, M12). Status: approved.

## D-016 · GitHub Actions Compute Offload & Multi-Stage Cloud Run Deployment
Context: Architecture 02 §1 & §8 and production VPS reliability rules require that compute-intensive builds (Cargo release linking, Astro SSG bundles, test matrices, Docker image construction) and scheduled telemetry/retention batches do not overload the production host or local dev environments.
Decision: Offload all heavy computation to GitHub Actions workflows: `.github/workflows/ci.yml` (multi-crate verification, clippy, engine tests, Astro client build), `.github/workflows/deploy-cloud-run.yml` (Docker Buildx multi-stage image builds with GitHub Actions layer cache, deploying pre-built images to Cloud Run), and `.github/workflows/scheduled-maintenance.yml` (daily cron running GDPR retention purge and exposure telemetry aggregation). The production VPS runs only the lightweight runtime container with minimal CPU, memory, and disk pressure.
Spec: 02 §1, 02 §8, 11 (M12). Status: approved.

## D-017 · Idempotency Caching, Rapid-Click Rate Limiting, and In-Process Job Queue
Context: Architecture 02 §3–§5 mandates idempotent submission contracts, concurrency state versioning, rapid response rate-limiting (min 750ms interval, flagged but accepted), and an in-process persistent job queue for productive ratings and background tasks.
Decision: Implement in-memory `idempotency_cache` and `jobs` queue within `AppState`. In `POST /api/sessions/:id/responses`, `submit_speaking`, and `submit_writing`, inspect `Idempotency-Key` headers; duplicate requests return cached responses immediately without redundant engine execution. Enforce 750ms minimum response interval; requests faster than 750ms receive the `rapid_response_burst` flag. Concurrency is guarded by incrementing and exposing `state_version` on every valid state transition.
Spec: 02 §3.4, 02 §4, 02 §5, 02 §7. Status: approved.

## D-018 · Production VPS Zero-Compute Policy & GitHub Actions Offload
Context: The production VPS must serve the live application reliably with minimal CPU, memory, and disk pressure. Running compilation (Rust `cargo build`, Astro bundling), test suites, or Docker image builds on a production VPS causes CPU spikes, memory exhaustion, and disk bloat that degrade live candidate sessions.
Decision: Strictly enforce a Zero-Compute Host policy. All compilation, linting, testing, and Docker image composition are executed exclusively on GitHub Actions runners via `.github/workflows/ci.yml`, `.github/workflows/deploy-cloud-run.yml`, and `.github/workflows/deploy-vps.yml`. Production VPS deployments pull pre-built, multi-stage optimized runtime images from GitHub Container Registry (`ghcr.io`) with strict resource constraints (1.0 CPU, 512MB RAM cap) and automated dangling image pruning (`docker image prune -f`), leaving the production host dedicated 100% to serving live traffic.
Spec: 02 §1, 02 §8, 11 (M12). Status: approved.

## D-020 · Model tiering placeholder fix (MODEL_RATER distinct from MODEL_FAST/MODEL_TTS)
Context: `.env.example` had `MODEL_RATER`, `MODEL_FAST` and `MODEL_TTS` all pinned to the same flash model, contradicting
`02_ARCHITECTURE.md §1` ("MODEL_RATER (Pro-class...)") and Q-002 (agent runs `models.list` at M1 to pin real IDs). No
`GEMINI_API_KEY` is available in this build environment, so a live `models.list` call cannot be made.
Decision: pin `MODEL_RATER=gemini-2.5-pro` (distinct Pro-class placeholder) and leave `MODEL_FAST`/`MODEL_TTS` on the
flash model, with an inline comment marking these as placeholders pending a real `models.list` run once a key exists.
Spec: 02 §1, Q-002. Reversible: yes. Status: proposed.

## D-021 · Postgres + self-issued JWT replaces Firestore + Firebase Auth (product owner directive)
Context: `00_MASTER_SYSTEM_PROMPT.md §4`, `02_ARCHITECTURE.md §1` and `03_DATA_MODEL.md §5` specify Firestore + Firebase
Auth (anonymous candidates, Google sign-in for staff) as the persistence/identity layer. The product owner explicitly
overrode this mid-build: **no Firebase in any form** (not even the local Emulator Suite, which needs a JVM this
environment should not have installed) — Postgres for storage, plain Rust for everything else, and all DB-dependent
verification runs in GitHub Actions (a Postgres service container), never against a database on the dev machine.
Decision:
- **Persistence**: `server/src/db.rs` (`PostgresClient`, via `tokio-postgres` + `deadpool-postgres`, no query-macro/ORM
  crate — avoids a compile-time-checked-query tool needing a live DB at build time, and keeps the dependency tree
  small on a slow-building toolchain). Three tables (`sessions`, `jobs`, `review_queue`) share one shape —
  `id TEXT PRIMARY KEY, data JSONB, version BIGINT` — so the whole domain model (`SessionState`, `BackgroundJob`,
  `FlaggedSessionReview`) round-trips through `serde_json` unchanged; `repos.rs`'s `SessionRepo`/`JobRepo`/
  `ReviewQueueRepo` kept their exact public API from the Firestore design, only their backing client changed.
  Optimistic concurrency uses an integer `version` column (`UPDATE ... WHERE version = $expected`) in place of
  Firestore's `updateTime` precondition. Migrations are one plain SQL file (`server/migrations/001_init.sql`), embedded
  into the binary via `include_str!` and run idempotently at startup — no migration-framework crate for a handful of
  tables at pilot scale.
- **Auth**: `server/src/auth.rs` issues its own HS256 JWTs (`JwtService`, secret from `JWT_SECRET`) instead of
  verifying externally-issued Firebase ID tokens — this removes the JWKS-fetching/emulator-vs-prod branching entirely,
  not just the dependency. Candidates need no external identity provider at all: `POST /api/sessions` mints a fresh
  anonymous `candidate_uid` and returns a token in the same response (`CreateSessionResponse.token`), so the client
  never needs a separate "sign in" step first. Reviewers/admins log in with email + password
  (`POST /api/auth/login`) against a new `staff_users` table (Argon2-hashed passwords, `tools/create_staff_user`
  provisions the first accounts) instead of Google sign-in + custom claims. The `AuthUser`/`StaffAuth`/`AdminAuth`
  Axum extractors and every `api.rs` handler signature are unchanged — only what they verify against changed.
- **Rules enforcement**: Firestore's `firestore.rules`/`storage.rules` (and their emulator-based unit tests) are
  removed — there is no separate rules layer with Postgres; the same ownership/role checks that already lived in
  `api.rs` (`require_session_owner`, `assert_owns_or_staff`) are now the *only* enforcement point, which is exactly
  what they always were for anything routed through the API (D-009).
- **Local dev**: nothing is installed or run locally to support this. `cargo check`/`cargo test` compile and run the
  DB-independent parts (the 47 pure-engine tests) without any database. Anything that needs a live Postgres connection
  is verified exclusively in `.github/workflows/ci.yml` (`postgres:16-alpine` service container) — see D-016/D-018's
  existing GitHub-Actions-offload precedent, now extended to cover this too.

Alternatives considered: `sqlx` (heavier dependency tree, its compile-time query macros need a reachable DB at build
time — directly conflicts with "nothing DB-related runs locally"); keeping the Firestore Emulator Suite for local/CI
parity (rejected outright by the product owner — it requires a JVM, which must not be installed on this machine).
Spec: 00 §4, 02 §1, 03 §5, 09 §1-§2, 11 (M4, M5, M10). Reversible: architecturally yes (the repo-layer abstraction
means swapping backends again is a `db.rs`/`auth.rs` change, not a `services.rs`/`api.rs` rewrite) but not a decision
expected to be revisited. Status: approved.

---
<!-- Agent appends from here. Next id: D-022 -->

## D-022 — Candidate-view sanitization of productive task payloads + M12 hardening gates
Context: the M12 "log redaction test" / "grep secrets in bundle" / "bundle < 400 kB gz" acceptance items were
never actually implemented — and implementing the bundle scan immediately found two real server-side leaks that
the e2e security spec had structurally missed (it only scanned the receptive journey's traffic):
(1) `GET /api/sessions/:id/speaking|writing/start` serialized the full `SpeakingTask`/`WritingTask` structs,
including `audio_script` (present on 12 SPK + 6 WRT seed tasks) and `interlocutor_line` — hand-delivering
rating material (model scripts, interlocutor target lines) to every candidate's network tab; (2) the client
bundle shipped a dead `RestrictedKey` zod schema declaring the shape of the restricted answer-key collection
(`key_option_id`, `authoring_letter`, `answer_text`, `rationale` field names) — no values, but the shape is
server-only material per AGENTS.md's "Never" list. The Gemini client also carried its API key in the request
URL (`?key=`), where any reqwest error Display (which embeds the URL) could leak it into logs.
Decision:
- **Server-side candidate view**: `get_speaking_tasks`/`get_writing_tasks` (server/src/services.rs) null out
`audio_script`/`interlocutor_line` before returning. Root-cause fix at the single serialization point every
route shares, rather than serde `skip_serializing` on the shared engine model (the server itself needs those
fields internally for rating prompts and audio production).
- **Client schemas match the wire**: `client/src/schemas/api.ts` drops `audio_script`/`interlocutor_line` from
`SpeakingTask`/`WritingTask` and deletes the dead `RestrictedKey` schema entirely; a schema test now asserts
those field names are absent from the candidate-facing shapes. The e2e `security.spec.ts` extends its leak
scan to the productive phase (drives through `/speaking/start`, fetches `/writing/start` with the session
token, and fails on the field *names* `audio_script`/`interlocutor_line` as well as verbatim script text from
all three seed banks).
- **Credential hygiene**: Gemini API key moves from URL query to the `x-goog-api-key` header
(`GeminiClient::generate_content_url`), so no reqwest error Display can ever embed it.
- **Logging redaction, two layers** (AGENTS.md §3 redaction list): a static source-scan test
(`server/src/logging.rs::log_statement_source_scan`) fails `cargo test` if any `tracing::*!` macro in
`server/src` interpolates a redacted identifier (string-literal contents are stripped first, so message prose
can't false-positive); and a CI runtime test boots the built container with canary secret values plus a wrong
DB password (to actually exercise the pool-error/migration-failure paths) and greps the container's full log
output — including tower_http TraceLayer and deadpool internal events — for those canaries.
- **CI bundle gates**: `scripts/scan_bundle_secrets.py` (secret-shaped strings, restricted-key markers,
listening/seed script text) and `scripts/check_bundle_size.py` (initial gzipped payload < 400 kB per spec,
single-chunk < 250 kB) run right after `npm run build:client` in CI.
Alternatives: serde `skip_serializing` on the engine models (rejected: the server needs the scripts
internally); scanning only built output without the server-side strip (rejected: the leak was server-side,
the bundle scan was just what surfaced it); per-endpoint allowlist DTOs (heavier; the null-out at the shared
service boundary covers every current and future route through the same function).
Spec: 09 §1–§2, 11 (M12), AGENTS.md §3 "Logging" + "Never" list, 06 §4–§8. Reversible: yes (but reversing it
reintroduces the leaks). Status: approved.



## D-023 — Deploy target: shared-platform VPS profile (no per-app database, no published ports)
Context: Q-008 (the final M12 item) needed a deploy target. The owner directed that the production VPS runs
**shared infrastructure** — the platform stack at `/opt/platform` on the shared box (rules in
`/opt/platform/PLATFORM-RULES.md`, VPS-only doc; the same pattern UBAG already follows there): one shared
Postgres instance for every project on the box, per-project credentials generated by
`/opt/platform/bin/provision-project.sh` into `/opt/platform/projects/<app>.env` (never committed), ingress
through the shared Nginx Proxy Manager container, and zero compute on the box (D-016/D-018). Two defects in
the previous deploy path violated this and are fixed: `docker-compose.prod.yml` declared a dedicated
`postgres` service (per-app database — wrong under the shared-infra policy), and it published host port
8080 (the box has no active firewall, so a 0.0.0.0-bound port is directly internet-reachable; the NPM-only
ingress pattern exists precisely to avoid this). The workflow also interpolated `${{ github.repository }}`
(`jerryboganda/GEPA`) directly into a docker image ref, which must be lowercase, and its smoke check curled
`localhost:8080` on the host — unreachable once no ports are published.
Decision:
- `docker-compose.prod.yml` becomes the **shared-platform profile**: a single `gepa-server` service joining
  the external `platform` network (shared `platform-postgres` via `DATABASE_URL`) and the external
  `nginx-proxy-manager_default` network (NPM reaches `gepa-server-prod:8080` by container name). Required
  env (`DATABASE_URL`, `JWT_SECRET`) fails fast (`:?`) with a pointer to RUNBOOK §9.6 instead of starting
  degraded. Resource limits unchanged (1.0 CPU / 512 MB cap on the shared box).
- `deploy-vps.yml` `deploy-to-vps` job: gated on build-and-push success (was `if: always()`, which could
  print a green deploy notice after a failed build); prepares `/opt/docker/gepa`, copies the **exact tagged**
  compose file from the workflow checkout via `appleboy/scp-action@v1.0.0`, lowercases the image repo, pulls
  and starts with `--env-file /opt/platform/projects/gepa.env`, and smokes `/healthz` via
  `docker compose exec` inside the compose network with retry headroom for first-boot migration.
- GEPA consumes **only** shared Postgres: the server's code path is Postgres-only (self-issued JWT, no
  queue, no object store). Shared Redis/NATS/MinIO are available on the `platform` network but deliberately
  not referenced — no invented wiring (AGENTS.md "Never" list).
- The GHCR package must stay **private**: the image bakes in the whole `seed/` directory including
  `RESTRICTED_answer_keys.json` / `RESTRICTED_keys_review.md` (AGENTS.md: RESTRICTED_* are server-only
  assets). The workflow authenticates the pull with its short-lived `GITHUB_TOKEN` (expanded Actions-side,
  masked in logs), so no long-lived PAT is needed.
Alternatives: keeping a per-app Postgres in the compose (rejected: owner's shared-infrastructure policy;
duplicates state and memory on a 6-core box); Cloud Run (deferred — needs a GCP billing project, Q-006);
publishing a 127.0.0.1-bound port for host-side smoke (rejected: policy is NPM-only ingress, no new
listeners on an unfiltered host).
Spec: RUNBOOK §9.6 + §10.3, QUESTIONS.md Q-008, AGENTS.md §5, 02 §8. Reversible: yes (compose is
re-deployable per revision). Status: approved.
