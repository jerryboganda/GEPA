# GEPA Session Handoff — 2026-09-12/13 (session 3)

**Read this first if you're a fresh session picking up this project.**
This file is intentionally uncommitted (local only). Delete or overwrite it next session.

## TL;DR — where things stand

- **`main` is at `50901a0`, CI fully green (both jobs verified `success` on that exact commit, run `34726271140`); daily
  maintenance workflow also re-validated green on demand (run `34726276684`).**
- **PR #1 was squash-merged** (`8d3c925`, branch deleted). The repo is `jerryboganda/GEPA`, public.
- All owner-independent M12 hardening items are **done and gated in CI**, and the last cosmetic item (Node 20 deprecation
  annotations) was closed on 2026-09-13: all workflow actions bumped to Node 24-native majors (`50901a0`), release notes
  checked first, zero annotations remain in CI logs. What remains is **owner-only**: choosing a deploy target (Q-008) and
  the standing non-blocking Q-001..Q-007 confirmations.
- Nothing is mid-flight. Working tree is clean except this file (untracked). No open PRs.

Verify current state with:
```
cd D:\Projects\GEPA
git status                  # clean on main, only HANDOFF.md untracked
gh run list --branch main --limit 3
```

## Standing constraints (hard-learned — do not violate)

1. **Never install local system deps** (JDK incident, session 1). Ask first, always.
2. **All compute-heavy work goes to GitHub Actions** — explicit user directive this session. This machine is genuinely too
   slow for Rust: `cargo check -p server` took 5m20s; `cargo test --lib` timed out at 300s AND 600s twice. The workflow is:
   edit locally → sanity-check only the *cheap* things locally → push → read CI failure → fix. (User quote: "Move builds,
   tests, data processing, automation, and other heavy workloads to GitHub Actions wherever feasible.")
3. Local checks that ARE fine (all seconds-fast): `npm --prefix client run typecheck` (tsc), `npm --prefix client run test`
   (node schema tests), `npm --prefix client run build` (Astro, ~5-7s), `python scripts/scan_bundle_secrets.py`,
   `python scripts/check_bundle_size.py`. These need a rebuilt `client/dist` first for the last two.
4. **Multi-line commit messages on this PowerShell setup**: use `git commit -F <file>` (write the file first). Inline
   `git commit -m "multi\nline"` gets mangled by PowerShell quoting — cost one failed commit this session.
5. `gh run watch` can die on transient network errors — prefer polling `gh run view <id> --json status,conclusion`.
6. Frontend changes must match the existing design system. Commits end with
   `Co-Authored-By: Claude Sonnet 5 <noreply@anthropic.com>`.

## What this session did (chronological)

1. Merged PR #1 (squash) per handoff instruction "continue ⇒ merge". CI green on `main` post-merge.
2. Audited M12 (docs/11_MILESTONES_TASKS.md) against what actually existed. Three owner-independent items were missing:
   **log redaction test, secrets-in-bundle grep, bundle-size gate**. Deploy steps need owner infra (no repo secrets exist —
   `gh secret list` is empty).
3. Implemented all three, and the new gates immediately caught **two real leaks** (details below — worth reading).
4. Fixed the leaks at the root, tightened the gates, iterated CI ×4 (two of the rounds were the gates catching
   over-strictness in *themselves*, not app bugs), final state green.

### The real leaks found and fixed (the valuable part)

**Leak A — productive task payloads shipped admin scripts.** `GET /api/sessions/:id/speaking/start` and `/writing/start`
serialized the full `SpeakingTask`/`WritingTask` structs, including `audio_script` (12 SPK + 6 WRT seed tasks carry real
scripts) and `interlocutor_line`. Every candidate could read rating material in the network tab. The old `security.spec.ts`
never caught it because it only scanned the *receptive* journey. Fix, layered:
- `server/src/services.rs` `get_speaking_tasks`/`get_writing_tasks` null out `audio_script`/`interlocutor_line` before
  returning (all routes share these functions; `e2e_seed_session` too).
- `shared/engine/src/models.rs`: `#[serde(skip_serializing_if = "Option::is_none")]` on those fields — otherwise serde
  ships `"audio_script":null` and the field NAME still reaches the client. (Safe: reviewer views hand-build their JSON,
  never serialize these structs; the seed loader only deserializes.)
- `client/src/schemas/api.ts`: dropped `audio_script`/`interlocutor_line` from candidate schemas; deleted dead
  `RestrictedKey` and `ListeningStimulusAdmin` zod schemas (they shipped restricted-collection field *shapes* into the
  public bundle — no values, but shape is server-only per AGENTS.md "Never").
- `client/tests/schemas.test.ts` asserts those field names are absent from the shapes.
- `security.spec.ts` now drives through the productive phase (clicks `receptive-continue-speaking`, waits
  `mic-check-begin-speaking`, fetches `/writing/start` with the token from `localStorage` — keys: `gepa_token`,
  `gepa_active_session`) and fails on the field names `audio_script`/`interlocutor_line`/`key_option_id`/
  `authoring_letter`/`"correct":true|false` plus verbatim scripts.

**Leak B — Gemini API key in the URL.** `?key={api_key}` meant any reqwest error Display (embeds the URL) could leak the
credential into logs. Moved to the `x-goog-api-key` header via `GeminiClient::generate_content_url`
(server/src/ai/client.rs). Also hardened `db.rs`'s pool-error log to a static message (its Display can embed the DSN
with password).

