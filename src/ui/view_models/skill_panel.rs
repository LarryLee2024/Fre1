//! SkillPanelVm — 技能面板视图模型
//!
//! 详见 `docs/06-ui/04-data-flow/projection-viewmodel.md` §3

use std::collections::HashMap;

use bevy::prelude::*;

/// 技能槽位视图模型
#[derive(Clone, Reflect, Default)]
pub struct SkillSlotVm {
    /// 技能 ID
    pub skill_id: u32,
    /// 技能名称（本地化 Key）
    pub name_key: &'static str,
    /// 剩余冷却回合数
    pub cooldown_remaining: u32,
    /// 最大冷却回合数
    pub max_cooldown: u32,
    /// 是否可用
    pub is_usable: bool,
    /// AP 消耗
    pub ap_cost: u32,
}

/// 技能面板视图模型
#[derive(Clone, Reflect, Default)]
pub struct SkillPanelVm {
    /// 技能数据映射（skill_id → SkillSlotVm）
    pub skills: HashMap<u32, SkillSlotVm>,
}
