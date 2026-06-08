use crate::gameplay::attribute::AttributeKind;
use crate::gameplay::effect::EffectDef;
use crate::gameplay::tag::{GameplayTag, TagName};
use bevy::prelude::*;
use ron::de::from_bytes;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::{read, read_dir};

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
        source_attrs: &crate::gameplay::attribute::Attributes,
        source_tags: &crate::gameplay::tag::GameplayTags,
        target_tags: Option<&crate::gameplay::tag::GameplayTags>,
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

// ── 技能注册表 ──

/// 技能注册表资源
#[derive(Resource, Default)]
pub struct SkillRegistry {
    pub skills: HashMap<String, SkillData>,
}

impl SkillRegistry {
    pub fn get(&self, id: &str) -> Option<&SkillData> {
        self.skills.get(id)
    }

    /// 注册一个技能
    pub fn register(&mut self, skill: SkillData) {
        self.skills.insert(skill.id.clone(), skill);
    }

    /// 从 assets/skills/ 目录加载所有 .ron 文件
    pub fn load_from_dir(dir: &str) -> Self {
        let mut registry = SkillRegistry::default();
        let Ok(entries) = read_dir(dir) else {
            bevy::log::warn!("技能目录不存在，使用默认技能: {}", dir);
            registry.register_defaults();
            return registry;
        };

        let mut loaded = false;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "ron") {
                match read(&path) {
                    Ok(bytes) => match from_bytes::<SkillDef>(&bytes) {
                        Ok(def) => {
                            let id = def.id.clone();
                            registry.register(def.into());
                            bevy::log::info!("加载技能: {}", id);
                            loaded = true;
                        }
                        Err(e) => {
                            bevy::log::error!("解析技能文件 {:?} 失败: {}", path, e);
                        }
                    },
                    Err(e) => {
                        bevy::log::error!("读取技能文件 {:?} 失败: {}", path, e);
                    }
                }
            }
        }

        // 目录存在但为空或全部解析失败，加载默认技能
        if !loaded {
            bevy::log::warn!("技能目录为空，使用默认技能");
            registry.register_defaults();
        }

        registry
    }

    /// 注册内置默认技能（确保基础功能可用）
    fn register_defaults(&mut self) {
        // 普通攻击
        self.register(SkillData {
            id: BASIC_ATTACK_ID.into(),
            name: "普通攻击".into(),
            description: "基础物理攻击".into(),
            cost_mp: 0,
            range: 0,
            targeting: SkillTargeting::SingleEnemy,
            effects: vec![EffectDef::Damage {
                multiplier: 1.0,
                ignore_def_percent: 0.0,
            }],
            tags: vec![],
            conditions: vec![],
            cooldown: 0,
            priority: 0,
        });

        // 冲锋
        self.register(SkillData {
            id: "charge".into(),
            name: "冲锋".into(),
            description: "1.5倍伤害，需要近战".into(),
            cost_mp: 0,
            range: 0,
            targeting: SkillTargeting::SingleEnemy,
            effects: vec![EffectDef::Damage {
                multiplier: 1.5,
                ignore_def_percent: 0.0,
            }],
            tags: vec![],
            conditions: vec![],
            cooldown: 2,
            priority: 5,
        });

        // 穿刺
        self.register(SkillData {
            id: "pierce".into(),
            name: "穿刺".into(),
            description: "无视50%防御".into(),
            cost_mp: 0,
            range: 0,
            targeting: SkillTargeting::SingleEnemy,
            effects: vec![EffectDef::Damage {
                multiplier: 1.2,
                ignore_def_percent: 50.0,
            }],
            tags: vec![],
            conditions: vec![],
            cooldown: 3,
            priority: 8,
        });

        // 火球
        self.register(SkillData {
            id: "fireball".into(),
            name: "火球".into(),
            description: "远程火属性攻击".into(),
            cost_mp: 0,
            range: 3,
            targeting: SkillTargeting::SingleEnemy,
            effects: vec![
                EffectDef::Damage {
                    multiplier: 1.5,
                    ignore_def_percent: 0.0,
                },
                EffectDef::ApplyBuff {
                    buff_id: "burn".into(),
                    duration: 2,
                },
            ],
            tags: vec![],
            conditions: vec![],
            cooldown: 2,
            priority: 10,
        });

        // 治疗
        self.register(SkillData {
            id: "heal".into(),
            name: "治疗".into(),
            description: "恢复友方生命值".into(),
            cost_mp: 0,
            range: 2,
            targeting: SkillTargeting::SingleAlly,
            effects: vec![EffectDef::Heal { amount: 8 }],
            tags: vec![],
            conditions: vec![],
            cooldown: 2,
            priority: 15,
        });

        // 净化
        self.register(SkillData {
            id: "cleanse_skill".into(),
            name: "净化".into(),
            description: "驱散友方所有负面效果".into(),
            cost_mp: 0,
            range: 2,
            targeting: SkillTargeting::SingleAlly,
            effects: vec![EffectDef::Cleanse],
            tags: vec![],
            conditions: vec![],
            cooldown: 3,
            priority: 12,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── SkillTargeting ──

    #[test]
    fn 目标类型_需要选择目标() {
        assert!(SkillTargeting::SingleEnemy.requires_target_selection());
        assert!(SkillTargeting::SingleAlly.requires_target_selection());
        assert!(!SkillTargeting::SelfOnly.requires_target_selection());
        assert!(!SkillTargeting::NoTarget.requires_target_selection());
    }

    // ── SkillData::can_use ──

    fn make_attrs(hp: f32, max_hp: f32, mp: f32) -> crate::gameplay::attribute::Attributes {
        let mut attrs = crate::gameplay::attribute::Attributes::default();
        attrs.set_base(AttributeKind::Hp, hp);
        attrs.set_base(AttributeKind::MaxHp, max_hp);
        attrs.set_base(AttributeKind::Mp, mp);
        attrs
    }

    #[test]
    fn 条件_冷却中不可使用() {
        let skill = SkillData {
            id: "fireball".into(),
            name: "火球".into(),
            description: String::new(),
            cost_mp: 0,
            range: 3,
            targeting: SkillTargeting::SingleEnemy,
            effects: vec![],
            tags: vec![],
            conditions: vec![],
            cooldown: 3,
            priority: 0,
        };
        let attrs = make_attrs(20.0, 20.0, 10.0);
        let tags = crate::gameplay::tag::GameplayTags::default();
        let result = skill.can_use(&attrs, &tags, None, 2);
        assert_eq!(result, Err(SkillUseError::OnCooldown { remaining: 2 }));
    }

    #[test]
    fn 条件_mp不足不可使用() {
        let skill = SkillData {
            id: "fireball".into(),
            name: "火球".into(),
            description: String::new(),
            cost_mp: 10,
            range: 3,
            targeting: SkillTargeting::SingleEnemy,
            effects: vec![],
            tags: vec![],
            conditions: vec![SkillCondition::MpCost(10)],
            cooldown: 0,
            priority: 0,
        };
        let attrs = make_attrs(20.0, 20.0, 5.0);
        let tags = crate::gameplay::tag::GameplayTags::default();
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
    fn 条件_缺少标签不可使用() {
        let skill = SkillData {
            id: "fireball".into(),
            name: "火球".into(),
            description: String::new(),
            cost_mp: 0,
            range: 3,
            targeting: SkillTargeting::SingleEnemy,
            effects: vec![],
            tags: vec![],
            conditions: vec![SkillCondition::RequireTag(GameplayTag::MAGE)],
            cooldown: 0,
            priority: 0,
        };
        let attrs = make_attrs(20.0, 20.0, 10.0);
        let tags = crate::gameplay::tag::GameplayTags::default();
        let result = skill.can_use(&attrs, &tags, None, 0);
        assert_eq!(
            result,
            Err(SkillUseError::MissingTag {
                tag: GameplayTag::MAGE
            })
        );
    }

    #[test]
    fn 条件_满足条件可使用() {
        let skill = SkillData {
            id: "fireball".into(),
            name: "火球".into(),
            description: String::new(),
            cost_mp: 5,
            range: 3,
            targeting: SkillTargeting::SingleEnemy,
            effects: vec![],
            tags: vec![],
            conditions: vec![SkillCondition::MpCost(5)],
            cooldown: 0,
            priority: 0,
        };
        let attrs = make_attrs(20.0, 20.0, 10.0);
        let tags = crate::gameplay::tag::GameplayTags::default();
        assert!(skill.can_use(&attrs, &tags, None, 0).is_ok());
    }

    #[test]
    fn 条件_hp低于阈值() {
        let skill = SkillData {
            id: "desperate".into(),
            name: "背水一战".into(),
            description: String::new(),
            cost_mp: 0,
            range: 1,
            targeting: SkillTargeting::SingleEnemy,
            effects: vec![],
            tags: vec![],
            conditions: vec![SkillCondition::HpBelow(0.5)],
            cooldown: 0,
            priority: 0,
        };
        let attrs_low = make_attrs(5.0, 20.0, 10.0);
        let attrs_ok = make_attrs(15.0, 20.0, 10.0);
        let tags = crate::gameplay::tag::GameplayTags::default();
        assert!(skill.can_use(&attrs_low, &tags, None, 0).is_ok());
        assert_eq!(
            skill.can_use(&attrs_ok, &tags, None, 0),
            Err(SkillUseError::HpNotBelow { threshold: 0.5 })
        );
    }

    // ── SkillDef → SkillData 转换 ──

    #[test]
    fn skill_def_转换为_skill_data() {
        let def = SkillDef {
            id: "test".into(),
            name: "测试".into(),
            description: "测试技能".into(),
            cost_mp: 5,
            range: 3,
            targeting: SkillTargeting::SingleEnemy,
            effects: vec![EffectDef::Damage {
                multiplier: 1.5,
                ignore_def_percent: 0.0,
            }],
            tags: vec![TagName::Fire, TagName::SkillActive],
            conditions: vec![SkillConditionDef::RequireTag(TagName::Mage)],
            cooldown: 2,
            priority: 10,
        };
        let data: SkillData = def.into();
        assert_eq!(data.id, "test");
        assert_eq!(
            data.tags,
            vec![GameplayTag::FIRE, GameplayTag::SKILL_ACTIVE]
        );
        assert_eq!(data.conditions.len(), 1);
        assert!(matches!(
            data.conditions[0],
            SkillCondition::RequireTag(GameplayTag::MAGE)
        ));
    }

    // ── RON 反序列化 ──

    #[test]
    fn ron_反序列化_技能定义() {
        let ron_str = r#"
            (
                id: "test_skill",
                name: "测试技能",
                description: "一个测试技能",
                cost_mp: 10,
                range: 3,
                targeting: SingleEnemy,
                effects: [
                    Damage(multiplier: 2.0, ignore_def_percent: 50.0),
                    ApplyBuff(buff_id: "burn", duration: 2),
                ],
                tags: [FIRE, SKILL_ACTIVE],
                conditions: [
                    MpCost(10),
                    RequireTag(MAGE),
                ],
                cooldown: 3,
                priority: 20,
            )
        "#;
        let def: SkillDef = from_bytes(ron_str.as_bytes()).unwrap();
        assert_eq!(def.id, "test_skill");
        assert_eq!(def.tags, vec![TagName::Fire, TagName::SkillActive]);
        assert_eq!(def.effects.len(), 2);
        assert_eq!(def.conditions.len(), 2);
    }
}
