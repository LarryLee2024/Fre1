// 修饰计算器 trait 与内置实现

use bevy::prelude::*;

use super::types::ModifierEffect;

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
