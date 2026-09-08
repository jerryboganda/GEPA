# 03 — Data Model (Rust Serde Models on Backend & zod Schemas on Astro Frontend)

Implements blueprint §15. Canonical domain models are defined in Rust (`shared/engine` / `server/schemas`) with `serde::{Serialize, Deserialize}`, and mirrored as zod schemas in the Astro client (`client/src/schemas/api.ts`). Field names below are binding; add fields only via
`DECISIONS.md`. Seed files in `/seed` already conform to the *seed* shapes; the loader maps them into these schemas.

## 1. Enums

```ts
export const Band = z.enum(["Pre-A1","A1","A2","B1","B2","C1","C2"]);           // ordinal index = position
export const Route = z.enum(["PreA1-A1","A1-A2","A2-B1","B1-B2","B2-C1","C1-C2"]);
export const Module = z.enum(["LS","RD","LSN","SPK","WRT"]);
export const Domain = z.enum(["personal","public","educational","occupational"]);
export const Confidence = z.enum(["Low","Moderate"]);                            // "High" intentionally absent
export const ModuleStatus = z.enum(["pending","in_progress","paused","complete","abandoned","not_measured"]);
export const Mode = z.enum(["free_beta","verified"]);                             // verified = future
export const Role = z.enum(["candidate","reviewer","admin"]);
```

## 2. Bank objects (candidate-safe unless marked RESTRICTED)

```ts
export const Option = z.object({ option_id: z.string(), text: z.string() });        // no letter, no correctness

export const ObjectiveItem = z.object({
  item_id: z.string(), module: z.enum(["LS","RD","LSN"]), band: Band, version: z.number().int().default(1),
  status: z.enum(["draft","trial","active","retired"]).default("trial"),
  stem: z.string(), options: z.array(Option).min(3).max(4),
  construct: z.string().optional(),            // LS
  evidence_focus: z.string().optional(),       // RD/LSN
  domain: Domain.optional(), topic_family: z.string().optional(),
  locator_candidate: z.boolean().default(false),
  stimulus_id: z.string().optional(),          // RD/LSN
  is_anchor: z.boolean().default(false), is_pretest: z.boolean().default(false),
  accessibility_alternative: z.string().optional(),
  reviewer_provenance: z.array(z.object({ reviewer: z.string(), date: z.string(), note: z.string() })).default([]),
  exposure: z.object({ delivered: z.number().int(), correct: z.number().int(), medianMs: z.number().nullable() }).optional(),
});

export const ReadingStimulus = z.object({ stimulus_id: z.string(), band: Band, stimulus_type: z.string(),
  domain: Domain, topic_family: z.string(), text: z.string(), word_count: z.number().int(), item_ids: z.array(z.string()) });

// Candidate-safe listening stimulus: NO script text.
export const ListeningStimulusPublic = z.object({ stimulus_id: z.string(), band: Band, speaker_count: z.number().int(),
  domain: Domain, topic_family: z.string(), media: z.object({ storagePath: z.string(), durationSec: z.number(),
  loudnessLufs: z.number(), checksum: z.string(), codec: z.string() }).nullable(), item_ids: z.array(z.string()) });

// RESTRICTED (collection stimulus_admin): script + production metadata.
export const ListeningStimulusAdmin = ListeningStimulusPublic.extend({ script: z.string(), speakers: z.string(),
  target_wpm: z.number().int(), word_count: z.number().int(), est_duration_sec: z.number(),
  accents: z.array(z.string()).default([]), voice_cast: z.record(z.string()).default({}) });

export const SpeakingTask = z.object({ task_id: z.string(), route: Route, route_bands: z.tuple([Band, Band]),
  task_type: z.enum(["oral_reading","sentence_reconstruction","functional_situation","retell_summarise",
    "extended_response","simulated_interaction"]), task_group: z.string(), turn: z.number().int().nullable(),
  label: z.string(), prompt: z.string(), candidate_sees_text: z.boolean(),
  delivery: z.enum(["text_read_aloud","audio_only_repeat","text_prompt","audio_then_speak","interlocutor_audio_then_speak"]),
  audio_script: z.string().nullable().optional(), interlocutor_line: z.string().nullable().optional(),
  audio: z.object({ storagePath: z.string(), durationSec: z.number() }).nullable().default(null),
  weight: z.number(), spontaneous_or_interactive: z.boolean(), traits_scored: z.array(z.string()), topic_family: z.string(),
  prepSeconds: z.number().int(), maxSpeakSeconds: z.number().int(), allowsRerecord: z.boolean() });

export const WritingTask = z.object({ task_id: z.string(), route: Route, route_bands: z.tuple([Band, Band]),
  task_type: z.enum(["functional_communication","mediation_synthesis","extended_response","integrated_accuracy_diagnostic"]),
  label: z.string(), prompt: z.string(), focus: z.string(), word_guidance: z.object({ min: z.number(), max: z.number() }).nullable(),
  weight: z.number(), scored_in_writing_level: z.boolean(), topic_family: z.string(), timeLimitSeconds: z.number().int(),
  audio_script: z.string().nullable().optional(), audio: z.object({ storagePath: z.string(), durationSec: z.number() }).nullable().default(null) });

// RESTRICTED (collection restricted_keys). Never leaves server code.
export const RestrictedKey = z.object({ item_id: z.string(), module: z.enum(["LS","RD","LSN"]), band: Band,
  key_option_id: z.string(), authoring_letter: z.enum(["A","B","C","D"]), answer_text: z.string(),
  option_count: z.number().int(), evidence_focus: z.string().nullable(), rationale: z.string().nullable() });
```

