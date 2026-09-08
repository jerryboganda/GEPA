# 10 — Testing and QA

The `verify` suite (`cargo test` + `cargo clippy` + Astro build + Playwright e2e) must be green at the end of every milestone. Tests are specifications: if a rule in `04`/`05` has no
test, the milestone is not done.

## 1. Unit tests (`cargo test`) — `shared/engine` (Rust crate) ≥95% branch coverage
- `routing_test.rs` — every case in `04 §4` (locator, confirmation, productive route) + property tests:
  never >8 locator items; no item reuse; outcome band ∈ {L−1, L, U}; no numeric level; deterministic with seeded RNG.
- `objective_scoring_test.rs` — key identity, permutation stored, omissions, shuffle determinism per (session,item).
- `evidence_rules_test.rs` — Speaking/Writing decisions: fixtures for lower-only, upper, below_route, insufficient (<3 / <2
  usable), core-trait floor (grammar 1 blocks upper), independent-response count (INT turns count once), diagnostic tasks
  never change band, listen-to-write never changes band.
- `result_assembly_test.rs` — headline: even (lower median), uneven (range), one skill insufficient → none, foundation-only →
  none + label; cap invariant `headline ≤ min + 1`; confidence: every trigger yields Low; never High; readiness text per
  target; wording guard throws on forbidden phrase.
- `form_assembler_test.rs` — enemy high-severity avoidance, conflict logging when unavoidable, domain coverage, least-exposed
  preference, pretest insertion when flag on, ≤1 inversion.
- `payload_schemas_test.rs` — candidate item/stimulus/task payload schemas reject any object containing keys:
  `key`, `key_option_id`, `correct`, `authoring_letter`, `rationale`, `script`, `answer_text`, listening `text`.
- `copy_lint_test.rs` — fixtures of forbidden and allowed strings (`05 §4`).

## 2. Seed validation (`seed:validate`)
Schemas, checksums, counts (52/21+42/21+42/48/24), 3/4-option rule, key↔option existence, key-letter distributions
equal manifest, Speaking weight caps, Writing weights, enemy group members exist, route coverage (6 routes × 8 speaking,
6 × 4 writing). Fails the build on any deviation.

## 3. Server tests (Rust / Axum)
- Route handlers with `axum::test` / `tower::ServiceExt`: auth required; Serde/validator rejections (400); idempotency; `responses` rejects an item that is not
  current (409); listening URL third-play refusal; rate limiting; no `correct` in any response body (regex over JSON).
- Firestore rules tests (emulator): candidate cannot read `restricted_keys`, `stimulus_admin`, `ratings`, other sessions,
  `responses`; can read own session and result.
- Rating pipeline with a mocked Gemini client: schema validation, trait-key equality, inconsistency re-run, second-opinion
  disagreement flag, retry/backoff → `usable:false` after 5 failures, original preserved on rescore.
- Job worker resumes pending jobs on startup.

## 4. E2E (Playwright)
- `journey.full.spec.ts`: start → worked example → LS → RD → LSN → receptive result → speaking (mocked mic via fake media
  stream + fixture audio) → writing → full result, with a deterministic seeded session (test-only endpoint enabled by
  `E2E_MODE`), desktop 1280 and mobile 390 widths.
- `journey.keyboardOnly.spec.ts`: same journey without pointer events.
- `journey.partial.spec.ts`: stop after receptive → "Foundation and Receptive Profile" label; Speaking skipped → not measured.
- `resume.spec.ts`: reload mid-module restores state; simulated offline during listening → same audio replays once.
- `claims.spec.ts`: results DOM contains no forbidden phrases; skills rendered above and larger than headline; no
  external exam score text; confidence ∈ {Low, Moderate}.
- `a11y.spec.ts`: axe-core WCAG 2.2 AA on every screen state (including dialogs, timer warning, recording) → 0 violations.
- `security.spec.ts`: intercept network — no response contains `key_option_id`, `authoring_letter`, listening script text.

## 5. Fixtures (`tests/fixtures`)
- `routingTraces.json`: the tables in `04 §4` encoded as inputs → expected outputs.
- `ratings/*.json`: synthetic ProductiveRating sets for every evidence-rule branch.
- `audio/`: 3 short fixture recordings (clear speech, silence, clipped) for the quality gate.
- `copy/forbidden.txt`, `copy/allowed.txt`.

## 6. Manual QA checklist (per release)
Worked example feedback correct; timers match `06`; extended time ×1.5 applied everywhere; replay logged not penalised;
mic-check failure → skip path; re-record limits; autosave restores text after reload; results order/prominence;
"Delete my data" works; reviewer rescore preserves original; admin form check and exposure report run; deployed
`/healthz` OK; secrets not present in client bundle (`grep -r "GEMINI" dist/` → nothing).

## 7. Definition of "verified" in milestone reports
Paste the command and the summary line (tests passed/failed, coverage %). A milestone claiming tests that were not run
is a defect; fix by running them.
