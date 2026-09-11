use crate::db::{DbError, PostgresClient};
use crate::state::{BackgroundJob, FlaggedSessionReview, SessionState};
use serde_json::Value;
use shared_engine::models::*;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Default)]
pub struct SeedBank {
    pub ls_items: Vec<ObjectiveItem>,
    pub rd_stimuli: Vec<ReadingStimulus>,
    pub rd_items: Vec<ObjectiveItem>,
    pub lsn_stimuli: Vec<ListeningStimulusAdmin>,
    pub lsn_items: Vec<ObjectiveItem>,
    pub restricted_keys: HashMap<String, RestrictedKey>,
    pub speaking_tasks: Vec<SpeakingTask>,
    pub writing_tasks: Vec<WritingTask>,
    pub enemy_groups: Vec<Value>,
}

impl SeedBank {
    pub fn load_from_dir<P: AsRef<Path>>(seed_dir: P) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let dir = seed_dir.as_ref();

        // 1. Language Systems
        let ls_data = fs::read_to_string(dir.join("language_systems.json"))?;
        let ls_json: Value = serde_json::from_str(&ls_data)?;
        let ls_items: Vec<ObjectiveItem> = serde_json::from_value(ls_json["items"].clone())?;

        // 2. Reading
        let rd_data = fs::read_to_string(dir.join("reading.json"))?;
        let rd_json: Value = serde_json::from_str(&rd_data)?;
        let rd_stimuli: Vec<ReadingStimulus> = serde_json::from_value(rd_json["stimuli"].clone())?;
        let rd_items: Vec<ObjectiveItem> = serde_json::from_value(rd_json["items"].clone())?;

        // 3. Listening
        let lsn_data = fs::read_to_string(dir.join("listening.json"))?;
        let lsn_json: Value = serde_json::from_str(&lsn_data)?;
        let lsn_stimuli: Vec<ListeningStimulusAdmin> = serde_json::from_value(lsn_json["stimuli"].clone())?;
        let lsn_items: Vec<ObjectiveItem> = serde_json::from_value(lsn_json["items"].clone())?;

        // 4. Restricted Keys
        let keys_data = fs::read_to_string(dir.join("RESTRICTED_answer_keys.json"))?;
        let keys_json: Value = serde_json::from_str(&keys_data)?;
        let keys_map: HashMap<String, KeyDetail> = serde_json::from_value(keys_json["keys"].clone())?;
        let mut restricted_keys = HashMap::new();
        for (item_id, detail) in keys_map {
            restricted_keys.insert(
                item_id.clone(),
                RestrictedKey {
                    item_id,
                    module: detail.module,
                    band: detail.band,
                    key_option_id: detail.key_option_id,
                    authoring_letter: detail.authoring_letter,
                    answer_text: detail.answer_text,
                    option_count: detail.option_count,
                    evidence_focus: detail.evidence_focus,
                    rationale: detail.rationale,
                },
            );
        }

        // 5. Speaking
        let spk_data = fs::read_to_string(dir.join("speaking_tasks.json"))?;
        let spk_json: Value = serde_json::from_str(&spk_data)?;
        let speaking_tasks: Vec<SpeakingTask> = serde_json::from_value(spk_json["tasks"].clone())?;

        // 6. Writing
        let wrt_data = fs::read_to_string(dir.join("writing_tasks.json"))?;
        let wrt_json: Value = serde_json::from_str(&wrt_data)?;
        let writing_tasks: Vec<WritingTask> = serde_json::from_value(wrt_json["tasks"].clone())?;

        // 7. Enemy groups
        let eg_data = fs::read_to_string(dir.join("enemy_groups.json"))?;
        let eg_json: Value = serde_json::from_str(&eg_data)?;
        let enemy_groups = eg_json["enemy_groups"].as_array().cloned().unwrap_or_default();

        Ok(Self {
            ls_items,
            rd_stimuli,
            rd_items,
            lsn_stimuli,
            lsn_items,
            restricted_keys,
            speaking_tasks,
            writing_tasks,
            enemy_groups,
        })
    }
}

fn decode_err(e: serde_json::Error) -> DbError {
    DbError::Query(format!("json decode: {e}"))
}

/// A session row plus the `version` counter needed to write it back with an
/// optimistic-concurrency precondition.
pub struct FetchedSession {
    pub session: SessionState,
    pub version: i64,
}