## 3. Ruleset, form, session

```ts
export const RoutingRuleset = z.object({ ruleset_id: z.string(), version: z.string(), start_band: Band.default("B1"),
  locator_pair_size: z.literal(2), locator_hard_cap: z.number().int().default(8),
  confirmation: z.object({ lower_block: z.number().int().default(5), upper_block: z.number().int().default(5),
    strong_min: z.number().default(0.8), borderline: z.number().default(0.6), weak_max: z.number().default(0.4),
    boundary_items: z.number().int().default(4), extension_items: z.number().int().default(4), descent_items: z.number().int().default(5),
    aberrant_gap: z.number().int().default(3) }),
  productive: z.object({ lift_when_both_receptive_above_ls: z.boolean().default(true), wide_window_span: z.number().int().default(2) }),
  effective_from: z.string() });

export const FormSpec = z.object({ form_id: z.string(), version: z.string(), blueprint_version: z.string(),
  ruleset_id: z.string(), item_ids_by_module: z.record(z.array(z.string())), anchors: z.array(z.string()),
  key_balance_report: z.record(z.record(z.number())), domain_coverage_report: z.record(z.any()),
  enemy_group_validation: z.object({ conflicts: z.array(z.object({ family: z.string(), members: z.array(z.string()), severity: z.string() })) }),
  status: z.enum(["draft","beta","retired"]) });

export const LocatorEvent = z.object({ band: Band, itemIds: z.array(z.string()), correct: z.number().int(), kind: z.enum(["pair","tie"]) });
export const ConfirmationBlock = z.object({ band: Band, kind: z.enum(["lower","upper","boundary","extension","descent","aberrant_recheck"]),
  itemIds: z.array(z.string()), correct: z.number().int(), size: z.number().int() });

export const ObjectiveModuleState = z.object({ module: z.enum(["LS","RD","LSN"]), status: ModuleStatus,
  phase: z.enum(["locator","confirmation","done"]), currentBand: Band, direction: z.enum(["none","up","down"]),
  locatorItemsUsed: z.number().int(), locatorTrace: z.array(LocatorEvent),
  bracket: z.tuple([Band, Band]).nullable(), confirmationTrace: z.array(ConfirmationBlock),
  usedItemIds: z.array(z.string()), currentUnit: z.object({ stimulusId: z.string().nullable(), itemIds: z.array(z.string()),
    deadlineAt: z.string() }).nullable(),
  outcome: z.object({ band: Band.nullable(), range: z.tuple([Band, Band]).nullable(), notes: z.array(z.string()),
    flags: z.array(z.string()), evidenceShortfall: z.boolean() }).nullable(),
  stateVersion: z.number().int() });

export const ProductiveModuleState = z.object({ module: z.enum(["SPK","WRT"]), status: ModuleStatus, route: Route.nullable(),
  taskIds: z.array(z.string()), currentTaskId: z.string().nullable(), deadlineAt: z.string().nullable(),
  attempts: z.record(z.number().int()),               // taskId -> recordings/submissions used (re-record rule)
  micCheck: z.object({ passed: z.boolean(), at: z.string() }).nullable(),      // SPK only
  ratingStatus: z.enum(["not_started","pending","complete","failed"]), stateVersion: z.number().int() });

export const Session = z.object({ session_id: z.string(), candidate_uid: z.string(), mode: Mode, createdAt: z.string(),
  targetGoal: z.enum(["OET","IELTS","TOEFL iBT","PTE Academic","Cambridge/DET/other","University","Work","General","none"]),
  uiLanguage: z.string(), consent: z.object({ privacy: z.literal(true), research: z.boolean().default(false) }),
  accommodations: z.object({ extendedTime: z.boolean(), transcriptAccess: z.boolean(), oralReadingAlternative: z.boolean(),
    highContrast: z.boolean(), textScale: z.number(), spacing: z.enum(["normal","wide"]), keyboardOnly: z.boolean() }),
  form_id: z.string(), ruleset_id: z.string(), modules: z.record(z.union([ObjectiveModuleState, ProductiveModuleState])),
  productiveRoute: z.object({ route: Route, wideWindow: z.boolean(), lifted: z.boolean(), basis: z.array(z.string()) }).nullable(),
  device: z.object({ class: z.enum(["mobile","tablet","desktop"]), ua: z.string(), network: z.string().nullable() }),
  flags: z.array(z.object({ code: z.string(), module: Module.optional(), detail: z.string().optional(), at: z.string() })),
  interruptions: z.array(z.object({ module: Module, itemId: z.string().optional(), kind: z.string(), at: z.string(), recovered: z.boolean() })),
  profileType: z.enum(["none","foundation_receptive","full"]).default("none") });
```

