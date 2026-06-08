// 技能系统：数据驱动 + 目标类型 + 条件 + 冷却 + 预览
// 完全解耦：技能定义独立于棋子，棋子只持有 skill_ids
// 支持从 assets/skills/*.ron 外部配置文件加载

use crate::core::effect::EffectDef;
use crate::core::tag::{GameplayTag, TagName};
use crate::core::attribute::AttributeKind;
use bevy::prelude::*;
use ron::de::from_bytes;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::{read, read_dir};

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
        source_attrs: &crate::core::attribute::Attributes,
        source_tags: &crate::core::tag::GameplayTags,
        target_tags: Option<&crate::core::tag::GameplayTags>,
        current_cooldown: u32,
    ) -> Result<(), SkillUseError> {
        // 冷却检查
        if current_cooldown > 0 {
            return Err(SkillUseError::OnCooldown { remaining: current_cooldown });
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

// ── 技能槽组件 ──

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

    /// 获取所有技能 ID（迭代器）
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.skill_ids.iter().map(|s| s.as_str())
    }
}

// ── 运行时冷却追踪 ──

/// 运行时技能冷却追踪组件
#[derive(Component, Default, Debug, Clone)]
pub struct SkillCooldowns {
    /// skill_id → 剩余冷却回合数
    cooldowns: HashMap<String, u32>,
}

impl SkillCooldowns {
    /// 获取技能当前冷却
    pub fn get(&self, skill_id: &str) -> u32 {
        self.cooldowns.get(skill_id).copied().unwrap_or(0)
    }

    /// 设置技能冷却
    pub fn set(&mut self, skill_id: &str, turns: u32) {
        if turns > 0 {
            self.cooldowns.insert(skill_id.to_string(), turns);
        }
    }

    /// 回合结束：递减所有冷却
    pub fn tick(&mut self) {
        self.cooldowns.retain(|_, cd| {
            *cd = cd.saturating_sub(1);
            *cd > 0
        });
    }

    /// 清除所有冷却
    pub fn clear(&mut self) {
        self.cooldowns.clear();
    }
}

// ── 技能执行上下文 ──

/// 技能执行上下文：封装一次技能释放的所有信息
#[derive(Clone, Debug)]
pub struct SkillExecutionContext {
    pub source: Entity,
    pub target: Entity,
    pub skill_id: String,
    pub source_attrs: crate::core::attribute::Attributes,
    pub target_attrs: crate::core::attribute::Attributes,
    pub source_tags: crate::core::tag::GameplayTags,
    pub target_tags: crate::core::tag::GameplayTags,
    pub terrain: crate::map::Terrain,
}

impl SkillExecutionContext {
    /// 从 ECS 查询构建上下文（纯数据快照，避免借用冲突）
    pub fn from_query(
        source: Entity,
        target: Entity,
        skill_id: &str,
        source_attrs: &crate::core::attribute::Attributes,
        target_attrs: &crate::core::attribute::Attributes,
        source_tags: &crate::core::tag::GameplayTags,
        target_tags: &crate::core::tag::GameplayTags,
        terrain: crate::map::Terrain,
    ) -> Self {
        Self {
            source,
            target,
            skill_id: skill_id.to_string(),
            source_attrs: source_attrs.clone(),
            target_attrs: target_attrs.clone(),
            source_tags: source_tags.clone(),
            target_tags: target_tags.clone(),
            terrain,
        }
    }
}

// ── 效果预览 ──

/// 技能效果预览结果
#[derive(Clone, Debug)]
pub struct SkillPreview {
    pub skill_id: String,
    pub skill_name: String,
    pub predictions: Vec<EffectPreview>,
}

/// 单个效果的预览
#[derive(Clone, Debug)]
pub enum EffectPreview {
    Damage { amount: i32, lethal: bool },
    Heal { amount: i32 },
    BuffApplied { buff_name: String },
    Cleanse,
}

