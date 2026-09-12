import test from 'node:test';
import assert from 'node:assert/strict';
import {
  Band,
  Route,
  Module,
  Domain,
  Confidence,
  ModuleStatus,
  Mode,
  Role,
  Option,
  OptionChoice,
  ReviewerProvenance,
  ItemExposure,
  ObjectiveItem,
  ReadingStimulus,
  StimulusMedia,
  ListeningStimulusPublic,
  TaskAudio,
  SpeakingTask,
  WordGuidance,
  WritingTask,
  ConfirmationConfig,
  ProductiveRulesetConfig,
  RoutingRuleset,
  EnemyGroupConflict,
  EnemyGroupValidation,
  FormSpec,
  LocatorEvent,
  ConfirmationBlock,
  CurrentUnit,
  ObjectiveModuleOutcome,
  ObjectiveModuleState,
  MicCheck,
  ProductiveModuleState,
  SessionConsent,
  SessionAccommodations,
  Accommodations,
  ProductiveRouteInfo,
  DeviceMetadata,
  SessionFlag,
  SessionInterruption,
  Session,
  ObjectiveResponse,
  TraitScores,
  ProductiveRating,
  SkillResult,
  LanguageSystemsDiagnostic,
  ListenToWriteDiagnostic,
  DiagnosticsSummary,
  Headline,
  ReadinessLayer,
  ResultReport,
} from '../src/schemas/api.ts';

test('1. Enums match 03_DATA_MODEL specification', () => {
  assert.deepEqual(Band.options, ['Pre-A1', 'A1', 'A2', 'B1', 'B2', 'C1', 'C2']);
  assert.deepEqual(Route.options, ['PreA1-A1', 'A1-A2', 'A2-B1', 'B1-B2', 'B2-C1', 'C1-C2']);
  assert.deepEqual(Module.options, ['LS', 'RD', 'LSN', 'SPK', 'WRT']);
  assert.deepEqual(Domain.options, ['personal', 'public', 'educational', 'occupational']);
  assert.deepEqual(Confidence.options, ['Low', 'Moderate']);
  assert.deepEqual(ModuleStatus.options, ['pending', 'in_progress', 'paused', 'complete', 'abandoned', 'not_measured']);
  assert.deepEqual(Mode.options, ['free_beta', 'verified']);
  assert.deepEqual(Role.options, ['candidate', 'reviewer', 'admin']);

  // "High" confidence must be rejected
  assert.throws(() => Confidence.parse('High'));
  // Suffix levels must be rejected
  assert.throws(() => Band.parse('B1+'));
  assert.throws(() => Band.parse('B2-'));
});