## 4. Responses, ratings, results

```ts
export const ObjectiveResponse = z.object({ response_id: z.string(), session_id: z.string(), item_id: z.string(),
  module: z.enum(["LS","RD","LSN"]), band: Band, permutation: z.array(z.string()),   // option_ids in delivered order
  selected_option_id: z.string().nullable(), omitted: z.boolean(), correct: z.boolean(),    // server-computed; NEVER sent to client
  responseMs: z.number().int(), replayCount: z.number().int().default(0), submittedAt: z.string(),
  purpose: z.enum(["locator","tie","confirmation","boundary","extension","descent","pretest"]) });
// Client-facing echo of a response contains only { item_id, accepted:true, next:... } — never `correct`.

export const TraitScores = z.record(z.number().int().min(0).max(5));                 // trait -> 0..5
export const ProductiveRating = z.object({ rating_id: z.string(), session_id: z.string(), task_id: z.string(),
  module: z.enum(["SPK","WRT"]), route: Route, rater: z.enum(["ai","human"]),
  model: z.string().nullable(), promptVersion: z.string(), rubricVersion: z.string(), benchmarkSet: z.string().nullable(),
  usable: z.boolean(), unusableReason: z.string().nullable(),
  atLower: TraitScores, atUpper: TraitScores,        // dual-reference (see 05 and DECISIONS D-003)
  transcript: z.string().nullable(),                 // SPK: model transcript (server-only)
  rationale: z.string(), flags: z.array(z.string()), createdAt: z.string(),
  supersededBy: z.string().nullable(), regenerationReason: z.string().nullable(), humanReviewStatus: z.enum(["none","queued","reviewed"]) });

export const SkillResult = z.object({ skill: z.enum(["RD","LSN","SPK","WRT"]),
  status: z.enum(["measured","insufficient_evidence","not_measured"]), band: Band.nullable(),
  range: z.tuple([Band, Band]).nullable(), notes: z.array(z.string()), canDo: z.array(z.string()) });

export const ResultReport = z.object({ session_id: z.string(), profileType: z.enum(["foundation_receptive","full"]),
  skills: z.array(SkillResult), diagnostics: z.object({ languageSystems: z.object({ band: Band.nullable(), range: z.tuple([Band,Band]).nullable(),
    constructsStrong: z.array(z.string()), constructsWeak: z.array(z.string()) }), listenToWrite: z.object({ accuracy: z.string() }).nullable() }),
  headline: z.object({ kind: z.enum(["indicative_overall","uneven","none"]), band: Band.nullable(), range: z.tuple([Band,Band]).nullable() }),
  confidence: Confidence, confidenceReasons: z.array(z.string()),
  readiness: z.object({ target: z.string(), text: z.string(), disclaimer: z.string() }).nullable(),
  retestAdvice: z.string(), wordingVersion: z.string(), generatedAt: z.string() });
```

## 5. Firestore collections and access

| Collection | Contents | Client access |
|---|---|---|
| `items` | `ObjectiveItem` (candidate-safe) | none (API only) |
| `stimuli_reading` | `ReadingStimulus` | none (API only) |
| `stimuli_listening` | `ListeningStimulusPublic` | none (API only) |
| `stimulus_admin` | `ListeningStimulusAdmin` **RESTRICTED** | deny |
| `restricted_keys` | `RestrictedKey` **RESTRICTED** | deny |
| `speaking_tasks`, `writing_tasks` | tasks | none (API only) |
| `rulesets`, `forms`, `rubrics`, `claims_config` | config | none |
| `sessions/{id}` | `Session` | read own (`resource.data.candidate_uid == request.auth.uid`), no write |
| `sessions/{id}/responses/{rid}` | `ObjectiveResponse` **contains `correct`** | deny |
| `sessions/{id}/productive/{taskId}` | drafts, storage paths, metrics | read own draft text only via API |
| `ratings` | `ProductiveRating` | deny |
| `results/{sessionId}` | `ResultReport` | read own |
| `jobs`, `telemetry`, `exposure` | ops | deny |
| `users/{uid}` | role claims mirror | read own |

Rules unit tests (M4) must prove: a candidate cannot read `restricted_keys`, `stimulus_admin`, another session,
any `responses` sub-collection, or any `ratings` document.

## 6. Storage layout
`audio/stimuli/{stimulusId}/{version}.{opus|mp3}` (signed URL only), `audio/prompts/{taskId}.{opus|mp3}`,
`audio/responses/{sessionId}/{taskId}/{attempt}.webm` (private; server-side read for rating; deleted per retention policy in `09`).
