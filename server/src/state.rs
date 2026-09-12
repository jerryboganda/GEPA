use crate::repos::SeedBank;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use shared_engine::models::*;
use shared_engine::routing::{ConfirmationOutcome, LocatorState};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeObjectiveState {
    pub module: String, // "LS", "RD", "LSN"
    pub status: String, // "pending", "in_progress", "complete"
    pub locator: LocatorState,
    pub bracket: Option<(Band, Band)>,
    pub confirmation_stage: String, // "lower", "upper", "done"
    pub lower_items: Vec<String>,
    pub upper_items: Vec<String>,
    pub lower_correct: u32,
    pub lower_total: u32,
    pub upper_correct: u32,
    pub upper_total: u32,
    pub used_item_ids: HashSet<String>,
    pub current_delivery_unit: Option<CandidateDeliveryUnit>,
    pub outcome: Option<ConfirmationOutcome>,
    pub responses: Vec<ObjectiveResponseRecord>,
    #[serde(default)]
    pub current_pair_scores: Vec<bool>,
}

impl RuntimeObjectiveState {
    pub fn new(module: &str) -> Self {
        Self {
            module: module.to_string(),
            status: "pending".to_string(),
            locator: LocatorState::new(),
            bracket: None,
            confirmation_stage: "none".to_string(),
            lower_items: Vec::new(),
            upper_items: Vec::new(),
            lower_correct: 0,
            lower_total: 0,
            upper_correct: 0,
            upper_total: 0,
            used_item_ids: HashSet::new(),
            current_delivery_unit: None,
            outcome: None,
            responses: Vec::new(),
            current_pair_scores: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeProductiveState {
    pub module: String, // "SPK", "WRT"
    pub status: String,
    pub route: Option<Route>,
    pub task_ids: Vec<String>,
    pub current_task_idx: usize,
    pub drafts: HashMap<String, String>,
    pub submitted_tasks: HashSet<String>,
    pub ratings: Vec<ProductiveRating>,
}

impl RuntimeProductiveState {
    pub fn new(module: &str) -> Self {
        Self {
            module: module.to_string(),
            status: "pending".to_string(),
            route: None,
            task_ids: Vec::new(),
            current_task_idx: 0,
            drafts: HashMap::new(),
            submitted_tasks: HashSet::new(),
            ratings: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub session_id: String,
    pub candidate_uid: String,
    #[serde(default = "default_session_mode")]
    pub mode: String,
    #[serde(default = "default_device_class")]
    pub device_class: String,
    #[serde(default = "default_state_version")]
    pub state_version: u64,
    /// Wall-clock timestamp of the last objective response, used for the
    /// 750ms rapid-click throttle (`02_ARCHITECTURE.md §4`). Must be a real
    /// (persistable) timestamp, not `std::time::Instant` (monotonic, meaningless
    /// once the session round-trips through Postgres between requests).
    #[serde(default)]
    pub last_response_time: Option<chrono::DateTime<chrono::Utc>>,
    pub target_goal: String,
    pub ui_language: String,
    pub accommodations: Accommodations,
    pub flags: Vec<String>,
    pub created_at: String,
    #[serde(default)]
    pub identity_metadata: Option<serde_json::Value>,
    #[serde(default)]
    pub proctoring_metadata: Option<serde_json::Value>,
    pub ls_state: RuntimeObjectiveState,
    pub rd_state: RuntimeObjectiveState,
    pub lsn_state: RuntimeObjectiveState,
    pub spk_state: RuntimeProductiveState,
    pub wrt_state: RuntimeProductiveState,
    pub productive_route: Option<Route>,
    pub result_report: Option<ResultReport>,
    #[serde(default)]
    pub previous_reports: Vec<ResultReport>,
}

fn default_session_mode() -> String {
    "free_beta".to_string()
}

fn default_device_class() -> String {
    "desktop".to_string()
}

fn default_state_version() -> u64 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundJob {
    pub id: String,
    pub job_type: String,
    pub session_id: Option<String>,
    pub task_id: Option<String>,
    pub status: String,
    pub attempts: u32,
    pub max_attempts: u32,
    pub last_error: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlaggedSessionReview {
    pub session_id: String,
    pub flag_type: String,
    pub module: String,
    pub details: String,
    pub timestamp: String,
}

pub struct AppState {
    pub seed_bank: RwLock<SeedBank>,
    /// Raw pool access for the one table that isn't a generic JSONB
    /// document (`staff_users` — see `api.rs::login`); everything else goes
    /// through the typed repos below.
    pub db: Arc<crate::db::PostgresClient>,
    pub session_repo: crate::repos::SessionRepo,
    pub job_repo: crate::repos::JobRepo,
    pub review_queue_repo: crate::repos::ReviewQueueRepo,
    /// Short-lived request dedupe cache only (`Idempotency-Key` header,
    /// `02_ARCHITECTURE.md §4`) — deliberately kept in-process rather than in
    /// Postgres: losing it on restart just means a duplicate request
    /// re-executes once, which is a non-event, not a data-durability bug.
    pub idempotency_cache: RwLock<HashMap<String, serde_json::Value>>,
    pub gemini_client: Arc<crate::ai::GeminiClient>,
    pub jwt: Arc<crate::auth::JwtService>,
    pub media_manifest: RwLock<HashMap<String, MediaAsset>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaAsset {
    pub opus_path: String,
    pub mp3_path: String,
    pub duration_sec: f64,
}

pub type SharedState = Arc<AppState>;
