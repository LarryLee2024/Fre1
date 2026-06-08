// 修饰规则：数据驱动的效果修饰系统
// 替代 modify_effects 中的硬编码 if-else

use crate::gameplay::tag::{GameplayTag, TagName};
use bevy::prelude::*;
use ron::de::from_bytes;
use serde::Deserialize;
use std::fs::{read, read_dir};

/// 修饰效果类型
#[derive(Clone, Debug)]
pub enum ModifierEffect {
    /// 伤害倍率修饰
    DamageMultiplier(f32),
    /// 伤害加成（固定值）
    DamageBonus(i32),
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
}

/// 修饰规则（RON 反序列化用）
#[derive(Clone, Debug, Deserialize)]
pub struct ModifierRuleDef {
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
            },
        }
    }
}

/// 修饰规则注册表资源
#[derive(Resource, Default)]
pub struct ModifierRuleRegistry {
    pub rules: Vec<ModifierRule>,
}

impl ModifierRuleRegistry {
    /// 从 assets/rules/ 目录加载所有 .ron 文件
    pub fn load_from_dir(dir: &str) -> Self {
        let mut registry = ModifierRuleRegistry::default();
        let Ok(entries) = read_dir(dir) else {
            bevy::log::warn!("修饰规则目录不存在: {}", dir);
            return registry;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "ron") {
                match read(&path) {
                    Ok(bytes) => match from_bytes::<Vec<ModifierRuleDef>>(&bytes) {
                        Ok(defs) => {
                            for def in defs {
                                let name = def.name.clone();
                                registry.rules.push(def.into());
                                bevy::log::info!("加载修饰规则: {}", name);
                            }
                        }
                        Err(e) => {
                            bevy::log::error!("解析修饰规则文件 {:?} 失败: {}", path, e);
                        }
                    },
                    Err(e) => {
                        bevy::log::error!("读取修饰规则文件 {:?} 失败: {}", path, e);
                    }
                }
            }
        }
        registry
    }

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

    /// 应用所有修饰规则到伤害值
    pub fn apply_damage_modifiers(
        &self,
        amount: i32,
        source_tags: &[GameplayTag],
        target_tags: &crate::gameplay::tag::GameplayTags,
    ) -> i32 {
        let mut result = amount as f32;
        for rule in &self.rules {
            if !source_tags.contains(&rule.source_tag) {
                continue;
            }
            if !target_tags.has(rule.target_tag) {
                continue;
            }
            match rule.effect {
                ModifierEffect::DamageMultiplier(mul) => {
                    result *= mul;
                }
                ModifierEffect::DamageBonus(bonus) => {
                    result += bonus as f32;
                }
            }
        }
        result.max(1.0) as i32
    }
}

/// 修饰规则插件
pub struct ModifierRulePlugin;

impl Plugin for ModifierRulePlugin {
    fn build(&self, app: &mut App) {
        let mut registry = ModifierRuleRegistry::load_from_dir("assets/rules");
        registry.register_defaults();
        app.insert_resource(registry);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gameplay::tag::GameplayTags;

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
        };

        let mut target_tags = GameplayTags::default();
        target_tags.add(GameplayTag::FIRE);

        let result = registry.apply_damage_modifiers(
            10,
            &[GameplayTag::FIRE],
            &target_tags,
        );
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
        };

        let target_tags = GameplayTags::default(); // 无 FIRE 标签
        let result = registry.apply_damage_modifiers(
            10,
            &[GameplayTag::FIRE],
            &target_tags,
        );
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
        };

        let mut target_tags = GameplayTags::default();
        target_tags.add(GameplayTag::POISON);

        let result = registry.apply_damage_modifiers(
            10,
            &[GameplayTag::POISON],
            &target_tags,
        );
        assert_eq!(result, 15); // 10 + 5 = 15
    }
}
