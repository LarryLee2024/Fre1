// 单位模板：数据驱动的单位定义，替代硬编码数组
// 支持从 assets/units/*.ron 外部配置文件加载

use super::components::Faction;
use crate::core::attribute::AttributeKind;
use crate::skill::BASIC_ATTACK_ID;
use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

/// 单位模板（运行时）
#[derive(Clone, Debug)]
pub struct UnitTemplate {
    pub id: String,
    pub name: String,
    pub faction: Faction,
    pub race: String,
    pub background: String,
    pub class: String,
    /// 核心属性基础值（仅8维核心属性）
    pub base_attributes: HashMap<AttributeKind, f32>,
    /// 基础攻击范围（由职业/装备决定）
    pub base_attack_range: u32,
    pub skill_ids: Vec<String>,
    pub trait_ids: Vec<String>,
    pub ai_behavior: String,
}

/// 单位模板（RON 反序列化用）
#[derive(Clone, Debug, Deserialize)]
pub struct UnitTemplateDef {
    #[serde(default)]
    pub version: u32,
    pub id: String,
    pub name: String,
    pub faction: FactionDef,
    pub race: String,
    pub background: String,
    pub class: String,
    /// 核心属性基础值（仅8维核心属性，RON 中使用 PascalCase 名如 Might: 5.0）
    pub base_attributes: HashMap<AttributeKind, f32>,
    pub base_attack_range: u32,
    pub skill_ids: Vec<String>,
    pub trait_ids: Vec<String>,
    pub ai_behavior: String,
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
            race: def.race,
            background: def.background,
            class: def.class,
            base_attributes: def.base_attributes,
            base_attack_range: def.base_attack_range,
            skill_ids: def.skill_ids,
            trait_ids: def.trait_ids,
            ai_behavior: def.ai_behavior,
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

    /// 注册一个单位模板
    pub fn register(&mut self, template: UnitTemplate) {
        self.templates.insert(template.id.clone(), template);
    }

    /// 从 assets/units/ 目录加载所有 .ron 文件
    pub fn load_from_dir(dir: &str) -> Self {
        let mut registry = UnitTemplateRegistry::default();
        let (defs, loaded) =
            crate::core::loader::load_dir_single::<UnitTemplateDef>(dir, "单位模板");
        for def in defs {
            let id = def.id.clone();
            registry.register(def.into());
            bevy::log::info!("加载单位模板: {}", id);
        }
        if !loaded {
            registry.register_defaults();
        }
        registry
    }

