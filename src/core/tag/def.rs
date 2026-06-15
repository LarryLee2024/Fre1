// 标签定义注册表：标签元数据外部化，支持 UI 自动生成和标签查询
// GameplayTag 仍为位掩码（运行时 O(1) 查询），显示元数据从 RON 加载

use crate::core::registry_loader::RegistryLoader;
use crate::core::tag::{GameplayTag, TagName};
use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

/// 标签分类
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum TagCategory {
    Element,
    Status,
    Weapon,
    WeaponType,
    Class,
    Movement,
    SkillType,
    BuffType,
    ItemType,
    EquipmentAttribute,
}

impl TagCategory {
    /// 根据 TagName 返回默认分类（RON 缺失时的回退）
    pub fn default_for(name: &TagName) -> Self {
        match name {
            TagName::Fire | TagName::Ice | TagName::Poison => Self::Element,
            TagName::Stun | TagName::Burn | TagName::Regen => Self::Status,
            TagName::Melee | TagName::Ranged => Self::Weapon,
            TagName::Sword | TagName::Axe | TagName::Bow | TagName::Staff => Self::WeaponType,
            TagName::Warrior | TagName::Archer | TagName::Mage => Self::Class,
            TagName::Flying | TagName::Mounted | TagName::Swimming => Self::Movement,
            TagName::SkillActive | TagName::SkillPassive => Self::SkillType,
            TagName::Buff | TagName::Debuff => Self::BuffType,
            TagName::Consumable
            | TagName::Ammo
            | TagName::Material
            | TagName::Currency
            | TagName::QuestItem
            | TagName::Healing
            | TagName::Potion
            | TagName::Scroll
            | TagName::Food => Self::ItemType,
            TagName::HeavyArmor
            | TagName::LightArmor
            | TagName::Shield
            | TagName::TwoHanded
            | TagName::Martial
            | TagName::Simple => Self::EquipmentAttribute,
        }
    }
}

/// 标签定义（RON 反序列化用）
#[derive(Clone, Debug, Deserialize)]
pub struct TagDefinition {
    /// 配置版本号（预留，用于未来存档兼容性检查）
    #[serde(default)]
    pub version: u32,
    pub tag: TagName,
    /// 旧字段：直接文本（向后兼容）
    #[serde(default)]
    pub display_name: String,
    /// 旧字段：直接文本（向后兼容）
    #[serde(default)]
    pub description: String,
    /// 新字段：本地化 Key（优先使用）
    #[serde(default)]
    pub display_name_key: Option<String>,
    /// 新字段：本地化 Key（优先使用）
    #[serde(default)]
    pub desc_key: Option<String>,
    pub category: TagCategory,
}

/// 标签注册表资源
#[derive(Resource, Default)]
pub struct TagRegistry {
    pub definitions: HashMap<GameplayTag, TagDefinition>,
}

/// GameplayTag 默认显示名（RON 缺失时的回退）
fn default_tag_display_name(tag: GameplayTag) -> &'static str {
    match tag {
        GameplayTag::FIRE => "火焰",
        GameplayTag::ICE => "冰霜",
        GameplayTag::POISON => "毒素",
        GameplayTag::STUN => "晕眩",
        GameplayTag::BURN => "燃烧",
        GameplayTag::REGEN => "恢复",
        GameplayTag::MELEE => "近战",
        GameplayTag::RANGED => "远程",
        GameplayTag::WARRIOR => "战士",
        GameplayTag::ARCHER => "弓手",
        GameplayTag::MAGE => "法师",
        GameplayTag::FLYING => "飞行",
        GameplayTag::MOUNTED => "骑兵",
        GameplayTag::SWIMMING => "水生",
        GameplayTag::CONSUMABLE => "消耗品",
        GameplayTag::AMMO => "弹药",
        GameplayTag::MATERIAL => "材料",
        GameplayTag::CURRENCY => "货币",
        GameplayTag::QUEST_ITEM => "任务物品",
        GameplayTag::HEALING => "治疗",
        GameplayTag::POTION => "药水",
        GameplayTag::SCROLL => "卷轴",
        GameplayTag::FOOD => "食物",
        GameplayTag::SKILL_ACTIVE => "主动技能",
        GameplayTag::SKILL_PASSIVE => "被动技能",
        GameplayTag::BUFF => "增益",
        GameplayTag::DEBUFF => "减益",
        GameplayTag::HEAVY_ARMOR => "重甲",
        GameplayTag::LIGHT_ARMOR => "轻甲",
        GameplayTag::SHIELD => "盾牌",
        GameplayTag::TWO_HANDED => "双手",
        GameplayTag::MARTIAL => "军用",
        GameplayTag::SIMPLE => "简易",
        GameplayTag::SWORD => "剑",
        GameplayTag::AXE => "斧",
        GameplayTag::BOW => "弓",
        GameplayTag::STAFF => "法杖",
        _ => "未知",
    }
}