/// Postgres-backed `sessions` table (DECISIONS.md D-021). Replaces the old
/// in-process `RwLock<HashMap<String, SessionState>>` — every call
/// round-trips to Postgres, so session state genuinely survives a server
/// restart. The whole `SessionState` is stored as one JSONB `data` column;
/// see `db.rs` for why (keeps this repo's shape stable across the earlier
/// Firestore design and this one).
#[derive(Clone)]
pub struct SessionRepo {
    db: Arc<PostgresClient>,
}

impl SessionRepo {
    pub fn new(db: Arc<PostgresClient>) -> Self {
        Self { db }
    }

    pub async fn create(&self, session: &SessionState) -> Result<(), DbError> {
        let value = serde_json::to_value(session).map_err(decode_err)?;
        self.db.create("sessions", &session.session_id, &value).await
    }

    pub async fn get(&self, id: &str) -> Result<Option<FetchedSession>, DbError> {
        let fetched = match self.db.get("sessions", id).await? {
            Some(f) => f,
            None => return Ok(None),
        };
        let session: SessionState = serde_json::from_value(fetched.data).map_err(decode_err)?;
        Ok(Some(FetchedSession { session, version: fetched.version }))
    }

    /// Overwrite the whole row. Pass `expected_version` (from a prior
    /// `get`) to make this a compare-and-swap — `Err(Conflict)` means
    /// another request modified the session in between; the caller should
    /// re-fetch and retry (see `services.rs`'s helper `load_session`).
    pub async fn set(&self, session: &SessionState, expected_version: i64) -> Result<(), DbError> {
        let value = serde_json::to_value(session).map_err(decode_err)?;
        self.db.set("sessions", &session.session_id, &value, expected_version).await
    }

    pub async fn delete(&self, id: &str) -> Result<bool, DbError> {
        self.db.delete("sessions", id).await
    }

    /// Every session. Used only by admin reports and the retention job,
    /// which genuinely need to see every row anyway.
    /// # ponytail: full-table scan, fine at pilot volume — add a real
    /// index/query if session counts grow past the tens of thousands.
    pub async fn list_all(&self) -> Result<Vec<SessionState>, DbError> {
        let rows = self.db.list("sessions").await?;
        Ok(rows.into_iter().filter_map(|(_, v)| serde_json::from_value(v).ok()).collect())
    }
}

/// Postgres-backed `jobs` table (background rating-job audit trail,
/// `02_ARCHITECTURE.md §5`).
#[derive(Clone)]
pub struct JobRepo {
    db: Arc<PostgresClient>,
}

impl JobRepo {
    pub fn new(db: Arc<PostgresClient>) -> Self {
        Self { db }
    }

    pub async fn create(&self, job: &BackgroundJob) -> Result<(), DbError> {
        let value = serde_json::to_value(job).map_err(decode_err)?;
        self.db.create("jobs", &job.id, &value).await
    }

    pub async fn list_all(&self) -> Result<Vec<BackgroundJob>, DbError> {
        let rows = self.db.list("jobs").await?;
        Ok(rows.into_iter().filter_map(|(_, v)| serde_json::from_value(v).ok()).collect())
    }
}

/// Postgres-backed reviewer flag queue (each row a `FlaggedSessionReview`,
/// keyed by a server-generated id).
#[derive(Clone)]
pub struct ReviewQueueRepo {
    db: Arc<PostgresClient>,
}

impl ReviewQueueRepo {
    pub fn new(db: Arc<PostgresClient>) -> Self {
        Self { db }
    }

    pub async fn push(&self, item: &FlaggedSessionReview) -> Result<(), DbError> {
        let id = format!("rq_{}", &Uuid::new_v4().to_string().replace('-', "")[..16]);
        let value = serde_json::to_value(item).map_err(decode_err)?;
        self.db.create("review_queue", &id, &value).await
    }

    pub async fn list_all(&self) -> Result<Vec<FlaggedSessionReview>, DbError> {
        let rows = self.db.list("review_queue").await?;
        Ok(rows.into_iter().filter_map(|(_, v)| serde_json::from_value(v).ok()).collect())
    }

    /// Dedupe check used before pushing a `profile_inconsistency` flag twice
    /// for the same session (mirrors the old `Vec::iter().any(...)` check).
    pub async fn exists(&self, session_id: &str, flag_type: &str) -> Result<bool, DbError> {
        let all = self.list_all().await?;
        Ok(all.iter().any(|item| item.session_id == session_id && item.flag_type == flag_type))
    }
}
