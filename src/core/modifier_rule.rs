// 修饰规则：数据驱动的效果修饰系统
// 替代 modify_effects 中的硬编码 if-else

use crate::core::registry_loader::RegistryLoader;
use crate::core::tag::{GameplayTag, TagName};
use bevy::prelude::*;
use serde::Deserialize;

/// 修饰效果类型
#[derive(Clone, Debug)]
pub enum ModifierEffect {
    /// 伤害倍率修饰
    DamageMultiplier(f32),
    /// 伤害加成（固定值）
    DamageBonus(i32),
    /// 治疗倍率修饰
    HealMultiplier(f32),
    /// 治疗加成（固定值）
    HealBonus(i32),
}

impl ModifierEffect {
    /// 返回效果类型名（与 enum variant 名对应）
    pub fn type_name(&self) -> &'static str {
        match self {
            ModifierEffect::DamageMultiplier(_) => "DamageMultiplier",
            ModifierEffect::DamageBonus(_) => "DamageBonus",
            ModifierEffect::HealMultiplier(_) => "HealMultiplier",
            ModifierEffect::HealBonus(_) => "HealBonus",
        }
    }
}

/// 修饰计算规则 trait：描述如何计算一种修饰效果
pub trait ModifierCalculator: Send + Sync + 'static {
    /// 效果类型名（与 ModifierEffect variant 名对应）
    fn type_name(&self) -> &'static str;
    /// 是否适用于伤害修饰
    fn applies_to_damage(&self) -> bool;
    /// 是否适用于治疗修饰
    fn applies_to_heal(&self) -> bool;
    /// 计算修饰后的值
    fn calculate(&self, effect: &ModifierEffect, current: f32) -> f32;
}

// ---- 内置 Calculator 实现 ----

/// 伤害倍率计算器
pub struct DamageMultiplierCalculator;

impl ModifierCalculator for DamageMultiplierCalculator {
    fn type_name(&self) -> &'static str {
        "DamageMultiplier"
    }
    fn applies_to_damage(&self) -> bool {
        true
    }
    fn applies_to_heal(&self) -> bool {
        false
    }
    fn calculate(&self, effect: &ModifierEffect, current: f32) -> f32 {
        if let ModifierEffect::DamageMultiplier(mul) = effect {
            current * mul
        } else {
            current
        }
    }
}

/// 伤害加成计算器
pub struct DamageBonusCalculator;

impl ModifierCalculator for DamageBonusCalculator {
    fn type_name(&self) -> &'static str {
        "DamageBonus"
    }
    fn applies_to_damage(&self) -> bool {
        true
    }
    fn applies_to_heal(&self) -> bool {
        false
    }
    fn calculate(&self, effect: &ModifierEffect, current: f32) -> f32 {
        if let ModifierEffect::DamageBonus(bonus) = effect {
            current + *bonus as f32
        } else {
            current
        }
    }
}

/// 治疗倍率计算器
pub struct HealMultiplierCalculator;

impl ModifierCalculator for HealMultiplierCalculator {
    fn type_name(&self) -> &'static str {
        "HealMultiplier"
    }
    fn applies_to_damage(&self) -> bool {
        false
    }
    fn applies_to_heal(&self) -> bool {
        true
    }
    fn calculate(&self, effect: &ModifierEffect, current: f32) -> f32 {
        if let ModifierEffect::HealMultiplier(mul) = effect {
            current * mul
        } else {
            current
        }
    }
}

/// 治疗加成计算器
pub struct HealBonusCalculator;

impl ModifierCalculator for HealBonusCalculator {
    fn type_name(&self) -> &'static str {
        "HealBonus"
    }
    fn applies_to_damage(&self) -> bool {
        false
    }
    fn applies_to_heal(&self) -> bool {
        true
    }
    fn calculate(&self, effect: &ModifierEffect, current: f32) -> f32 {
        if let ModifierEffect::HealBonus(bonus) = effect {
            current + *bonus as f32
        } else {
            current
        }
    }
}

/// 修饰计算器注册表资源
#[derive(Resource)]
pub struct ModifierCalculatorRegistry {
    calculators: Vec<Box<dyn ModifierCalculator>>,
}

impl Default for ModifierCalculatorRegistry {
    fn default() -> Self {
        Self::with_defaults()
    }
}

