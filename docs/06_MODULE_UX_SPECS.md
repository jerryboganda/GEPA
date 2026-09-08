# 06 — Module UX Specifications

Blueprint §3, §5–§9, §12. All timers are server-issued deadlines (client displays only). Extended-time accommodation
multiplies every limit by 1.5. Every screen: keyboard operable, visible focus, live-region announcements for timer
warnings (at 60 s, 15 s), no drag-only interactions, min target 44×44 px, text scalable to 200%.

## 1. Start screen
- Fields: target goal (single-select chips incl. "Just general English"; optional), interface language (EN default;
  AR/UR/others for instructions only), consent checkbox (privacy essentials, link to policy), optional "About you"
  (age band, L1, device) — all skippable, collapsible, and deferrable to the end.
- Accessibility panel (also available from every screen via the persistent "Display & access" button): text size,
  spacing, high contrast, extended time (self-declared; sets flag), transcript access for Listening (explains that
  Listening will be reported as not measured), alternative to Oral Reading.
- Primary CTA "Start". Secondary "What is this test?" → claims text ("indicative placement estimate…").

## 2. Worked example (unscored)
One LS-style MCQ with instant feedback and an explanation of controls (select, change answer, submit, timer). Shows
how audio works if the candidate's route will include Listening (play button demo with a 3 s tone).

## 3. Language Systems player
- One item per screen: stem with a clear blank (`___` rendered as an underlined gap), 3–4 options as a radio group
  (arrow keys move, Space selects, Enter submits). "Submit" enabled after selection; "I don't know" is not offered
  (timeouts are recorded as omissions).
- Time limit per item: 60 s (Pre-A1/A1 items 75 s). Warning at 15 s. On timeout: auto-submit as omitted, brief neutral
  notice "Time is up for this question", continue.
- Progress indicator: "Question 7" only — never total count (adaptive), never band names.
- Pause: allowed at item boundary; resume shows the next item with a fresh deadline.

## 4. Reading player
- Testlet layout: stimulus on the left/top (scrollable, resizable text), the stimulus' 2 items on the right/below.
  Candidate can move freely between the items of the testlet and change answers until "Submit testlet".
- Testlet time limits: Pre-A1/A1 2 min; A2 3 min; B1 4 min; B2 5 min; C1/C2 6 min.
- Stimulus types render semantically: signs/notices as `<figure>` with large text; emails with header block; two-text
  stimuli as labelled sections ("Notice A", "Notice B"); `[GAP]` in insertion tasks rendered as a highlighted marker.
- Lock at submission; pause allowed between testlets.

## 5. Listening player
- **Questions visible before the first play and remain visible** (both items of the stimulus).
- Play control: large button "Play (1 of 2)". First play required before answering is enabled. Audio cannot be paused or
  scrubbed. A second play "Play again (2 of 2)" is available after the first ends; it is **logged and never penalised**.
- Timing: preview — the candidate presses Play when ready (auto-start after 45 s idle); after the audio ends, answer time
  = max(45 s, 1.5 × audio duration). Replay pauses the answer timer while playing, then resumes.
- Pre-buffer the next stimulus audio silently once the current testlet is submitted (never before; no item leakage:
  audio URLs are for the current/next stimulus only). Buffer failure → "Checking your connection" state with retry;
  after 2 failures replace the unit (technical, no penalty).
- Volume slider + "test sound" before the module; captions are **not** shown (transcript accommodation routes to the
  transcript pathway instead: text shown, answers still collected for study feedback, module marked `not_measured`).
- Pause/resume: only between recordings. Mid-recording disconnection → resume replays the same audio (counts as first play).

## 6. Receptive result screen ("Foundation and Receptive Profile")
Shows RD and LSN (band or range/not measured), LS as diagnostic beneath, confidence + reasons, then two CTAs:
"Continue to Speaking and Writing (about 35–60 min)" and "Finish here and see my study guidance". Copy must say
this is a partial profile and that Speaking/Writing are not yet measured.

