import { z } from 'zod';

export const Band = z.enum(['Pre-A1', 'A1', 'A2', 'B1', 'B2', 'C1', 'C2']);
export const Route = z.enum(['PreA1-A1', 'A1-A2', 'A2-B1', 'B1-B2', 'B2-C1', 'C1-C2']);
export const Module = z.enum(['LS', 'RD', 'LSN', 'SPK', 'WRT']);
export const Confidence = z.enum(['Low', 'Moderate']); // 'High' intentionally absent

export const OptionChoice = z.object({
  option_id: z.string(),
  text: z.string(),
});

export const CandidateItemPayload = z.object({
  item_id: z.string(),
  module: z.string(),
  stem: z.string(),
  options: z.array(OptionChoice),
});

export const CandidateDeliveryUnit = z.object({
  stimulus_id: z.string().nullable().optional(),
  stimulus_text: z.string().nullable().optional(),
  stimulus_type: z.string().nullable().optional(),
  audio_url: z.string().nullable().optional(),
  items: z.array(CandidateItemPayload),
  deadline_at: z.string(),
  module_complete: z.boolean(),
});

export const Accommodations = z.object({
  extended_time: z.boolean().default(false),
  transcript_access: z.boolean().default(false),
  oral_reading_alternative: z.boolean().default(false),
  high_contrast: z.boolean().default(false),
  text_scale: z.number().default(1.0),
  spacing: z.enum(['normal', 'wide']).default('normal'),
  keyboard_only: z.boolean().default(false),
});

export const CreateSessionRequest = z.object({
  target_goal: z.string().optional(),
  ui_language: z.string().optional(),
  accommodations: Accommodations.optional(),
  mode: z.string().optional(),
  device_class: z.string().optional(),
  identity_metadata: z.any().optional(),
  proctoring_metadata: z.any().optional(),
});

export const CreateSessionResponse = z.object({
  session_id: z.string(),
  next_step: z.string(),
});

export const SingleItemResponse = z.object({
  item_id: z.string(),
  selected_option_id: z.string().nullable().optional(),
  response_ms: z.number().int().default(0),
});

export const SubmitResponseRequest = z.object({
  item_id: z.string().optional(),
  selected_option_id: z.string().nullable().optional(),
  response_ms: z.number().int().default(0),
  replay_count: z.number().int().default(0),
  responses: z.array(SingleItemResponse).optional(),
});

export const SkillResult = z.object({
  skill: z.enum(['RD', 'LSN', 'SPK', 'WRT']),
  status: z.enum(['measured', 'insufficient_evidence', 'not_measured']),
  band: Band.nullable(),
  range: z.tuple([Band, Band]).nullable(),
  notes: z.array(z.string()),
  can_do: z.array(z.string()),
  growth_areas: z.array(z.string()),
});

export const Headline = z.object({
  kind: z.enum(['indicative_overall', 'uneven', 'none']),
  band: Band.nullable(),
  range: z.tuple([Band, Band]).nullable(),
});

export const LanguageSystemsDiagnostic = z.object({
  band: Band.nullable(),
  range: z.tuple([Band, Band]).nullable(),
  constructs_strong: z.array(z.string()),
  constructs_weak: z.array(z.string()),
});

export const ListenToWriteDiagnostic = z.object({
  accuracy: z.string(),
});

export const DiagnosticsSummary = z.object({
  language_systems: LanguageSystemsDiagnostic,
  listen_to_write: ListenToWriteDiagnostic.nullable().optional(),
  pronunciation_notes: z.array(z.string()),
  fluency_notes: z.array(z.string()),
});

export const ReadinessLayer = z.object({
  target: z.string(),
  text: z.string(),
  disclaimer: z.string(),
  currency_note: z.string().nullable().optional(),
});

export const ResultReport = z.object({
  session_id: z.string(),
  profile_type: z.enum(['foundation_receptive', 'full']),
  skills: z.array(SkillResult),
  diagnostics: DiagnosticsSummary,
  headline: Headline,
  confidence: Confidence,
  confidence_reasons: z.array(z.string()),
  readiness: ReadinessLayer.nullable().optional(),
  retest_advice: z.string(),
  wording_version: z.string(),
  generated_at: z.string(),
});

export const BackgroundJob = z.object({
  id: z.string(),
  job_type: z.string(),
  session_id: z.string().nullable().optional(),
  task_id: z.string().nullable().optional(),
  status: z.enum(['pending', 'in_progress', 'completed', 'failed']),
  attempts: z.number().int(),
  max_attempts: z.number().int(),
  last_error: z.string().nullable().optional(),
  created_at: z.string(),
  updated_at: z.string(),
});

export const SubmitResponseResult = z.object({
  accepted: z.boolean(),
  next: CandidateDeliveryUnit.nullable().optional(),
  state_version: z.number().int().optional(),
});