impl TagRegistry {
    pub fn get(&self, tag: GameplayTag) -> Option<&TagDefinition> {
        self.definitions.get(&tag)
    }

    /// 获取标签显示名称（找不到则回退到默认显示名）
    pub fn display_name(&self, tag: GameplayTag) -> &str {
        self.definitions
            .get(&tag)
            .map(|d| d.display_name.as_str())
            .unwrap_or(default_tag_display_name(tag))
    }

    /// 按分类查询标签
    pub fn tags_by_category(&self, category: TagCategory) -> Vec<GameplayTag> {
        self.definitions
            .iter()
            .filter(|(_, def)| def.category == category)
            .map(|(tag, _)| *tag)
            .collect()
    }

    /// 从 TagName 枚举生成默认元数据（RON 缺失时的回退方案）
    pub fn from_enum_defaults() -> Self {
        let mut registry = Self::default();
        for name in TagName::ALL {
            let tag = name.to_tag();
            registry.definitions.insert(
                tag,
                TagDefinition {
                    version: 0,
                    tag: *name,
                    display_name: default_tag_display_name(tag).to_string(),
                    description: String::new(),
                    display_name_key: None,
                    desc_key: None,
                    category: TagCategory::default_for(name),
                },
            );
        }
        registry
    }

    /// 检查 RON 中是否有缺失的 TagName 定义
    pub fn ron_missing_tags(&self) -> Vec<GameplayTag> {
        TagName::ALL
            .iter()
            .filter(|name| !self.definitions.contains_key(&name.to_tag()))
            .map(|name| name.to_tag())
            .collect()
    }

    /// 检查位掩码使用率（≥80% 时返回 true）
    pub fn bit_mask_usage_warning(&self) -> bool {
        let used_bits = GameplayTag::used_bits();
        used_bits >= 51 // 64 * 0.8 = 51.2
    }
}

impl RegistryLoader for TagRegistry {
    type Item = TagDefinition;

    fn register_item(&mut self, item: TagDefinition) {
        self.definitions.insert(item.tag.to_tag(), item);
    }

    fn register_defaults(&mut self) {
        if !self.definitions.is_empty() {
            return;
        }
        // 回退到从枚举生成默认数据
        let defaults = Self::from_enum_defaults();
        self.definitions = defaults.definitions;
    }

    fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }

    fn registry_name() -> &'static str {
        "标签定义"
    }
}

/// 标签定义插件
pub struct TagDefPlugin;

impl Plugin for TagDefPlugin {
    fn build(&self, app: &mut App) {
        let mut registry = TagRegistry::load_from_file("content/definitions/tags.ron");

        // 校验：RON 覆盖所有 TagName 变体
        let missing = registry.ron_missing_tags();
        if !missing.is_empty() {
            bevy::log::warn!(
                target: "core",
                event = "tag_coverage_missing",
                missing_tags = ?missing,
                "以下标签在 RON 中缺少定义，将使用默认元数据"
            );
        }

        // 校验：位掩码使用率
        if registry.bit_mask_usage_warning() {
            bevy::log::warn!(
                target: "core",
                event = "tag_bitmask_warning",
                used_bits = GameplayTag::used_bits(),
                "GameplayTag 位掩码使用率已超过 80%，请准备迁移到分层标签系统"
            );
        }

        app.insert_resource(registry);
    }
}

#[cfg(test)]
mod tests {
    // ================================================
    // Bevy SRPG AI宪法 v1.1 自检结果（测试专用）
    // ================================================
    // ✅ 测行为不测实现：是 — 断言验证标签定义查询，不验证内部存储
    // ✅ 符合领域规则：是 — 覆盖 INV-REG-5 标签注册表不变量
    // ✅ 确定性：是 — 硬编码标签定义
    // ✅ 使用标准数据：是 — 使用标准 TagRegistry
    // ✅ 无越界测试：是 — 仅测试公共 API
    // ✅ 未测试私有实现：是 — 仅通过 pub 接口测试
    // ================================================
    use super::*;
    use ron::de::from_bytes;

