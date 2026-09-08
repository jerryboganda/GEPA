# 08 — Item Bank, Seed Files, Form Assembly and Exposure

Blueprint §6–§9, §13, §19; Item Bank §1 and §7; Keys §1 and §10. The pilot bank is for expert review, cognitive labs and
closed beta — **not** a public production bank (production needs ≥6× an operational form, parallel forms, exposure control).

## 1. Seed files (`/seed`, generated from the PDF by `scripts/parse_pdf_to_seed.py`; checksums in `manifest.json`)

| File | Count | Notes |
|---|---|---|
| `language_systems.json` | 52 items (Pre-A1 4; A1–C2 8 each) | `locator_candidate` on first two items per A1–C2 band; 3 options below B1, 4 from B1 |
| `reading.json` | 21 stimuli / 42 items (3 stimuli × 2 items per band) | stimulus text is candidate-facing |
| `listening.json` | 21 scripts / 42 items | `text` = **admin script** (server-only), `target_wpm`, `est_duration_sec`, `speaker_count` |
| `speaking_tasks.json` | 48 prompts (6 routes × 8) | `audio_script` (SR full sentence, RT quoted message), `interlocutor_line` (INT) for TTS; `weight`, `traits_scored`, kit-assigned `topic_family` |
| `writing_tasks.json` | 24 (6 routes × 3 scored + 1 diagnostic) | `word_guidance`, `weight` 0.3/0.3/0.4/0 |
| `RESTRICTED_answer_keys.json` | 136 keys | `key_option_id` (opaque), `authoring_letter`, `answer_text`, `rationale` (LS) — **server-only** |
| `rubrics.json` | — | trait descriptors, weights, evidence thresholds, integrity rules |
| `routing_ruleset_v2beta.json` | — | all router parameters |
| `results_and_claims.json` | — | headline/confidence rules, allowed/forbidden wording, readiness layer |
| `enemy_groups.json` | 22 groups | cross-skill overlaps with severity |
| `item_bank_review.md`, `RESTRICTED_keys_review.md` | — | human-readable renderings |

Option identity: `option_id = "opt_" + sha256("GEPA-v2|<item_id>|<letter>")[:10]` — deterministic, opaque, stable
across regenerations. The client never sees letters.

## 2. Seed loader (`scripts/seed_load.ts`, admin-only endpoint wraps it)
1. `seed:validate` — parse every file with the zod schemas in `03`, verify `manifest.json` sha256, verify invariants:
   counts; 3/4-option rule; every key's option exists in its item; authoring letter distributions equal manifest;
   Speaking per route: OR+SR ≤ 0.20, spontaneous ≥ 0.60, weights sum 1.0; Writing weights sum 1.0.
2. Write candidate-safe collections (`items`, `stimuli_reading`, `stimuli_listening` **without** `text`,
   `speaking_tasks`, `writing_tasks`, `rulesets`, `rubrics`, `claims_config`).
3. Write restricted collections in a **separate** step (`restricted_keys`, `stimulus_admin` with script + production
   metadata). Log counts only.
4. Idempotent: documents are versioned (`version`, `status`); re-running updates in place and records a `seed_runs` doc.
5. Enrich Speaking tasks with `prepSeconds`, `maxSpeakSeconds`, `allowsRerecord` from the `06` timing table; Writing tasks
   with `timeLimitSeconds` from the `06` route table.

## 3. Form assembly (`server/services/formAssembler.ts`)
The beta form is the whole pilot bank; the assembler works **per session** to pick delivery order and to enforce rules:
- **Domain coverage:** each session's Reading and Listening must draw from ≥2 domains per visited band (pilot bands
  already mix public/educational/occupational/personal; assert, do not fail).
- **Enemy rules (`enemy_groups.json`):** when choosing the next stimulus/item, avoid members of a group already
  delivered in this session where severity is `high`; for `medium`/`low` prefer alternatives and, if none exist
  (pilot reality — single productive task set per route), deliver anyway and log `enemy_conflict{family,severity}` on
  the session. Reviewer report lists conflict frequency per family — this is evidence for authoring alternates.
- **LS block rule:** ≤1 inversion item per level block (only `LS-C1-01` exists; keep the check generic on `construct`).
- **No hidden arithmetic / specialist knowledge:** authoring QA checklist item; assembler does nothing at runtime.
- **Exposure caps:** per item `delivered` counter; the assembler prefers the least-exposed eligible item/stimulus
  within a band (ties broken by seeded RNG). Cap is informational in pilot (report), enforced in production.
- **Pretest seeding:** `FEATURE_PRETEST_ITEMS` inserts 2–3 `is_pretest` items per public session (unscored, excluded
  from routing). Off in pilot; code path must exist and be tested.
- **Key-position balance:** runtime shuffle guarantees uniform expectation; `forms:check` reports the delivered-position
  distribution from `exposure` and fails if any position > 40% for any module over ≥200 deliveries.

## 4. Status lifecycle
`draft → trial → active → retired`. Only `trial`/`active` items are deliverable in beta. Any edit creates a new
`version` and keeps the parent `item_id` (audit). Reviewer provenance entries required before `active`
(≥2 independent reviewers, adjudication note) — enforced by the admin UI, not by the runtime.

## 5. Known content overlaps to surface to reviewers (from `enemy_groups.json`)
- Identical/near-identical prompts across Speaking and Writing at the same route (e.g. meeting-time reply,
  class-booking, workshop places, learning alone vs group, the half-past-ten sentence) — high severity: writing after
  speaking rehearses content. The assembler logs them; authors should create alternates before public beta.
- Objective cross-skill overlap: `RD-C2-S3` and `LSN-C2-S3` share the heritage/"authentic" argument — a C2 candidate may
  meet both; the assembler must prefer another C2 listening stimulus first and log the conflict if unavoidable.
- Upper-level thematic clustering around simplification/uncertainty/claims persists across RD-C1, LSN-C1/C2 and the
  B2-C1/C1-C2 productive sets (medium severity).

## 6. Exposure and telemetry reports (`report:exposure`)
Per item: deliveries, p-value (proportion correct), median response ms, omission rate, replay rate (listening),
position distribution, sessions by band bracket. Per stimulus: deliveries, replay rate, buffer failures. Per productive
task: usable rate, mean traits at lower/upper, flag rates. Export CSV for Phase C/D2 analysis. No candidate identifiers.
