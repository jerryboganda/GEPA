use serde_json::Value;
use shared_engine::models::*;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

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