### The M12 gates now in CI (`.github/workflows/ci.yml`)

- **Static log-redaction scan** — `server/src/logging.rs` (new module, wired in `lib.rs`): a `cargo test` source scan
  that fails if any `tracing::*!` macro in `server/src` interpolates a redaction-list identifier. Two matching surfaces:
  code tokens outside string literals (named fields, positional args) AND `{ident}` inline captures inside literals;
  prose inside literals can't false-positive. `REDACTED_IDENTIFIERS` is the exported list (audio_url, transcript,
  option_id, key, script, email, api_key, secret, password, database_url, …). Unit tests pin the scanner's behavior.
- **Runtime log-redaction test** (container job, "Log Redaction Runtime Test (M12)"): boots a second container with
  canary `JWT_SECRET`/`GEMINI_API_KEY` and a wrong-password `DATABASE_URL` (so the pool-error/migration-failure paths
  actually execute), then greps full `docker logs` for the canaries. Confirmed running in the green run.
- **Bundle secrets scan** — `scripts/scan_bundle_secrets.py` after every client build: secret-shaped strings
  (AIza/sk-/JWT-shape/DSN-with-password), restricted field-name declarations (`"key_option_id"` etc. in the bundle),
  verbatim seed script text (listening stimuli + productive scripts not quoted in their own prompts).
- **Bundle size gate** — `scripts/check_bundle_size.py`: initial gzipped payload (index.html + exactly what it
  references) < 400 kB per spec, single chunk < 250 kB. Current: **~84 kB**.

### Critical domain knowledge discovered (don't re-learn this the hard way)

