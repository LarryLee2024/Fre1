// 属性定义注册表：属性元数据外部化，支持 UI 自动生成
// AttributeKind 仍为枚举（运行时类型安全），显示元数据从 RON 加载

use crate::gameplay::attribute::AttributeKind;
use bevy::prelude::*;
use ron::de::from_bytes;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::read;

/// 属性定义（RON 反序列化用）
#[derive(Clone, Debug, Deserialize)]
pub struct AttributeDefinition {
    pub kind: AttributeKind,
    pub display_name: String,
    pub description: String,
    pub default_value: f32,
    pub min_value: f32,
    pub max_value: f32,
}

/// 属性注册表资源
#[derive(Resource, Default)]
pub struct AttributeRegistry {
    pub definitions: HashMap<AttributeKind, AttributeDefinition>,
}

impl AttributeRegistry {
    pub fn get(&self, kind: AttributeKind) -> Option<&AttributeDefinition> {
        self.definitions.get(&kind)
    }

    /// 获取属性显示名称（找不到则回退到 kind 的 label）
    pub fn display_name(&self, kind: AttributeKind) -> &str {
        self.definitions
            .get(&kind)
            .map(|d| d.display_name.as_str())
            .unwrap_or(kind.label())
    }

    /// 从 RON 文件加载
    pub fn load_from_file(path: &str) -> Self {
        let mut registry = AttributeRegistry::default();

        match read(path) {
            Ok(bytes) => match from_bytes::<Vec<AttributeDefinition>>(&bytes) {
                Ok(defs) => {
                    for def in defs {
                        registry.definitions.insert(def.kind, def);
                    }
                    bevy::log::info!("加载属性定义: {} 种", registry.definitions.len());
                }
                Err(e) => {
                    bevy::log::error!("解析属性定义文件 {} 失败: {}", path, e);
                    registry.register_defaults();
                }
            },
            Err(e) => {
                bevy::log::warn!("属性定义文件 {} 不存在: {}, 使用默认值", path, e);
                registry.register_defaults();
            }
        }

        registry
    }

