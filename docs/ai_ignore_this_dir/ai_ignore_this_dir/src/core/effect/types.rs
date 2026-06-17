// 效果管道数据类型：EffectDef、PendingEffectData、EffectResult、EffectQueue
// ADR-026 §二：Buff 统一为带 Duration 的 Effect
// DEPRECATED: ApplyBuff 已废弃，请使用 ApplyModifier

use crate::core::modifier::ModifierEntry;
use crate::core::stacking::StackingRule;
use crate::core::tag::GameplayTag;
use bevy::prelude::*;
use serde::Deserialize;

/// 持续时间定义（ADR-026 §二）
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
/// 使用 stacking::StackingRule 作为运行时类型，此为 RON 反序列化用
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

impl From<StackingDef> for StackingRule {
    fn from(def: StackingDef) -> Self {
        match def {
            StackingDef::Replace => StackingRule::Replace,
            StackingDef::RefreshDuration => StackingRule::RefreshDuration,
            StackingDef::StackAdd => StackingRule::StackAdd,
            StackingDef::StackMax { max_stack } => StackingRule::StackMax(max_stack),
        }
    }
}

/// 效果定义（ADR-026 §二：Buff 统一为带 Duration 的 Effect）
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
    /// 应用修饰器效果（ADR-026 推荐）
    ApplyModifier {
        modifier_id: String,
        duration: DurationDef,
        stacking: StackingDef,
    },
    /// 清除所有减益效果
    Cleanse,
}

impl EffectDef {
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Damage { .. } => "Damage",
            Self::Heal { .. } => "Heal",
            Self::ApplyModifier { .. } => "ApplyModifier",
            Self::Cleanse => "Cleanse",
        }
    }

    pub fn duration(&self) -> Option<DurationDef> {
        match self {
            Self::ApplyModifier { duration, .. } => Some(*duration),
            _ => None,
        }
    }

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
        base_amount: Option<i32>,
        modifiers: Vec<ModifierEntry>,
    },
    Heal {
        amount: i32,
        base_amount: Option<i32>,
        modifiers: Vec<ModifierEntry>,
    },
    /// 应用修饰器效果（ADR-026 推荐）
    ApplyModifier {
        modifier_id: String,
        duration: DurationDef,
        stacking: StackingDef,
    },
    Cleanse,
}

impl PendingEffectData {
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Damage { .. } => "Damage",
            Self::Heal { .. } => "Heal",
            Self::ApplyModifier { .. } => "ApplyModifier",
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

/// 伤害计算辅助函数（兼容旧系统）
/// ADR-026：新代码应使用 Execution 模块进行数值计算
#[deprecated(note = "Use Execution module instead (ADR-026)")]
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
    use super::*;

    #[test]
    fn 持续时间_默认为瞬时() {
        assert_eq!(DurationDef::default(), DurationDef::Instant);
    }

    #[test]
    fn 叠层策略_默认为替换() {
        assert_eq!(StackingDef::default(), StackingDef::Replace);
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
            EffectDef::ApplyModifier {
                modifier_id: "burn".into(),
                duration: DurationDef::TurnLimited(2),
                stacking: StackingDef::StackAdd,
            }
            .type_name(),
            "ApplyModifier"
        );
        assert_eq!(EffectDef::Cleanse.type_name(), "Cleanse");
    }

    #[test]
    fn 叠层定义转换() {
        let def = StackingDef::StackMax { max_stack: 5 };
        let rule: StackingRule = def.into();
        assert_eq!(rule, StackingRule::StackMax(5));
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
}