impl ModifierCalculatorRegistry {
    /// 创建包含所有内置计算器的注册表
    pub fn with_defaults() -> Self {
        ModifierCalculatorRegistry {
            calculators: vec![
                Box::new(DamageMultiplierCalculator),
                Box::new(DamageBonusCalculator),
                Box::new(HealMultiplierCalculator),
                Box::new(HealBonusCalculator),
            ],
        }
    }

    /// 注册自定义计算器
    pub fn register(&mut self, calculator: Box<dyn ModifierCalculator>) {
        self.calculators.push(calculator);
    }

    /// 查找能处理指定效果类型的伤害计算器
    pub fn find_damage_calculator(
        &self,
        effect: &ModifierEffect,
    ) -> Option<&dyn ModifierCalculator> {
        self.calculators
            .iter()
            .find(|c| c.type_name() == effect.type_name() && c.applies_to_damage())
            .map(|c| c.as_ref())
    }

    /// 查找能处理指定效果类型的治疗计算器
    pub fn find_heal_calculator(&self, effect: &ModifierEffect) -> Option<&dyn ModifierCalculator> {
        self.calculators
            .iter()
            .find(|c| c.type_name() == effect.type_name() && c.applies_to_heal())
            .map(|c| c.as_ref())
    }
}

/// 修饰规则（运行时）
#[derive(Clone, Debug)]
pub struct ModifierRule {
    pub name: String,
    /// 攻击方技能需要包含的标签
    pub source_tag: GameplayTag,
    /// 目标需要包含的标签
    pub target_tag: GameplayTag,
    /// 修饰效果
    pub effect: ModifierEffect,
}

/// 修饰效果类型（RON 反序列化用）
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ModifierEffectDef {
    DamageMultiplier(f32),
    DamageBonus(i32),
    HealMultiplier(f32),
    HealBonus(i32),
}

/// 修饰规则（RON 反序列化用）
#[derive(Clone, Debug, Deserialize)]
pub struct ModifierRuleDef {
    /// 配置版本号（预留，用于未来存档兼容性检查）
    #[serde(default)]
    pub version: u32,
    pub name: String,
    pub source_tag: TagName,
    pub target_tag: TagName,
    pub effect: ModifierEffectDef,
}

impl From<ModifierRuleDef> for ModifierRule {
    fn from(def: ModifierRuleDef) -> Self {
        ModifierRule {
            name: def.name,
            source_tag: def.source_tag.to_tag(),
            target_tag: def.target_tag.to_tag(),
            effect: match def.effect {
                ModifierEffectDef::DamageMultiplier(v) => ModifierEffect::DamageMultiplier(v),
                ModifierEffectDef::DamageBonus(v) => ModifierEffect::DamageBonus(v),
                ModifierEffectDef::HealMultiplier(v) => ModifierEffect::HealMultiplier(v),
                ModifierEffectDef::HealBonus(v) => ModifierEffect::HealBonus(v),
            },
        }
    }
}

/// 修饰规则注册表资源
#[derive(Resource, Default)]
pub struct ModifierRuleRegistry {
    pub rules: Vec<ModifierRule>,
    /// 计算器注册表，用于 trait 分发替代 match
    calculators: ModifierCalculatorRegistry,
}

impl ModifierRuleRegistry {
    /// 兜底默认值
    pub fn register_defaults(&mut self) {
        if self.rules.is_empty() {
            self.rules.push(ModifierRule {
                name: "火焰共鸣".into(),
                source_tag: GameplayTag::FIRE,
                target_tag: GameplayTag::FIRE,
                effect: ModifierEffect::DamageMultiplier(1.5),
            });
        }
    }

    /// 注册自定义计算器
    pub fn register_calculator(&mut self, calculator: Box<dyn ModifierCalculator>) {
        self.calculators.register(calculator);
    }

    /// 应用所有修饰规则到伤害值
    pub fn apply_damage_modifiers(
        &self,
        amount: i32,
        source_tags: &[GameplayTag],
        target_tags: &crate::core::tag::GameplayTags,
    ) -> i32 {
        let mut result = amount as f32;
        for rule in &self.rules {
            if !source_tags.contains(&rule.source_tag) {
                continue;
            }
            if !target_tags.has(rule.target_tag) {
                continue;
            }
            // 通过 calculator registry 查找并计算，替代 match 分发
            if let Some(calc) = self.calculators.find_damage_calculator(&rule.effect) {
                result = calc.calculate(&rule.effect, result);
            }
        }
        result.max(1.0) as i32
    }

