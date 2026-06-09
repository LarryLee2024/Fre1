use crate::core::attribute::{AttributeKind, AttributeModifierDef, ModifierOp};
use crate::core::registry_loader::RegistryLoader;
use crate::core::tag::{GameplayTag, TagName};
use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

/// Buff 数据定义（运行时）
#[derive(Clone, Debug)]
pub struct BuffData {
    pub id: String,
    pub name: String,
    pub default_duration: u32,
    pub modifiers: Vec<AttributeModifierDef>,
    pub tags: Vec<GameplayTag>,
    pub dot_damage: i32,
    pub hot_heal: i32,
    pub is_stun: bool,
    pub is_cleanse: bool,
    pub is_buff: bool,
}

/// Buff 数据定义（RON 反序列化用，TagName 替代 GameplayTag）
#[derive(Clone, Debug, Deserialize)]
pub struct BuffDef {
    #[serde(default)]
    pub version: u32,
    pub id: String,
    pub name: String,
    pub default_duration: u32,
    pub modifiers: Vec<AttributeModifierDef>,
    pub tags: Vec<TagName>,
    pub dot_damage: i32,
    pub hot_heal: i32,
    pub is_stun: bool,
    pub is_cleanse: bool,
    pub is_buff: bool,
}

impl From<BuffDef> for BuffData {
    fn from(def: BuffDef) -> Self {
        BuffData {
            id: def.id,
            name: def.name,
            default_duration: def.default_duration,
            modifiers: def.modifiers,
            tags: def.tags.iter().map(|t| t.to_tag()).collect(),
            dot_damage: def.dot_damage,
            hot_heal: def.hot_heal,
            is_stun: def.is_stun,
            is_cleanse: def.is_cleanse,
            is_buff: def.is_buff,
        }
    }
}

impl BuffData {
    pub fn is_debuff(&self) -> bool {
        !self.is_buff
    }
}

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
    fn register_defaults(&mut self) {
        if !self.buffs.is_empty() {
            return;
        }
        // 攻击力增加
        self.buffs.insert(
            "attack_up".into(),
            BuffData {
                id: "attack_up".into(),
                name: "攻+5".into(),
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
        bevy::log::info!("加载Buff: {}", id);
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
    use ron::de::from_bytes;

    // ── BuffDef → BuffData 转换 ──

    #[test]
    fn buff_def_转换为_buff_data() {
        let def = BuffDef {
            version: 0,
            id: "test_buff".into(),
            name: "测试增益".into(),
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
    }

    #[test]
    fn is_debuff_增益返回false() {
        let data = BuffData {
            id: "test".into(),
            name: "test".into(),
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
}
