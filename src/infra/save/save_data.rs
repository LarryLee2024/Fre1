use serde::{Deserialize, Serialize};

use super::resources::PersistentEntityId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldSaveData {
    pub save_version: u32,
    pub metadata: SaveMetadataData,
    pub combat: CombatSaveData,
    pub party: PartySaveData,
    pub progression: ProgressionSaveData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveMetadataData {
    pub label: String,
    pub location: String,
    pub playtime_seconds: u64,
    pub player_level: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatSaveData {
    pub phase: String,
    pub round_number: u32,
    pub current_index: usize,
    pub participants: Vec<CombatEntityData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatEntityData {
    pub persistent_id: u64,
    pub team_id: String,
    pub initiative: u32,
    pub is_dead: bool,
    pub action_points: Option<ActionPointsData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionPointsData {
    pub standard_action: bool,
    pub bonus_action: bool,
    pub reaction: bool,
    pub movement: f32,
    pub max_movement: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartySaveData {
    pub formation: String,
    pub max_active: u32,
    pub max_total: u32,
    pub active_members: Vec<PartyMemberSaveData>,
    pub reserve_members: Vec<u64>,
    pub active_bonds: Vec<ActiveBondSaveData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartyMemberSaveData {
    pub persistent_id: u64,
    pub slot_index: u32,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveBondSaveData {
    pub bond_id: String,
    pub level: u32,
    pub participant_ids: Vec<u64>,
    pub accumulated_battles: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressionSaveData {
    pub entities: Vec<ProgressionEntityData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressionEntityData {
    pub persistent_id: u64,
    pub experience: ExperienceData,
    pub class_levels: ClassLevelsData,
    pub talent_tree: TalentTreeData,
    pub subclass_choices: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperienceData {
    pub current_xp: u64,
    pub level: u32,
    pub total_xp_earned: u64,
    pub is_max_level: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassLevelsData {
    pub entries: Vec<ClassLevelEntryData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassLevelEntryData {
    pub class_id: String,
    pub level: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TalentTreeData {
    pub unlocked_talents: Vec<String>,
    pub available_points: u32,
}