    /// 注册内置默认属性定义（8核心 + 3资源 + 13衍生 = 24种）
    fn register_defaults(&mut self) {
        let defaults = vec![
            // ── 8 维核心属性 ──
            AttributeDefinition {
                kind: AttributeKind::Might,
                display_name: "力量".into(),
                description: "物理攻击力".into(),
                default_value: 0.0,
                min_value: 0.0,
                max_value: 999.0,
            },
            AttributeDefinition {
                kind: AttributeKind::Dexterity,
                display_name: "技巧".into(),
                description: "命中、暴击、远程".into(),
                default_value: 0.0,
                min_value: 0.0,
                max_value: 999.0,
            },
            AttributeDefinition {
                kind: AttributeKind::Agility,
                display_name: "敏捷".into(),
                description: "行动顺序、闪避、移动".into(),
                default_value: 0.0,
                min_value: 0.0,
                max_value: 999.0,
            },
            AttributeDefinition {
                kind: AttributeKind::Vitality,
                display_name: "体质".into(),
                description: "生命、物防".into(),
                default_value: 0.0,
                min_value: 0.0,
                max_value: 999.0,
            },
            AttributeDefinition {
                kind: AttributeKind::Intelligence,
                display_name: "智力".into(),
                description: "法术攻击、法力".into(),
                default_value: 0.0,
                min_value: 0.0,
                max_value: 999.0,
            },
            AttributeDefinition {
                kind: AttributeKind::Willpower,
                display_name: "意志".into(),
                description: "魔防、治疗、异常抵抗".into(),
                default_value: 0.0,
                min_value: 0.0,
                max_value: 999.0,
            },
            AttributeDefinition {
                kind: AttributeKind::Presence,
                display_name: "魅力".into(),
                description: "光环、召唤、指挥".into(),
                default_value: 0.0,
                min_value: 0.0,
                max_value: 999.0,
            },
            AttributeDefinition {
                kind: AttributeKind::Luck,
                display_name: "幸运".into(),
                description: "暴击、掉落、随机事件".into(),
                default_value: 0.0,
                min_value: 0.0,
                max_value: 999.0,
            },
            // ── 生命资源 ──
            AttributeDefinition {
                kind: AttributeKind::Hp,
                display_name: "生命值".into(),
                description: "当前生命值".into(),
                default_value: 0.0,
                min_value: 0.0,
                max_value: 9999.0,
            },
            AttributeDefinition {
                kind: AttributeKind::Mp,
                display_name: "魔法值".into(),
                description: "当前魔法值".into(),
                default_value: 0.0,
                min_value: 0.0,
                max_value: 9999.0,
            },
            AttributeDefinition {
                kind: AttributeKind::Stamina,
                display_name: "耐力值".into(),
                description: "当前耐力值".into(),
                default_value: 0.0,
                min_value: 0.0,
                max_value: 9999.0,
            },
            // ── 衍生属性 ──
            AttributeDefinition {
                kind: AttributeKind::MaxHp,
                display_name: "最大生命值".into(),
                description: "生命值上限".into(),
                default_value: 0.0,
                min_value: 0.0,
                max_value: 9999.0,
            },
            AttributeDefinition {
                kind: AttributeKind::MaxMp,
                display_name: "最大魔法值".into(),
                description: "魔法值上限".into(),
                default_value: 0.0,
                min_value: 0.0,
                max_value: 9999.0,
            },
            AttributeDefinition {
                kind: AttributeKind::MaxStamina,
                display_name: "最大耐力值".into(),
                description: "耐力值上限".into(),
                default_value: 0.0,
                min_value: 0.0,
                max_value: 9999.0,
            },
            AttributeDefinition {
                kind: AttributeKind::Attack,
                display_name: "物理攻击力".into(),
                description: "物理伤害基础".into(),
                default_value: 0.0,
                min_value: 0.0,
                max_value: 9999.0,
            },
            AttributeDefinition {
                kind: AttributeKind::Defense,
                display_name: "物理防御力".into(),
                description: "物理伤害减免".into(),
                default_value: 0.0,
                min_value: 0.0,
                max_value: 9999.0,
            },
            AttributeDefinition {
                kind: AttributeKind::MagicAttack,
                display_name: "魔法攻击力".into(),
                description: "魔法伤害基础".into(),
                default_value: 0.0,
                min_value: 0.0,
                max_value: 9999.0,
            },
            AttributeDefinition {
                kind: AttributeKind::MagicDefense,
                display_name: "魔法防御力".into(),
                description: "魔法伤害减免".into(),
                default_value: 0.0,
                min_value: 0.0,
                max_value: 9999.0,
            },
            AttributeDefinition {
                kind: AttributeKind::Accuracy,
                display_name: "命中率".into(),
                description: "攻击命中概率".into(),
                default_value: 80.0,
                min_value: 0.0,
                max_value: 9999.0,
            },
            AttributeDefinition {
                kind: AttributeKind::Evasion,
                display_name: "闪避率".into(),
                description: "躲避攻击概率".into(),
                default_value: 0.0,
                min_value: 0.0,
                max_value: 9999.0,
            },
            AttributeDefinition {
                kind: AttributeKind::CritRate,
                display_name: "暴击率".into(),
                description: "暴击触发概率".into(),
                default_value: 5.0,
                min_value: 0.0,
                max_value: 100.0,
            },
            AttributeDefinition {
                kind: AttributeKind::MoveRange,
                display_name: "移动力".into(),
                description: "每回合可移动格数".into(),
                default_value: 0.0,
                min_value: 0.0,
                max_value: 99.0,
            },
            AttributeDefinition {
                kind: AttributeKind::Initiative,
                display_name: "行动速度".into(),
                description: "决定行动顺序".into(),
                default_value: 0.0,
                min_value: 0.0,
                max_value: 9999.0,
            },
            AttributeDefinition {
                kind: AttributeKind::AttackRange,
                display_name: "攻击范围".into(),
                description: "攻击距离（格数）".into(),
                default_value: 1.0,
                min_value: 1.0,
                max_value: 99.0,
            },
        ];

        for def in defaults {
            self.definitions.insert(def.kind, def);
        }
    }
}

/// 属性定义插件
pub struct AttributeDefPlugin;

