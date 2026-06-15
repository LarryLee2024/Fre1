// 技能类型定义：SkillData, SkillCondition, SkillDef, SkillUseError
// SkillTargeting 已迁移至 crate::core::targeting::types

use crate::core::attribute::AttributeKind;
use crate::core::effect::EffectDef;
use crate::core::tag::{GameplayTag, TagName};
use crate::core::targeting::types::SkillTargeting;
use bevy::prelude::*;
use serde::Deserialize;

/// 基础攻击技能 ID 常量
pub const BASIC_ATTACK_ID: &str = "basic_attack";

// ── 技能使用条件 ──

/// 技能使用条件（运行时）
#[derive(Clone, Debug, PartialEq, Reflect)]
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
#[derive(Clone, Debug, Reflect)]
pub struct SkillData {
    pub id: String,
    /// 旧字段：直接文本（向后兼容）
    pub name: String,
    /// 旧字段：直接文本（向后兼容）
    pub description: String,
    /// 新字段：本地化 Key（优先使用）
    pub name_key: Option<String>,
    /// 新字段：本地化 Key（优先使用）
    pub desc_key: Option<String>,
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
    /// 旧字段：直接文本（向后兼容）
    #[serde(default)]
    pub name: String,
    /// 旧字段：直接文本（向后兼容）
    #[serde(default)]
    pub description: String,
    /// 新字段：本地化 Key（优先使用）
    #[serde(default)]
    pub name_key: Option<String>,
    /// 新字段：本地化 Key（优先使用）
    #[serde(default)]
    pub desc_key: Option<String>,
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
            name_key: def.name_key,
            desc_key: def.desc_key,
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

impl Default for SkillData {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            description: String::new(),
            name_key: None,
            desc_key: None,
            cost_mp: 0,
            range: 1,
            targeting: SkillTargeting::SingleEnemy,
            effects: vec![],
            tags: vec![],
            conditions: vec![],
            cooldown: 0,
            priority: 0,
        }
    }
}

impl SkillData {
    /// 扣除技能消耗（MP）- 属于 Cost 阶段
    /// 根据 skill.cost_mp 字段扣除，确保 cost_mp 是实际消耗的唯一来源
    pub fn deduct_cost(&self, attrs: &mut crate::core::attribute::Attributes) {
        if self.cost_mp > 0 {
            let current = attrs.get(AttributeKind::Mp);
            let new_mp = (current - self.cost_mp as f32).max(0.0);
            attrs.set_vital(AttributeKind::Mp, new_mp);
        }
    }