test('2. Bank objects schema validation and defaults', () => {
  // Option
  const opt = Option.parse({ option_id: 'opt_1', text: 'Option 1' });
  assert.equal(opt.option_id, 'opt_1');
  assert.equal(OptionChoice, Option);

  // ObjectiveItem with defaults
  const objItem = ObjectiveItem.parse({
    item_id: 'LS-B1-01',
    module: 'LS',
    band: 'B1',
    stem: 'Select the best phrase.',
    options: [
      { option_id: 'o1', text: 'First' },
      { option_id: 'o2', text: 'Second' },
      { option_id: 'o3', text: 'Third' },
    ],
  });
  assert.equal(objItem.version, 1);
  assert.equal(objItem.status, 'trial');
  assert.equal(objItem.locator_candidate, false);
  assert.equal(objItem.is_anchor, false);
  assert.equal(objItem.is_pretest, false);
  assert.deepEqual(objItem.reviewer_provenance, []);

  // ObjectiveItem option bounds (min 3, max 4)
  assert.throws(() =>
    ObjectiveItem.parse({
      item_id: 'LS-B1-01',
      module: 'LS',
      band: 'B1',
      stem: 'Stem',
      options: [{ option_id: 'o1', text: 'A' }, { option_id: 'o2', text: 'B' }],
    })
  );

  // ReadingStimulus
  const rdStim = ReadingStimulus.parse({
    stimulus_id: 'RD-B1-S1',
    band: 'B1',
    stimulus_type: 'article',
    domain: 'educational',
    topic_family: 'campus',
    text: 'Article body text.',
    word_count: 50,
    item_ids: ['RD-B1-01', 'RD-B1-02'],
  });
  assert.equal(rdStim.word_count, 50);

  // ListeningStimulusPublic (the admin variant that carried `script` was
  // removed — restricted collection shapes don't ship in the client bundle)
  const media = {
    storagePath: 'audio/stimuli/LSN-B1-S1/v1.opus',
    durationSec: 30.5,
    loudnessLufs: -16.0,
    checksum: 'sha256:123',
    codec: 'opus',
  };
  const lsnPub = ListeningStimulusPublic.parse({
    stimulus_id: 'LSN-B1-S1',
    band: 'B1',
    speaker_count: 1,
    domain: 'personal',
    topic_family: 'travel',
    media,
    item_ids: ['LSN-B1-01'],
  });
  assert.equal(lsnPub.speaker_count, 1);
  // No script-bearing field may exist on the candidate-safe shape.
  assert.equal('script' in ListeningStimulusPublic.shape, false);

  // SpeakingTask
  const spk = SpeakingTask.parse({
    task_id: 'SPK-B1B2-OR',
    route: 'B1-B2',
    route_bands: ['B1', 'B2'],
    task_type: 'oral_reading',
    task_group: 'group_1',
    turn: null,
    label: 'Oral Reading',
    prompt: 'Read this paragraph aloud.',
    candidate_sees_text: true,
    delivery: 'text_read_aloud',
    weight: 0.1,
    spontaneous_or_interactive: false,
    traits_scored: ['intelligibility', 'fluency'],
    topic_family: 'daily_life',
    prepSeconds: 15,
    maxSpeakSeconds: 45,
    allowsRerecord: true,
  });
  assert.equal(spk.prepSeconds, 15);

  // WritingTask
  const wrt = WritingTask.parse({
    task_id: 'WRT-B1B2-1',
    route: 'B1-B2',
    route_bands: ['B1', 'B2'],
    task_type: 'functional_communication',
    label: 'Email inquiry',
    prompt: 'Write an email asking for information.',
    focus: 'polite register',
    word_guidance: { min: 60, max: 100 },
    weight: 0.3,
    scored_in_writing_level: true,
    topic_family: 'customer_service',
    timeLimitSeconds: 360,
  });
  assert.equal(wrt.timeLimitSeconds, 360);

  // The candidate-facing task schemas must NOT accept server-only fields
  // (AGENTS.md "Never" list): if a future server regression re-adds
  // `audio_script` to the wire, this parse must fail loudly — wait, no:
  // zod objects ignore unknown keys by default, so the *server* strip is
  // the enforcement point (server/src/services.rs). What we assert here
  // is that the schema itself no longer declares them, so no client code
  // can ever read them via the typed surface.
  assert.equal('audio_script' in SpeakingTask.shape, false);
  assert.equal('interlocutor_line' in SpeakingTask.shape, false);
  assert.equal('audio_script' in WritingTask.shape, false);
});

test('3. Ruleset, form, session schema validation and defaults', () => {
  // RoutingRuleset defaults
  const ruleset = RoutingRuleset.parse({
    ruleset_id: 'ruleset_pilot',
    version: '2.0',
    confirmation: {},
    productive: {},
    effective_from: '2026-09-01T00:00:00Z',
  });
  assert.equal(ruleset.start_band, 'B1');
  assert.equal(ruleset.locator_pair_size, 2);
  assert.equal(ruleset.locator_hard_cap, 8);
  assert.equal(ruleset.confirmation.lower_block, 5);
  assert.equal(ruleset.confirmation.strong_min, 0.8);
  assert.equal(ruleset.productive.lift_when_both_receptive_above_ls, true);

  // FormSpec
  const form = FormSpec.parse({
    form_id: 'beta_form',
    version: '1.0',
    blueprint_version: 'v2.0',
    ruleset_id: 'ruleset_pilot',
    item_ids_by_module: { LS: ['LS-B1-01'] },
    anchors: ['LS-B1-01'],
    key_balance_report: { LS: { A: 0.25, B: 0.25, C: 0.25, D: 0.25 } },
    domain_coverage_report: {},
    enemy_group_validation: { conflicts: [] },
    status: 'beta',
  });
  assert.equal(form.status, 'beta');

  // ObjectiveModuleState
  const objState = ObjectiveModuleState.parse({
    module: 'LS',
    status: 'in_progress',
    phase: 'locator',
    currentBand: 'B1',
    direction: 'none',
    locatorItemsUsed: 2,
    locatorTrace: [
      { band: 'B1', itemIds: ['LS-B1-01', 'LS-B1-02'], correct: 2, kind: 'pair' },
    ],
    bracket: null,
    confirmationTrace: [],
    usedItemIds: ['LS-B1-01', 'LS-B1-02'],
    currentUnit: null,
    outcome: null,
    stateVersion: 1,
  });
  assert.equal(objState.status, 'in_progress');

  // ProductiveModuleState
  const prodState = ProductiveModuleState.parse({
    module: 'SPK',
    status: 'pending',
    route: 'B1-B2',
    taskIds: ['SPK-B1B2-OR'],
    currentTaskId: 'SPK-B1B2-OR',
    deadlineAt: null,
    attempts: { 'SPK-B1B2-OR': 1 },
    micCheck: { passed: true, at: '2026-09-11T12:00:00Z' },
    ratingStatus: 'not_started',
    stateVersion: 1,
  });
  assert.equal(prodState.module, 'SPK');

  // Session
  const session = Session.parse({
    session_id: 'sess_123',
    candidate_uid: 'cand_456',
    mode: 'free_beta',
    createdAt: '2026-09-11T12:00:00Z',
    targetGoal: 'IELTS',
    uiLanguage: 'en',
    consent: { privacy: true, research: true },
    accommodations: {
      highContrast: true,
      textScale: 1.2,
      spacing: 'wide',
    },
    form_id: 'beta_form',
    ruleset_id: 'ruleset_pilot',
    modules: { LS: objState, SPK: prodState },
    productiveRoute: {
      route: 'B1-B2',
      wideWindow: false,
      lifted: false,
      basis: ['LS', 'RD'],
    },
    device: { class: 'desktop', ua: 'Chrome', network: null },
    flags: [],
    interruptions: [],
    profileType: 'full',
  });
  assert.equal(session.targetGoal, 'IELTS');
  assert.equal(session.accommodations.textScale, 1.2);
  assert.equal(session.accommodations.spacing, 'wide');

  // Consent without privacy: true must fail
  assert.throws(() =>
    SessionConsent.parse({ privacy: false, research: false })
  );
});

