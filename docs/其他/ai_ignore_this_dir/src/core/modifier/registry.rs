// 修饰规则注册表资源

use crate::core::registry_loader::RegistryLoader;
use bevy::prelude::*;

use super::calculator::ModifierCalculator;
use super::types::{ModifierEntry, ModifierRule, ModifierRuleDef};

/// 修饰规则注册表资源
#[derive(Resource, Default)]
pub struct ModifierRuleRegistry {
    pub rules: Vec<ModifierRule>,
    pub(crate) calculators: super::calculator::ModifierCalculatorRegistry,
}

impl ModifierRuleRegistry {
    /// 兜底默认值
    /// 🟥 不硬编码规则内容（宪法 1.1.3 Rule/Content 分离，1.1.5 数据驱动）
    /// 规则必须从 RON 配置加载，此处仅记录警告
    pub fn register_defaults(&mut self) {
        if self.rules.is_empty() {
            bevy::log::warn!(
                target: "core",
                "修饰规则注册表为空，请检查 content/modifiers/ 目录下的 RON 配置文件"
            );
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
        source_tags: &[crate::core::tag::GameplayTag],
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
        source_tags: &[crate::core::tag::GameplayTag],
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

    /// 应用所有修饰规则到伤害值，同时记录每步修饰详情
    pub fn apply_damage_modifiers_with_breakdown(
        &self,
        amount: i32,
        source_tags: &[crate::core::tag::GameplayTag],
        target_tags: &crate::core::tag::GameplayTags,
    ) -> (i32, Vec<ModifierEntry>) {
        let mut result = amount as f32;
        let mut entries = Vec::new();
        for rule in &self.rules {
            if !source_tags.contains(&rule.source_tag) {
                continue;
            }
            if !target_tags.has(rule.target_tag) {
                continue;
            }
            if let Some(calc) = self.calculators.find_damage_calculator(&rule.effect) {
                let before = result;
                result = calc.calculate(&rule.effect, result);
                entries.push(ModifierEntry {
                    before: before as i32,
                    after: result as i32,
                    rule_name: rule.name.clone(),
                });
            }
        }
        (result.max(1.0) as i32, entries)
    }

    /// 应用所有修饰规则到治疗值，同时记录每步修饰详情
    pub fn apply_heal_modifiers_with_breakdown(
        &self,
        amount: i32,
        source_tags: &[crate::core::tag::GameplayTag],
        target_tags: &crate::core::tag::GameplayTags,
    ) -> (i32, Vec<ModifierEntry>) {
        let mut result = amount as f32;
        let mut entries = Vec::new();
        for rule in &self.rules {
            if !source_tags.contains(&rule.source_tag) {
                continue;
            }
            if !target_tags.has(rule.target_tag) {
                continue;
            }
            if let Some(calc) = self.calculators.find_heal_calculator(&rule.effect) {
                let before = result;
                result = calc.calculate(&rule.effect, result);
                entries.push(ModifierEntry {
                    before: before as i32,
                    after: result as i32,
                    rule_name: rule.name.clone(),
                });
            }
        }
        (result.max(0.0) as i32, entries)
    }
}

impl RegistryLoader for ModifierRuleRegistry {
    type Item = ModifierRuleDef;

    fn register_item(&mut self, item: ModifierRuleDef) {
        let name = item.name.clone();
        self.rules.push(item.into());
        bevy::log::info!(target: "core", event = "modifier_rule_loaded", name = %name, "修饰规则已加载");
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
