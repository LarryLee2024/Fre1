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
#[derive(Clone, Reflect)]
pub struct SkillPanelVm {
    /// 技能数据映射（skill_id → SkillSlotVm）
    pub skills: HashMap<u32, SkillSlotVm>,
}

impl Default for SkillPanelVm {
    /// 默认提供三个示例技能，使 UI 在首次投影前有显示内容。
    /// 进入战斗后，投影函数（on_effect_applied、on_turn_started_for_skills）
    /// 会更新冷却状态和可用性。
    fn default() -> Self {
        let mut skills = HashMap::new();
        skills.insert(
            1,
            SkillSlotVm {
                skill_id: 1,
                name_key: "ui.skill.attack",
                cooldown_remaining: 0,
                max_cooldown: 0,
                is_usable: true,
                ap_cost: 1,
            },
        );
        skills.insert(
            2,
            SkillSlotVm {
                skill_id: 2,
                name_key: "ui.skill.fireball",
                cooldown_remaining: 0,
                max_cooldown: 3,
                is_usable: true,
                ap_cost: 2,
            },
        );
        skills.insert(
            3,
            SkillSlotVm {
                skill_id: 3,
                name_key: "ui.skill.heal",
                cooldown_remaining: 0,
                max_cooldown: 2,
                is_usable: true,
                ap_cost: 1,
            },
        );
        Self { skills }
    }
}
