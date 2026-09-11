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

---
<!-- Agent appends from here. Next id: D-019 -->


