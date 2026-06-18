//! ECS Components — 成长养成领域组件
//!
//! 定义经验、等级、天赋树、子职选择等 ECS 组件。
//! 详见 docs/02-domain/domains/progression_domain.md
//! 详见 docs/04-data/domains/progression_schema.md

use std::collections::HashMap;

use bevy::prelude::*;

// ─── ID 类型 ──────────────────────────────────────────────────────

/// 职业标识符。
///
/// 使用 String，内容系统接入后可通过 Registry 的 DefinitionId 桥接。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect)]
pub struct ClassId(pub String);

impl ClassId {
    /// 从字符串创建 ClassId。
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// 返回内部字符串引用。
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ClassId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for ClassId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for ClassId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// 天赋标识符。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect)]
pub struct TalentId(pub String);

impl TalentId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for TalentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for TalentId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for TalentId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// 子职标识符。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect)]
pub struct SubclassId(pub String);

impl SubclassId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for SubclassId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for SubclassId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for SubclassId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

// ─── 辅助数据类型 ──────────────────────────────────────────────────

/// 职业等级条目（多职业用）。
#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct ClassLevelEntry {
    /// 职业 ID
    pub class_id: ClassId,
    /// 在该职业上的等级
    pub level: u32,
}

impl ClassLevelEntry {
    pub fn new(class_id: impl Into<ClassId>, level: u32) -> Self {
        Self {
            class_id: class_id.into(),
            level,
        }
    }
}

/// ASI 选择类型。
#[derive(Debug, Clone, PartialEq, Reflect)]
pub enum ASIChoice {
    /// 单一属性 +2
    SingleAttribute(String),
    /// 两项属性各 +1
    TwoAttributes(String, String),
    /// 选择一个专长
    Feat(String),
}

/// 经验获取来源。
#[derive(Debug, Clone, PartialEq, Reflect)]
pub enum XpSource {
    /// 战斗胜利
    Combat,
    /// 任务完成
    Quest,
    /// 探索发现
    Discovery,
    /// 对话奖励
    Dialogue,
    /// 特殊事件
    Special,
}

// ─── ECS Components ───────────────────────────────────────────────

/// 角色经验值组件。
///
/// 管理角色的当前经验值积累和等级状态。
/// 不变量 3.2：经验值只增不减，仅升级时自动扣除升级所需量。
#[derive(Component, Debug, Clone, PartialEq, Reflect)]
#[reflect(Component)]
pub struct Experience {
    /// 当前累积的经验值（用于判断升级）
    pub current_xp: u64,
    /// 角色总等级（1-20，不变量 3.1）
    pub level: u32,
    /// 一生获得的总经验值（只增不减）
    pub total_xp_earned: u64,
    /// 是否为满级（等级 >= 20）
    pub is_max_level: bool,
}

impl Experience {
    /// 创建初始经验组件（等级 1）。
    pub fn new() -> Self {
        Self {
            current_xp: 0,
            level: 1,
            total_xp_earned: 0,
            is_max_level: false,
        }
    }

    /// 检查是否可以升级（不变量 3.1：等级上限 20）。
    pub fn can_level_up(&self, xp_for_next: u64) -> bool {
        !self.is_max_level && self.current_xp >= xp_for_next
    }

    /// 增加经验值（不变量 3.2：只增不减）。
    ///
    /// 返回增加后的 total_xp_earned。
    pub fn add_xp(&mut self, amount: u64) -> u64 {
        if self.is_max_level {
            return self.total_xp_earned;
        }
        self.current_xp += amount;
        self.total_xp_earned += amount;
        self.total_xp_earned
    }

    /// 升级时消耗经验值。
    ///
    /// 扣除升级所需的经验值，等级 +1。
    /// 如果达到满级，设置 is_max_level = true。
    pub fn apply_level_up(&mut self, xp_cost: u64) {
        if self.is_max_level {
            return;
        }
        self.current_xp -= xp_cost;
        self.level += 1;
        if self.level >= 20 {
            self.is_max_level = true;
        }
    }
}

impl Default for Experience {
    fn default() -> Self {
        Self::new()
    }
}