    #[test]
    fn ron_反序列化_标签定义() {
        let ron_str = r#"
            [
                (tag: FIRE, display_name: "火焰", description: "火属性", category: Element),
                (tag: WARRIOR, display_name: "战士", description: "战士职业", category: Class),
            ]
        "#;
        let defs: Vec<TagDefinition> = from_bytes(ron_str.as_bytes()).unwrap();
        assert_eq!(defs.len(), 2);
        assert_eq!(defs[0].display_name, "火焰");
        assert_eq!(defs[1].category, TagCategory::Class);
    }

    #[test]
    fn tag_registry_查询() {
        let registry = TagRegistry::from_enum_defaults();

        let def = registry.get(GameplayTag::FIRE).unwrap();
        assert_eq!(def.display_name, "火焰");
        assert_eq!(def.category, TagCategory::Element);
    }

    #[test]
    fn tag_registry_按分类查询() {
        let registry = TagRegistry::from_enum_defaults();

        let elements = registry.tags_by_category(TagCategory::Element);
        assert_eq!(elements.len(), 3); // Fire, Ice, Poison
        assert!(elements.contains(&GameplayTag::FIRE));
    }

    #[test]
    fn tag_registry_显示名称回退() {
        let registry = TagRegistry::default();
        assert_eq!(registry.display_name(GameplayTag::FIRE), "火焰");
    }

    #[test]
    fn tag_registry_from_enum_defaults_覆盖所有标签() {
        let registry = TagRegistry::from_enum_defaults();
        let missing = registry.ron_missing_tags();
        assert!(missing.is_empty(), "from_enum_defaults 应覆盖所有 TagName");
    }

    #[test]
    fn tag_registry_new_categories() {
        let registry = TagRegistry::from_enum_defaults();
        // WeaponType
        let weapon_types = registry.tags_by_category(TagCategory::WeaponType);
        assert_eq!(weapon_types.len(), 4); // SWORD, AXE, BOW, STAFF
        // ItemType
        let item_types = registry.tags_by_category(TagCategory::ItemType);
        assert_eq!(item_types.len(), 9); // 5 main + 4 consumable subtypes
        // EquipmentAttribute
        let equip_attrs = registry.tags_by_category(TagCategory::EquipmentAttribute);
        assert_eq!(equip_attrs.len(), 6); // HEAVY_ARMOR, LIGHT_ARMOR, SHIELD, TWO_HANDED, MARTIAL, SIMPLE
    }

    #[test]
    fn tag_category_default_for() {
        assert_eq!(
            TagCategory::default_for(&TagName::Fire),
            TagCategory::Element
        );
        assert_eq!(
            TagCategory::default_for(&TagName::Sword),
            TagCategory::WeaponType
        );
        assert_eq!(
            TagCategory::default_for(&TagName::Consumable),
            TagCategory::ItemType
        );
        assert_eq!(
            TagCategory::default_for(&TagName::HeavyArmor),
            TagCategory::EquipmentAttribute
        );
    }

    #[test]
    fn tag_name_all_count() {
        assert_eq!(TagName::ALL.len(), 37, "TagName::ALL 应包含 37 个变体");
    }

    #[test]
    fn ron_coverage_all_tags() {
        let registry = TagRegistry::load_from_file("content/definitions/tags.ron");
        let missing = registry.ron_missing_tags();
        assert!(
            missing.is_empty(),
            "以下标签在 RON 中缺少定义: {:?}",
            missing
        );
    }

    #[test]
    fn tag_category_coverage_all_tags() {
        let registry = TagRegistry::load_from_file("content/definitions/tags.ron");
        let all_categories = [
            TagCategory::Element,
            TagCategory::Status,
            TagCategory::Weapon,
            TagCategory::WeaponType,
            TagCategory::Class,
            TagCategory::Movement,
            TagCategory::SkillType,
            TagCategory::BuffType,
            TagCategory::ItemType,
            TagCategory::EquipmentAttribute,
        ];
        for category in &all_categories {
            assert!(
                !registry.tags_by_category(*category).is_empty(),
                "TagCategory {:?} 没有对应的标签定义",
                category
            );
        }
    }
}
