use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Band {
    #[serde(rename = "Pre-A1")]
    PreA1,
    #[serde(rename = "A1")]
    A1,
    #[serde(rename = "A2")]
    A2,
    #[serde(rename = "B1")]
    B1,
    #[serde(rename = "B2")]
    B2,
    #[serde(rename = "C1")]
    C1,
    #[serde(rename = "C2")]
    C2,
}

impl Band {
    pub const ALL: [Band; 7] = [
        Band::PreA1,
        Band::A1,
        Band::A2,
        Band::B1,
        Band::B2,
        Band::C1,
        Band::C2,
    ];

    pub fn index(self) -> usize {
        match self {
            Band::PreA1 => 0,
            Band::A1 => 1,
            Band::A2 => 2,
            Band::B1 => 3,
            Band::B2 => 4,
            Band::C1 => 5,
            Band::C2 => 6,
        }
    }

    pub fn from_index(index: usize) -> Option<Band> {
        match index {
            0 => Some(Band::PreA1),
            1 => Some(Band::A1),
            2 => Some(Band::A2),
            3 => Some(Band::B1),
            4 => Some(Band::B2),
            5 => Some(Band::C1),
            6 => Some(Band::C2),
            _ => None,
        }
    }

    pub fn prev(self) -> Option<Band> {
        if self.index() == 0 {
            None
        } else {
            Self::from_index(self.index() - 1)
        }
    }

    pub fn next(self) -> Option<Band> {
        if self.index() >= 6 {
            None
        } else {
            Self::from_index(self.index() + 1)
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Band::PreA1 => "Pre-A1",
            Band::A1 => "A1",
            Band::A2 => "A2",
            Band::B1 => "B1",
            Band::B2 => "B2",
            Band::C1 => "C1",
            Band::C2 => "C2",
        }
    }
}

