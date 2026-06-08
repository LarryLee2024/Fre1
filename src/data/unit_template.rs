// 单位模板：数据驱动的单位定义，替代硬编码数组
// 支持从 assets/units/*.ron 外部配置文件加载

use crate::core::attribute::AttributeKind;
use crate::core::tag::{GameplayTag, TagName};
use crate::unit::Faction;
use bevy::prelude::*;
use ron::de::from_bytes;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::{read, read_dir};

/// 单位模板（运行时）
#[derive(Clone, Debug)]
pub struct UnitTemplate {
    pub id: String,
    pub name: String,
    pub faction: Faction,
    pub class_tag: GameplayTag,
    pub base_attributes: HashMap<AttributeKind, f32>,
    pub skill_ids: Vec<String>,
}

/// 单位模板（RON 反序列化用）
#[derive(Clone, Debug, Deserialize)]
pub struct UnitTemplateDef {
    pub id: String,
    pub name: String,
    pub faction: FactionDef,
    pub class_tag: TagName,
    pub base_attributes: HashMap<AttributeKind, f32>,
    pub skill_ids: Vec<String>,
}

/// 阵营定义（RON 反序列化用）
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum FactionDef {
    Player,
    Enemy,
}

impl From<FactionDef> for Faction {
    fn from(def: FactionDef) -> Self {
        match def {
            FactionDef::Player => Faction::Player,
            FactionDef::Enemy => Faction::Enemy,
        }
    }
}

impl From<UnitTemplateDef> for UnitTemplate {
    fn from(def: UnitTemplateDef) -> Self {
        UnitTemplate {
            id: def.id,
            name: def.name,
            faction: def.faction.into(),
            class_tag: def.class_tag.to_tag(),
            base_attributes: def.base_attributes,
            skill_ids: def.skill_ids,
        }
    }
}

/// 单位模板注册表资源
#[derive(Resource, Default)]
pub struct UnitTemplateRegistry {
    pub templates: HashMap<String, UnitTemplate>,
}

impl UnitTemplateRegistry {
    pub fn get(&self, id: &str) -> Option<&UnitTemplate> {
        self.templates.get(id)
    }

    /// 从 assets/units/ 目录加载所有 .ron 文件
    pub fn load_from_dir(dir: &str) -> Self {
        let mut registry = UnitTemplateRegistry::default();
        let Ok(entries) = read_dir(dir) else {
            bevy::log::warn!("单位模板目录不存在: {}", dir);
            return registry;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "ron") {
                match read(&path) {
                    Ok(bytes) => match from_bytes::<UnitTemplateDef>(&bytes) {
                        Ok(def) => {
                            let id = def.id.clone();
                            registry.templates.insert(id.clone(), def.into());
                            bevy::log::info!("加载单位模板: {}", id);
                        }
                        Err(e) => {
                            bevy::log::error!("解析单位模板文件 {:?} 失败: {}", path, e);
                        }
                    },
                    Err(e) => {
                        bevy::log::error!("读取单位模板文件 {:?} 失败: {}", path, e);
                    }
                }
            }
        }
        registry
    }
}

/// 单位模板插件
pub struct UnitTemplatePlugin;

impl Plugin for UnitTemplatePlugin {
    fn build(&self, app: &mut App) {
        let registry = UnitTemplateRegistry::load_from_dir("assets/units");
        app.insert_resource(registry);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ron_反序列化_单位模板() {
        let ron_str = r#"
            (
                id: "player_warrior",
                name: "战士",
                faction: Player,
                class_tag: WARRIOR,
                base_attributes: {
                    Hp: 30.0, MaxHp: 30.0,
                    Atk: 10.0, Def: 5.0,
                    Mov: 5.0, AttackRange: 1.0,
                },
                skill_ids: ["basic_attack", "charge"],
            )
        "#;
        let def: UnitTemplateDef = from_bytes(ron_str.as_bytes()).unwrap();
        assert_eq!(def.id, "player_warrior");
        assert_eq!(def.faction, FactionDef::Player);
        assert_eq!(def.class_tag, TagName::Warrior);
        assert_eq!(def.skill_ids, vec!["basic_attack", "charge"]);
    }

    #[test]
    fn unit_template_def_转换为_unit_template() {
        let def = UnitTemplateDef {
            id: "test".into(),
            name: "测试".into(),
            faction: FactionDef::Enemy,
            class_tag: TagName::Mage,
            base_attributes: {
                let mut m = HashMap::new();
                m.insert(AttributeKind::Hp, 20.0);
                m.insert(AttributeKind::MaxHp, 20.0);
                m
            },
            skill_ids: vec!["basic_attack".into()],
        };
        let template: UnitTemplate = def.into();
        assert_eq!(template.id, "test");
        assert_eq!(template.faction, Faction::Enemy);
        assert_eq!(template.class_tag, GameplayTag::MAGE);
    }
}
