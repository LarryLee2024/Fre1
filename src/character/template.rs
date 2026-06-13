// 单位模板：数据驱动的单位定义，替代硬编码数组
// 支持从 assets/units/*.ron 外部配置文件加载

use super::components::Faction;
use crate::core::attribute::AttributeKind;
use crate::core::registry_loader::RegistryLoader;
use crate::equipment::EquipmentSlot;
use crate::skill::BASIC_ATTACK_ID;
use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

/// 单位模板（运行时）
#[derive(Clone, Debug)]
#[allow(dead_code)]
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
    /// 初始装备：槽位 → 装备定义 ID
    pub initial_equipment: Vec<(EquipmentSlot, String)>,
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
    /// 初始装备：槽位 → 装备定义 ID
    #[serde(default)]
    pub initial_equipment: Vec<(EquipmentSlot, String)>,
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
            initial_equipment: def.initial_equipment,
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
                initial_equipment: vec![
                    (EquipmentSlot::MainHand, "iron_sword".into()),
                    (EquipmentSlot::Body, "leather_armor".into()),
                ],
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
                initial_equipment: vec![(EquipmentSlot::MainHand, "iron_sword".into())],
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
                initial_equipment: vec![],
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
                initial_equipment: vec![
                    (EquipmentSlot::MainHand, "iron_sword".into()),
                    (EquipmentSlot::Body, "leather_armor".into()),
                    (EquipmentSlot::OffHand, "iron_shield".into()),
                ],
            },
        );
    }
}

impl RegistryLoader for UnitTemplateRegistry {
    type Item = UnitTemplateDef;

    fn register_item(&mut self, item: UnitTemplateDef) {
        let id = item.id.clone();
        self.register(item.into());
        bevy::log::info!(target: "character", event = "unit_template_loaded", id = %id, "单位模板已加载");
    }

    fn register_defaults(&mut self) {
        UnitTemplateRegistry::register_defaults(self);
    }

    fn is_empty(&self) -> bool {
        self.templates.is_empty()
    }

    fn registry_name() -> &'static str {
        "单位模板"
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
    // ================================================
    // AI Self-Check (test_spec.md §13.1)
    // ================================================
    // ✅ 测试行为，不是实现
    // ✅ 符合领域规则
    // ✅ 测试是确定性的
    // ✅ 使用标准测试数据
    // ✅ 没有测试私有实现
    // ✅ 没有生成不在范围内的测试
    // ================================================

    use super::*;
    use ron::de::from_bytes;

    /// Test ID: CHR-TPL-001
    /// Title: RON 反序列化单位模板
    ///
    /// Given: 有效的 RON 字符串
    /// When: 反序列化为 UnitTemplateDef
    /// Then: 所有字段正确解析
    ///
    /// Assertions: id, faction, race, class, skill_ids, trait_ids 正确
    #[test]
    fn ron_deserialize_unit_template() {
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

    /// Test ID: CHR-TPL-002
    /// Title: UnitTemplateDef 转换为 UnitTemplate
    ///
    /// Given: 一个 UnitTemplateDef 实例
    /// When: 调用 .into() 转换为 UnitTemplate
    /// Then: 所有字段正确转换
    ///
    /// Assertions: id, faction, race, class, trait_ids, ai_behavior 正确
    #[test]
    fn unit_template_def_converts_to_unit_template() {
        // Given
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
            initial_equipment: vec![],
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

    /// Test ID: CHR-TPL-003
    /// Title: UnitTemplateRegistry 默认注册 4 个模板
    ///
    /// Given: 一个空的 UnitTemplateRegistry
    /// When: 调用 register_defaults()
    /// Then: 注册 4 个默认模板
    ///
    /// Assertions: 4 个模板均存在
    #[test]
    fn unit_template_registry_default_templates() {
        // Given
        let mut registry = UnitTemplateRegistry::default();

        // When
        registry.register_defaults();

        // Then
        assert!(registry.get("player_warrior").is_some());
        assert!(registry.get("player_archer").is_some());
        assert!(registry.get("enemy_goblin").is_some());
        assert!(registry.get("enemy_dark_knight").is_some());
    }

    /// Test ID: CHR-TPL-004
    /// Title: UnitTemplateRegistry 查询已注册模板
    ///
    /// Given: 已注册默认模板的 Registry
    /// When: 查询 "player_warrior"
    /// Then: 返回正确的模板信息
    ///
    /// Assertions: name, faction, base_attack_range 正确
    #[test]
    fn unit_template_registry_query() {
        // Given
        let mut registry = UnitTemplateRegistry::default();
        registry.register_defaults();

        // When
        let warrior = registry.get("player_warrior").unwrap();

        // Then
        assert_eq!(warrior.name, "战士");
        assert_eq!(warrior.faction, Faction::Player);
        assert_eq!(warrior.base_attack_range, 1);
    }

    /// Test ID: CHR-TPL-005
    /// Title: UnitTemplateRegistry 查询未注册模板返回 None
    ///
    /// Given: 已注册默认模板的 Registry
    /// When: 查询 "nonexistent"
    /// Then: 返回 None
    ///
    /// Assertions: result.is_none()
    #[test]
    fn unit_template_registry_query_unregistered_returns_none() {
        // Given
        let mut registry = UnitTemplateRegistry::default();
        registry.register_defaults();

        // When
        let result = registry.get("nonexistent");

        // Then
        assert!(result.is_none());
    }

    /// Test ID: CHR-TPL-006
    /// Title: FactionDef::Player 转换为 Faction::Player
    ///
    /// Given: FactionDef::Player
    /// When: 调用 .into() 转换
    /// Then: 返回 Faction::Player
    ///
    /// Assertions: faction == Faction::Player
    #[test]
    fn faction_def_player_converts() {
        // Given
        let def = FactionDef::Player;

        // When
        let faction: Faction = def.into();

        // Then
        assert_eq!(faction, Faction::Player);
    }

    /// Test ID: CHR-TPL-007
    /// Title: FactionDef::Enemy 转换为 Faction::Enemy
    ///
    /// Given: FactionDef::Enemy
    /// When: 调用 .into() 转换
    /// Then: 返回 Faction::Enemy
    ///
    /// Assertions: faction == Faction::Enemy
    #[test]
    fn faction_def_enemy_converts() {
        // Given
        let def = FactionDef::Enemy;

        // When
        let faction: Faction = def.into();

        // Then
        assert_eq!(faction, Faction::Enemy);
    }

    /// Test ID: CHR-TPL-008
    /// Title: RON 反序列化旧配置（无 version 字段）兼容
    ///
    /// Given: 不含 version 字段的 RON 字符串
    /// When: 反序列化为 UnitTemplateDef
    /// Then: version 默认为 0
    ///
    /// Assertions: id == "old_unit", version == 0
    #[test]
    fn ron_deserialize_old_config_without_version() {
        // Given
        let ron_str = format!(
            r#"
            (
                id: "old_unit",
                name: "旧单位",
                race: "人类",
                background: "无",
                class: "战士",
                faction: Player,
                base_attributes: {{}},
                base_attack_range: 1,
                skill_ids: [],
                trait_ids: [],
                ai_behavior: "default",
            )
        "#
        );

        // When
        let def: UnitTemplateDef = from_bytes(ron_str.as_bytes()).unwrap();

        // Then
        assert_eq!(def.id, "old_unit");
        assert_eq!(def.version, 0);
    }
}
