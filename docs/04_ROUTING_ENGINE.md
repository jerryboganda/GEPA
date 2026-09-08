# 04 — Routing Engine (v2 beta, pre-calibration, rule-based)

Implements blueprint §4 and Keys §5. This is deliberately **not** CAT/IRT. Every rule below is a beta operating
hypothesis; parameters live in `seed/routing_ruleset_v2beta.json` and must be re-estimated after calibration.
The engine is pure TypeScript in `shared/engine/routing.ts` and `shared/engine/productiveRoute.ts`.

## 0. Vocabulary
- **Band index** `bi(band)`: Pre-A1=0 … C2=6. Used only to move up/down and compare order. **Never averaged.**
- **Delivery unit**: LS → one item; RD/LSN → one stimulus with its two items (always delivered together).
- **Locator pair**: 2 items at one band (LS: the two `locator_candidate` items if unused, else any two unused items of
  different constructs; RD/LSN: the first unused stimulus of the band).
- **Tie item**: one same-band item (LS: any unused item, different construct; RD/LSN: next unused stimulus — only its
  first item routes, the second item's response is stored as confirmation evidence with `purpose:"confirmation"`).
- **Bracket** `[L, U]`: two adjacent bands, L below U.

## 1. Locator phase (per objective module, independent of other modules)

```
state ← { band: B1, direction: none, used: 0, pairResults: {}, trace: [] }
loop:
  unit ← nextLocatorPair(state.band)                      // 2 items
  c ← correctCount(unit); state.used += 2; state.pairResults[band] = c; push trace
  if c == 2:
      if band ∈ {C1, C2}: return bracket [C1, C2], flags += c2_provisional            // "strong at C1 opens C1/C2"
      if pairResults[band+1] == 0: return bracket [band, band+1]                      // reversal (came down, now 2/2)
      band += 1; direction = up
  else if c == 0:
      if band == A1: return bracket [Pre-A1, A1], flags += preA1_confirmation        // never force A1
      if pairResults[band-1] == 2: return bracket [band-1, band]                      // reversal (came up, now 0/2)
      band -= 1; direction = down
  else (c == 1):
      tie ← nextTieItem(band); state.used += 1; push trace(kind:"tie")
      if tie correct: return bracket [band, band+1]     (band == C1 → [C1,C2] + c2_provisional)
      else:           return bracket [band-1, band]     (band == A1 → [Pre-A1,A1] + preA1_confirmation)
  if state.used >= ruleset.locator_hard_cap (8):
      // two most plausible adjacent bands from the last move
      return direction == up ? [band-1, band] : [band, band+1], flags += boundary_confirmation
```
Locator bands are therefore always A1…C1 (Pre-A1 and C2 are only reached as bracket ends). Max locator length with
this rule is 6 items; the cap is a safety net.

## 2. Confirmation phase

```
[L, U] ← bracket
lowerBlock ← buildBlock(L, target=5)      // see 2.1
upperBlock ← buildBlock(U, target=5)
deliver lowerBlock then upperBlock (RD/LSN: stimulus by stimulus)
cL, nL ← correct/size of lower evidence;  cU, nU ← same for upper
pL ← cL/nL; pU ← cU/nU
classify(p): p >= strong_min(0.8) → strong; p <= weak_max(0.4) → weak; else borderline

1. if cU - cL >= aberrant_gap(3):                                   // non-monotonic
       recheck ← buildBlock(L, target=4, kind:"aberrant_recheck")  (if < 2 items available → skip, go to 1b)
       if recheck.correct/recheck.size >= 0.75: outcome band=L, notes+="upper possible; non-monotonic pattern", flags+=aberrant_pattern, confidenceCap=Low
       else 1b: outcome band=null, range=[L,U], flags+=aberrant_pattern, confidenceCap=Low
       STOP
2. if strong(pL) and weak(pU):          outcome band=L, notes+="upper not supported"
3. if strong(pL) and borderline(pU):    boundary ← buildBlock(U, target=4, kind:"boundary")
       if boundary.size < 2:            outcome band=L, notes+="upper possible", flags+=boundary_unresolved, confidenceCap=Low
       else if boundary.correct >= ceil(0.75*boundary.size): outcome band=U
       else:                            outcome band=L, notes+="upper possible"
4. if strong(pL) and strong(pU):        outcome band=U
       if U == C2: notes+="provisional C2-level evidence"
       else: ext ← buildBlock(U+1, target=4, kind:"extension"); if ext.size >= 2 and ext.correct >= ceil(0.75*ext.size):
             notes+="upper end of band; next band possible", flags+=extension_strong   // band stays U in beta
5. if not strong(pL):                                                 // lower 0–3/5
       if L == Pre-A1:                  outcome band=Pre-A1, flags+=floor_unresolved, confidenceCap=Low
       else if pairResults[L-1] == 2:   outcome band=L-1  (notes+="lower possible" if pL >= 0.6)      // floor already established
       else descent ← buildBlock(L-1, target=5, kind:"descent")
            if descent.size < 2:        outcome band=L-1, flags+=floor_unresolved, confidenceCap=Low
            else if strong(descent):    outcome band=L-1 (notes+="lower possible" if pL >= 0.6)
            else:                       outcome band=L-1, flags+=floor_unresolved, confidenceCap=Low
       if strong(pU): flags+=inconsistent_pattern, confidenceCap=Low
```
Outcome object: `{ band | null, range | null, notes[], flags[], evidenceShortfall, itemsDelivered }`.
**No ± positions, no numeric level, no cross-module capping.** Grammar/Vocabulary (LS) outcome is stored the same way
but reported as a diagnostic, with construct-level strengths/weaknesses derived from item constructs.

### 2.1 `buildBlock(band, target, kind)` — pilot-bank-aware
- Candidates = unused items at `band` (RD/LSN: whole unused stimuli; block sizes are then even).
- Prefer constructs / stimuli not used by the locator pair; respect `≤1 inversion item per LS block`.
- If unused items < target: also count already-answered locator/tie responses at that band toward the block
  (`evidenceShortfall=true` when total evidence < 3). Never re-deliver an item within a session.
- Returns `{ itemIds(new deliveries), correct(total incl. counted prior responses), size(total) }`.
- Pilot depth reality: LS 8 items/band (Pre-A1: 4); RD/LSN 6 items/band. Boundary/extension/descent blocks will often
  be unavailable in the pilot → the `size < 2` branches above are expected, not exceptional. Log `block_unavailable`.

### 2.2 Timeouts, omissions, recovery (blueprint §5)
- Timeout → response `omitted:true`, scored incorrect for routing, separate omission flag; ≥3 omissions in a module →
  flag `effort_omissions`, confidenceCap=Low.
- Disconnection → restore the current unit once with a fresh deadline (Listening: allow replay); a second failure on the
  same unit → replace the unit (mark `technical_replaced`), no penalty.
- Abandonment → module `abandoned`; profile partial; other modules unaffected.

## 3. Productive route (`shared/engine/productiveRoute.ts`)

```
routeIndexOf(moduleOutcome) = min(bi(outcome.band ?? outcome.range[0]), 5)      // route = [band, band+1]; C2 → C1-C2
valid = objective modules with status complete and an outcome
if valid is empty: route = B1-B2, flags += productive_route_default
r = median(routeIndexOf(m) for m in valid)        // 3 → middle; 2 → the lower; 1 → itself
if LS valid and RD valid and LSN valid and idx(RD) >= idx(LS)+1 and idx(LSN) >= idx(LS)+1: r = max(r, idx(LS)+1), lifted=true
if max(idx) - min(idx) >= 2: wideWindow=true, confidenceCap=Low
return { route: ROUTES[r], lifted, wideWindow, basis }
```
Upper extension (blueprint §4.3 last bullet): when Speaking upper evidence is met with margin (mean-at-upper ≥ 3.5 on
≥3 responses), offer one Extended Response from the next route as an **optional extra** on the results screen
(`FEATURE_PRODUCTIVE_EXTENSION`, default off for the pilot; see DECISIONS D-006).

## 4. Required unit tests (names are binding; add more)

Locator:
| Test | Trace | Expected |
|---|---|---|
| `locator.up.up.up` | B1 2/2 → B2 2/2 → C1 2/2 | [C1,C2], used 6, flag c2_provisional |
| `locator.down.down.down` | B1 0/2 → A2 0/2 → A1 0/2 | [Pre-A1,A1], used 6, flag preA1_confirmation |
| `locator.tie.correct` | B1 1/2 → tie ✓ | [B1,B2], used 3 |
| `locator.tie.incorrect` | B1 1/2 → tie ✗ | [A2,B1], used 3 |
| `locator.reversal.upThenDown` | B1 2/2 → B2 0/2 | [B1,B2], used 4 |
| `locator.reversal.downThenUp` | B1 0/2 → A2 2/2 | [A2,B1], used 4 |
| `locator.up.tie.correct` | B1 2/2 → B2 1/2 → tie ✓ | [B2,C1], used 5 |
| `locator.down.tie.incorrect` | B1 0/2 → A2 1/2 → tie ✗ | [A1,A2], used 5 |
| `locator.a1.tie.incorrect` | B1 0/2 → A2 0/2 → A1 1/2 → tie ✗ | [Pre-A1,A1], flag preA1_confirmation |
| `locator.c1.tie.correct` | B1 2/2 → B2 2/2 → C1 1/2 → tie ✓ | [C1,C2], flag c2_provisional |
| `locator.noItemReuse` | any | every delivered item_id unique within module |
| `locator.rdlsn.pairIsWholeStimulus` | RD | first unit = 2 items sharing stimulus_id |

Confirmation:
| Test | Inputs | Expected |
|---|---|---|
| `confirm.lowerStrong.upperWeak` | [B1,B2] 5/5, 1/5 | B1 |
| `confirm.borderline.boundaryPass` | [B1,B2] 4/5, 3/5, boundary 3/4 | B2 |
| `confirm.borderline.boundaryFail` | [B1,B2] 4/5, 3/5, boundary 2/4 | B1 + "upper possible" |
| `confirm.borderline.noBoundaryItems` | [B1,B2] 4/5, 3/5, 0 items available | B1, flag boundary_unresolved, cap Low |
| `confirm.bothStrong.extensionStrong` | [B1,B2] 5/5, 4/5, ext 3/4 | B2, flag extension_strong, note |
| `confirm.bothStrong.ceiling` | [C1,C2] 5/5, 5/5 | C2, note "provisional C2-level evidence", no extension |
| `confirm.aberrant.recheckHigh` | [B1,B2] 2/5, 5/5, recheck 3/4 | B1, flag aberrant_pattern, cap Low |
| `confirm.aberrant.recheckLow` | [B1,B2] 1/5, 4/5, recheck 1/4 | band null, range [B1,B2], flag aberrant_pattern |
| `confirm.lowerWeak.descentStrong` | [A2,B1] 2/5, 0/5, descent 5/5 | A1 |
| `confirm.lowerWeak.descentWeak` | [A2,B1] 2/5, 0/5, descent 3/5 | A1, flag floor_unresolved, cap Low |
| `confirm.lowerWeak.floorFromLocator` | [A2,B1] 3/5, 0/5, pairResults[A1]=2 | A1 + "lower possible" |
| `confirm.lowerWeak.preA1Floor` | [Pre-A1,A1] 2/4, 1/5 | Pre-A1, flag floor_unresolved |
| `confirm.fractionThresholds.size4` | [B1,B2] 3/4, 2/4 | lower 0.75 = not strong → descent path |
| `confirm.evidenceShortfall` | band with 3 items total | evidenceShortfall=true |

Productive route:
| Test | Inputs (LS, RD, LSN) | Expected |
|---|---|---|
| `route.median` | B2, B2, C1 | B2-C1 |
| `route.lift` | A2, B1, B1 | B1-B2, lifted |
| `route.wideWindow` | B1, B2, A2 | B1-B2, wideWindow, cap Low |
| `route.twoValid.lower` | —, B1, B2 | B1-B2 |
| `route.ceiling` | C2, C2, C2 | C1-C2 |
| `route.noneValid` | —, —, — | B1-B2, flag productive_route_default |
| `route.rangeUsesLower` | LS range [B1,B2], RD B2, LSN B2 | B2-C1 (median of 3,4,4 = 4) |

Property tests (fast-check or hand-rolled): routing never delivers > 8 locator items; never reuses an item; outcome band
∈ bracket ∪ {L-1}; no code path produces a numeric level or a ± suffix.
