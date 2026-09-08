# GEPA v2 — Agentic Build Kit for Google AI Studio

Everything an autonomous coding agent needs to build the **General English Placement Assessment (v2 Free
Diagnostic Beta)** from the Master Draft v2 review package: one master system prompt, twelve numbered
specifications, agent operating rules, and machine-readable seed data parsed and validated from the source PDF.

## Contents

| Path | Purpose |
|---|---|
| `00_MASTER_SYSTEM_PROMPT.md` | Paste into AI Studio Build **System Instructions**. Governs everything. |
| `AGENTS.md` | Agent operating rules, commands, conventions (also usable as `GEMINI.md`). |
| `01_PRD.md` | Product requirements, journey, modes, claims policy, out-of-scope. |
| `02_ARCHITECTURE.md` | Stack (Astro.js + Rust), layers, folder layout, API surface, deployment. |
| `03_DATA_MODEL.md` | Rust Serde & Astro zod schemas, Firestore collections, security boundaries. |
| `04_ROUTING_ENGINE.md` | Exact adaptive-bracketing algorithm + confirmation + productive route, with test cases. |
| `05_SCORING_RESULTS_CLAIMS.md` | Objective scoring, evidence rules, headline/confidence rules, wording lint. |
| `06_MODULE_UX_SPECS.md` | Screen-by-screen behaviour for every module incl. pause/resume and timers. |
| `07_AI_SCORING_SPEC.md` | Gemini rating pipeline, prompts, JSON schemas, versioning, review queue. |
| `08_ITEM_BANK_AND_SEED.md` | Seed file schemas, loader, form assembly, exposure, key balance. |
| `09_SECURITY_PRIVACY_ACCESSIBILITY.md` | Threat model, Firestore rules, privacy, WCAG 2.2 AA, accommodations. |
| `10_TESTING_QA.md` | Test matrix, fixtures, forbidden-wording lint, `verify` command. |
| `11_MILESTONES_TASKS.md` | M0–M12 backlog with acceptance criteria — the agent's work order. |
| `12_AUDIO_PRODUCTION.md` | Listening/Speaking audio generation spec (TTS, loudness, WPM checks). |
| `DECISIONS.md` | Pre-seeded interpretation decisions; agent appends. |
| `QUESTIONS.md` | Blocked items for the human; agent appends. |
| `seed/` | Item bank JSON (no keys), `RESTRICTED_answer_keys.json` (server-only), rubrics, routing ruleset, results/claims config, enemy groups, manifest with checksums. |
| `scripts/parse_pdf_to_seed.py` | Regenerates `seed/` from an updated review PDF and re-validates counts/key balance. |

## How to run it in Google AI Studio (Build mode)

1. **Create a GitHub repo** (private) containing this kit at the root:
   `docs/` ← all `.md` files except `AGENTS.md`; `AGENTS.md` at root (copy as `GEMINI.md` too); `seed/`; `scripts/`.
2. In AI Studio → **Build**, start a new full-stack app and **import the repo** (or create the app, then add
   the files through the editor). Enable the **Firebase integration** (Firestore + Authentication).
3. Open **Settings → System instructions**, paste `00_MASTER_SYSTEM_PROMPT.md` in full.
4. First prompt to the agent:
   > Read /docs/AGENTS.md, then /docs/11_MILESTONES_TASKS.md. Start Milestone M0 and follow the autonomous loop
   > in the system instructions. Report in the required shape after each milestone and continue.
5. Add secrets when the agent asks (card appears in chat): `GEMINI_API_KEY` is auto-configured; add
   `ELEVENLABS_API_KEY` only if you choose that TTS provider (see `12_AUDIO_PRODUCTION.md`).
6. Review `DECISIONS.md` and `QUESTIONS.md` after M3, M7, M9 and before M12 (deploy).

Works equally with Gemini CLI / Antigravity IDE: keep `GEMINI.md` = `AGENTS.md` and paste the master prompt as the
session system prompt.

## Regenerating seed data

```bash
python3 scripts/parse_pdf_to_seed.py GEPA_Master_Draft_v2_All_in_One_Review.pdf seed
# exits non-zero if counts or authoring key-letter distributions differ from the blueprint
```

## Status of this package

Derived from **GEPA Master Draft v2 — Review Edition (8 Sept 2026)**. It is a build specification for the
**Free Diagnostic Beta**. It is not a validated test, not a certificate, and not the final developer handover
described in blueprint §19 — the agent's `DECISIONS.md` output plus your approvals become that handover.
