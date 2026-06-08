// 技能数据：数据驱动的技能定义，替代 Skill 枚举

use crate::core::effect::EffectDef;
use crate::core::tag::GameplayTag;
use bevy::prelude::*;
use std::collections::HashMap;

/// 技能数据定义
#[derive(Clone, Debug)]
pub struct SkillData {
    pub id: String,
    pub name: String,
    pub cost_mp: i32,
    pub range: u32,
    pub effects: Vec<EffectDef>,
    pub tags: Vec<GameplayTag>,
}

/// 单位的技能槽组件
#[derive(Component, Default, Debug, Clone)]
pub struct SkillSlots {
    pub skill_ids: Vec<String>,
}

impl SkillSlots {
    pub fn new(skill_ids: Vec<String>) -> Self {
        Self { skill_ids }
    }

    /// 获取默认攻击技能 ID
    pub fn default_attack(&self) -> &str {
        self.skill_ids.first().map(|s| s.as_str()).unwrap_or("basic_attack")
    }

    /// 获取特殊技能 ID（第二个技能，如果有）
    pub fn special_skill(&self) -> Option<&str> {
        self.skill_ids.get(1).map(|s| s.as_str())
    }
}

/// 技能注册表资源
#[derive(Resource, Default)]
pub struct SkillRegistry {
    pub skills: HashMap<String, SkillData>,
}

impl SkillRegistry {
    pub fn get(&self, id: &str) -> Option<&SkillData> {
        self.skills.get(id)
    }

    /// 初始化所有技能定义
    pub fn populate(&mut self) {
        let skills = vec![
            SkillData {
                id: "basic_attack".into(),
                name: "普通攻击".into(),
                cost_mp: 0,
                range: 0, // 使用单位自身攻击范围
                effects: vec![EffectDef::Damage {
                    multiplier: 1.0,
                    ignore_def_percent: 0.0,
                }],
                tags: vec![],
            },
            SkillData {
                id: "charge".into(),
                name: "冲锋".into(),
                cost_mp: 0,
                range: 1,
                effects: vec![EffectDef::Damage {
                    multiplier: 1.5,
                    ignore_def_percent: 0.0,
                }],
                tags: vec![GameplayTag::MELEE, GameplayTag::SKILL_ACTIVE],
            },
            SkillData {
                id: "pierce".into(),
                name: "穿透箭".into(),
                cost_mp: 0,
                range: 4,
                effects: vec![EffectDef::Damage {
                    multiplier: 1.3,
                    ignore_def_percent: 50.0,
                }],
                tags: vec![GameplayTag::RANGED, GameplayTag::SKILL_ACTIVE],
            },
            SkillData {
                id: "fireball".into(),
                name: "火球".into(),
                cost_mp: 0,
                range: 3,
                effects: vec![
                    EffectDef::Damage {
                        multiplier: 1.8,
                        ignore_def_percent: 0.0,
                    },
                    EffectDef::ApplyBuff {
                        buff_id: "burn".into(),
                        duration: 2,
                    },
                ],
                tags: vec![GameplayTag::FIRE, GameplayTag::SKILL_ACTIVE],
            },
        ];

        for skill in skills {
            self.skills.insert(skill.id.clone(), skill);
        }
    }
}

/// 获取技能的有效范围（考虑单位基础攻击范围）
pub fn effective_skill_range(skill_data: &SkillData, base_attack_range: u32) -> u32 {
    if skill_data.range > 0 {
        skill_data.range
    } else {
        base_attack_range
    }
}

/// 技能数据插件
pub struct SkillDataPlugin;

impl Plugin for SkillDataPlugin {
    fn build(&self, app: &mut App) {
        let mut registry = SkillRegistry::default();
        registry.populate();
        app.insert_resource(registry);
    }
}
