/// Buff 领域错误（BuffError, BuffResult）
mod buff_error;
/// Buff 数据模型类型（BuffData, BuffDef, DurationPolicy, StackPolicy 等）
mod types;

pub use buff_error::*;
pub use types::*;

use crate::core::attribute::{AttributeKind, AttributeModifierDef, ModifierOp};
use crate::core::registry_loader::RegistryLoader;
use crate::core::tag::GameplayTag;
use bevy::prelude::*;
use std::collections::HashMap;

/// Buff 注册表资源
#[derive(Resource, Default, Clone, Debug)]
pub struct BuffRegistry {
    pub buffs: HashMap<String, BuffData>,
}

impl BuffRegistry {
    pub fn get(&self, id: &str) -> Option<&BuffData> {
        self.buffs.get(id)
    }

    /// 注册一个 Buff
    pub fn register(&mut self, buff: BuffData) {
        self.buffs.insert(buff.id.clone(), buff);
    }

    /// 注册内置默认 Buff（确保基础功能可用）
    pub fn register_defaults(&mut self) {
        if !self.buffs.is_empty() {
            return;
        }
        // 攻击力增加
        self.buffs.insert(
            "attack_up".into(),
            BuffData {
                id: "attack_up".into(),
                name: "攻+5".into(),
                name_key: Some("buff.b_001.name".into()),
                description: String::new(),
                effects: vec![],
                duration: DurationPolicy::Turns(3),
                stack: StackPolicy::NoStack,
                conditions: vec![],
                default_duration: 3,
                modifiers: vec![AttributeModifierDef {
                    kind: AttributeKind::Attack,
                    op: ModifierOp::Add,
                    value: 5.0,
                }],
                tags: vec![GameplayTag::BUFF],
                dot_damage: 0,
                hot_heal: 0,
                is_stun: false,
                is_cleanse: false,
                is_buff: true,
            },
        );

        // 攻击力减少
        self.buffs.insert(
            "attack_down".into(),
            BuffData {
                id: "attack_down".into(),
                name: "攻-5".into(),
                name_key: Some("buff.b_002.name".into()),
                description: String::new(),
                effects: vec![],
                duration: DurationPolicy::Turns(3),
                stack: StackPolicy::NoStack,
                conditions: vec![],
                default_duration: 3,
                modifiers: vec![AttributeModifierDef {
                    kind: AttributeKind::Attack,
                    op: ModifierOp::Add,
                    value: -5.0,
                }],
                tags: vec![GameplayTag::DEBUFF],
                dot_damage: 0,
                hot_heal: 0,
                is_stun: false,
                is_cleanse: false,
                is_buff: false,
            },
        );

        // 防御力增加
        self.buffs.insert(
            "defense_up".into(),
            BuffData {
                id: "defense_up".into(),
                name: "防+5".into(),
                name_key: Some("buff.b_003.name".into()),
                description: String::new(),
                effects: vec![],
                duration: DurationPolicy::Turns(3),
                stack: StackPolicy::NoStack,
                conditions: vec![],
                default_duration: 3,
                modifiers: vec![AttributeModifierDef {
                    kind: AttributeKind::Defense,
                    op: ModifierOp::Add,
                    value: 5.0,
                }],
                tags: vec![GameplayTag::BUFF],
                dot_damage: 0,
                hot_heal: 0,
                is_stun: false,
                is_cleanse: false,
                is_buff: true,
            },
        );

        // 防御力减少
        self.buffs.insert(
            "defense_down".into(),
            BuffData {
                id: "defense_down".into(),
                name: "防-5".into(),
                name_key: Some("buff.b_004.name".into()),
                description: String::new(),
                effects: vec![],
                duration: DurationPolicy::Turns(3),
                stack: StackPolicy::NoStack,
                conditions: vec![],
                default_duration: 3,
                modifiers: vec![AttributeModifierDef {
                    kind: AttributeKind::Defense,
                    op: ModifierOp::Add,
                    value: -5.0,
                }],
                tags: vec![GameplayTag::DEBUFF],
                dot_damage: 0,
                hot_heal: 0,
                is_stun: false,
                is_cleanse: false,
                is_buff: false,
            },
        );

        // 灼烧
        self.buffs.insert(
            "burn".into(),
            BuffData {
                id: "burn".into(),
                name: "灼-2".into(),
                name_key: Some("buff.b_005.name".into()),
                description: String::new(),
                effects: vec![],
                duration: DurationPolicy::Turns(2),
                stack: StackPolicy::NoStack,
                conditions: vec![],
                default_duration: 2,
                modifiers: vec![AttributeModifierDef {
                    kind: AttributeKind::Defense,
                    op: ModifierOp::Add,
                    value: -2.0,
                }],
                tags: vec![GameplayTag::DEBUFF, GameplayTag::BURN, GameplayTag::FIRE],
                dot_damage: 2,
                hot_heal: 0,
                is_stun: false,
                is_cleanse: false,
                is_buff: false,
            },
        );

        // 中毒
        self.buffs.insert(
            "poison".into(),
            BuffData {
                id: "poison".into(),
                name: "毒-3".into(),
                name_key: Some("buff.b_006.name".into()),
                description: String::new(),
                effects: vec![],
                duration: DurationPolicy::Turns(3),
                stack: StackPolicy::NoStack,
                conditions: vec![],
                default_duration: 3,
                modifiers: vec![],
                tags: vec![GameplayTag::DEBUFF, GameplayTag::POISON],
                dot_damage: 3,
                hot_heal: 0,
                is_stun: false,
                is_cleanse: false,
                is_buff: false,
            },
        );

        // 再生
        self.buffs.insert(
            "regen".into(),
            BuffData {
                id: "regen".into(),
                name: "愈+4".into(),
                name_key: Some("buff.b_007.name".into()),
                description: String::new(),
                effects: vec![],
                duration: DurationPolicy::Turns(3),
                stack: StackPolicy::NoStack,
                conditions: vec![],
                default_duration: 3,
                modifiers: vec![],
                tags: vec![GameplayTag::BUFF],
                dot_damage: 0,
                hot_heal: 4,
                is_stun: false,
                is_cleanse: false,
                is_buff: true,
            },
        );

        // 眩晕
        self.buffs.insert(
            "stun".into(),
            BuffData {
                id: "stun".into(),
                name: "晕眩".into(),
                name_key: Some("buff.b_008.name".into()),
                description: String::new(),
                effects: vec![],
                duration: DurationPolicy::Turns(1),
                stack: StackPolicy::NoStack,
                conditions: vec![],
                default_duration: 1,
                modifiers: vec![],
                tags: vec![GameplayTag::DEBUFF, GameplayTag::POISON],
                dot_damage: 3,
                hot_heal: 0,
                is_stun: false,
                is_cleanse: false,
                is_buff: false,
            },
        );

        // 再生
        self.buffs.insert(
            "regen".into(),
            BuffData {
                id: "regen".into(),
                name: "愈+4".into(),
                name_key: Some("buff.b_007.name".into()),
                description: "每回合恢复生命".into(),
                effects: vec![],
                duration: DurationPolicy::Turns(3),
                stack: StackPolicy::NoStack,
                conditions: vec![],
                default_duration: 3,
                modifiers: vec![],
                tags: vec![GameplayTag::BUFF],
                dot_damage: 0,
                hot_heal: 4,
                is_stun: false,
                is_cleanse: false,
                is_buff: true,
            },
        );

        // 眩晕
        self.buffs.insert(
            "stun".into(),
            BuffData {
                id: "stun".into(),
                name: "晕眩".into(),
                name_key: Some("buff.b_008.name".into()),
                description: "无法行动".into(),
                effects: vec![],
                duration: DurationPolicy::Turns(1),
                stack: StackPolicy::NoStack,
                conditions: vec![],
                default_duration: 1,
                modifiers: vec![],
                tags: vec![GameplayTag::DEBUFF, GameplayTag::STUN],
                dot_damage: 0,
                hot_heal: 0,
                is_stun: true,
                is_cleanse: false,
                is_buff: false,
            },
        );
    }
}