/// 多职业等级组件。
///
/// 记录角色在各职业上的等级分布。
/// D&D 5e 多职业规则：总等级 = 各职业等级之和，熟练加值按总等级计算。
#[derive(Component, Debug, Clone, PartialEq, Reflect)]
#[reflect(Component)]
pub struct ClassLevels {
    /// 各职业的等级条目
    pub entries: Vec<ClassLevelEntry>,
}

impl ClassLevels {
    /// 创建带初始职业的等级数据。
    pub fn new(initial_class: impl Into<ClassId>) -> Self {
        Self {
            entries: vec![ClassLevelEntry::new(initial_class, 1)],
        }
    }

    /// 获取总等级（所有职业等级之和，不变量 3.1 <= 20）。
    pub fn total_level(&self) -> u32 {
        self.entries.iter().map(|e| e.level).sum()
    }

    /// 获取指定职业的等级（0 表示未选择该职业）。
    pub fn level_in_class(&self, class_id: &ClassId) -> u32 {
        self.entries
            .iter()
            .find(|e| e.class_id == *class_id)
            .map(|e| e.level)
            .unwrap_or(0)
    }

    /// 增加指定职业的等级（升级或开始新职业）。
    ///
    /// 如果角色已有该职业则等级 +1，否则添加新职业（等级 1）。
    pub fn advance_class(&mut self, class_id: ClassId) {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.class_id == class_id) {
            entry.level += 1;
        } else {
            self.entries.push(ClassLevelEntry::new(class_id, 1));
        }
    }

    /// 检查是否满足多职业前置条件（属性需求）。
    pub fn has_class(&self, class_id: &ClassId) -> bool {
        self.entries.iter().any(|e| e.class_id == *class_id)
    }
}

/// 天赋树组件。
///
/// 记录角色的天赋解锁状态。
/// 不变量 3.3：天赋前置链完整性——解锁前必须检查全部前置条件。
#[derive(Component, Debug, Clone, PartialEq, Reflect)]
#[reflect(Component)]
pub struct TalentTree {
    /// 已解锁的天赋 ID 列表
    pub unlocked_talents: Vec<TalentId>,
    /// 可用的天赋点数（尚未分配的）
    pub available_points: u32,
}

impl TalentTree {
    /// 创建空的天赋树。
    pub fn new() -> Self {
        Self {
            unlocked_talents: Vec::new(),
            available_points: 0,
        }
    }

    /// 解锁天赋（不变量 3.3：调用方必须确保前置条件已满足）。
    pub fn unlock(&mut self, talent_id: TalentId) {
        if !self.unlocked_talents.contains(&talent_id) {
            self.unlocked_talents.push(talent_id);
        }
    }

    /// 检查天赋是否已解锁。
    pub fn is_unlocked(&self, talent_id: &TalentId) -> bool {
        self.unlocked_talents.contains(talent_id)
    }

    /// 添加天赋点数。
    pub fn add_points(&mut self, points: u32) {
        self.available_points += points;
    }

    /// 消耗天赋点数。
    ///
    /// 返回是否成功消耗。
    pub fn spend_point(&mut self) -> bool {
        if self.available_points > 0 {
            self.available_points -= 1;
            true
        } else {
            false
        }
    }
}

impl Default for TalentTree {
    fn default() -> Self {
        Self::new()
    }
}

/// 子职选择组件。
///
/// 记录角色在各职业上选择的子职。
/// 不变量 3.4：同一职业只能选择一个子职，选择后不可更改。
#[derive(Component, Debug, Clone, PartialEq, Reflect)]
#[reflect(Component)]
pub struct SubclassChoice {
    /// 职业 ID → 子职 ID 映射
    pub choices: HashMap<ClassId, SubclassId>,
}

impl SubclassChoice {
    /// 创建空的子职选择记录。
    pub fn new() -> Self {
        Self {
            choices: HashMap::new(),
        }
    }

    /// 选择子职（不变量 3.4：已选择的职业不可更改）。
    ///
    /// 返回 `Ok(())` 或 `Err` 如果该职业已有子职。
    pub fn choose(&mut self, class_id: ClassId, subclass_id: SubclassId) -> Result<(), String> {
        if self.choices.contains_key(&class_id) {
            Err(format!(
                "Class {:?} already has subclass {:?}",
                class_id, self.choices[&class_id]
            ))
        } else {
            self.choices.insert(class_id, subclass_id);
            Ok(())
        }
    }

