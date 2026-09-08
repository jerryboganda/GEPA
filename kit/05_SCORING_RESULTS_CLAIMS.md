# 05 — Scoring, Results and Claims

Implements blueprint §8.1, §9.1, §10, §11, §16 and Keys §1, §6–§10. Pure functions live in `shared/engine/`:
`objectiveScoring.ts`, `evidenceRules.ts`, `resultAssembly.ts`, `wordingPolicy.ts`. Config: `seed/rubrics.json`,
`seed/results_and_claims.json`.

## 1. Objective scoring (LS, RD, LSN)

- **Key identity:** `restricted_keys/{item_id}.key_option_id`. Correct ⇔ `selected_option_id === key_option_id`.
- **Shuffling:** on delivery, `permutation = shuffle(item.options.map(o => o.option_id), seededRng(session_id + item_id))`.
  Store `permutation` on the response record. The client receives options in that order and returns an `option_id`.
- **Server-side only:** the `correct` field exists only on `ObjectiveResponse` (client-inaccessible collection). The API
  response to a submit is `{ accepted:true, next:<unit|moduleComplete> }` — no correctness feedback during the test
  (worked example is the only place with feedback).
- **Omissions:** `selected_option_id:null, omitted:true, correct:false` for routing; omission flag kept separately.
- **Key balance (form check):** after assembling a beta form, compute delivered-position distribution per module; the
  runtime shuffle makes it uniform in expectation; the `forms:check` script asserts no authoring letter > 40% in any
  module block and reports `seed/manifest.json` distributions (LS A15/B16/C13/D8; RD & LSN A10/B14/C14/D4).

## 2. Productive rating inputs (from `07_AI_SCORING_SPEC.md`)
Each Speaking response / Writing task yields a `ProductiveRating` with `usable`, `atLower` and `atUpper` trait maps
(0–5 integers), `flags`. **Dual-reference rating** (D-003): a trait score of 3 = "meets what a typical candidate at that
band produces on this task"; `atLower` is judged against the route's lower band, `atUpper` against its upper band.

### 2.1 Per-response weighted trait mean
```
traits = rubrics.<module>.per_task_trait_override[task_type] ?? rubrics.<module>.traits
w = rubrics.<module>.trait_weights_default restricted to `traits`, renormalised to sum 1
mean_at(ref) = Σ w[t] * scores[ref][t]
```
### 2.2 Speaking decision (`evidenceRules.speaking(ratings, route)`)
```
spont = ratings where task.spontaneous_or_interactive && usable      // FS, RT, ER1, ER2, INT1, INT2 (max 6)
if |spont| < 3 → status insufficient_evidence, reason "fewer than 3 usable spontaneous responses"
lowerMeets = mean over spont of mean_at(lower) >= 2.75  AND  count(spont where atLower.communication >= 3) >= 2
upperMeets = mean over spont of mean_at(upper) >= 3.0
             AND for every spont r: min(atUpper.grammar, atUpper.vocabulary, atUpper.communication) >= 2
             AND count(spont where mean_at(upper) >= 3.0) >= 2
band = upperMeets && lowerMeets ? route.upper : lowerMeets ? route.lower : "below_route"
```
"below_route" → status `measured`, band = lower band − 1 **only if** ≥3 spontaneous responses are usable and
mean_at(lower) ≥ 1.75; otherwise status `insufficient_evidence` (never force a low level from thin evidence). Note
`below_route` in `notes` and set confidenceCap Low. Diagnostic tasks (OR, SR) never change the band; their
intelligibility/fluency scores feed the diagnostics panel. The INT two-turn task is one task: its two turns are two
responses in `spont` but count once toward "independent responses" (D-004).

### 2.3 Writing decision (`evidenceRules.writing(ratings, route)`)
```
scored = ratings for tasks with scored_in_writing_level && usable        // max 3
if |scored| < 2 → insufficient_evidence ("Writing: not measured")
lowerMeets = count(scored where mean_at(lower) >= 2.75 && atLower.task_fulfilment >= 3) >= 2
upperMeets = count(scored where mean_at(upper) >= 3.0) >= 2
             AND every scored r: min(atUpper.task_fulfilment, atUpper.organisation) >= 2
band as for Speaking; below_route rule identical (threshold mean_at(lower) >= 1.75 on ≥2 tasks)
```
Task weights (30/30/40) are used for the **diagnostics summary** (weighted profile of strengths) and for tie-breaking
notes, not to override the count-based rules above. The listen-to-write diagnostic reports an accuracy label
(`exact | minor_errors | major_errors | unusable`) and never touches the band.

