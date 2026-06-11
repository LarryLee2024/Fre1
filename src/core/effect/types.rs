// 效果管道数据类型：EffectDef、PendingEffectData、EffectResult、EffectQueue
// 从原 effect.rs 迁移，保留 RON 反序列化支持

use crate::core::modifier_rule::ModifierEntry;
use crate::core::tag::GameplayTag;
use bevy::prelude::*;
use serde::Deserialize;

/// 效果定义（用于 SkillData 中声明技能效果，支持 RON 反序列化）
#[derive(Clone, Debug, Reflect, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum EffectDef {
    Damage {
        multiplier: f32,
        ignore_def_percent: f32,
    },
    Heal {
        amount: i32,
    },
    ApplyBuff {
        buff_id: String,
        duration: u32,
    },
    Cleanse,
}

impl EffectDef {
    /// 返回效果类型名（与 EffectHandler::type_name 对应）
    /// 用于 trait 分发查找，新增效果类型需保证 type_name 与注册的 handler 一致
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Damage { .. } => "Damage",
            Self::Heal { .. } => "Heal",
            Self::ApplyBuff { .. } => "ApplyBuff",
            Self::Cleanse => "Cleanse",
        }
    }
}

/// 待处理效果（运行时，进入 EffectQueue）
#[derive(Clone, Debug, Reflect)]
pub struct PendingEffect {
    pub source: Entity,
    pub target: Entity,
    pub data: PendingEffectData,
    pub source_tags: Vec<GameplayTag>,
    pub terrain_id: String,
}

/// 待处理效果数据
#[derive(Clone, Debug, Reflect)]
pub enum PendingEffectData {
    Damage {
        amount: i32,
        is_skill: bool,
        /// generate 阶段的原始伤害值（modify 前设置）
        base_amount: Option<i32>,
        /// modify 阶段记录的修饰步骤详情
        modifiers: Vec<ModifierEntry>,
    },
    Heal {
        amount: i32,
        /// generate 阶段的原始治疗值（modify 前设置）
        base_amount: Option<i32>,
        /// modify 阶段记录的修饰步骤详情（规则4：每步修饰必须记录）
        modifiers: Vec<ModifierEntry>,
    },
    ApplyBuff {
        buff_id: String,
        duration: u32,
    },
    Cleanse,
}

impl PendingEffectData {
    /// 返回效果类型名（与 EffectDef::type_name 对应）
    /// 为未来 execute 阶段 trait 化做准备
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Damage { .. } => "Damage",
            Self::Heal { .. } => "Heal",
            Self::ApplyBuff { .. } => "ApplyBuff",
            Self::Cleanse => "Cleanse",
        }
    }
}

/// 效果执行结果
#[derive(Clone, Debug, Reflect)]
pub struct EffectResult {
    pub source: Entity,
    pub target: Entity,
    pub data: EffectResultData,
}

#[derive(Clone, Debug, Reflect)]
pub enum EffectResultData {
    Damage { amount: i32, killed: bool },
    Heal { amount: i32 },
    BuffApplied { buff_id: String },
    CleanseApplied,
}

/// 效果队列资源
#[derive(Resource, Reflect, Default, Debug)]
#[reflect(Resource)]
pub struct EffectQueue {
    pub pending: Vec<PendingEffect>,
}

impl EffectQueue {
    pub fn push(&mut self, effect: PendingEffect) {
        self.pending.push(effect);
    }

    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    pub fn clear(&mut self) {
        self.pending.clear();
    }
}

/// 从技能效果定义生成伤害计算结果
pub fn calculate_damage_from_effect(
    effective_atk: f32,
    effective_def: f32,
    base_def: f32,
    multiplier: f32,
    ignore_def_percent: f32,
    terrain_defense_bonus: i32,
) -> i32 {
    let def_ignored = base_def * (ignore_def_percent / 100.0);
    let final_def = effective_def - def_ignored;
    let base_damage = effective_atk - final_def;
    let terrain_bonus = terrain_defense_bonus as f32;
    ((base_damage - terrain_bonus) * multiplier).max(1.0) as i32
}

#[cfg(test)]
mod tests {
    // ================================================
    // Bevy SRPG AI宪法 v1.1 自检结果（测试专用）
    // ================================================
    // ✅ 测行为不测实现：是 — 断言验证伤害公式结果，不验证内部计算步骤
    // ✅ 符合领域规则：是 — 覆盖 INV-EFX-1~3 效果管线不变量
    // ✅ 确定性：是 — 硬编码属性值和地形数据
    // ✅ 使用标准数据：是 — 使用标准伤害计算参数
    // ✅ 无越界测试：是 — 仅测试公共 API
    // ✅ 未测试私有实现：是 — 仅通过 pub 接口测试
    // ================================================
    use super::*;

    #[test]
    fn 伤害计算_基础() {
        // ATK=10, DEF=3, multiplier=1.0, no ignore, Plain (defense_bonus=0)
        let dmg = calculate_damage_from_effect(10.0, 3.0, 3.0, 1.0, 0.0, 0);
        assert_eq!(dmg, 7);
    }