- **The seed quotes every script inside its own candidate-visible prompt.** RT tasks: `You hear: "<script>"`.
  INT tasks: the interlocutor line appears in the prompt. Listen-to-write: `Listen and write: "<sentence>"`. So verbatim
  script text in a task response is *indistinguishable from the prompt it legitimately ships with*. Therefore:
  verbatim scans must use **containment** (only flag script text NOT contained in its own task's prompt) — equality
  comparison was the bug in round 2 of CI. The genuinely enforceable invariants for this seed are: (a) field names never
  on the wire (skip_serializing_if + regex field checks — both green), (b) listening stimuli scripts never in any
  response (they're never in any candidate-visible text).
- serde serializes `Option::None` as `"field":null` — nulling a value does NOT remove the field name from the wire.
- `db.rs` never panics on a bad `DATABASE_URL` — server boots degraded (healthz 503, static assets still served). That's
  what makes the canary redaction test possible without a DB.
- The e2e security spec runs in both browser projects (desktop + mobile) — one failing session ID per project is normal
  in a failure (two `ses_…` URLs = same test, two projects).

## Commits this session (on main)

- `8d3c925` — PR #1 squash merge (Postgres + JWT auth)
- `25b6890` — security fixes + M12 gates (the big one: leaks fixed, logging.rs, both scan scripts, CI wiring, docs)
- `6a5c423` — logging scanner detects inline captures inside literals (its own self-test caught the gap)
- `d56b289` — skip_serializing_if (field names off the wire) + scan containment scoping + ListeningStimulusAdmin removal
- `cab2f54` — verbatim script scan scoped to non-quoted material (final, CI green)
- `50901a0` — CI: actions bumped to Node 24-native majors (session 4, zero deprecation annotations left; CI run
  `34726271140` + dispatched maintenance `34726276684` both green)

Docs updated alongside: `docs/DECISIONS.md` D-022, `docs/CHANGELOG.md` 2.0.0-beta.5 entry, `docs/RUNBOOK.md` §9.4–§9.5,
`docs/QUESTIONS.md` Q-008.

## What remains for "100%" (all owner-side; nothing is blocked)

1. **Q-008 — deploy target decision**: (a) VPS: set `VPS_HOST`/`VPS_SSH_KEY` (+ optional `VPS_USERNAME`/`VPS_PORT`) repo
   secrets → `deploy-vps.yml` pulls the pre-built GHCR image and brings up `docker-compose.prod.yml`; (b) Cloud Run:
   set `GCP_WORKLOAD_IDENTITY_PROVIDER`/`GCP_SERVICE_ACCOUNT` (needs billing project — overlaps Q-006); (c) later —
   pushing a `v*` tag publishes the image to GHCR regardless. Deploy workflows trigger on `v*` tags.
   Suggested next concrete step if the owner picks (a) or (b): tag `v2.0.0-beta.5` and watch the deploy workflow.
2. Q-001..Q-007 confirmations (all non-blocking; Q-007 = accept 19/52 audio assets outside ±8% WPM tolerance from the
   offline flite voice, or supply `GEMINI_API_KEY`/`ELEVENLABS_API_KEY` — audio_producer auto-upgrades, no code change).
3. ~~Optional polish: Node.js 20 deprecation annotations~~ — **done 2026-09-13** (`50901a0`): checkout→v7, setup-node→v7,
   upload-artifact→v7, setup-buildx→v4, login→v4, metadata→v6, build-push→v7 (release notes of every intervening major
   checked; all runtime/ESM swaps). Deliberately NOT bumped: `google-github-actions/auth@v2`, `deploy-cloudrun@v2`,
   `appleboy/ssh-action@v1.0.3` — no warnings, and they only execute in deploy workflows that stay untestable until
   Q-008 secrets exist; bump them during the first deploy dry-run instead.

## Debugging patterns that worked (unchanged from session 2)

- CI failure → `gh run view <run-id> --log-failed` (or `--log --job <job-id>`) → grep `##[error]` / the Playwright failure
  block → read the real error, never guess. The `Error:` lines in Playwright output name exact URLs and leak reasons.
- Server-side request tracing: set `RUST_LOG: 'server=info,tower_http=debug'` + `stdout/stderr: 'pipe'` on the server
  `webServer` entry in `client/playwright.config.ts` temporarily (revert after).
- E2E timing races: a click Playwright reports as landed doesn't mean React committed state — verify-and-retry
  (see `answerKeys.ts` / `journey.full.spec.ts` comments). `workers: 1` in playwright.config is load-bearing.

## File map for anything touched again

- `server/src/api.rs` — handlers (`start_speaking` ~L398, `e2e_seed_session` ~L1008, `review_session` ~L619)
- `server/src/services.rs` — `AssessmentService` (sanitization in `get_speaking_tasks`/`get_writing_tasks` ~L580-670)
- `server/src/logging.rs` — redaction scanner (self-contained)
- `server/src/ai/client.rs` — Gemini client (header auth, `generate_content_url`)
- `shared/engine/src/models.rs` — `SpeakingTask`/`WritingTask` (skip_serializing_if fields)
- `client/src/schemas/api.ts` — candidate-facing zod schemas (restricted shapes deleted)
- `client/src/components/CandidateJourney.tsx` — main UI (~1.8k lines)
- `client/tests/e2e/security.spec.ts` — the extended leak scan
- `scripts/scan_bundle_secrets.py`, `scripts/check_bundle_size.py` — CI gates
- `.github/workflows/ci.yml` — verify job + container job (redaction runtime test)
- `docs/{DECISIONS,CHANGELOG,RUNBOOK,QUESTIONS}.md` — D-022 / beta.5 / §9.4–9.5 / Q-008

## Suggested first moves for the next session

1. `gh run list --branch main --limit 3` — confirm still green (scheduled maintenance workflow also runs daily).
2. Ask the owner Q-008 (deploy target) and Q-007 (audio tolerance) — those are the only gates between "CI green" and
   "deployed product".
3. If owner says merge/tag: `git tag v2.0.0-beta.5 && git push origin v2.0.0-beta.5` triggers image publish + deploy
   (depending on secrets present).
