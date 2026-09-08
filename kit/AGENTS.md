# AGENTS.md — operating rules for the GEPA coding agent

Read this after `00_MASTER_SYSTEM_PROMPT.md`. It is the practical rulebook: repo layout, commands, conventions,
what to log, what never to do. Also copied as `GEMINI.md` for Gemini CLI / Antigravity.

## 1. Repository layout (logical; adapt names to the AI Studio runtime, keep the boundaries)

```
/docs                      ← this kit (specs). Read-only for you except DECISIONS.md, QUESTIONS.md, CHANGELOG.md
/seed                      ← item bank + config. Read-only. RESTRICTED_* files are server-only assets.
/scripts                   ← PDF→seed parser, seed loader, audio production job, exposure report
/shared
  /schemas                 ← Rust Serde models & client zod schemas (Item, Stimulus, Session, Response, Scoring…)
  /engine                  ← Rust PURE functions (crate): routing, objective_scoring, evidence_rules, headline, confidence
  /copy-policy             ← forbidden/allowed wording lists (from seed/results_and_claims.json)
/server                    ← Rust backend & APIs (Tokio + Axum)
  /src/api                 ← route handlers (thin): sessions, items, responses, media, results, admin, review
  /src/services            ← session_service, form_assembler, rating_service, audio_service, exposure_service
  /src/repos               ← Firestore access (Google Cloud / Admin SDK) — only place touching restricted collections
  /src/ai                  ← Gemini client (reqwest / Google GenAI), prompt registry, JSON schemas for structured output
  /rules                   ← firestore.rules, storage.rules
/client                    ← Astro.js application
  /src/pages               ← Astro routes: index, session, exam modules, results, review, admin
  /src/components          ← accessible primitives & islands (Timer, AudioPlayer, Recorder, Editor, Button, RadioGroup)
  /src/copy                ← all user-facing strings (lint-checked)
  /src/a11y                ← focus management, live regions, preferences (contrast, spacing, text size)
/tests
  /unit                    ← Rust cargo test for engine + services
  /fixtures                ← routing traces, rating fixtures, forbidden-wording cases
  /e2e                     ← Playwright journeys + axe scans
```

## 2. Commands you must provide (workspace scripts)

| Script | Does |
|---|---|
| `dev` | Astro client (`astro dev`) + Rust API server (`cargo watch -x run` or `cargo run`) |
| `typecheck` | `cargo check --workspace && tsc --noEmit` across all packages |
| `lint` | `cargo clippy` + ESLint + Prettier check + `copy-lint` (forbidden wording scan of `client/src/copy` and results templates) |
| `test` | `cargo test --workspace` with coverage; fails if `shared/engine` branch coverage < 95% |
| `test:e2e` | Playwright smoke journey + axe-core WCAG 2.2 AA scan on every screen |
| `seed:validate` | validates `/seed/*.json` against schemas + manifest checksums + count/key-balance invariants |
| `seed:load` | loads seed into Firestore (candidate-safe collections + restricted collections separately) |
| `audio:produce` | generates listening/speaking audio assets per `12_AUDIO_PRODUCTION.md`, writes QC report |
| `forms:check` | key-position balance, domain coverage, enemy-group conflicts for the assembled beta form |
| `report:exposure` | item exposure and response-time report |
| `verify` | `typecheck && lint && test && seed:validate && test:e2e` — must be green at every milestone end |

## 3. Conventions

- **Types:** define canonical models in Rust (`shared/engine` / `server/schemas`) with `serde::{Serialize, Deserialize}`; export/mirror client zod schemas in `client/src/schemas`. No `any`; use strict types.
- **Engine:** pure, deterministic, synchronous Rust crate. Inputs are plain structs; randomness (shuffling) is injected as a
  seeded RNG so tests are reproducible. Every branch in `04_ROUTING_ENGINE.md` and `05_SCORING…md` maps to a
  test named after the rule (e.g. `locator_one_of_two_tie_correct_brackets_current_next`).
- **Server handlers:** `auth → Serde/validator parse → service → Result<Json<T>, AppError>`. Never return raw Firestore docs.
- **Candidate item payload** contains exactly: `item_id, module, band(hidden from UI), stem, options[{option_id,text}]`
  in the **session's shuffled order**, plus stimulus text (Reading) or a **signed audio URL** (Listening —
  never the script). A unit test asserts the payload schema has no `key`, `correct`, `authoring_letter`,
  `rationale`, `script`, `text` (for listening stimulus) fields.
- **IDs:** keep seed IDs (`LS-B1-04`, `RD-B2-S2`, `SPK-B1B2-ER1`, `WRT-C1C2-3`) as document IDs; sessions and
  responses use ULIDs.
- **Time:** all timestamps UTC ISO-8601 on the server; the client never decides timeouts — it displays the
  server-issued deadline and the server enforces it with a 3-second grace window.
- **Copy:** every string shown to a candidate lives in `client/src/copy`. The results copy is generated from
  `seed/results_and_claims.json` allowed wording. The `copy-lint` fails on any forbidden phrase (see
  `10_TESTING_QA.md`), case-insensitive, including "CEFR-aligned", "validated", "certified", "your level is",
  "B1+", "B2-", "%" inside confidence text, "IELTS band", "OET grade".
- **Accessibility:** semantic HTML first; ARIA only to fill gaps. Every interactive element keyboard-reachable
  and visible-focus. Audio player and recorder are custom accessible components (see `06_MODULE_UX_SPECS.md`).
- **Errors:** typed `AppError{code, httpStatus, safeMessage}`. Candidate-facing messages never mention items,
  keys, models or internals.
- **Logging:** pino/structured JSON; redaction list: audio URLs, transcripts, option ids, keys, emails.

## 4. Logs you maintain

- `docs/DECISIONS.md` — `D-###` entries: context, decision, alternatives, spec reference, reversible?
- `docs/QUESTIONS.md` — `Q-###` entries: what is blocked, what you did meanwhile, options for the human.
- `docs/CHANGELOG.md` — one section per milestone: files, tests, verify output summary.

## 5. Never

- Never put keys, letters, rationales, listening scripts or reviewer notes in any client-reachable place.
- Never average CEFR bands numerically; never output ±/quarter levels; never show "High" confidence.
- Never write UI copy that predicts an external exam score or uses forbidden wording.
- Never penalise a replay, a slow response, an accommodation, an empty recording or a suspected AI text.
- Never fabricate test results, model IDs, or library versions — check them (`models.list`, `npm view`).
- Never delete or weaken a test to pass a milestone. Never leave `TODO`, `FIXME`, stub bodies or mock data in shipped paths.
- Never block on a question when a conservative default exists — log it and continue.

## 6. When the runtime constrains you

AI Studio may impose a single-app structure, its own dev server or its own auth wiring. Accept that, but keep:
(a) the pure engine as its own importable module with its own tests; (b) all restricted data behind server code;
(c) the endpoints in `02_ARCHITECTURE.md` (names may change, contracts may not). Record any forced deviation as a
`D-###` entry with the reason "runtime constraint".