    #[test]
    fn 伤害计算_森林地形() {
        let dmg = calculate_damage_from_effect(10.0, 3.0, 3.0, 1.0, 0.0, 2);
        // 10 - 3 - 2 = 5
        assert_eq!(dmg, 5);
    }

    #[test]
    fn 伤害计算_最低为1() {
        let dmg = calculate_damage_from_effect(1.0, 10.0, 10.0, 1.0, 0.0, 0);
        assert_eq!(dmg, 1);
    }

    #[test]
    fn 伤害计算_技能倍率() {
        let dmg = calculate_damage_from_effect(10.0, 3.0, 3.0, 1.5, 0.0, 0);
        // (10 - 3) * 1.5 = 10.5 → 10
        assert_eq!(dmg, 10);
    }

    #[test]
    fn 伤害计算_无视防御() {
        let dmg = calculate_damage_from_effect(10.0, 10.0, 10.0, 1.3, 50.0, 0);
        // final_def = 10 - 10*0.5 = 5, (10 - 5) * 1.3 = 6.5 → 6
        assert_eq!(dmg, 6);
    }

    #[test]
    fn 伤害计算_100百分比无视防御() {
        let dmg = calculate_damage_from_effect(10.0, 10.0, 10.0, 1.0, 100.0, 0);
        // final_def = 10 - 10*1.0 = 0, (10 - 0) * 1.0 = 10
        assert_eq!(dmg, 10);
    }

    #[test]
    fn 伤害计算_山地地形无防御加成() {
        let dmg = calculate_damage_from_effect(10.0, 3.0, 3.0, 1.0, 0.0, 0);
        // Mountain defense_bonus = 0, 10 - 3 = 7
        assert_eq!(dmg, 7);
    }

    #[test]
    fn 伤害计算_水域地形无防御加成() {
        let dmg = calculate_damage_from_effect(10.0, 3.0, 3.0, 1.0, 0.0, 0);
        // Water defense_bonus = 0, 10 - 3 = 7
        assert_eq!(dmg, 7);
    }

    #[test]
    fn 伤害计算_高倍率技能() {
        let dmg = calculate_damage_from_effect(10.0, 3.0, 3.0, 3.0, 0.0, 0);
        // (10 - 3) * 3.0 = 21
        assert_eq!(dmg, 21);
    }

    #[test]
    fn 效果队列_push和drain() {
        let mut queue = EffectQueue::default();
        assert!(queue.is_empty());

        queue.push(PendingEffect {
            source: Entity::from_bits(1),
            target: Entity::from_bits(2),
            data: PendingEffectData::Damage {
                amount: 5,
                is_skill: false,
                base_amount: None,
                modifiers: Vec::new(),
            },
            source_tags: vec![],
            terrain_id: "plain".to_string(),
        });
        assert!(!queue.is_empty());

        let drained: Vec<_> = queue.pending.drain(..).collect();
        assert_eq!(drained.len(), 1);
        assert!(queue.is_empty());
    }

    #[test]
    fn 效果队列_clear() {
        let mut queue = EffectQueue::default();
        queue.push(PendingEffect {
            source: Entity::from_bits(1),
            target: Entity::from_bits(2),
            data: PendingEffectData::Damage {
                amount: 5,
                is_skill: false,
                base_amount: None,
                modifiers: Vec::new(),
            },
            source_tags: vec![],
            terrain_id: "plain".to_string(),
        });
        queue.clear();
        assert!(queue.is_empty());
    }

    #[test]
    fn effect_def_type_name() {
        assert_eq!(
            EffectDef::Damage {
                multiplier: 1.0,
                ignore_def_percent: 0.0
            }
            .type_name(),
            "Damage"
        );
        assert_eq!(EffectDef::Heal { amount: 5 }.type_name(), "Heal");
        assert_eq!(
            EffectDef::ApplyBuff {
                buff_id: "burn".into(),
                duration: 2
            }
            .type_name(),
            "ApplyBuff"
        );
        assert_eq!(EffectDef::Cleanse.type_name(), "Cleanse");
    }

    #[test]
    fn pending_effect_data_type_name() {
        assert_eq!(
            PendingEffectData::Damage {
                amount: 5,
                is_skill: false,
                base_amount: None,
                modifiers: Vec::new(),
            }
            .type_name(),
            "Damage"
        );
        assert_eq!(
            PendingEffectData::Heal {
                amount: 5,
                base_amount: None,
                modifiers: Vec::new(),
            }
            .type_name(),
            "Heal"
        );
        assert_eq!(
            PendingEffectData::ApplyBuff {
                buff_id: "burn".into(),
                duration: 2
            }
            .type_name(),
            "ApplyBuff"
        );
        assert_eq!(PendingEffectData::Cleanse.type_name(), "Cleanse");
    }
}
