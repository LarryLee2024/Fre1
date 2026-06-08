// 标签定义注册表：标签元数据外部化，支持 UI 自动生成和标签查询
// GameplayTag 仍为位掩码（运行时 O(1) 查询），显示元数据从 RON 加载

use crate::core::tag::{GameplayTag, TagName};
use bevy::prelude::*;
use ron::de::from_bytes;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::read;

/// 标签分类
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum TagCategory {
    Element,
    Status,
    Weapon,
    Class,
    SkillType,
    BuffType,
}

/// 标签定义（RON 反序列化用）
#[derive(Clone, Debug, Deserialize)]
pub struct TagDefinition {
    pub tag: TagName,
    pub display_name: String,
    pub description: String,
    pub category: TagCategory,
}

/// 标签注册表资源
#[derive(Resource, Default)]
pub struct TagRegistry {
    pub definitions: HashMap<GameplayTag, TagDefinition>,
}

impl TagRegistry {
    pub fn get(&self, tag: GameplayTag) -> Option<&TagDefinition> {
        self.definitions.get(&tag)
    }

    /// 获取标签显示名称（找不到则回退到 tag.label()）
    pub fn display_name(&self, tag: GameplayTag) -> &str {
        self.definitions
            .get(&tag)
            .map(|d| d.display_name.as_str())
            .unwrap_or(tag.label())
    }

    /// 按分类查询标签
    pub fn tags_by_category(&self, category: TagCategory) -> Vec<GameplayTag> {
        self.definitions
            .iter()
            .filter(|(_, def)| def.category == category)
            .map(|(tag, _)| *tag)
            .collect()
    }

    /// 从 RON 文件加载
    pub fn load_from_file(path: &str) -> Self {
        let mut registry = TagRegistry::default();

        match read(path) {
            Ok(bytes) => match from_bytes::<Vec<TagDefinition>>(&bytes) {
                Ok(defs) => {
                    for def in defs {
                        registry.definitions.insert(def.tag.to_tag(), def);
                    }
                    bevy::log::info!("加载标签定义: {} 种", registry.definitions.len());
                }
                Err(e) => {
                    bevy::log::error!("解析标签定义文件 {} 失败: {}", path, e);
                    registry.register_defaults();
                }
            },
            Err(e) => {
                bevy::log::warn!("标签定义文件 {} 不存在: {}, 使用默认值", path, e);
                registry.register_defaults();
            }
        }

        registry
    }

    /// 注册内置默认标签定义
    fn register_defaults(&mut self) {
        let defaults = vec![
            // 元素
            TagDefinition { tag: TagName::Fire, display_name: "火焰".into(), description: "火属性".into(), category: TagCategory::Element },
            TagDefinition { tag: TagName::Ice, display_name: "冰霜".into(), description: "冰属性".into(), category: TagCategory::Element },
            TagDefinition { tag: TagName::Poison, display_name: "毒素".into(), description: "毒属性".into(), category: TagCategory::Element },
            // 状态
            TagDefinition { tag: TagName::Stun, display_name: "晕眩".into(), description: "无法行动".into(), category: TagCategory::Status },
            TagDefinition { tag: TagName::Burn, display_name: "燃烧".into(), description: "每回合受到火焰伤害".into(), category: TagCategory::Status },
            TagDefinition { tag: TagName::Regen, display_name: "恢复".into(), description: "每回合恢复生命值".into(), category: TagCategory::Status },
            // 武器
            TagDefinition { tag: TagName::Melee, display_name: "近战".into(), description: "近战攻击".into(), category: TagCategory::Weapon },
            TagDefinition { tag: TagName::Ranged, display_name: "远程".into(), description: "远程攻击".into(), category: TagCategory::Weapon },
            // 职业
            TagDefinition { tag: TagName::Warrior, display_name: "战士".into(), description: "战士职业".into(), category: TagCategory::Class },
            TagDefinition { tag: TagName::Archer, display_name: "弓手".into(), description: "弓手职业".into(), category: TagCategory::Class },
            TagDefinition { tag: TagName::Mage, display_name: "法师".into(), description: "法师职业".into(), category: TagCategory::Class },
            // 技能类型
            TagDefinition { tag: TagName::SkillActive, display_name: "主动技能".into(), description: "主动施放的技能".into(), category: TagCategory::SkillType },
            TagDefinition { tag: TagName::SkillPassive, display_name: "被动技能".into(), description: "被动触发的技能".into(), category: TagCategory::SkillType },
            // Buff 类型
            TagDefinition { tag: TagName::Buff, display_name: "增益".into(), description: "正面效果".into(), category: TagCategory::BuffType },
            TagDefinition { tag: TagName::Debuff, display_name: "减益".into(), description: "负面效果".into(), category: TagCategory::BuffType },
        ];

        for def in defaults {
            self.definitions.insert(def.tag.to_tag(), def);
        }
    }
}

/// 标签定义插件
pub struct TagDefPlugin;

impl Plugin for TagDefPlugin {
    fn build(&self, app: &mut App) {
        let registry = TagRegistry::load_from_file("assets/definitions/tags.ron");
        app.insert_resource(registry);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let mut registry = TagRegistry::default();
        registry.register_defaults();

        let def = registry.get(GameplayTag::FIRE).unwrap();
        assert_eq!(def.display_name, "火焰");
        assert_eq!(def.category, TagCategory::Element);
    }

    #[test]
    fn tag_registry_按分类查询() {
        let mut registry = TagRegistry::default();
        registry.register_defaults();

        let elements = registry.tags_by_category(TagCategory::Element);
        assert_eq!(elements.len(), 3); // Fire, Ice, Poison
        assert!(elements.contains(&GameplayTag::FIRE));
    }

    #[test]
    fn tag_registry_显示名称回退() {
        let registry = TagRegistry::default();
        assert_eq!(registry.display_name(GameplayTag::FIRE), "火焰");
    }
}