    /// 注册内置默认单位模板（确保基础功能可用）
    fn register_defaults(&mut self) {
        if !self.templates.is_empty() {
            return;
        }
        // 战士
        let warrior_attrs: HashMap<AttributeKind, f32> = {
            let mut m = HashMap::new();
            m.insert(AttributeKind::Might, 5.0);
            m.insert(AttributeKind::Dexterity, 3.0);
            m.insert(AttributeKind::Agility, 6.0);
            m.insert(AttributeKind::Vitality, 5.0);
            m.insert(AttributeKind::Intelligence, 2.0);
            m.insert(AttributeKind::Willpower, 3.0);
            m.insert(AttributeKind::Presence, 2.0);
            m.insert(AttributeKind::Luck, 2.0);
            m
        };
        self.templates.insert(
            "player_warrior".into(),
            UnitTemplate {
                id: "player_warrior".into(),
                name: "战士".into(),
                faction: Faction::Player,
                race: "人类".into(),
                background: "士兵".into(),
                class: "战士".into(),
                base_attributes: warrior_attrs,
                base_attack_range: 1,
                skill_ids: vec![BASIC_ATTACK_ID.into(), "charge".into()],
                trait_ids: vec!["warrior_mastery".into()],
                ai_behavior: "default".into(),
            },
        );

        // 弓手
        let archer_attrs: HashMap<AttributeKind, f32> = {
            let mut m = HashMap::new();
            m.insert(AttributeKind::Might, 4.0);
            m.insert(AttributeKind::Dexterity, 6.0);
            m.insert(AttributeKind::Agility, 6.0);
            m.insert(AttributeKind::Vitality, 3.0);
            m.insert(AttributeKind::Intelligence, 3.0);
            m.insert(AttributeKind::Willpower, 2.0);
            m.insert(AttributeKind::Presence, 2.0);
            m.insert(AttributeKind::Luck, 3.0);
            m
        };
        self.templates.insert(
            "player_archer".into(),
            UnitTemplate {
                id: "player_archer".into(),
                name: "弓手".into(),
                faction: Faction::Player,
                race: "人类".into(),
                background: "猎人".into(),
                class: "弓手".into(),
                base_attributes: archer_attrs,
                base_attack_range: 3,
                skill_ids: vec![BASIC_ATTACK_ID.into()],
                trait_ids: vec!["archer_mastery".into()],
                ai_behavior: "default".into(),
            },
        );

        // 哥布林
        let goblin_attrs: HashMap<AttributeKind, f32> = {
            let mut m = HashMap::new();
            m.insert(AttributeKind::Might, 4.0);
            m.insert(AttributeKind::Dexterity, 2.0);
            m.insert(AttributeKind::Agility, 4.0);
            m.insert(AttributeKind::Vitality, 3.0);
            m.insert(AttributeKind::Intelligence, 1.0);
            m.insert(AttributeKind::Willpower, 2.0);
            m.insert(AttributeKind::Presence, 1.0);
            m.insert(AttributeKind::Luck, 2.0);
            m
        };
        self.templates.insert(
            "enemy_goblin".into(),
            UnitTemplate {
                id: "enemy_goblin".into(),
                name: "哥布林".into(),
                faction: Faction::Enemy,
                race: "哥布林".into(),
                background: "部落".into(),
                class: "战士".into(),
                base_attributes: goblin_attrs,
                base_attack_range: 1,
                skill_ids: vec![BASIC_ATTACK_ID.into()],
                trait_ids: vec!["warrior_mastery".into()],
                ai_behavior: "aggressive".into(),
            },
        );

        // 暗黑骑士
        let dark_knight_attrs: HashMap<AttributeKind, f32> = {
            let mut m = HashMap::new();
            m.insert(AttributeKind::Might, 6.0);
            m.insert(AttributeKind::Dexterity, 3.0);
            m.insert(AttributeKind::Agility, 4.0);
            m.insert(AttributeKind::Vitality, 6.0);
            m.insert(AttributeKind::Intelligence, 2.0);
            m.insert(AttributeKind::Willpower, 4.0);
            m.insert(AttributeKind::Presence, 3.0);
            m.insert(AttributeKind::Luck, 2.0);
            m
        };
        self.templates.insert(
            "enemy_dark_knight".into(),
            UnitTemplate {
                id: "enemy_dark_knight".into(),
                name: "暗黑骑士".into(),
                faction: Faction::Enemy,
                race: "亡灵".into(),
                background: "堕落骑士".into(),
                class: "战士".into(),
                base_attributes: dark_knight_attrs,
                base_attack_range: 1,
                skill_ids: vec![BASIC_ATTACK_ID.into()],
                trait_ids: vec!["warrior_mastery".into(), "heavy_armor".into()],
                ai_behavior: "aggressive".into(),
            },
        );
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
    use ron::de::from_bytes;

    #[test]
    fn ron_反序列化_单位模板() {
        let ron_str = format!(
            r#"
            (
                id: "player_warrior",
                name: "战士",
                faction: Player,
                race: "人类",
                background: "士兵",
                class: "战士",
                base_attributes: {{
                    Might: 5.0, Dexterity: 3.0, Agility: 6.0,
                    Vitality: 5.0, Intelligence: 2.0, Willpower: 3.0,
                    Presence: 2.0, Luck: 2.0,
                }},
                base_attack_range: 1,
                skill_ids: ["{}", "charge"],
                trait_ids: ["warrior_mastery"],
                ai_behavior: "default",
            )
        "#,
            BASIC_ATTACK_ID
        );
        let def: UnitTemplateDef = from_bytes(ron_str.as_bytes()).unwrap();
        assert_eq!(def.id, "player_warrior");
        assert_eq!(def.faction, FactionDef::Player);
        assert_eq!(def.race, "人类");
        assert_eq!(def.class, "战士");
        assert_eq!(def.base_attack_range, 1);
        assert_eq!(def.skill_ids, vec![BASIC_ATTACK_ID, "charge"]);
        assert_eq!(def.trait_ids, vec!["warrior_mastery"]);
        assert_eq!(def.ai_behavior, "default");
    }

    #[test]
    fn unit_template_def_转换为_unit_template() {
        let def = UnitTemplateDef {
            version: 0,
            id: "test".into(),
            name: "测试".into(),
            faction: FactionDef::Enemy,
            race: "哥布林".into(),
            background: "部落".into(),
            class: "战士".into(),
            base_attributes: {
                let mut m = HashMap::new();
                m.insert(AttributeKind::Might, 4.0);
                m.insert(AttributeKind::Vitality, 3.0);
                m
            },
            base_attack_range: 1,
            skill_ids: vec![BASIC_ATTACK_ID.into()],
            trait_ids: vec!["warrior_mastery".into()],
            ai_behavior: "aggressive".into(),
        };
        let template: UnitTemplate = def.into();
        assert_eq!(template.id, "test");
        assert_eq!(template.faction, Faction::Enemy);
        assert_eq!(template.race, "哥布林");
        assert_eq!(template.class, "战士");
        assert_eq!(template.base_attack_range, 1);
        assert_eq!(template.trait_ids, vec!["warrior_mastery"]);
        assert_eq!(template.ai_behavior, "aggressive");
    }

    #[test]
    fn unit_template_registry_默认模板() {
        let mut registry = UnitTemplateRegistry::default();
        registry.register_defaults();
        assert!(registry.get("player_warrior").is_some());
        assert!(registry.get("player_archer").is_some());
        assert!(registry.get("enemy_goblin").is_some());
        assert!(registry.get("enemy_dark_knight").is_some());
    }

    #[test]
    fn unit_template_registry_查询() {
        let mut registry = UnitTemplateRegistry::default();
        registry.register_defaults();
        let warrior = registry.get("player_warrior").unwrap();
        assert_eq!(warrior.name, "战士");
        assert_eq!(warrior.faction, Faction::Player);
        assert_eq!(warrior.base_attack_range, 1);
    }

    #[test]
    fn unit_template_registry_查询未注册返回none() {
        let mut registry = UnitTemplateRegistry::default();
        registry.register_defaults();
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn faction_def_player转换() {
        let faction: Faction = FactionDef::Player.into();
        assert_eq!(faction, Faction::Player);
    }

    #[test]
    fn faction_def_enemy转换() {
        let faction: Faction = FactionDef::Enemy.into();
        assert_eq!(faction, Faction::Enemy);
    }
}
