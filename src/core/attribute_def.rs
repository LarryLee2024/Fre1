// 属性定义注册表：属性元数据外部化，支持 UI 自动生成
// AttributeKind 仍为枚举（运行时类型安全），显示元数据从 RON 加载

use crate::core::attribute::AttributeKind;
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

    /// 注册内置默认属性定义
    fn register_defaults(&mut self) {
        let defaults = vec![
            AttributeDefinition {
                kind: AttributeKind::Hp,
                display_name: "生命值".into(),
                description: "当前生命值".into(),
                default_value: 0.0,
                min_value: 0.0,
                max_value: 9999.0,
            },
            AttributeDefinition {
                kind: AttributeKind::MaxHp,
                display_name: "最大生命值".into(),
                description: "生命值上限".into(),
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
                kind: AttributeKind::MaxMp,
                display_name: "最大魔法值".into(),
                description: "魔法值上限".into(),
                default_value: 0.0,
                min_value: 0.0,
                max_value: 9999.0,
            },
            AttributeDefinition {
                kind: AttributeKind::Atk,
                display_name: "攻击力".into(),
                description: "物理攻击力".into(),
                default_value: 0.0,
                min_value: 0.0,
                max_value: 9999.0,
            },
            AttributeDefinition {
                kind: AttributeKind::Def,
                display_name: "防御力".into(),
                description: "物理防御力".into(),
                default_value: 0.0,
                min_value: 0.0,
                max_value: 9999.0,
            },
            AttributeDefinition {
                kind: AttributeKind::Mov,
                display_name: "移动力".into(),
                description: "每回合可移动格数".into(),
                default_value: 0.0,
                min_value: 0.0,
                max_value: 99.0,
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
                (kind: Atk, display_name: "攻击力", description: "物理攻击力", default_value: 0.0, min_value: 0.0, max_value: 9999.0),
            ]
        "#;
        let defs: Vec<AttributeDefinition> = from_bytes(ron_str.as_bytes()).unwrap();
        assert_eq!(defs.len(), 2);
        assert_eq!(defs[0].display_name, "生命值");
        assert_eq!(defs[1].kind, AttributeKind::Atk);
    }

    #[test]
    fn attribute_registry_查询() {
        let mut registry = AttributeRegistry::default();
        registry.register_defaults();

        let def = registry.get(AttributeKind::Atk).unwrap();
        assert_eq!(def.display_name, "攻击力");
        assert_eq!(def.max_value, 9999.0);
    }

    #[test]
    fn attribute_registry_显示名称回退() {
        let registry = AttributeRegistry::default();
        // 没有注册定义时回退到 kind.label()
        assert_eq!(registry.display_name(AttributeKind::Hp), "HP");
    }
}