## 7. Speaking player
- Intro: what will happen, 8 recordings, approximate time; then **microphone check** (record 3 s, playback, level meter).
  Failure → troubleshooting steps; option to skip Speaking (module `not_measured`), never a low score.
- Per task screen: instruction text (large), prompt (text or audio per `delivery`), prep countdown, then record with a
  visible level meter and countdown; "Stop" ends early. Playback + one **Re-record** (where `allowsRerecord`).
  Audio prompts (SR sentence, RT message, INT interlocutor line) play once automatically after "Ready"; SR allows no
  replay (it is the construct); RT and INT allow one replay.
- Timing table (seconds; upper routes = B2-C1, C1-C2):

| Task | Prep | Speak (lower routes / upper routes) | Re-record |
|---|---|---|---|
| Oral Reading | 15 | 30 / 40 | yes |
| Sentence Reconstruction | 0 (after audio) | 12 / 15 | yes (replays audio) |
| Functional Situation | 20 | 45 / 60 | yes |
| Retell / Summarise | 10 (after audio) | 45 / 75 | yes |
| Extended Response 1 & 2 | 30 | 60 / 90 | yes |
| Interaction turn 1 & 2 | 10 (after line) | 40 / 60 | turn 1 only |

- Client quality pre-check after each recording: duration ≥ 3 s, mean RMS above noise floor, clipping ratio < 1%,
  speech ratio > 25%. Failing → "We could not capture that clearly" with re-record (technical re-record does not consume
  the standard re-record). Never display a score or a judgement about pronunciation.
- Upload immediately in the background; if upload fails, retry with backoff; keep the blob in memory until confirmed.
- Speech-difference accommodation: Oral Reading replaced by an extra Functional Situation (weights re-normalised, see D-007).

## 8. Writing editor
- One task per screen: prompt (and, for mediation tasks, the source message(s) shown in a panel), word guidance
  (e.g. "Aim for 90–120 words") with a live word count that turns amber outside the range but never blocks submission,
  time limit per task with warning; autosave indicator ("Saved 5 s ago").
- Editor: plain textarea (no rich text), spellcheck **off** by default (configurable per form, logged), browser
  autocomplete off, paste allowed but logged (length, timestamp), focus-loss events logged. Mobile: sticky submit bar.
- Time limits by route (minutes): PreA1-A1 T1 5 / T2 5 / T3 8; A1-A2 6/6/10; A2-B1 7/8/12; B1-B2 8/10/14; B2-C1 9/11/17;
  C1-C2 10/12/20. Diagnostic (listen-to-write): audio plays once, one replay, 2 min.
- Upper routes show a non-blocking note recommending a keyboard/desktop.
- Submit freezes the text; the next task starts with a fresh deadline. Pause allowed between tasks.

## 9. Full result screen
Order and prominence per `05 §3.1`. Each skill card: band chip (or "Range B1–B2 — needs confirmation", "Insufficient
evidence", "Not measured"), 1–2 Can-Do lines, 1–2 gap lines. Confidence badge (Low/Moderate) with reasons in plain
words. Diagnostics accordion (Grammar/Vocabulary constructs, pronunciation/fluency notes, listen-to-write accuracy).
Headline block small and below the skills, only when the rule allows. Readiness block for the chosen target with the
fixed disclaimer. Retest advice. Download/share as PDF is **not** offered in beta (no certificate).

## 10. Global behaviours
- Reconnect banner with automatic retry; state restored from server on reload (`GET /api/sessions/:id/state`).
- Idle timeout 30 min → session paused (not abandoned); resume from last boundary within 7 days.
- Error copy never mentions items, keys, models or "AI"; use "assessment service".
- Language of instructions (not items) follows `uiLanguage`; item content is always English.
- Telemetry events (client → `events[]` in submits): focus_lost, paste, replay, rerecord, buffer_retry, timer_warning_seen.
