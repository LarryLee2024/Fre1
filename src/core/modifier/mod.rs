// 修饰规则：数据驱动的效果修饰系统
// 替代 modify_effects 中的硬编码 if-else

pub mod calculator;
pub mod registry;
pub mod types;

pub use calculator::*;
pub use registry::*;
pub use types::*;

use crate::core::registry_loader::RegistryLoader;
use bevy::prelude::*;

/// 修饰规则插件
pub struct ModifierRulePlugin;

impl Plugin for ModifierRulePlugin {
    fn build(&self, app: &mut App) {
        let mut registry = ModifierRuleRegistry::load_from_dir_vec("content/modifiers");
        registry.register_defaults();
        app.insert_resource(registry);
    }
}

#[cfg(test)]
mod tests {
    // ================================================
    // Bevy SRPG AI宪法 v1.1 自检结果（测试专用）
    // ================================================
    // ✅ 测行为不测实现：是 — 断言验证修饰结果，不验证内部规则匹配逻辑
    // ✅ 符合领域规则：是 — 覆盖 INV-MOD-1~8 修饰规则不变量
    // ✅ 确定性：是 — 硬编码规则和标签数据
    // ✅ 使用标准数据：是 — 使用标准 ModifierRule 结构
    // ✅ 无越界测试：是 — 仅测试公共 API
    // ✅ 未测试私有实现：是 — 仅通过 pub 接口测试
    // ================================================
    use super::*;
    use crate::core::tag::{GameplayTag, GameplayTags};
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
        // 🟥 不硬编码规则内容（Rule/Content 分离），空注册表应保持空
        assert!(registry.rules.is_empty());
    }
}