### 2.4 Unusable / integrity handling (Keys §8)
| Condition | Effect |
|---|---|
| empty / silence / < 3 s speech or < 5 words writing | `usable:false`, reason; never a low score |
| audio clipping / severe noise | `usable:false`, reason `technical`; re-record offered; review queue |
| off-topic | `usable:true`, communication/task_fulfilment ≤ 2, flag `off_topic` |
| suspected AI-generated writing (behavioural: paste of >60% of text, typing burst > 15 chars/s sustained, profile mismatch) | flag `ai_suspect_review` only; rating unchanged |
| productive vs receptive mismatch ≥ 2 bands | flag `profile_inconsistency`; queue for human review; confidence Low |
| rapid clicking (≥5 consecutive objective responses < 1.5 s) | flag `effort_rapid`; confidence Low |

## 3. Result assembly (`resultAssembly.build(session, outcomes, ratings)`)

1. **Skill statuses:** RD, LSN from module outcomes (`measured` with band; `measured` with range when band null;
   `not_measured` if abandoned/transcript accommodation); SPK, WRT from evidence rules.
2. **Diagnostics:** LS band/range + constructs (correct vs incorrect grouped by `construct`); OR/SR intelligibility
   summary; listen-to-write accuracy.
3. **Headline** (`headlineRule`): only when `profileType === "full"` and all four skills are `measured` with a band:
   ```
   bands sorted ascending → b1 ≤ b2 ≤ b3 ≤ b4
   if bi(b4) - bi(b1) <= 1: headline = { kind:"indicative_overall", band: b2 }       // lower median
   else: headline = { kind:"uneven", range:[b1,b4] }
   assert bi(headline.band) <= bi(b1) + 1                                              // always true for lower median; test it
   ```
   Any skill with a range (no band) or insufficient/not measured → `kind:"none"` and the UI shows skills only.
   Foundation-only profile → `kind:"none"`, `profileType:"foundation_receptive"`, label "Foundation and Receptive Profile".
4. **Confidence:** start `Moderate`; drop to `Low` if any: partial profile; any module flag in
   {boundary_unresolved, floor_unresolved, aberrant_pattern, inconsistent_pattern, evidenceShortfall, wideWindow,
   productive_route_default, effort_omissions, effort_rapid, profile_inconsistency, technical_replaced ≥ 2};
   any skill `insufficient_evidence`; ≥2 integrity flags. Never `High`. `confidenceReasons` lists the triggers in
   candidate-safe wording (e.g. "Some modules were not completed").
5. **Can-Do interpretation:** one or two short Can-Do statements per skill per band from `client/src/copy/canDo.ts`
   (write them in plain, non-technical English; CEFR-inspired, no "CEFR-aligned" claim).
6. **Readiness layer:** `readiness_layer.targets[targetGoal]` → `safe_use` text + fixed disclaimer
   "GEPA does not predict official exam scores." Never mention bands/grades of the external exam.
7. **Retest advice:** Low confidence → "retest after 2–4 weeks of study or when technical issues are resolved";
   Moderate → "retest after ~8–12 weeks of study".
8. **Wording version** stamped on every report; regenerating a report never deletes the previous one.

### 3.1 Screen order (binding for UI)
Skills (large) → confidence badge + reasons → diagnostics (smaller) → headline (small, only if kind ≠ none) →
readiness → next steps/retest → "What this result is and is not" (claims text). Individual skills must be
visually more prominent than the headline (font size, position) — e2e asserts the DOM order and relative sizes.

## 4. Claims policy lint (`shared/copy-policy` + `copy-lint`)
Forbidden (case-insensitive, whole phrase or token as noted) in any candidate-facing string or results template:
`CEFR-aligned`, `validated`, `certified`, `certificate`, `your level is`, `your CEFR level`, `official`, `guarantee`,
`IELTS band`, `OET grade`, `TOEFL score`, `PTE score`, `predict`, `High confidence`, `%` inside confidence strings,
regex `\b(A1|A2|B1|B2|C1|C2)[+\-]` (± levels), `reliability`, `accuracy of`. Allowed exceptions require an
explicit `// copy-lint-allow: <reason>` on the line (reviewed in DECISIONS). Runtime guard: `resultAssembly` runs
the same check on the generated report and throws if violated (fail closed, log, show a neutral fallback).

## 5. Rescoring
A rescore (new rubric/prompt/model version or human override) creates a **new** `ProductiveRating` with
`supersededBy` set on the old one and `regenerationReason`. Result reports are rebuilt from the latest non-superseded
ratings; previous reports remain queryable by `wordingVersion` + `generatedAt`.
