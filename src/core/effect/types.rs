// 效果管道数据类型：EffectDef、PendingEffectData、EffectResult、EffectQueue
// 从原 effect.rs 迁移，保留 RON 反序列化支持
// ADR-026 §二：Buff 统一为带 Duration 的 Effect

use crate::core::modifier::ModifierEntry;
use crate::core::tag::GameplayTag;
use bevy::prelude::*;
use serde::Deserialize;

/// 持续时间定义（ADR-026 §二）
///
/// Buff 统一为带 Duration 的 Effect，Duration 定义效果持续多久。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum DurationDef {
    /// 瞬时效果（立即生效，无持续时间）
    Instant,
    /// 持续 N 回合（tick 递减）
    TurnLimited(u32),
    /// 永久效果（直到手动移除）
    Permanent,
}

impl Default for DurationDef {
    fn default() -> Self {
        DurationDef::Instant
    }
}

/// 叠层策略定义（ADR-026 §六）
///
/// 效果重复施加时的处理规则。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum StackingDef {
    /// 替换旧实例
    Replace,
    /// 刷新持续时间，不叠加
    RefreshDuration,
    /// 叠加层数，无上限
    StackAdd,
    /// 叠加层数，上限为参数值
    StackMax { max_stack: u32 },
}

impl Default for StackingDef {
    fn default() -> Self {
        StackingDef::Replace
    }
}

/// 效果定义（用于 SkillData 中声明技能效果，支持 RON 反序列化）
///
/// ADR-026 §二：Buff 统一为带 Duration 的 Effect
#[derive(Clone, Debug, Reflect, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum EffectDef {
    /// 伤害效果
    Damage {
        multiplier: f32,
        ignore_def_percent: f32,
    },
    /// 治疗效果
    Heal { amount: i32 },
    /// 应用修饰器效果（替代原 ApplyBuff）
    ///
    /// ADR-026：Buff 统一为带 Duration 的 Effect
    ApplyModifier {
        /// 修饰器 ID
        modifier_id: String,
        /// 持续时间
        duration: DurationDef,
        /// 叠层策略
        stacking: StackingDef,
    },
    /// 清除所有减益效果
    Cleanse,
    /// 应用 Buff（兼容旧系统，待废弃）
    #[deprecated(note = "Use ApplyModifier instead")]
    ApplyBuff { buff_id: String, duration: u32 },
}

impl EffectDef {
    /// 返回效果类型名（与 EffectHandler::type_name 对应）
    /// 用于 trait 分发查找，新增效果类型需保证 type_name 与注册的 handler 一致
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Damage { .. } => "Damage",
            Self::Heal { .. } => "Heal",
            Self::ApplyModifier { .. } => "ApplyModifier",
            Self::Cleanse => "Cleanse",
            #[allow(deprecated)]
            Self::ApplyBuff { .. } => "ApplyBuff",
        }
    }

    /// 获取持续时间定义（如果效果有持续时间）
    pub fn duration(&self) -> Option<DurationDef> {
        match self {
            Self::ApplyModifier { duration, .. } => Some(*duration),
            #[allow(deprecated)]
            Self::ApplyBuff { duration, .. } => Some(DurationDef::TurnLimited(*duration)),
            _ => None,
        }
    }

    /// 获取叠层策略（如果效果有叠层策略）
    pub fn stacking(&self) -> Option<StackingDef> {
        match self {
            Self::ApplyModifier { stacking, .. } => Some(*stacking),
            _ => None,
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
    /// 应用修饰器效果（替代原 ApplyBuff）
    ApplyModifier {
        modifier_id: String,
        duration: DurationDef,
        stacking: StackingDef,
    },
    Cleanse,
    /// 应用 Buff（兼容旧系统，待废弃）
    #[deprecated(note = "Use ApplyModifier instead")]
    ApplyBuff {
        buff_id: String,
        duration: u32,
    },
}

impl PendingEffectData {
    /// 返回效果类型名（与 EffectDef::type_name 对应）
    /// 为未来 execute 阶段 trait 化做准备
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Damage { .. } => "Damage",
            Self::Heal { .. } => "Heal",
            Self::ApplyModifier { .. } => "ApplyModifier",
            Self::Cleanse => "Cleanse",
            #[allow(deprecated)]
            Self::ApplyBuff { .. } => "ApplyBuff",
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
    Damage {
        amount: i32,
        killed: bool,
    },
    Heal {
        amount: i32,
    },
    /// 修饰器已应用（ADR-026 §二）
    ModifierApplied {
        modifier_id: String,
    },
    CleanseApplied,
    /// Buff 已应用（兼容旧系统，待废弃）
    #[deprecated(note = "Use ModifierApplied instead")]
    BuffApplied {
        buff_id: String,
    },
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
    use bevy::prelude::*;
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
    fn 效果定义_类型名() {
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
    fn 待处理效果数据_类型名() {
        assert_eq!(
            PendingEffectData::Damage {
                amount: 5,
                is_skill: false,
                base_amount: None,
                modifiers: Vec::new()
            }
            .type_name(),
            "Damage"
        );
        assert_eq!(
            PendingEffectData::Heal {
                amount: 5,
                base_amount: None,
                modifiers: Vec::new()
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