    /// 应用所有修饰规则到治疗值
    pub fn apply_heal_modifiers(
        &self,
        amount: i32,
        source_tags: &[GameplayTag],
        target_tags: &crate::core::tag::GameplayTags,
    ) -> i32 {
        let mut result = amount as f32;
        for rule in &self.rules {
            if !source_tags.contains(&rule.source_tag) {
                continue;
            }
            if !target_tags.has(rule.target_tag) {
                continue;
            }
            // 通过 calculator registry 查找并计算，替代 match 分发
            if let Some(calc) = self.calculators.find_heal_calculator(&rule.effect) {
                result = calc.calculate(&rule.effect, result);
            }
        }
        result.max(0.0) as i32
    }
}

impl RegistryLoader for ModifierRuleRegistry {
    type Item = ModifierRuleDef;

    fn register_item(&mut self, item: ModifierRuleDef) {
        let name = item.name.clone();
        self.rules.push(item.into());
        bevy::log::info!("加载修饰规则: {}", name);
    }

    fn register_defaults(&mut self) {
        ModifierRuleRegistry::register_defaults(self);
    }

    fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    fn registry_name() -> &'static str {
        "修饰规则"
    }
}

/// 修饰规则插件
pub struct ModifierRulePlugin;

impl Plugin for ModifierRulePlugin {
    fn build(&self, app: &mut App) {
        let mut registry = ModifierRuleRegistry::load_from_dir_vec("assets/rules");
        registry.register_defaults();
        app.insert_resource(registry);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::tag::GameplayTags;
    use ron::de::from_bytes;

    #[test]
    fn ron_反序列化_修饰规则() {
        let ron_str = r#"
            [
                (
                    name: "火焰共鸣",
                    source_tag: FIRE,
                    target_tag: FIRE,
                    effect: DamageMultiplier(1.5),
                ),
                (
                    name: "冰火相克",
                    source_tag: FIRE,
                    target_tag: ICE,
                    effect: DamageMultiplier(0.5),
                ),
            ]
        "#;
        let defs: Vec<ModifierRuleDef> = from_bytes(ron_str.as_bytes()).unwrap();
        assert_eq!(defs.len(), 2);
    }

    #[test]
    fn 修饰规则_火焰增伤() {
        let registry = ModifierRuleRegistry {
            rules: vec![ModifierRule {
                name: "火焰共鸣".into(),
                source_tag: GameplayTag::FIRE,
                target_tag: GameplayTag::FIRE,
                effect: ModifierEffect::DamageMultiplier(1.5),
            }],
            ..Default::default()
        };

        let mut target_tags = GameplayTags::default();
        target_tags.add(GameplayTag::FIRE);

        let result = registry.apply_damage_modifiers(10, &[GameplayTag::FIRE], &target_tags);
        assert_eq!(result, 15); // 10 * 1.5 = 15
    }

    #[test]
    fn 修饰规则_无匹配规则不变() {
        let registry = ModifierRuleRegistry {
            rules: vec![ModifierRule {
                name: "火焰共鸣".into(),
                source_tag: GameplayTag::FIRE,
                target_tag: GameplayTag::FIRE,
                effect: ModifierEffect::DamageMultiplier(1.5),
            }],
            ..Default::default()
        };

        let target_tags = GameplayTags::default(); // 无 FIRE 标签
        let result = registry.apply_damage_modifiers(10, &[GameplayTag::FIRE], &target_tags);
        assert_eq!(result, 10); // 无匹配，不变
    }

    #[test]
    fn 修饰规则_固定加成() {
        let registry = ModifierRuleRegistry {
            rules: vec![ModifierRule {
                name: "毒伤加深".into(),
                source_tag: GameplayTag::POISON,
                target_tag: GameplayTag::POISON,
                effect: ModifierEffect::DamageBonus(5),
            }],
            ..Default::default()
        };

        let mut target_tags = GameplayTags::default();
        target_tags.add(GameplayTag::POISON);

        let result = registry.apply_damage_modifiers(10, &[GameplayTag::POISON], &target_tags);
        assert_eq!(result, 15); // 10 + 5 = 15
    }

    #[test]
    fn 修饰规则_治疗倍率() {
        let registry = ModifierRuleRegistry {
            rules: vec![ModifierRule {
                name: "火焰治愈".into(),
                source_tag: GameplayTag::FIRE,
                target_tag: GameplayTag::FIRE,
                effect: ModifierEffect::HealMultiplier(2.0),
            }],
            ..Default::default()
        };
        let mut target_tags = GameplayTags::default();
        target_tags.add(GameplayTag::FIRE);
        let result = registry.apply_heal_modifiers(10, &[GameplayTag::FIRE], &target_tags);
        assert_eq!(result, 20);
    }

    #[test]
    fn 修饰规则_治疗固定加成() {
        let registry = ModifierRuleRegistry {
            rules: vec![ModifierRule {
                name: "治愈加成".into(),
                source_tag: GameplayTag::REGEN,
                target_tag: GameplayTag::BUFF,
                effect: ModifierEffect::HealBonus(5),
            }],
            ..Default::default()
        };
        let mut target_tags = GameplayTags::default();
        target_tags.add(GameplayTag::BUFF);
        let result = registry.apply_heal_modifiers(10, &[GameplayTag::REGEN], &target_tags);
        assert_eq!(result, 15);
    }

    #[test]
    fn 修饰规则_治疗无匹配不变() {
        let registry = ModifierRuleRegistry {
            rules: vec![ModifierRule {
                name: "火焰治愈".into(),
                source_tag: GameplayTag::FIRE,
                target_tag: GameplayTag::FIRE,
                effect: ModifierEffect::HealMultiplier(2.0),
            }],
            ..Default::default()
        };
        let target_tags = GameplayTags::default();
        let result = registry.apply_heal_modifiers(10, &[], &target_tags);
        assert_eq!(result, 10);
    }

    #[test]
    fn 修饰规则_多规则叠加() {
        let registry = ModifierRuleRegistry {
            rules: vec![
                ModifierRule {
                    name: "倍率".into(),
                    source_tag: GameplayTag::FIRE,
                    target_tag: GameplayTag::FIRE,
                    effect: ModifierEffect::DamageMultiplier(1.5),
                },
                ModifierRule {
                    name: "加成".into(),
                    source_tag: GameplayTag::FIRE,
                    target_tag: GameplayTag::FIRE,
                    effect: ModifierEffect::DamageBonus(3),
                },
            ],
            ..Default::default()
        };
        let mut target_tags = GameplayTags::default();
        target_tags.add(GameplayTag::FIRE);
        let result = registry.apply_damage_modifiers(10, &[GameplayTag::FIRE], &target_tags);
        assert_eq!(result, 18);
    }

    #[test]
    fn 修饰规则_最低伤害为1() {
        let registry = ModifierRuleRegistry {
            rules: vec![ModifierRule {
                name: "减伤".into(),
                source_tag: GameplayTag::ICE,
                target_tag: GameplayTag::ICE,
                effect: ModifierEffect::DamageMultiplier(0.01),
            }],
            ..Default::default()
        };
        let mut target_tags = GameplayTags::default();
        target_tags.add(GameplayTag::ICE);
        let result = registry.apply_damage_modifiers(10, &[GameplayTag::ICE], &target_tags);
        assert_eq!(result, 1);
    }

    #[test]
    fn 修饰规则_最低治疗为0() {
        let registry = ModifierRuleRegistry {
            rules: vec![ModifierRule {
                name: "减少".into(),
                source_tag: GameplayTag::POISON,
                target_tag: GameplayTag::POISON,
                effect: ModifierEffect::HealMultiplier(0.01),
            }],
            ..Default::default()
        };
        let mut target_tags = GameplayTags::default();
        target_tags.add(GameplayTag::POISON);
        let result = registry.apply_heal_modifiers(10, &[GameplayTag::POISON], &target_tags);
        assert_eq!(result, 0);
    }

    #[test]
    fn 修饰规则_兜底默认值() {
        let mut registry = ModifierRuleRegistry::default();
        registry.register_defaults();
        assert!(!registry.rules.is_empty());
        assert_eq!(registry.rules[0].name, "火焰共鸣");
    }
}