impl Plugin for AttributeDefPlugin {
    fn build(&self, app: &mut App) {
        let registry = AttributeRegistry::load_from_file("assets/definitions/attributes.ron");
        app.insert_resource(registry);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ron_反序列化_属性定义() {
        let ron_str = r#"
            [
                (kind: Hp, display_name: "生命值", description: "当前生命值", default_value: 0.0, min_value: 0.0, max_value: 9999.0),
                (kind: Attack, display_name: "物理攻击力", description: "物理伤害基础", default_value: 0.0, min_value: 0.0, max_value: 9999.0),
            ]
        "#;
        let defs: Vec<AttributeDefinition> = from_bytes(ron_str.as_bytes()).unwrap();
        assert_eq!(defs.len(), 2);
        assert_eq!(defs[0].display_name, "生命值");
        assert_eq!(defs[1].kind, AttributeKind::Attack);
    }

    #[test]
    fn attribute_registry_查询() {
        let mut registry = AttributeRegistry::default();
        registry.register_defaults();

        let def = registry.get(AttributeKind::Attack).unwrap();
        assert_eq!(def.display_name, "物理攻击力");
        assert_eq!(def.max_value, 9999.0);
    }

    #[test]
    fn attribute_registry_显示名称回退() {
        let registry = AttributeRegistry::default();
        // 没有注册定义时回退到 kind.label()
        assert_eq!(registry.display_name(AttributeKind::Hp), "HP");
    }

    #[test]
    fn attribute_registry_defaults_包含所有属性() {
        let mut registry = AttributeRegistry::default();
        registry.register_defaults();
        // 8 核心属性
        for kind in [
            AttributeKind::Might, AttributeKind::Dexterity,
            AttributeKind::Agility, AttributeKind::Vitality,
            AttributeKind::Intelligence, AttributeKind::Willpower,
            AttributeKind::Presence, AttributeKind::Luck,
        ] {
            assert!(registry.get(kind).is_some(), "缺少核心属性: {:?}", kind);
        }
        // 3 生命资源
        for kind in [
            AttributeKind::Hp, AttributeKind::Mp, AttributeKind::Stamina,
        ] {
            assert!(registry.get(kind).is_some(), "缺少生命资源: {:?}", kind);
        }
        // 12 衍生属性
        for kind in [
            AttributeKind::MaxHp, AttributeKind::MaxMp, AttributeKind::MaxStamina,
            AttributeKind::Attack, AttributeKind::Defense,
            AttributeKind::MagicAttack, AttributeKind::MagicDefense,
            AttributeKind::Accuracy, AttributeKind::Evasion,
            AttributeKind::CritRate, AttributeKind::MoveRange,
            AttributeKind::Initiative, AttributeKind::AttackRange,
        ] {
            assert!(registry.get(kind).is_some(), "缺少衍生属性: {:?}", kind);
        }
    }

    #[test]
    fn attribute_registry_defaults_总数为24() {
        let mut registry = AttributeRegistry::default();
        registry.register_defaults();
        assert_eq!(registry.definitions.len(), 24);
    }

    #[test]
    fn attribute_registry_显示名称_已注册() {
        let mut registry = AttributeRegistry::default();
        registry.register_defaults();
        assert_eq!(registry.display_name(AttributeKind::Attack), "物理攻击力");
        assert_eq!(registry.display_name(AttributeKind::Might), "力量");
        assert_eq!(registry.display_name(AttributeKind::Defense), "物理防御力");
    }

    #[test]
    fn attribute_registry_查询所有默认属性() {
        let mut registry = AttributeRegistry::default();
        registry.register_defaults();
        // 核心属性
        assert_eq!(registry.get(AttributeKind::Might).unwrap().max_value, 999.0);
        // 生命资源
        assert_eq!(registry.get(AttributeKind::Hp).unwrap().max_value, 9999.0);
        assert_eq!(registry.get(AttributeKind::Mp).unwrap().max_value, 9999.0);
        assert_eq!(registry.get(AttributeKind::Stamina).unwrap().max_value, 9999.0);
        // 衍生属性
        assert_eq!(registry.get(AttributeKind::MaxHp).unwrap().max_value, 9999.0);
        assert_eq!(registry.get(AttributeKind::Defense).unwrap().max_value, 9999.0);
        assert_eq!(registry.get(AttributeKind::MoveRange).unwrap().max_value, 99.0);
        assert_eq!(registry.get(AttributeKind::AttackRange).unwrap().default_value, 1.0);
        assert_eq!(registry.get(AttributeKind::Accuracy).unwrap().default_value, 80.0);
        assert_eq!(registry.get(AttributeKind::CritRate).unwrap().max_value, 100.0);
    }
}