impl RegistryLoader for BuffRegistry {
    type Item = BuffDef;

    fn register_item(&mut self, item: BuffDef) {
        let id = item.id.clone();
        self.register(item.into());
        bevy::log::info!(target: "buff", event = "buff_loaded", id = %id, "Buff已加载");
    }

    fn register_defaults(&mut self) {
        BuffRegistry::register_defaults(self);
    }

    fn is_empty(&self) -> bool {
        self.buffs.is_empty()
    }

    fn registry_name() -> &'static str {
        "Buff"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::tag::TagName;
    use ron::de::from_bytes;

    // ── BuffDef → BuffData 转换 ──

    #[test]
    fn buff_def_转换为_buff_data() {
        let def = BuffDef {
            version: 0,
            id: "test_buff".into(),
            name: "测试增益".into(),
            name_key: None,
            description: String::new(),
            effects: vec![],
            duration: DurationDef::Turns(3),
            stack: StackDef::NoStack,
            conditions: vec![],
            default_duration: 3,
            modifiers: vec![AttributeModifierDef {
                kind: AttributeKind::Attack,
                op: ModifierOp::Add,
                value: 10.0,
            }],
            tags: vec![TagName::Buff, TagName::Fire],
            dot_damage: 0,
            hot_heal: 0,
            is_stun: false,
            is_cleanse: false,
            is_buff: true,
        };
        let data: BuffData = def.into();
        assert_eq!(data.id, "test_buff");
        assert_eq!(data.tags, vec![GameplayTag::BUFF, GameplayTag::FIRE]);
        assert_eq!(data.duration, DurationPolicy::Turns(3));
        assert_eq!(data.stack, StackPolicy::NoStack);
        assert!(data.effects.is_empty());
        assert!(data.conditions.is_empty());
    }

    // ── RON 反序列化 ──

