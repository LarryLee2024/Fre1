use serde::{Deserialize, Serialize};

use super::resources::PersistentEntityId;

/// 完整的世界存档数据——序列化到磁盘的顶层结构。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldSaveData {
    /// 存档格式版本号，用于迁移兼容性检查
    pub save_version: u32,
    /// 存档元数据（标签、位置、游玩时间等）
    pub metadata: SaveMetadataData,
    /// 战斗状态快照
    pub combat: CombatSaveData,
    /// 队伍状态快照
    pub party: PartySaveData,
    /// 角色成长状态快照
    pub progression: ProgressionSaveData,
}

/// 存档元数据——供 UI 显示和存档管理使用。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveMetadataData {
    /// 存档标签（如 "自动存档"、"手动存档 1"）
    pub label: String,
    /// 当前游戏位置描述（如 "遗忘森林 - 第 3 区域"）
    pub location: String,
    /// 累计游玩秒数
    pub playtime_seconds: u64,
    /// 队伍平均等级
    pub player_level: u32,
}

/// 战斗状态存档——记录战斗进行中的完整状态。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatSaveData {
    /// 战斗阶段（如 "Preparation"、"Battle"）
    pub phase: String,
    /// 当前回合数
    pub round_number: u32,
    /// 当前行动单位在参与者列表中的索引
    pub current_index: usize,
    /// 所有战斗参与者的快照
    pub participants: Vec<CombatEntityData>,
}

/// 单个战斗参与者的存档数据。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatEntityData {
    /// 持久化实体 ID（跨战斗唯一）
    pub persistent_id: u64,
    /// 队伍 ID
    pub team_id: String,
    /// 先攻值
    pub initiative: u32,
    /// 是否已死亡
    pub is_dead: bool,
    /// 行动点状态（战斗中才有）
    pub action_points: Option<ActionPointsData>,
}

/// 行动点状态存档。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionPointsData {
    /// 标准行动是否可用
    pub standard_action: bool,
    /// 附赠行动是否可用
    pub bonus_action: bool,
    /// 反应是否可用
    pub reaction: bool,
    /// 当前剩余移动力
    pub movement: f32,
    /// 最大移动力
    pub max_movement: f32,
}

/// 队伍状态存档。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartySaveData {
    /// 队伍阵型 ID
    pub formation: String,
    /// 最大出战人数
    pub max_active: u32,
    /// 最大总人数（出战 + 预备）
    pub max_total: u32,
    /// 出战成员列表
    pub active_members: Vec<PartyMemberSaveData>,
    /// 预备成员实体 ID 列表
    pub reserve_members: Vec<u64>,
    /// 已激活的羁绊列表
    pub active_bonds: Vec<ActiveBondSaveData>,
}

/// 单个队伍成员的存档数据。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartyMemberSaveData {
    /// 持久化实体 ID
    pub persistent_id: u64,
    /// 在队伍中的槽位索引
    pub slot_index: u32,
    /// 是否为出战成员（true = 出战，false = 预备）
    pub is_active: bool,
}

/// 已激活羁绊的存档数据。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveBondSaveData {
    /// 羁绊定义 ID
    pub bond_id: String,
    /// 羁绊等级
    pub level: u32,
    /// 参与该羁绊的实体持久化 ID 列表
    pub participant_ids: Vec<u64>,
    /// 累计共同战斗次数
    pub accumulated_battles: u32,
}

/// 角色成长系统存档。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressionSaveData {
    /// 所有角色的成长数据
    pub entities: Vec<ProgressionEntityData>,
}

/// 单个角色的成长状态存档。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressionEntityData {
    /// 持久化实体 ID
    pub persistent_id: u64,
    /// 经验值数据
    pub experience: ExperienceData,
    /// 职业等级数据
    pub class_levels: ClassLevelsData,
    /// 天赋树数据
    pub talent_tree: TalentTreeData,
    /// 子职选择（职业 ID → 子职 ID）
    pub subclass_choices: Vec<(String, String)>,
}

/// 经验值存档数据。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperienceData {
    /// 当前等级所需额外经验值
    pub current_xp: u64,
    /// 当前等级
    pub level: u32,
    /// 累计获得的总经验值
    pub total_xp_earned: u64,
    /// 是否已达等级上限
    pub is_max_level: bool,
}

/// 职业等级存档数据。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassLevelsData {
    /// 各职业的等级条目
    pub entries: Vec<ClassLevelEntryData>,
}

/// 单个职业等级条目。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassLevelEntryData {
    /// 职业 ID
    pub class_id: String,
    /// 该职业上的等级
    pub level: u32,
}

/// 天赋树存档数据。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TalentTreeData {
    /// 已解锁的天赋 ID 列表
    pub unlocked_talents: Vec<String>,
    /// 可用天赋点数
    pub available_points: u32,
}
