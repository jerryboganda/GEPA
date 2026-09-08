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

---
<!-- Agent appends from here. Next id: D-012 -->