test('4. Responses, ratings, results schema validation', () => {
  // ObjectiveResponse
  const resp = ObjectiveResponse.parse({
    response_id: 'resp_1',
    session_id: 'sess_123',
    item_id: 'LS-B1-01',
    module: 'LS',
    band: 'B1',
    permutation: ['o1', 'o2', 'o3'],
    selected_option_id: 'o2',
    omitted: false,
    correct: true,
    responseMs: 3200,
    replayCount: 0,
    submittedAt: '2026-09-11T12:05:00Z',
    purpose: 'locator',
  });
  assert.equal(resp.correct, true);
  assert.equal(resp.responseMs, 3200);

  // TraitScores constraint (0..5)
  const scores = TraitScores.parse({ fluency: 4, pronunciation: 3 });
  assert.equal(scores.fluency, 4);
  assert.throws(() => TraitScores.parse({ fluency: 6 }));
  assert.throws(() => TraitScores.parse({ fluency: -1 }));

  // ProductiveRating
  const rating = ProductiveRating.parse({
    rating_id: 'rate_1',
    session_id: 'sess_123',
    task_id: 'SPK-B1B2-OR',
    module: 'SPK',
    route: 'B1-B2',
    rater: 'ai',
    model: 'gemini-1.5-pro',
    promptVersion: 'speaking_rater@v2.0',
    rubricVersion: '2026.1',
    benchmarkSet: null,
    usable: true,
    unusableReason: null,
    atLower: { fluency: 3 },
    atUpper: { fluency: 2 },
    transcript: 'Candidate transcript',
    rationale: 'Satisfactory performance',
    flags: [],
    createdAt: '2026-09-11T12:10:00Z',
    supersededBy: null,
    regenerationReason: null,
    humanReviewStatus: 'none',
  });
  assert.equal(rating.usable, true);

  // ResultReport
  const report = ResultReport.parse({
    session_id: 'sess_123',
    profileType: 'full',
    skills: [
      {
        skill: 'RD',
        status: 'measured',
        band: 'B2',
        range: null,
        notes: ['Strong reading comprehension'],
        canDo: ['Can read complex texts'],
      },
    ],
    diagnostics: {
      languageSystems: {
        band: 'B2',
        range: null,
        constructsStrong: ['collocations'],
        constructsWeak: [],
      },
      listenToWrite: { accuracy: 'exact' },
    },
    headline: { kind: 'indicative_overall', band: 'B2', range: null },
    confidence: 'Moderate',
    confidenceReasons: ['All modules complete'],
    readiness: {
      target: 'OET',
      text: 'Good preparation',
      disclaimer: 'GEPA does not predict official exam scores.',
    },
    retestAdvice: 'Retest in 8-12 weeks',
    wordingVersion: '2.0.0-beta',
    generatedAt: '2026-09-11T12:20:00Z',
  });
  assert.equal(report.session_id, 'sess_123');
  assert.equal(report.confidence, 'Moderate');
});
