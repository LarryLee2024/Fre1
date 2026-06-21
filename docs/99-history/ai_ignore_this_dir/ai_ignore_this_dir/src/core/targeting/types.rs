use bevy::prelude::*;
use serde::Deserialize;

/// 技能目标类型：决定技能可以作用于谁
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum SkillTargeting {
    /// 对单个敌方单位使用
    SingleEnemy,
    /// 对单个友方单位使用
    SingleAlly,
    /// 对自身使用
    SelfOnly,
    /// 对自身周围的敌方单位使用（范围由 range 决定）
    AoeEnemies,
    /// 对自身周围的友方单位使用
    AoeAllies,
    /// 无需目标（直接对自身生效）
    NoTarget,
}

impl SkillTargeting {
    /// 技能目标 i18n key
    pub fn i18n_key(&self) -> &'static str {
        match self {
            Self::SingleEnemy => "targeting.single_enemy",
            Self::SingleAlly => "targeting.single_ally",
            Self::SelfOnly => "targeting.self_only",
            Self::AoeEnemies => "targeting.aoe_enemies",
            Self::AoeAllies => "targeting.aoe_allies",
            Self::NoTarget => "targeting.no_target",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::SingleEnemy => "单体敌方",
            Self::SingleAlly => "单体友方",
            Self::SelfOnly => "自身",
            Self::AoeEnemies => "范围敌方",
            Self::AoeAllies => "范围友方",
            Self::NoTarget => "无目标",
        }
    }

    /// 是否需要选择目标
    pub fn requires_target_selection(&self) -> bool {
        matches!(self, Self::SingleEnemy | Self::SingleAlly)
    }
}

/// 目标选择上下文：封装一次目标选择的所有信息
#[derive(Debug, Clone)]
pub struct TargetingContext {
    pub caster: Entity,
    pub ability_id: String,
    pub targeting_type: SkillTargeting,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn targeting_label() {
        assert_eq!(SkillTargeting::SingleEnemy.label(), "单体敌方");
        assert_eq!(SkillTargeting::SingleAlly.label(), "单体友方");
        assert_eq!(SkillTargeting::SelfOnly.label(), "自身");
        assert_eq!(SkillTargeting::AoeEnemies.label(), "范围敌方");
        assert_eq!(SkillTargeting::AoeAllies.label(), "范围友方");
        assert_eq!(SkillTargeting::NoTarget.label(), "无目标");
    }

    #[test]
    fn targeting_需要目标选择() {
        assert!(SkillTargeting::SingleEnemy.requires_target_selection());
        assert!(SkillTargeting::SingleAlly.requires_target_selection());
        assert!(!SkillTargeting::SelfOnly.requires_target_selection());
        assert!(!SkillTargeting::AoeEnemies.requires_target_selection());
        assert!(!SkillTargeting::AoeAllies.requires_target_selection());
        assert!(!SkillTargeting::NoTarget.requires_target_selection());
    }
}
