# 01 — Product Requirements (GEPA v2 Free Diagnostic Beta)

Source: Master Draft v2 blueprint §1–§3, §10–§12, §16–§18. Where this PRD summarises, the blueprint wording wins.

## 1. What GEPA is

A free, low-friction web placement assessment that estimates **general English proficiency** across receptive and
productive skills using CEFR as the reference framework, **without claiming formal CEFR alignment before validation**.
It samples personal, public, educational and occupational domains; profession-specific knowledge is excluded.
After measurement, an optional target (OET, IELTS, TOEFL iBT, PTE Academic, Cambridge/DET, university, work,
general) drives only the interpretation layer: readiness to start exam-specific preparation and which skill gaps
deserve priority.

## 2. Users

| User | Needs | Auth |
|---|---|---|
| Candidate (healthcare professionals and general learners, often on mobile, often L1 Arabic/Urdu/others) | Fast start, clear instructions, resume after interruption, honest result, next steps | Anonymous Firebase session (upgradeable to email later) |
| Reviewer (rater / QA) | Flag queue, listen/read responses, adjust or re-score with versioned rubric, see AI rationale | Google sign-in, role `reviewer` |
| Admin (product owner) | Seed load, form checks, audio production, exposure/telemetry reports, claims-copy control | Google sign-in, role `admin` |

## 3. Modes

- **Free Diagnostic Beta (this build):** account/session controls, option shuffling, exposure control, basic telemetry.
  Output = indicative profile, no certificate.
- **Future Verified Mode (design for, do not build):** identity verification, stronger proctoring, secure separate item
  pool, potential verified report. Data model must carry `mode` and leave room for identity/proctoring fields.

## 4. Candidate journey (must be implemented exactly in this order)

1. **Start screen** — optional target goal; consent/privacy essentials; all background questions skippable or deferred.
   Language of instructions selectable (English default; localised instructions + icons permitted for Pre-A1/A1 flows).
2. **Unscored worked example** for the first task type (one Language Systems MCQ with feedback).
3. **Foundation Profile** — Language Systems → Reading → Listening. Each module routes independently (`04`).
4. **Immediate provisional receptive result** — candidate may stop here and receive a clearly labelled
   **"Foundation and Receptive Profile"** (partial).
5. **Optional Full Profile** — Speaking (microphone check happens **only** when Speaking starts) → Writing.
6. **Full result** — skill profile, confidence + reasons, strengths, gaps, Can-Do interpretation, target-exam
   readiness (with disclaimers), retest advice.

## 5. Components and reporting

| Component | Primary construct | Reported? |
|---|---|---|
| Language Systems | Grammar and vocabulary in context | Diagnostic only (shown beneath skill profile) |
| Reading | Gist, detail, inference, cohesion, stance, discourse, cross-text synthesis | Yes |
| Listening | Detail, main point, inference, attitude, purpose, discourse, pragmatic meaning | Yes |
| Speaking | Intelligibility, fluency, grammar, vocabulary, discourse, pragmatics, interaction | Yes |
| Writing | Task achievement, organisation, grammar, vocabulary, mechanics/register, mediation | Yes |
| Integrated listen-to-write | Orthography / listening-to-writing accuracy | Diagnostic only (never in Writing level) |

CEFR scope note (must appear in the technical manual / about page): v2 samples mediation and digitally-mediated
interaction where practical; it does **not** claim to measure plurilingual/pluricultural competence.

## 6. Typical module length (beta)

| Module | Length | Pause / resume |
|---|---|---|
| Language Systems | 12–18 items, ~8–12 min | Resume at item boundary |
| Reading | 12–18 items, ~10–15 min | Free navigation inside a testlet; lock at testlet submission |
| Listening | 12–18 items, ~10–15 min | Resume between recordings only (mid-recording resume only for technical recovery) |
| Speaking | 8 recorded responses / 7 task types, ~12–18 min | Break before module; one standard re-record where task allows |
| Writing | 3 scored tasks (+1 diagnostic), 20–45 min by route | Autosave; break before module; desktop recommended for upper routes |

## 7. Result rules (summary — exact logic in `05`)

- Primary output = four-skill profile. Grammar/Vocabulary diagnostics beneath.
- Headline only when all four measured skills fall within one band; label "Indicative overall profile"; lower median
  band; never above lowest skill + 1. Otherwise "Uneven profile" with the full range.
- Confidence = **Moderate** or **Low** only.
- Partial completion → partial profile with explicit "not measured" skills.

## 8. Beta claims policy (enforced by lint + e2e)

| Allowed | Not allowed |
|---|---|
| Indicative placement estimate | "Your validated CEFR level is …" |
| Skill profile and observed diagnostic gaps | External exam score predictions |
| Study recommendations | Reliability/accuracy percentages |
| Target-exam readiness advice with disclaimers | Certificate / verified high-stakes claim |
| Low/Moderate confidence based on completeness/flags | High confidence, ±/quarter-level precision |

Permitted wording before calibration: "indicative placement estimate", "diagnostic profile". Never "CEFR-aligned",
"validated", "certified".

## 9. Target-exam readiness layer (interpretation only)

Use `seed/results_and_claims.json → readiness_layer`. Each target has `safe_use` (what to say) and `never_claim`.
Currency notes to keep visible in copy: TOEFL iBT moved to a 1–6 scale with multistage adaptive Reading/Listening in
January 2026; PTE Academic added two task types in August 2025; OET is a four-subtest healthcare-specific test.
GEPA never converts its result into any of these scores.

## 10. Fairness and accessibility policy (build requirements in `09`)

D/deaf or hard of hearing → transcript-access pathway for content access, Listening reported "not measured", partial
profile. Speech difference/stammering → alternative to Oral Reading; disorder-characteristic disfluency not scored;
extra attempts only as approved accommodation. Dyslexia/processing → extended time, spacing, text size, no speed
dependence. Low vision → high contrast, scaling, keyboard, AT use recorded where it changes modality. Motor → full
keyboard, large targets, no drag-only. Low bandwidth → pre-buffer, autosave, audio-quality gate, graceful recovery.
WCAG 2.2 AA. Accommodation status never lowers confidence by itself.

## 11. Out of scope for this build

CAT/IRT engine; certificates; identity verification/proctoring; production-scale item pool (pilot bank only);
numeric confidence intervals; plurilingual competence; any external-score mapping; payments.

## 12. Beta success metrics (instrument these)

Start→receptive-result completion rate; receptive→full completion rate; median time per module; technical failure
rate per module (audio capture, disconnection); % sessions with integrity flags; % sessions with boundary/aberrant
flags; item-level p-values and response times (feeds Phase C); reviewer agreement with AI ratings (feeds Phase D2).

## 13. Validation roadmap the build must support (blueprint §14)

Phase A expert review (2+ reviewers/item, provenance recorded) → A2 cognitive labs → B small pilot (150–250) →
C field calibration (≥250 responses/item) → D standard setting → D2 productive AI validation (double human scoring
subset) → E parallel forms → F production monitoring. Build implication: capture per-item response, time, permutation,
replay count, device class, accommodation status; version AI scoring by model/prompt/rubric/benchmark; preserve
originals on rescore; seed 2–3 unscored pretest items per public session (feature-flagged, off for pilot).
