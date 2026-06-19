//! AbilityDef — 技能定义（Definition 层）
//!
//! 技能的完整静态定义，从 RON 配置文件反序列化。
//! 参考 UE GameplayAbility 的 Tag 字段设计。
//!
//! 详见 docs/04-data/capabilities/ability_schema.md。

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::shared::localization_key::LocalizationKey;

/// 技能定义（Definition 层）。
///
/// 包含技能的全部静态参数：分类、消耗、冷却、标签、关联效果。
/// 参考 UE GameplayAbility 的 Tag 字段设计。
#[derive(Debug, Clone, PartialEq, Asset, Serialize, Deserialize, TypePath)]
pub struct AbilityDef {
    /// 技能唯一标识（格式：abl_ + 6 位数字，如 abl_000001）
    pub id: String,

    /// 技能名称本地化 Key
    pub name_key: LocalizationKey,

    /// 技能描述本地化 Key
    pub desc_key: LocalizationKey,

    /// 技能图标 Key
    pub icon_key: Option<LocalizationKey>,

    /// 技能分类标签（如 "Ability.Type.Active", "Ability.School.Fire"）
    #[serde(default)]
    pub ability_tags: Vec<String>,

    /// 激活时取消具有这些标签的其他技能（如沉默打断施法）
    #[serde(default)]
    pub cancel_by_tags: Vec<String>,

    /// 激活期间阻断具有这些标签的其他技能（如引导期间禁止其他技能）
    #[serde(default)]
    pub block_by_tags: Vec<String>,

    /// 激活期间授予自身的标签（如施法期间获得 "Ability.State.Casting"）
    #[serde(default)]
    pub activation_owned_tags: Vec<String>,

    /// 关联的效果定义 ID 列表（技能释放时依次应用）
    #[serde(default)]
    pub effect_ids: Vec<String>,

    /// 冷却回合数
    pub cooldown_turns: u32,

    /// 共享冷却组名（同组技能共享冷却）
    pub shared_cooldown_group: Option<String>,

    /// 资源消耗（属性 ID → 消耗量）
    #[serde(default)]
    pub costs: Vec<AbilityCostEntry>,

    /// 最大等级
    pub max_level: u8,

    /// 是否为被动技能（不需要手动激活）
    pub passive: bool,

    /// 是否可被取消/打断
    pub interruptible: bool,

    /// 施法时间（帧数，0 = 瞬发）
    pub cast_time_frames: u64,

    /// 是否可见（被动技能通常隐藏）
    pub visible: bool,
}

/// 技能资源消耗条目。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AbilityCostEntry {
    /// 消耗的资源属性 ID
    pub resource: String,
    /// 消耗量（基础值，按等级缩放）
    pub amount: f32,
    /// 每级增量
    #[serde(default)]
    pub per_level: f32,
}