    /// 检查单位是否满足使用条件（纯函数，不修改状态）
    /// ADR-014: 固定检查顺序：冷却 → cost_mp → 自定义条件
    pub fn can_use(
        &self,
        source_attrs: &crate::core::attribute::Attributes,
        source_tags: &crate::core::tag::GameplayTags,
        target_tags: Option<&crate::core::tag::GameplayTags>,
        current_cooldown: u32,
    ) -> Result<(), SkillUseError> {
        // 冷却检查（最先）
        if current_cooldown > 0 {
            return Err(SkillUseError::OnCooldown {
                remaining: current_cooldown,
            });
        }

        // cost_mp 字段检查（始终执行，ADR-013 要求 cost_mp 是必填字段）
        if self.cost_mp > 0 {
            let mp = source_attrs.get(AttributeKind::Mp);
            if mp < self.cost_mp as f32 {
                return Err(SkillUseError::InsufficientMp {
                    required: self.cost_mp,
                    current: mp as i32,
                });
            }
        }

        // 自定义条件列表（含 SkillCondition::MpCost 以保持 RON 向后兼容）
        for cond in &self.conditions {
            match cond {
                SkillCondition::MpCost(cost) => {
                    // cost_mp 字段已检查，此处仅用于兼容旧 RON 配置
                    // 避免重复错误：仅在 cost_mp 字段未覆盖时检查
                    if self.cost_mp <= 0 {
                        let mp = source_attrs.get(AttributeKind::Mp);
                        if mp < *cost as f32 {
                            return Err(SkillUseError::InsufficientMp {
                                required: *cost,
                                current: mp as i32,
                            });
                        }
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
#[derive(Clone, Debug, PartialEq, Reflect)]
pub enum SkillUseError {
    OnCooldown { remaining: u32 },
    InsufficientMp { required: i32, current: i32 },
    MissingTag { tag: GameplayTag },
    TargetMissingTag { tag: GameplayTag },
    HpNotBelow { threshold: f32 },
    HpNotAbove { threshold: f32 },
}

#[cfg(test)]
mod tests {
    // ================================================
    // Bevy SRPG AI宪法 v1.1 自检结果（测试专用）
    // ================================================
    // ✅ 测行为不测实现：是 — 断言验证 can_use 返回值，不验证内部条件匹配逻辑
    // ✅ 符合领域规则：是 — 覆盖 INV-SKILL-001~010 技能条件和目标不变量
    // ✅ 确定性：是 — 硬编码属性值和标签数据
    // ✅ 使用标准数据：是 — 使用标准 SkillCondition/SkillTargeting
    // ✅ 无越界测试：是 — 仅测试公共 API
    // ✅ 未测试私有实现：是 — 仅通过 pub 接口测试
    // ================================================
    use super::*;
    use crate::core::attribute::{AttributeKind, Attributes};
    use crate::core::tag::{GameplayTag, GameplayTags};
    use ron::de::from_bytes;

    #[test]
    fn ron_反序列化_旧配置无version字段() {
        let ron_str = r#"
            (
                id: "old_skill",
                name: "旧技能",
                description: "没有version字段",
                cost_mp: 5,
                range: 1,
                cooldown: 0,
                targeting: SingleEnemy,
                effects: [],
                tags: [],
                conditions: [],
                priority: 0,
            )
        "#;
        let def: SkillDef = from_bytes(ron_str.as_bytes()).unwrap();
        assert_eq!(def.id, "old_skill");
        assert_eq!(def.version, 0);
    }

    // ── SkillData::can_use 测试 ──

    fn make_skill(conditions: Vec<SkillCondition>) -> SkillData {
        SkillData {
            id: "test_skill".into(),
            name: "测试技能".into(),
            conditions,
            ..Default::default()
        }
    }

    fn make_attrs(mp: f32, hp: f32, vitality: f32) -> Attributes {
        let mut attrs = Attributes::default();
        // 先设置 Vitality，这样 MaxHp 才能正确计算
        attrs.set_base(AttributeKind::Vitality, vitality);
        attrs.fill_vital_resources();
        // 覆盖当前 HP 和 MP 为指定值
        attrs.set_vital(AttributeKind::Hp, hp);
        attrs.set_vital(AttributeKind::Mp, mp);
        attrs
    }

    // ── 冷却检查 ──

    #[test]
    fn can_use_冷却中返回错误() {
        let skill = make_skill(vec![]);
        let attrs = make_attrs(10.0, 30.0, 5.0); // MP=10, HP=30, Vitality=5 → MaxHp=30
        let tags = GameplayTags::default();

        let result = skill.can_use(&attrs, &tags, None, 3);
        assert_eq!(result, Err(SkillUseError::OnCooldown { remaining: 3 }));
    }

    #[test]
    fn can_use_冷却为0成功() {
        let skill = make_skill(vec![]);
        let attrs = make_attrs(10.0, 30.0, 5.0); // MP=10, HP=30, Vitality=5 → MaxHp=30
        let tags = GameplayTags::default();

        let result = skill.can_use(&attrs, &tags, None, 0);
        assert!(result.is_ok());
    }

    // ── MpCost 条件 ──

    #[test]
    fn can_use_mp_不足返回错误() {
        let skill = make_skill(vec![SkillCondition::MpCost(10)]);
        let attrs = make_attrs(5.0, 30.0, 5.0); // MP=5 < 10
        let tags = GameplayTags::default();

        let result = skill.can_use(&attrs, &tags, None, 0);
        assert_eq!(
            result,
            Err(SkillUseError::InsufficientMp {
                required: 10,
                current: 5
            })
        );
    }

    #[test]
    fn can_use_mp_足够成功() {
        let skill = make_skill(vec![SkillCondition::MpCost(10)]);
        let attrs = make_attrs(15.0, 30.0, 5.0); // MP=15 >= 10
        let tags = GameplayTags::default();

        let result = skill.can_use(&attrs, &tags, None, 0);
        assert!(result.is_ok());
    }

    // ── RequireTag 条件 ──

    #[test]
    fn can_use_缺少标签返回错误() {
        let skill = make_skill(vec![SkillCondition::RequireTag(GameplayTag::FIRE)]);
        let attrs = make_attrs(10.0, 30.0, 5.0);
        let tags = GameplayTags::default(); // 没有FIRE标签

        let result = skill.can_use(&attrs, &tags, None, 0);
        assert_eq!(
            result,
            Err(SkillUseError::MissingTag {
                tag: GameplayTag::FIRE
            })
        );
    }

    #[test]
    fn can_use_拥有标签成功() {
        let skill = make_skill(vec![SkillCondition::RequireTag(GameplayTag::FIRE)]);
        let attrs = make_attrs(10.0, 30.0, 5.0);
        let mut tags = GameplayTags::default();
        tags.add(GameplayTag::FIRE);

        let result = skill.can_use(&attrs, &tags, None, 0);
        assert!(result.is_ok());
    }

    // ── TargetRequireTag 条件 ──

    #[test]
    fn can_use_目标缺少标签返回错误() {
        let skill = make_skill(vec![SkillCondition::TargetRequireTag(GameplayTag::STUN)]);
        let attrs = make_attrs(10.0, 30.0, 5.0);
        let tags = GameplayTags::default();
        let target_tags = GameplayTags::default(); // 没有STUN标签

        let result = skill.can_use(&attrs, &tags, Some(&target_tags), 0);
        assert_eq!(
            result,
            Err(SkillUseError::TargetMissingTag {
                tag: GameplayTag::STUN
            })
        );
    }

    #[test]
    fn can_use_目标拥有标签成功() {
        let skill = make_skill(vec![SkillCondition::TargetRequireTag(GameplayTag::STUN)]);
        let attrs = make_attrs(10.0, 30.0, 5.0);
        let tags = GameplayTags::default();
        let mut target_tags = GameplayTags::default();
        target_tags.add(GameplayTag::STUN);

        let result = skill.can_use(&attrs, &tags, Some(&target_tags), 0);
        assert!(result.is_ok());
    }

    #[test]
    fn can_use_无目标标签检查跳过() {
        let skill = make_skill(vec![SkillCondition::TargetRequireTag(GameplayTag::STUN)]);
        let attrs = make_attrs(10.0, 30.0, 5.0);
        let tags = GameplayTags::default();

        // 不提供目标标签，应该跳过检查
        let result = skill.can_use(&attrs, &tags, None, 0);
        assert!(result.is_ok());
    }

    // ── HpBelow 条件 ──
    // MaxHp = 5 + Vitality * 5

    #[test]
    fn can_use_hp_不低于阈值返回错误() {
        let skill = make_skill(vec![SkillCondition::HpBelow(0.5)]); // 需要HP低于50%
        // Vitality=5 → MaxHp=30, HP=20 → HP%=20/30=66.7% >= 50%
        let attrs = make_attrs(10.0, 20.0, 5.0);
        let tags = GameplayTags::default();

        let result = skill.can_use(&attrs, &tags, None, 0);
        assert_eq!(result, Err(SkillUseError::HpNotBelow { threshold: 0.5 }));
    }

    #[test]
    fn can_use_hp_低于阈值成功() {
        let skill = make_skill(vec![SkillCondition::HpBelow(0.5)]); // 需要HP低于50%
        // Vitality=5 → MaxHp=30, HP=10 → HP%=10/30=33.3% < 50%
        let attrs = make_attrs(10.0, 10.0, 5.0);
        let tags = GameplayTags::default();

        let result = skill.can_use(&attrs, &tags, None, 0);
        assert!(result.is_ok());
    }

    // ── HpAbove 条件 ──

    #[test]
    fn can_use_hp_不高于阈值返回错误() {
        let skill = make_skill(vec![SkillCondition::HpAbove(0.5)]); // 需要HP高于50%
        // Vitality=5 → MaxHp=30, HP=10 → HP%=10/30=33.3% < 50%
        let attrs = make_attrs(10.0, 10.0, 5.0);
        let tags = GameplayTags::default();

        let result = skill.can_use(&attrs, &tags, None, 0);
        assert_eq!(result, Err(SkillUseError::HpNotAbove { threshold: 0.5 }));
    }

    #[test]
    fn can_use_hp_高于阈值成功() {
        let skill = make_skill(vec![SkillCondition::HpAbove(0.5)]); // 需要HP高于50%
        // Vitality=5 → MaxHp=30, HP=20 → HP%=20/30=66.7% > 50%
        let attrs = make_attrs(10.0, 20.0, 5.0);
        let tags = GameplayTags::default();

        let result = skill.can_use(&attrs, &tags, None, 0);
        assert!(result.is_ok());
    }

    // ── 多条件组合 ──

    #[test]
    fn can_use_多条件全部满足() {
        let skill = make_skill(vec![
            SkillCondition::MpCost(5),
            SkillCondition::RequireTag(GameplayTag::FIRE),
        ]);
        let attrs = make_attrs(10.0, 30.0, 5.0); // MP=10 >= 5
        let mut tags = GameplayTags::default();
        tags.add(GameplayTag::FIRE);

        let result = skill.can_use(&attrs, &tags, None, 0);
        assert!(result.is_ok());
    }

    #[test]
    fn can_use_多条件之一不满足() {
        let skill = make_skill(vec![
            SkillCondition::MpCost(5),
            SkillCondition::RequireTag(GameplayTag::FIRE),
        ]);
        let attrs = make_attrs(3.0, 30.0, 5.0); // MP=3 < 5
        let mut tags = GameplayTags::default();
        tags.add(GameplayTag::FIRE);

        let result = skill.can_use(&attrs, &tags, None, 0);
        assert!(result.is_err());
    }

    // ── SkillTargeting 测试 ──

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
