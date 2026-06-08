use crate::gameplay::attribute::{AttributeKind, AttributeModifierDef, ModifierOp};
use crate::gameplay::tag::{GameplayTag, TagName};
use bevy::prelude::*;
use ron::de::from_bytes;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::{read, read_dir};

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

    /// 从 assets/buffs/ 目录加载所有 .ron 文件
    pub fn load_from_dir(dir: &str) -> Self {
        let mut registry = BuffRegistry::default();
        let Ok(entries) = read_dir(dir) else {
            bevy::log::warn!("Buff 目录不存在，使用默认 Buff: {}", dir);
            registry.register_defaults();
            return registry;
        };

        let mut loaded = false;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "ron") {
                match read(&path) {
                    Ok(bytes) => match from_bytes::<BuffDef>(&bytes) {
                        Ok(def) => {
                            let id = def.id.clone();
                            registry.buffs.insert(id.clone(), def.into());
                            bevy::log::info!("加载 Buff: {}", id);
                            loaded = true;
                        }
                        Err(e) => {
                            bevy::log::error!("解析 Buff 文件 {:?} 失败: {}", path, e);
                        }
                    },
                    Err(e) => {
                        bevy::log::error!("读取 Buff 文件 {:?} 失败: {}", path, e);
                    }
                }
            }
        }

        // 目录存在但为空或全部解析失败，加载默认 Buff
        if !loaded {
            bevy::log::warn!("Buff 目录为空，使用默认 Buff");
            registry.register_defaults();
        }

        registry
    }

    /// 注册内置默认 Buff（确保基础功能可用）
    fn register_defaults(&mut self) {
        // 攻击力增加
        self.buffs.insert(
            "attack_up".into(),
            BuffData {
                id: "attack_up".into(),
                name: "攻+5".into(),
                default_duration: 3,
                modifiers: vec![AttributeModifierDef {
                    kind: AttributeKind::Atk,
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
                    kind: AttributeKind::Atk,
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
                    kind: AttributeKind::Def,
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
                    kind: AttributeKind::Def,
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
                    kind: AttributeKind::Def,
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

#[cfg(test)]
mod tests {
    use super::*;

    // ── BuffDef → BuffData 转换 ──

    #[test]
    fn buff_def_转换为_buff_data() {
        let def = BuffDef {
            id: "test_buff".into(),
            name: "测试增益".into(),
            default_duration: 3,
            modifiers: vec![AttributeModifierDef {
                kind: AttributeKind::Atk,
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
                    (kind: Atk, op: Add, value: 5.0),
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
