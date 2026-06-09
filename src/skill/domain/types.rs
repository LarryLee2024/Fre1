// 技能类型定义：SkillData, SkillCondition, SkillTargeting, SkillDef, SkillUseError

use crate::core::attribute::AttributeKind;
use crate::core::effect::EffectDef;
use crate::core::tag::{GameplayTag, TagName};
use serde::Deserialize;

/// 基础攻击技能 ID 常量
pub const BASIC_ATTACK_ID: &str = "basic_attack";

// ── 技能目标类型 ──

/// 技能目标类型：决定技能可以作用于谁
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
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

// ── 技能使用条件 ──

/// 技能使用条件（运行时）
#[derive(Clone, Debug, PartialEq)]
pub enum SkillCondition {
    /// 需要足够的 MP
    MpCost(i32),
    /// 需要拥有指定标签
    RequireTag(GameplayTag),
    /// 需要目标拥有指定标签
    TargetRequireTag(GameplayTag),
    /// 需要自身 HP 低于指定百分比 (0.0~1.0)
    HpBelow(f32),
    /// 需要自身 HP 高于指定百分比
    HpAbove(f32),
}

/// 技能使用条件（RON 反序列化用，TagName 替代 GameplayTag）
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum SkillConditionDef {
    MpCost(i32),
    RequireTag(TagName),
    TargetRequireTag(TagName),
    HpBelow(f32),
    HpAbove(f32),
}

impl From<SkillConditionDef> for SkillCondition {
    fn from(def: SkillConditionDef) -> Self {
        match def {
            SkillConditionDef::MpCost(v) => SkillCondition::MpCost(v),
            SkillConditionDef::RequireTag(t) => SkillCondition::RequireTag(t.to_tag()),
            SkillConditionDef::TargetRequireTag(t) => SkillCondition::TargetRequireTag(t.to_tag()),
            SkillConditionDef::HpBelow(v) => SkillCondition::HpBelow(v),
            SkillConditionDef::HpAbove(v) => SkillCondition::HpAbove(v),
        }
    }
}

// ── 技能数据定义 ──

/// 技能数据定义（注册表中的静态数据）
#[derive(Clone, Debug)]
pub struct SkillData {
    pub id: String,
    pub name: String,
    pub description: String,
    pub cost_mp: i32,
    pub range: u32,
    pub targeting: SkillTargeting,
    pub effects: Vec<EffectDef>,
    pub tags: Vec<GameplayTag>,
    pub conditions: Vec<SkillCondition>,
    pub cooldown: u32,
    pub priority: u32,
}

/// 技能数据定义（RON 反序列化用，TagName 替代 GameplayTag）
#[derive(Clone, Debug, Deserialize)]
pub struct SkillDef {
    #[serde(default)]
    pub version: u32,
    pub id: String,
    pub name: String,
    pub description: String,
    pub cost_mp: i32,
    pub range: u32,
    pub targeting: SkillTargeting,
    pub effects: Vec<EffectDef>,
    pub tags: Vec<TagName>,
    pub conditions: Vec<SkillConditionDef>,
    pub cooldown: u32,
    pub priority: u32,
}

impl From<SkillDef> for SkillData {
    fn from(def: SkillDef) -> Self {
        SkillData {
            id: def.id,
            name: def.name,
            description: def.description,
            cost_mp: def.cost_mp,
            range: def.range,
            targeting: def.targeting,
            effects: def.effects,
            tags: def.tags.iter().map(|t| t.to_tag()).collect(),
            conditions: def.conditions.into_iter().map(Into::into).collect(),
            cooldown: def.cooldown,
            priority: def.priority,
        }
    }
}

impl SkillData {
    /// 检查单位是否满足使用条件（纯函数，不修改状态）
    pub fn can_use(
        &self,
        source_attrs: &crate::core::attribute::Attributes,
        source_tags: &crate::core::tag::GameplayTags,
        target_tags: Option<&crate::core::tag::GameplayTags>,
        current_cooldown: u32,
    ) -> Result<(), SkillUseError> {
        // 冷却检查
        if current_cooldown > 0 {
            return Err(SkillUseError::OnCooldown {
                remaining: current_cooldown,
            });
        }

        for cond in &self.conditions {
            match cond {
                SkillCondition::MpCost(cost) => {
                    let mp = source_attrs.get(AttributeKind::Mp);
                    if mp < *cost as f32 {
                        return Err(SkillUseError::InsufficientMp {
                            required: *cost,
                            current: mp as i32,
                        });
                    }
                }
                SkillCondition::RequireTag(tag) => {
                    if !source_tags.has(*tag) {
                        return Err(SkillUseError::MissingTag { tag: *tag });
                    }
                }
                SkillCondition::TargetRequireTag(tag) => {
                    if let Some(t_tags) = target_tags {
                        if !t_tags.has(*tag) {
                            return Err(SkillUseError::TargetMissingTag { tag: *tag });
                        }
                    }
                }
                SkillCondition::HpBelow(pct) => {
                    let hp = source_attrs.get(AttributeKind::Hp);
                    let max_hp = source_attrs.get(AttributeKind::MaxHp);
                    if max_hp > 0.0 && hp / max_hp >= *pct {
                        return Err(SkillUseError::HpNotBelow { threshold: *pct });
                    }
                }
                SkillCondition::HpAbove(pct) => {
                    let hp = source_attrs.get(AttributeKind::Hp);
                    let max_hp = source_attrs.get(AttributeKind::MaxHp);
                    if max_hp > 0.0 && hp / max_hp < *pct {
                        return Err(SkillUseError::HpNotAbove { threshold: *pct });
                    }
                }
            }
        }
        Ok(())
    }
}

/// 技能使用失败原因
#[derive(Clone, Debug, PartialEq)]
pub enum SkillUseError {
    OnCooldown { remaining: u32 },
    InsufficientMp { required: i32, current: i32 },
    MissingTag { tag: GameplayTag },
    TargetMissingTag { tag: GameplayTag },
    HpNotBelow { threshold: f32 },
    HpNotAbove { threshold: f32 },
}