    #[test]
    fn ron_反序列化_buff定义() {
        let ron_str = r#"
            (
                id: "test_buff",
                name: "测试增益",
                default_duration: 2,
                modifiers: [
                    (kind: Attack, op: Add, value: 5.0),
                ],
                tags: [BUFF, FIRE],
                dot_damage: 0,
                hot_heal: 3,
                is_stun: false,
                is_cleanse: false,
                is_buff: true,
            )
        "#;
        let def: BuffDef = from_bytes(ron_str.as_bytes()).unwrap();
        assert_eq!(def.id, "test_buff");
        assert_eq!(def.tags, vec![TagName::Buff, TagName::Fire]);
        // 新字段使用默认值
        assert_eq!(def.duration, DurationDef::Turns(1)); // 默认 Turns(1)
        assert_eq!(def.stack, StackDef::NoStack); // 默认 NoStack
        assert!(def.effects.is_empty());
    }

    #[test]
    fn ron_反序列化_带duration和stack字段() {
        let ron_str = r#"
            (
                id: "new_buff",
                name: "新Buff",
                description: "测试描述",
                duration: Turns(5),
                stack: Stackable(3),
                default_duration: 2,
                modifiers: [],
                tags: [BUFF],
                dot_damage: 0,
                hot_heal: 0,
                is_stun: false,
                is_cleanse: false,
                is_buff: true,
            )
        "#;
        let def: BuffDef = from_bytes(ron_str.as_bytes()).unwrap();
        assert_eq!(def.duration, DurationDef::Turns(5));
        assert_eq!(def.stack, StackDef::Stackable(3));
        assert_eq!(def.description, "测试描述");
    }

    #[test]
    fn is_debuff_增益返回false() {
        let data = BuffData {
            id: "test".into(),
            name: "test".into(),
            name_key: None,
            description: String::new(),
            effects: vec![],
            duration: DurationPolicy::Turns(1),
            stack: StackPolicy::NoStack,
            conditions: vec![],
            default_duration: 1,
            modifiers: vec![],
            tags: vec![],
            dot_damage: 0,
            hot_heal: 0,
            is_stun: false,
            is_cleanse: false,
            is_buff: true,
        };
        assert!(!data.is_debuff());
    }

    #[test]
    fn is_debuff_减益返回true() {
        let data = BuffData {
            id: "test".into(),
            name: "test".into(),
            name_key: None,
            description: String::new(),
            effects: vec![],
            duration: DurationPolicy::Turns(1),
            stack: StackPolicy::NoStack,
            conditions: vec![],
            default_duration: 1,
            modifiers: vec![],
            tags: vec![],
            dot_damage: 0,
            hot_heal: 0,
            is_stun: false,
            is_cleanse: false,
            is_buff: false,
        };
        assert!(data.is_debuff());
    }

    #[test]
    fn buff_registry_查询已注册buff() {
        let mut registry = BuffRegistry::default();
        registry.buffs.insert(
            "test".into(),
            BuffData {
                id: "test".into(),
                name: "测试".into(),
                name_key: None,
                description: String::new(),
                effects: vec![],
                duration: DurationPolicy::Turns(1),
                stack: StackPolicy::NoStack,
                conditions: vec![],
                default_duration: 1,
                modifiers: vec![],
                tags: vec![],
                dot_damage: 0,
                hot_heal: 0,
                is_stun: false,
                is_cleanse: false,
                is_buff: true,
            },
        );
        assert!(registry.get("test").is_some());
    }

    #[test]
    fn buff_registry_查询未注册返回none() {
        let registry = BuffRegistry::default();
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn buff_registry_默认注册() {
        let mut registry = BuffRegistry::default();
        registry.register_defaults();
        assert!(registry.get("attack_up").is_some());
        assert!(registry.get("attack_down").is_some());
        assert!(registry.get("stun").is_some());
    }

    #[test]
    fn buff_registry_默认buff包含新字段() {
        let mut registry = BuffRegistry::default();
        registry.register_defaults();

        let attack_up = registry.get("attack_up").unwrap();
        assert_eq!(attack_up.duration, DurationPolicy::Turns(3));
        assert_eq!(attack_up.stack, StackPolicy::NoStack);
        assert!(attack_up.description.is_empty());
        assert!(attack_up.effects.is_empty());

        let poison = registry.get("poison").unwrap();
        assert_eq!(poison.duration, DurationPolicy::Turns(3));
        assert_eq!(poison.stack, StackPolicy::NoStack);
    }

    #[test]
    fn ron_反序列化_旧配置无version字段() {
        let ron_str = r#"
            (
                id: "old_buff",
                name: "旧Buff",
                is_buff: true,
                default_duration: 3,
                modifiers: [],
                tags: [],
                dot_damage: 0,
                hot_heal: 0,
                is_stun: false,
                is_cleanse: false,
            )
        "#;
        let def: BuffDef = from_bytes(ron_str.as_bytes()).unwrap();
        assert_eq!(def.id, "old_buff");
        assert_eq!(def.version, 0);
        // 旧配置无 duration/stack 字段，使用默认值
        assert_eq!(def.duration, DurationDef::Turns(1));
        assert_eq!(def.stack, StackDef::NoStack);
    }
}