impl fmt::Display for Band {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Band::PreA1 => write!(f, "Pre-A1"),
            Band::A1 => write!(f, "A1"),
            Band::A2 => write!(f, "A2"),
            Band::B1 => write!(f, "B1"),
            Band::B2 => write!(f, "B2"),
            Band::C1 => write!(f, "C1"),
            Band::C2 => write!(f, "C2"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Route {
    #[serde(rename = "PreA1-A1")]
    PreA1A1,
    #[serde(rename = "A1-A2")]
    A1A2,
    #[serde(rename = "A2-B1")]
    A2B1,
    #[serde(rename = "B1-B2")]
    B1B2,
    #[serde(rename = "B2-C1")]
    B2C1,
    #[serde(rename = "C1-C2")]
    C1C2,
}

impl Route {
    pub const ALL: [Route; 6] = [
        Route::PreA1A1,
        Route::A1A2,
        Route::A2B1,
        Route::B1B2,
        Route::B2C1,
        Route::C1C2,
    ];

    pub fn index(self) -> usize {
        match self {
            Route::PreA1A1 => 0,
            Route::A1A2 => 1,
            Route::A2B1 => 2,
            Route::B1B2 => 3,
            Route::B2C1 => 4,
            Route::C1C2 => 5,
        }
    }

    pub fn from_index(index: usize) -> Option<Route> {
        match index {
            0 => Some(Route::PreA1A1),
            1 => Some(Route::A1A2),
            2 => Some(Route::A2B1),
            3 => Some(Route::B1B2),
            4 => Some(Route::B2C1),
            5 => Some(Route::C1C2),
            _ => None,
        }
    }

    pub fn bands(self) -> (Band, Band) {
        match self {
            Route::PreA1A1 => (Band::PreA1, Band::A1),
            Route::A1A2 => (Band::A1, Band::A2),
            Route::A2B1 => (Band::A2, Band::B1),
            Route::B1B2 => (Band::B1, Band::B2),
            Route::B2C1 => (Band::B2, Band::C1),
            Route::C1C2 => (Band::C1, Band::C2),
        }
    }

    pub fn lower_band(self) -> Band {
        self.bands().0
    }

    pub fn upper_band(self) -> Band {
        self.bands().1
    }
}

impl fmt::Display for Route {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Route::PreA1A1 => write!(f, "PreA1-A1"),
            Route::A1A2 => write!(f, "A1-A2"),
            Route::A2B1 => write!(f, "A2-B1"),
            Route::B1B2 => write!(f, "B1-B2"),
            Route::B2C1 => write!(f, "B2-C1"),
            Route::C1C2 => write!(f, "C1-C2"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Module {
    LS,
    RD,
    LSN,
    SPK,
    WRT,
}

impl fmt::Display for Module {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Module::LS => write!(f, "LS"),
            Module::RD => write!(f, "RD"),
            Module::LSN => write!(f, "LSN"),
            Module::SPK => write!(f, "SPK"),
            Module::WRT => write!(f, "WRT"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Domain {
    Personal,
    Public,
    Educational,
    Occupational,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Confidence {
    Low,
    Moderate,
}

impl fmt::Display for Confidence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Confidence::Low => write!(f, "Low"),
            Confidence::Moderate => write!(f, "Moderate"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModuleStatus {
    Pending,
    InProgress,
    Paused,
    Complete,
    Abandoned,
    NotMeasured,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    FreeBeta,
    Verified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Candidate,
    Reviewer,
    Admin,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OptionChoice {
    pub option_id: String,
    pub text: String,
}

pub type OptionItem = OptionChoice;
pub type BankOption = OptionChoice;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ReviewerProvenance {
    pub reviewer: String,
    pub date: String,
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemExposure {
    pub delivered: u32,
    pub correct: u32,
    #[serde(rename = "medianMs", alias = "median_ms")]
    pub median_ms: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObjectiveItem {
    pub item_id: String,
    pub module: String, // "LS" | "RD" | "LSN"
    pub band: Band,
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default = "default_status")]
    pub status: String,
    pub stem: String,
    pub options: Vec<OptionChoice>,
    #[serde(default)]
    pub construct: Option<String>,
    #[serde(default)]
    pub evidence_focus: Option<String>,
    #[serde(default)]
    pub domain: Option<Domain>,
    #[serde(default)]
    pub topic_family: Option<String>,
    #[serde(default)]
    pub locator_candidate: bool,
    #[serde(default)]
    pub stimulus_id: Option<String>,
    #[serde(default)]
    pub is_anchor: bool,
    #[serde(default)]
    pub is_pretest: bool,
    #[serde(default)]
    pub accessibility_alternative: Option<String>,
    #[serde(default)]
    pub reviewer_provenance: Vec<ReviewerProvenance>,
    #[serde(default)]
    pub exposure: Option<ItemExposure>,
}

fn default_version() -> u32 {
    1
}
fn default_status() -> String {
    "trial".to_string()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadingStimulus {
    pub stimulus_id: String,
    pub band: Band,
    pub stimulus_type: String,
    pub domain: Domain,
    pub topic_family: String,
    pub text: String,
    pub word_count: u32,
    pub item_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StimulusMedia {
    #[serde(rename = "storagePath", alias = "storage_path")]
    pub storage_path: String,
    #[serde(rename = "durationSec", alias = "duration_sec")]
    pub duration_sec: f64,
    #[serde(rename = "loudnessLufs", alias = "loudness_lufs")]
    pub loudness_lufs: f64,
    pub checksum: String,
    pub codec: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListeningStimulusPublic {
    pub stimulus_id: String,
    pub band: Band,
    pub speaker_count: u32,
    pub domain: Domain,
    pub topic_family: String,
    pub media: Option<StimulusMedia>,
    pub item_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListeningStimulusAdmin {
    pub stimulus_id: String,
    pub band: Band,
    pub speaker_count: u32,
    pub domain: Domain,
    pub topic_family: String,
    #[serde(default)]
    pub media: Option<StimulusMedia>,
    pub item_ids: Vec<String>,
    #[serde(alias = "text")]
    pub script: String,
    pub speakers: String,
    pub target_wpm: u32,
    pub word_count: u32,
    pub est_duration_sec: f64,
    #[serde(default)]
    pub accents: Vec<String>,
    #[serde(default)]
    pub voice_cast: HashMap<String, String>,
    #[serde(default)]
    pub admin_only: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskAudio {
    #[serde(rename = "storagePath", alias = "storage_path")]
    pub storage_path: String,
    #[serde(rename = "durationSec", alias = "duration_sec")]
    pub duration_sec: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpeakingTask {
    pub task_id: String,
    pub route: Route,
    pub route_bands: (Band, Band),
    pub task_type: String,
    pub task_group: String,
    #[serde(default)]
    pub turn: Option<u32>,
    pub label: String,
    pub prompt: String,
    pub candidate_sees_text: bool,
    pub delivery: String,
    #[serde(default)]
    pub audio_script: Option<String>,
    #[serde(default)]
    pub interlocutor_line: Option<String>,
    #[serde(default)]
    pub audio: Option<TaskAudio>,
    pub weight: f64,
    pub spontaneous_or_interactive: bool,
    pub traits_scored: Vec<String>,
    pub topic_family: String,
    #[serde(rename = "prepSeconds", alias = "prep_seconds", default = "default_speaking_prep")]
    pub prep_seconds: u32,
    #[serde(rename = "maxSpeakSeconds", alias = "max_speak_seconds", default = "default_speaking_max")]
    pub max_speak_seconds: u32,
    #[serde(rename = "allowsRerecord", alias = "allows_rerecord", default = "default_true")]
    pub allows_rerecord: bool,
}

fn default_speaking_prep() -> u32 {
    15
}
fn default_speaking_max() -> u32 {
    45
}
fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WordGuidance {
    pub min: u32,
    pub max: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WritingTask {
    pub task_id: String,
    pub route: Route,
    pub route_bands: (Band, Band),
    pub task_type: String,
    pub label: String,
    pub prompt: String,
    #[serde(default)]
    pub target: Option<String>,
    pub focus: String,
    #[serde(default)]
    pub word_guidance: Option<WordGuidance>,
    pub weight: f64,
    pub scored_in_writing_level: bool,
    pub topic_family: String,
    #[serde(rename = "timeLimitSeconds", alias = "time_limit_seconds", default = "default_writing_limit")]
    pub time_limit_seconds: u32,
    #[serde(default)]
    pub audio_script: Option<String>,
    #[serde(default)]
    pub audio: Option<TaskAudio>,
}

fn default_writing_limit() -> u32 {
    300
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyDetail {
    pub module: String,
    pub band: Band,
    pub key_option_id: String,
    pub authoring_letter: String, // "A" | "B" | "C" | "D"
    pub answer_text: String,
    pub option_count: u32,
    #[serde(default)]
    pub evidence_focus: Option<String>,
    #[serde(default)]
    pub rationale: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestrictedKey {
    pub item_id: String,
    pub module: String,
    pub band: Band,
    pub key_option_id: String,
    pub authoring_letter: String, // "A" | "B" | "C" | "D"
    pub answer_text: String,
    pub option_count: u32,
    #[serde(default)]
    pub evidence_focus: Option<String>,
    #[serde(default)]
    pub rationale: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidateDeliveryUnit {
    pub stimulus_id: Option<String>,
    pub stimulus_text: Option<String>, // Reading only; Listening is signed audio URL
    pub stimulus_type: Option<String>,
    pub audio_url: Option<String>,     // Listening only
    pub items: Vec<CandidateItemPayload>,
    pub deadline_at: String, // ISO-8601 UTC
    pub module_complete: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidateItemPayload {
    pub item_id: String,
    pub module: String,
    pub stem: String,
    pub options: Vec<OptionChoice>,
}

// ---------------------------------------------------------------------------
// Ruleset, Form, Session (§3)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfirmationConfig {
    #[serde(rename = "lower_block", default = "default_block_size")]
    pub lower_block: u32,
    #[serde(rename = "upper_block", default = "default_block_size")]
    pub upper_block: u32,
    #[serde(rename = "strong_min", default = "default_strong_min")]
    pub strong_min: f64,
    #[serde(rename = "borderline", default = "default_borderline")]
    pub borderline: f64,
    #[serde(rename = "weak_max", default = "default_weak_max")]
    pub weak_max: f64,
    #[serde(rename = "boundary_items", default = "default_boundary_items")]
    pub boundary_items: u32,
    #[serde(rename = "extension_items", default = "default_extension_items")]
    pub extension_items: u32,
    #[serde(rename = "descent_items", default = "default_descent_items")]
    pub descent_items: u32,
    #[serde(rename = "aberrant_gap", default = "default_aberrant_gap")]
    pub aberrant_gap: u32,
}

impl Default for ConfirmationConfig {
    fn default() -> Self {
        Self {
            lower_block: 5,
            upper_block: 5,
            strong_min: 0.8,
            borderline: 0.6,
            weak_max: 0.4,
            boundary_items: 4,
            extension_items: 4,
            descent_items: 5,
            aberrant_gap: 3,
        }
    }
}

fn default_block_size() -> u32 {
    5
}
fn default_strong_min() -> f64 {
    0.8
}
fn default_borderline() -> f64 {
    0.6
}
fn default_weak_max() -> f64 {
    0.4
}
fn default_boundary_items() -> u32 {
    4
}
fn default_extension_items() -> u32 {
    4
}
fn default_descent_items() -> u32 {
    5
}
fn default_aberrant_gap() -> u32 {
    3
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductiveRulesetConfig {
    #[serde(rename = "lift_when_both_receptive_above_ls", default = "default_true")]
    pub lift_when_both_receptive_above_ls: bool,
    #[serde(rename = "wide_window_span", default = "default_wide_window_span")]
    pub wide_window_span: u32,
}

impl Default for ProductiveRulesetConfig {
    fn default() -> Self {
        Self {
            lift_when_both_receptive_above_ls: true,
            wide_window_span: 2,
        }
    }
}

fn default_wide_window_span() -> u32 {
    2
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoutingRuleset {
    pub ruleset_id: String,
    pub version: String,
    #[serde(default = "default_start_band")]
    pub start_band: Band,
    #[serde(default = "default_locator_pair_size")]
    pub locator_pair_size: u32,
    #[serde(default = "default_locator_hard_cap")]
    pub locator_hard_cap: u32,
    pub confirmation: ConfirmationConfig,
    pub productive: ProductiveRulesetConfig,
    pub effective_from: String,
}

fn default_start_band() -> Band {
    Band::B1
}
fn default_locator_pair_size() -> u32 {
    2
}
fn default_locator_hard_cap() -> u32 {
    8
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnemyGroupConflict {
    pub family: String,
    pub members: Vec<String>,
    pub severity: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnemyGroupValidation {
    pub conflicts: Vec<EnemyGroupConflict>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormSpec {
    pub form_id: String,
    pub version: String,
    pub blueprint_version: String,
    pub ruleset_id: String,
    pub item_ids_by_module: HashMap<String, Vec<String>>,
    pub anchors: Vec<String>,
    pub key_balance_report: HashMap<String, HashMap<String, f64>>,
    pub domain_coverage_report: HashMap<String, serde_json::Value>,
    pub enemy_group_validation: EnemyGroupValidation,
    pub status: String, // "draft" | "beta" | "retired"
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocatorEvent {
    pub band: Band,
    #[serde(rename = "itemIds", alias = "item_ids", default)]
    pub item_ids: Vec<String>,
    pub correct: u32,
    pub kind: String, // "pair" or "tie"
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfirmationBlock {
    pub band: Band,
    pub kind: String, // "lower", "upper", "boundary", "extension", "descent", "aberrant_recheck"
    #[serde(rename = "itemIds", alias = "item_ids")]
    pub item_ids: Vec<String>,
    pub correct: u32,
    pub size: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurrentUnit {
    #[serde(rename = "stimulusId", alias = "stimulus_id")]
    pub stimulus_id: Option<String>,
    #[serde(rename = "itemIds", alias = "item_ids")]
    pub item_ids: Vec<String>,
    #[serde(rename = "deadlineAt", alias = "deadline_at")]
    pub deadline_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObjectiveModuleOutcome {
    pub band: Option<Band>,
    pub range: Option<(Band, Band)>,
    pub notes: Vec<String>,
    pub flags: Vec<String>,
    #[serde(rename = "evidenceShortfall", alias = "evidence_shortfall", default)]
    pub evidence_shortfall: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObjectiveModuleState {
    pub module: String, // "LS" | "RD" | "LSN"
    pub status: ModuleStatus,
    pub phase: String, // "locator" | "confirmation" | "done"
    #[serde(rename = "currentBand", alias = "current_band")]
    pub current_band: Band,
    pub direction: String, // "none" | "up" | "down"
    #[serde(rename = "locatorItemsUsed", alias = "locator_items_used")]
    pub locator_items_used: u32,
    #[serde(rename = "locatorTrace", alias = "locator_trace")]
    pub locator_trace: Vec<LocatorEvent>,
    pub bracket: Option<(Band, Band)>,
    #[serde(rename = "confirmationTrace", alias = "confirmation_trace")]
    pub confirmation_trace: Vec<ConfirmationBlock>,
    #[serde(rename = "usedItemIds", alias = "used_item_ids")]
    pub used_item_ids: Vec<String>,
    #[serde(rename = "currentUnit", alias = "current_unit")]
    pub current_unit: Option<CurrentUnit>,
    pub outcome: Option<ObjectiveModuleOutcome>,
    #[serde(rename = "stateVersion", alias = "state_version")]
    pub state_version: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MicCheck {
    pub passed: bool,
    pub at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProductiveModuleState {
    pub module: String, // "SPK" | "WRT"
    pub status: ModuleStatus,
    pub route: Option<Route>,
    #[serde(rename = "taskIds", alias = "task_ids")]
    pub task_ids: Vec<String>,
    #[serde(rename = "currentTaskId", alias = "current_task_id")]
    pub current_task_id: Option<String>,
    #[serde(rename = "deadlineAt", alias = "deadline_at")]
    pub deadline_at: Option<String>,
    pub attempts: HashMap<String, u32>,
    #[serde(rename = "micCheck", alias = "mic_check")]
    pub mic_check: Option<MicCheck>,
    #[serde(rename = "ratingStatus", alias = "rating_status")]
    pub rating_status: String, // "not_started" | "pending" | "complete" | "failed"
    #[serde(rename = "stateVersion", alias = "state_version")]
    pub state_version: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionConsent {
    pub privacy: bool,
    #[serde(default)]
    pub research: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionAccommodations {
    #[serde(rename = "extendedTime", alias = "extended_time", default)]
    pub extended_time: bool,
    #[serde(rename = "transcriptAccess", alias = "transcript_access", default)]
    pub transcript_access: bool,
    #[serde(rename = "oralReadingAlternative", alias = "oral_reading_alternative", default)]
    pub oral_reading_alternative: bool,
    #[serde(rename = "highContrast", alias = "high_contrast", default)]
    pub high_contrast: bool,
    #[serde(rename = "textScale", alias = "text_scale", default = "default_text_scale")]
    pub text_scale: f64,
    #[serde(default = "default_spacing")]
    pub spacing: String,
    #[serde(rename = "keyboardOnly", alias = "keyboard_only", default)]
    pub keyboard_only: bool,
}

impl Default for SessionAccommodations {
    fn default() -> Self {
        Self {
            extended_time: false,
            transcript_access: false,
            oral_reading_alternative: false,
            high_contrast: false,
            text_scale: 1.0,
            spacing: "normal".to_string(),
            keyboard_only: false,
        }
    }
}

pub type Accommodations = SessionAccommodations;

fn default_text_scale() -> f64 {
    1.0
}
fn default_spacing() -> String {
    "normal".to_string()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductiveRouteInfo {
    pub route: Route,
    #[serde(rename = "wideWindow", alias = "wide_window")]
    pub wide_window: bool,
    pub lifted: bool,
    pub basis: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceMetadata {
    pub class: String, // "mobile" | "tablet" | "desktop"
    pub ua: String,
    #[serde(default)]
    pub network: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionFlag {
    pub code: String,
    #[serde(default)]
    pub module: Option<Module>,
    #[serde(default)]
    pub detail: Option<String>,
    pub at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionInterruption {
    pub module: Module,
    #[serde(rename = "itemId", alias = "item_id", default)]
    pub item_id: Option<String>,
    pub kind: String,
    pub at: String,
    pub recovered: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SessionModuleState {
    Objective(ObjectiveModuleState),
    Productive(ProductiveModuleState),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Session {
    pub session_id: String,
    pub candidate_uid: String,
    pub mode: Mode,
    #[serde(rename = "createdAt", alias = "created_at")]
    pub created_at: String,
    #[serde(rename = "targetGoal", alias = "target_goal")]
    pub target_goal: String,
    #[serde(rename = "uiLanguage", alias = "ui_language")]
    pub ui_language: String,
    pub consent: SessionConsent,
    pub accommodations: SessionAccommodations,
    pub form_id: String,
    pub ruleset_id: String,
    pub modules: HashMap<String, SessionModuleState>,
    #[serde(rename = "productiveRoute", alias = "productive_route")]
    pub productive_route: Option<ProductiveRouteInfo>,
    pub device: DeviceMetadata,
    pub flags: Vec<SessionFlag>,
    pub interruptions: Vec<SessionInterruption>,
    #[serde(rename = "profileType", alias = "profile_type", default = "default_profile_type")]
    pub profile_type: String,
}

fn default_profile_type() -> String {
    "none".to_string()
}

// ---------------------------------------------------------------------------
// Responses, Ratings, Results (§4)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObjectiveResponse {
    pub response_id: String,
    pub session_id: String,
    pub item_id: String,
    pub module: String, // "LS" | "RD" | "LSN"
    pub band: Band,
    pub permutation: Vec<String>,
    pub selected_option_id: Option<String>,
    pub omitted: bool,
    pub correct: bool,
    #[serde(rename = "responseMs", alias = "response_ms")]
    pub response_ms: u64,
    #[serde(rename = "replayCount", alias = "replay_count", default)]
    pub replay_count: u32,
    #[serde(rename = "submittedAt", alias = "submitted_at")]
    pub submitted_at: String,
    pub purpose: String, // "locator" | "tie" | "confirmation" | "boundary" | "extension" | "descent" | "pretest"
}

pub type ObjectiveResponseRecord = ObjectiveResponse;
pub type TraitScores = HashMap<String, u32>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProductiveRating {
    pub rating_id: String,
    pub session_id: String,
    pub task_id: String,
    pub module: String, // "SPK" | "WRT"
    pub route: Route,
    pub rater: String, // "ai" | "human"
    pub model: Option<String>,
    #[serde(rename = "promptVersion", alias = "prompt_version")]
    pub prompt_version: String,
    #[serde(rename = "rubricVersion", alias = "rubric_version")]
    pub rubric_version: String,
    #[serde(rename = "benchmarkSet", alias = "benchmark_set")]
    pub benchmark_set: Option<String>,
    pub usable: bool,
    #[serde(rename = "unusableReason", alias = "unusable_reason")]
    pub unusable_reason: Option<String>,
    #[serde(rename = "atLower", alias = "at_lower")]
    pub at_lower: TraitScores,
    #[serde(rename = "atUpper", alias = "at_upper")]
    pub at_upper: TraitScores,
    pub transcript: Option<String>,
    pub rationale: String,
    pub flags: Vec<String>,
    #[serde(rename = "createdAt", alias = "created_at")]
    pub created_at: String,
    #[serde(rename = "supersededBy", alias = "superseded_by")]
    pub superseded_by: Option<String>,
    #[serde(rename = "regenerationReason", alias = "regeneration_reason")]
    pub regeneration_reason: Option<String>,
    #[serde(rename = "humanReviewStatus", alias = "human_review_status")]
    pub human_review_status: String, // "none" | "queued" | "reviewed"
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkillResult {
    pub skill: String,  // "RD" | "LSN" | "SPK" | "WRT"
    pub status: String, // "measured" | "insufficient_evidence" | "not_measured"
    pub band: Option<Band>,
    pub range: Option<(Band, Band)>,
    pub notes: Vec<String>,
    #[serde(rename = "canDo", alias = "can_do")]
    pub can_do: Vec<String>,
    #[serde(rename = "growthAreas", alias = "growth_areas", default)]
    pub growth_areas: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Headline {
    pub kind: String, // "indicative_overall" | "uneven" | "none"
    pub band: Option<Band>,
    pub range: Option<(Band, Band)>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LanguageSystemsDiagnostic {
    pub band: Option<Band>,
    pub range: Option<(Band, Band)>,
    #[serde(rename = "constructsStrong", alias = "constructs_strong")]
    pub constructs_strong: Vec<String>,
    #[serde(rename = "constructsWeak", alias = "constructs_weak")]
    pub constructs_weak: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListenToWriteDiagnostic {
    pub accuracy: String, // "exact" | "minor_errors" | "major_errors" | "unusable"
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiagnosticsSummary {
    #[serde(rename = "languageSystems", alias = "language_systems")]
    pub language_systems: LanguageSystemsDiagnostic,
    #[serde(rename = "listenToWrite", alias = "listen_to_write")]
    pub listen_to_write: Option<ListenToWriteDiagnostic>,
    #[serde(rename = "pronunciationNotes", alias = "pronunciation_notes", default)]
    pub pronunciation_notes: Vec<String>,
    #[serde(rename = "fluencyNotes", alias = "fluency_notes", default)]
    pub fluency_notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadinessLayer {
    pub target: String,
    pub text: String,
    pub disclaimer: String,
    #[serde(rename = "currencyNote", alias = "currency_note", default)]
    pub currency_note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResultReport {
    pub session_id: String,
    #[serde(rename = "profileType", alias = "profile_type")]
    pub profile_type: String, // "foundation_receptive" | "full"
    pub skills: Vec<SkillResult>,
    pub diagnostics: DiagnosticsSummary,
    pub headline: Headline,
    pub confidence: Confidence,
    #[serde(rename = "confidenceReasons", alias = "confidence_reasons")]
    pub confidence_reasons: Vec<String>,
    pub readiness: Option<ReadinessLayer>,
    #[serde(rename = "retestAdvice", alias = "retest_advice")]
    pub retest_advice: String,
    #[serde(rename = "wordingVersion", alias = "wording_version")]
    pub wording_version: String,
    #[serde(rename = "generatedAt", alias = "generated_at")]
    pub generated_at: String,
}