/// 预览技能效果（纯函数，不修改任何状态）
pub fn preview_skill_effects(
    ctx: &SkillExecutionContext,
    skill_data: &SkillData,
    buff_registry: &crate::data::buff_data::BuffRegistry,
) -> SkillPreview {
    let mut predictions = Vec::new();

    for effect_def in &skill_data.effects {
        match effect_def {
            EffectDef::Damage { multiplier, ignore_def_percent } => {
                let effective_atk = ctx.source_attrs.get(AttributeKind::Atk);
                let effective_def = ctx.target_attrs.get(AttributeKind::Def);
                let base_def = ctx.target_attrs.base.get(&AttributeKind::Def).copied().unwrap_or(0.0);

                let amount = crate::core::effect::calculate_damage_from_effect(
                    effective_atk, effective_def, base_def,
                    *multiplier, *ignore_def_percent, ctx.terrain,
                );
                let current_hp = ctx.target_attrs.get(AttributeKind::Hp);
                predictions.push(EffectPreview::Damage {
                    amount,
                    lethal: current_hp - amount as f32 <= 0.0,
                });
            }
            EffectDef::Heal { amount } => {
                let max_hp = ctx.target_attrs.get(AttributeKind::MaxHp);
                let current_hp = ctx.target_attrs.get(AttributeKind::Hp);
                let actual = (*amount as f32).min(max_hp - current_hp).max(0.0) as i32;
                predictions.push(EffectPreview::Heal { amount: actual });
            }
            EffectDef::ApplyBuff { buff_id, .. } => {
                let buff_name = buff_registry
                    .get(buff_id)
                    .map(|b| b.name.as_str())
                    .unwrap_or(buff_id);
                predictions.push(EffectPreview::BuffApplied { buff_name: buff_name.to_string() });
            }
            EffectDef::Cleanse => {
                predictions.push(EffectPreview::Cleanse);
            }
        }
    }

    SkillPreview {
        skill_id: skill_data.id.clone(),
        skill_name: skill_data.name.clone(),
        predictions,
    }
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
            bevy::log::warn!("技能目录不存在: {}", dir);
            return registry;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "ron") {
                match read(&path) {
                    Ok(bytes) => match from_bytes::<SkillDef>(&bytes) {
                        Ok(def) => {
                            let id = def.id.clone();
                            registry.register(def.into());
                            bevy::log::info!("加载技能: {}", id);
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
        registry
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
        let registry = SkillRegistry::load_from_dir("assets/skills");
        app.insert_resource(registry);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── SkillSlots ──

    #[test]
    fn 技能槽_默认攻击() {
        let slots = SkillSlots::new(vec!["basic_attack".into(), "charge".into()]);
        assert_eq!(slots.default_attack(), "basic_attack");
    }

    #[test]
    fn 技能槽_默认攻击_空列表回退() {
        let slots = SkillSlots::new(vec![]);
        assert_eq!(slots.default_attack(), "basic_attack");
    }

    #[test]
    fn 技能槽_特殊技能() {
        let slots = SkillSlots::new(vec!["basic_attack".into(), "charge".into()]);
        assert_eq!(slots.special_skill(), Some("charge"));
    }

    #[test]
    fn 技能槽_特殊技能_只有一个技能() {
        let slots = SkillSlots::new(vec!["basic_attack".into()]);
        assert_eq!(slots.special_skill(), None);
    }

    #[test]
    fn 技能槽_特殊技能_空列表() {
        let slots = SkillSlots::new(vec![]);
        assert_eq!(slots.special_skill(), None);
    }

    #[test]
    fn 技能槽_迭代器() {
        let slots = SkillSlots::new(vec!["basic_attack".into(), "charge".into()]);
        let ids: Vec<&str> = slots.iter().collect();
        assert_eq!(ids, vec!["basic_attack", "charge"]);
    }

    // ── effective_skill_range ──

    #[test]
    fn 技能范围_技能自带范围() {
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
            cooldown: 0,
            priority: 0,
        };
        assert_eq!(effective_skill_range(&skill, 1), 3);
    }

    #[test]
    fn 技能范围_使用单位基础范围() {
        let skill = SkillData {
            id: "basic_attack".into(),
            name: "普通攻击".into(),
            description: String::new(),
            cost_mp: 0,
            range: 0,
            targeting: SkillTargeting::SingleEnemy,
            effects: vec![],
            tags: vec![],
            conditions: vec![],
            cooldown: 0,
            priority: 0,
        };
        assert_eq!(effective_skill_range(&skill, 3), 3);
    }

    // ── SkillTargeting ──

    #[test]
    fn 目标类型_需要选择目标() {
        assert!(SkillTargeting::SingleEnemy.requires_target_selection());
        assert!(SkillTargeting::SingleAlly.requires_target_selection());
        assert!(!SkillTargeting::SelfOnly.requires_target_selection());
        assert!(!SkillTargeting::NoTarget.requires_target_selection());
    }

    // ── SkillCooldowns ──

    #[test]
    fn 冷却_初始为0() {
        let cds = SkillCooldowns::default();
        assert_eq!(cds.get("fireball"), 0);
    }

    #[test]
    fn 冷却_设置和查询() {
        let mut cds = SkillCooldowns::default();
        cds.set("fireball", 3);
        assert_eq!(cds.get("fireball"), 3);
    }

    #[test]
    fn 冷却_tick递减() {
        let mut cds = SkillCooldowns::default();
        cds.set("fireball", 2);
        cds.tick();
        assert_eq!(cds.get("fireball"), 1);
        cds.tick();
        assert_eq!(cds.get("fireball"), 0); // 归零后被移除
    }

    #[test]
    fn 冷却_clear清空() {
        let mut cds = SkillCooldowns::default();
        cds.set("fireball", 3);
        cds.set("pierce", 2);
        cds.clear();
        assert_eq!(cds.get("fireball"), 0);
        assert_eq!(cds.get("pierce"), 0);
    }

    // ── SkillData::can_use ──

    fn make_attrs(hp: f32, max_hp: f32, mp: f32) -> crate::core::attribute::Attributes {
        let mut attrs = crate::core::attribute::Attributes::default();
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
        let tags = crate::core::tag::GameplayTags::default();
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
        let tags = crate::core::tag::GameplayTags::default();
        let result = skill.can_use(&attrs, &tags, None, 0);
        assert_eq!(result, Err(SkillUseError::InsufficientMp { required: 10, current: 5 }));
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
        let tags = crate::core::tag::GameplayTags::default();
        let result = skill.can_use(&attrs, &tags, None, 0);
        assert_eq!(result, Err(SkillUseError::MissingTag { tag: GameplayTag::MAGE }));
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
        let tags = crate::core::tag::GameplayTags::default();
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
        let tags = crate::core::tag::GameplayTags::default();
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
            effects: vec![EffectDef::Damage { multiplier: 1.5, ignore_def_percent: 0.0 }],
            tags: vec![TagName::Fire, TagName::SkillActive],
            conditions: vec![SkillConditionDef::RequireTag(TagName::Mage)],
            cooldown: 2,
            priority: 10,
        };
        let data: SkillData = def.into();
        assert_eq!(data.id, "test");
        assert_eq!(data.tags, vec![GameplayTag::FIRE, GameplayTag::SKILL_ACTIVE]);
        assert_eq!(data.conditions.len(), 1);
        assert!(matches!(data.conditions[0], SkillCondition::RequireTag(GameplayTag::MAGE)));
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

    // ── 效果预览 ──

    #[test]
    fn 预览_伤害预览() {
        let mut source_attrs = crate::core::attribute::Attributes::default();
        source_attrs.set_base(AttributeKind::Atk, 10.0);
        let mut target_attrs = crate::core::attribute::Attributes::default();
        target_attrs.set_base(AttributeKind::Def, 3.0);
        target_attrs.set_base(AttributeKind::Hp, 20.0);

        let ctx = SkillExecutionContext {
            source: Entity::from_bits(1),
            target: Entity::from_bits(2),
            skill_id: "basic_attack".into(),
            source_attrs,
            target_attrs,
            source_tags: crate::core::tag::GameplayTags::default(),
            target_tags: crate::core::tag::GameplayTags::default(),
            terrain: crate::map::Terrain::Plain,
        };

        // 使用硬编码构建 SkillData 用于测试（不依赖文件系统）
        let skill = SkillData {
            id: "basic_attack".into(),
            name: "普通攻击".into(),
            description: String::new(),
            cost_mp: 0,
            range: 0,
            targeting: SkillTargeting::SingleEnemy,
            effects: vec![EffectDef::Damage { multiplier: 1.0, ignore_def_percent: 0.0 }],
            tags: vec![],
            conditions: vec![],
            cooldown: 0,
            priority: 0,
        };
        let buff_reg = crate::data::buff_data::BuffRegistry::default();
        let preview = preview_skill_effects(&ctx, &skill, &buff_reg);

        assert_eq!(preview.skill_id, "basic_attack");
        assert_eq!(preview.predictions.len(), 1);
        if let EffectPreview::Damage { amount, lethal } = &preview.predictions[0] {
            assert_eq!(*amount, 7); // 10 - 3 = 7
            assert!(!lethal);
        } else {
            panic!("应该是伤害预览");
        }
    }

    #[test]
    fn 预览_致死伤害标记() {
        let mut source_attrs = crate::core::attribute::Attributes::default();
        source_attrs.set_base(AttributeKind::Atk, 50.0);
        let mut target_attrs = crate::core::attribute::Attributes::default();
        target_attrs.set_base(AttributeKind::Def, 3.0);
        target_attrs.set_base(AttributeKind::Hp, 5.0);

        let ctx = SkillExecutionContext {
            source: Entity::from_bits(1),
            target: Entity::from_bits(2),
            skill_id: "basic_attack".into(),
            source_attrs,
            target_attrs,
            source_tags: crate::core::tag::GameplayTags::default(),
            target_tags: crate::core::tag::GameplayTags::default(),
            terrain: crate::map::Terrain::Plain,
        };

        let skill = SkillData {
            id: "basic_attack".into(),
            name: "普通攻击".into(),
            description: String::new(),
            cost_mp: 0,
            range: 0,
            targeting: SkillTargeting::SingleEnemy,
            effects: vec![EffectDef::Damage { multiplier: 1.0, ignore_def_percent: 0.0 }],
            tags: vec![],
            conditions: vec![],
            cooldown: 0,
            priority: 0,
        };
        let buff_reg = crate::data::buff_data::BuffRegistry::default();
        let preview = preview_skill_effects(&ctx, &skill, &buff_reg);

        if let EffectPreview::Damage { lethal, .. } = &preview.predictions[0] {
            assert!(lethal);
        }
    }

    #[test]
    fn 预览_治疗预览() {
        let mut source_attrs = crate::core::attribute::Attributes::default();
        let mut target_attrs = crate::core::attribute::Attributes::default();
        target_attrs.set_base(AttributeKind::Hp, 12.0);
        target_attrs.set_base(AttributeKind::MaxHp, 20.0);

        let ctx = SkillExecutionContext {
            source: Entity::from_bits(1),
            target: Entity::from_bits(2),
            skill_id: "heal".into(),
            source_attrs,
            target_attrs,
            source_tags: crate::core::tag::GameplayTags::default(),
            target_tags: crate::core::tag::GameplayTags::default(),
            terrain: crate::map::Terrain::Plain,
        };

        let skill = SkillData {
            id: "heal".into(),
            name: "治疗".into(),
            description: String::new(),
            cost_mp: 0,
            range: 3,
            targeting: SkillTargeting::SingleAlly,
            effects: vec![EffectDef::Heal { amount: 8 }],
            tags: vec![],
            conditions: vec![],
            cooldown: 2,
            priority: 15,
        };
        let buff_reg = crate::data::buff_data::BuffRegistry::default();
        let preview = preview_skill_effects(&ctx, &skill, &buff_reg);

        if let EffectPreview::Heal { amount } = &preview.predictions[0] {
            assert_eq!(*amount, 8);
        }
    }

    #[test]
    fn 预览_治疗不超过最大hp() {
        let mut source_attrs = crate::core::attribute::Attributes::default();
        let mut target_attrs = crate::core::attribute::Attributes::default();
        target_attrs.set_base(AttributeKind::Hp, 18.0);
        target_attrs.set_base(AttributeKind::MaxHp, 20.0);

        let ctx = SkillExecutionContext {
            source: Entity::from_bits(1),
            target: Entity::from_bits(2),
            skill_id: "heal".into(),
            source_attrs,
            target_attrs,
            source_tags: crate::core::tag::GameplayTags::default(),
            target_tags: crate::core::tag::GameplayTags::default(),
            terrain: crate::map::Terrain::Plain,
        };

        let skill = SkillData {
            id: "heal".into(),
            name: "治疗".into(),
            description: String::new(),
            cost_mp: 0,
            range: 3,
            targeting: SkillTargeting::SingleAlly,
            effects: vec![EffectDef::Heal { amount: 8 }],
            tags: vec![],
            conditions: vec![],
            cooldown: 2,
            priority: 15,
        };
        let buff_reg = crate::data::buff_data::BuffRegistry::default();
        let preview = preview_skill_effects(&ctx, &skill, &buff_reg);

        if let EffectPreview::Heal { amount } = &preview.predictions[0] {
            assert_eq!(*amount, 2); // min(8, 20-18) = 2
        }
    }
}