    /// 获取指定职业的子职。
    pub fn get(&self, class_id: &ClassId) -> Option<&SubclassId> {
        self.choices.get(class_id)
    }
}

impl Default for SubclassChoice {
    fn default() -> Self {
        Self::new()
    }
}

/// 成长系统标记组件。
///
/// 标记具有完整成长系统（经验/等级/天赋/子职）的实体。
/// 用于 System 查询过滤。
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
#[reflect(Component)]
pub struct ProgressionMarker;

// ─── 资源 ─────────────────────────────────────────────────────────

/// 等级成长配置表（Resource）。
///
/// 定义经验曲线、熟练加值和 ASI 时机。
/// 当前使用默认的 D&D 5e 经验表，内容系统接入后可替换为配置加载。
#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct LevelProgressionTable {
    /// 等级上限
    pub max_level: u32,
    /// 各等级所需的累计经验值阈值 [0..max_level-1]
    /// 索引 0 = 1→2 级所需经验, 索引 18 = 19→20 级所需经验
    pub exp_thresholds: Vec<u64>,
    /// 各等级的熟练加值 [0..max_level-1]
    pub proficiency_by_level: Vec<i32>,
    /// ASI 触发等级
    pub asi_levels: Vec<u32>,
}

impl Default for LevelProgressionTable {
    /// 创建默认的 D&D 5e 等级表。
    fn default() -> Self {
        Self {
            max_level: 20,
            // D&D 5e 累计经验阈值（索引 0 = 1 级 0 XP, 索引 1 = 2 级 300 XP, ...）
            exp_thresholds: vec![
                0, 300, 900, 2700, 6500, 14000, 23000, 34000, 48000, 64000, 85000, 100000, 120000,
                140000, 165000, 195000, 225000, 265000, 305000, 355000,
            ],
            // 各等级熟练加值（索引 0 = 1 级）
            proficiency_by_level: vec![2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 6, 6, 6, 6],
            // ASI 触发等级
            asi_levels: vec![4, 8, 12, 16, 19],
        }
    }
}

impl LevelProgressionTable {
    /// 获取指定等级所需的累计经验值。
    ///
    /// level=1 返回 0（起始值），level=2 返回 300（D&D 5e 标准）。
    /// 如果 level >= max_level，返回 u64::MAX（无需经验）。
    /// 如果 level == 0，返回 0。
    pub fn xp_for_level(&self, level: u32) -> u64 {
        if level >= self.max_level {
            return u64::MAX;
        }
        if level == 0 {
            return 0;
        }
        let idx = (level - 1) as usize;
        if idx < self.exp_thresholds.len() {
            self.exp_thresholds[idx]
        } else {
            u64::MAX
        }
    }

    /// 获取从当前等级升到下一级所需的经验值。
    ///
    /// 返回 (当前级所需, 下一级所需)。
    pub fn xp_range_for_level(&self, level: u32) -> (u64, u64) {
        let current = self.xp_for_level(level);
        let next = self.xp_for_level(level + 1);
        (current, next)
    }

    /// 获取从当前经验值计算应处的等级。
    ///
    /// 二分查找找到第一个超过 total_xp 的阈值。
    pub fn level_from_xp(&self, total_xp: u64) -> u32 {
        let mut level = 1u32;
        for (i, &threshold) in self.exp_thresholds.iter().enumerate() {
            if total_xp >= threshold {
                level = (i + 1) as u32;
            } else {
                break;
            }
        }
        level.min(self.max_level)
    }

    /// 获取指定等级的熟练加值。
    pub fn proficiency_bonus(&self, level: u32) -> i32 {
        let idx = (level as usize).saturating_sub(1);
        if idx < self.proficiency_by_level.len() {
            self.proficiency_by_level[idx]
        } else {
            self.proficiency_by_level[self.proficiency_by_level.len() - 1]
        }
    }

    /// 检查指定等级是否为 ASI 等级。
    pub fn is_asi_level(&self, level: u32) -> bool {
        self.asi_levels.contains(&level)
    }
}
