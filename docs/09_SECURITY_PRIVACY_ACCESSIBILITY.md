# 09 — Security, Privacy and Accessibility

## 1. Threat model (free beta)
| Threat | Control |
|---|---|
| Key exposure via bundle / API / logs | Keys in `restricted_keys` (deny rules); scoring only in `server/repos` + `server/services`; payload schema tests; log redaction; no `correct` on any client type |
| Item harvesting (scraping items, listening scripts) | API delivers one unit at a time; listening served as short-lived signed audio URLs, never scripts; exposure counters; rate limiting; anomaly flags (many sessions per uid/IP) |
| Answer-position guessing | Per-session shuffle, permutation stored; key balance checks |
| Replay/scrubbing to defeat listening construct | Player has no scrub/pause; second play counted; server rejects a third play URL request |
| Session tampering (posting responses for items not current) | Server validates `item_id ∈ currentUnit`, `stateVersion`, idempotency key; Firestore transactions |
| Impersonation of reviewer/admin | Google sign-in + custom claims; server verifies ID token on every admin/review route; rules mirror |
| Prompt injection in candidate text/audio | Content wrapped as data; structured output; ratings never contain instructions; reviewer UI escapes text |
| Abuse of Gemini quota | Per-session call budget; job queue with backoff; 429 handling; admin alert counter |

Not in scope (Verified Mode later): identity verification, proctoring, lockdown browser.

## 2. Firestore & Storage rules (deploy with the app; unit-test with the emulator)
```
match /sessions/{sid}            { allow read: if request.auth != null && resource.data.candidate_uid == request.auth.uid; allow write: if false; }
match /sessions/{sid}/responses/{r} { allow read, write: if false; }
match /sessions/{sid}/productive/{t} { allow read, write: if false; }   // drafts flow through the API
match /results/{sid}             { allow read: if request.auth != null && resource.data.candidate_uid == request.auth.uid; allow write: if false; }
match /users/{uid}               { allow read: if request.auth != null && request.auth.uid == uid; allow write: if false; }
match /{document=**}             { allow read, write: if false; }     // items, keys, scripts, ratings, jobs, config
```
Storage: all paths private; client uploads only through signed PUT URLs scoped to `audio/responses/{sid}/{taskId}/`;
client downloads only through signed GET URLs for the current/next stimulus or prompt.

## 3. Privacy (UK GDPR / data minimisation)
- Data collected: session state, responses, timing, device class, optional background answers, audio recordings,
  writing text, accommodation flags (used only to interpret evidence). No name/email required in free beta.
- Consent screen states purposes (placement + anonymised research to improve the test) with a separate optional
  research consent; copy links to the privacy policy.
- Retention: audio recordings 90 days (then delete unless research consent → 24 months anonymised); text 24 months;
  telemetry aggregated after 12 months. `scripts/retention.ts` scheduled job.
- Candidate rights: "Delete my data" action on the results screen (session-scoped) → hard delete of audio, text, drafts;
  keep aggregated item statistics only.
- AI processing disclosure: results screen "How this was scored" explains automated rating with human review, and that
  it is indicative. No AI-detection accusations are ever shown.
- Accommodation status is never shown as a negative signal and never lowers confidence by itself.

## 4. Accessibility — WCAG 2.2 AA (tested with axe-core on every screen + manual keyboard pass)
- Keyboard: complete operation without a mouse; logical focus order; focus visible (≥3:1 contrast, 2 px); skip links;
  no keyboard traps in audio/recording components; escape closes dialogs.
- Timing (2.2.1): every timed screen exposes "Extended time" (×1.5) via accommodation and warns before expiry with an
  accessible live region; timer text is not the only indicator (progress ring + text).
- Text: base 16 px, scalable to 200% without loss; spacing control (line-height 1.5→1.8, letter-spacing +0.05em);
  dyslexia-friendly option (increased spacing, no justified text). Contrast ≥4.5:1 (7:1 in high-contrast mode).
- Targets ≥44×44 px (2.5.8); no drag-only interaction (2.5.7) — insertion/ordering tasks use select/move buttons;
  radio groups for MCQ; consistent help location (3.2.6): the "Display & access" button and help link stay in the same place.
- Audio: visible play state, level meters with text equivalents ("Recording… good level"), captions not used for test
  audio (construct) but all instructional audio has text; transcript pathway for D/deaf candidates.
- Language: `lang` attributes for localised instruction text vs English items; icons for Pre-A1/A1 controls with labels.
- Motion: no auto-animations beyond the timer; respect `prefers-reduced-motion`.
- Screen-reader announcements: question count, timer warnings, recording start/stop, autosave, errors.
- Forms: errors identified in text, suggestions provided, redundant entry avoided (3.3.7).

## 5. Accommodation matrix → implementation
| Need | Behaviour |
|---|---|
| Transcript access (D/deaf) | Listening shows script text as a readable pathway; answers collected; module `not_measured`; profile partial; copy explains why |
| Speech difference | Oral Reading replaced by an extra Functional Situation; disfluency flag `possible_speech_difference` passed to rater as context; extra attempts only if enabled by admin |
| Dyslexia / processing | Extended time; spacing/text scale; no speed-based inference (response time never affects score) |
| Low vision | High contrast, 200% scale, keyboard; AT-use event logged when it changes modality |
| Motor | Full keyboard, large targets, no drag |
| Low bandwidth | Pre-buffer next audio, autosave, upload retries, graceful replacement of failed units |

## 6. DIF readiness
Store L1, region, age band, gender (optional), device class, accommodation status **separately** from responses with
a session key so DIF analysis can run once sample sizes permit. Never display these to reviewers alongside ratings.
