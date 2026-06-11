// 角色模板：消除 warrior_attrs() 等重复代码

use bevy::prelude::*;
use tactical_rpg::buff::ActiveBuffs;
use tactical_rpg::character::TraitCollection;
use tactical_rpg::character::{Faction, PersistentTags, Unit, UnitId, UnitName};
use tactical_rpg::core::attribute::{AttributeKind, Attributes};
use tactical_rpg::core::tag::GameplayTags;
use tactical_rpg::equipment::EquipmentSlots;
use tactical_rpg::inventory::container::Container;

/// 角色构建器：流式 API 构建测试角色
pub struct UnitBuilder {
    faction: Faction,
    attrs: Attributes,
    name: &'static str,
}

impl UnitBuilder {
    /// 战士模板：Might=5, Vitality=5, Agility=6
    pub fn warrior() -> Self {
        let mut attrs = Attributes::default();
        attrs.set_base(AttributeKind::Might, 5.0);
        attrs.set_base(AttributeKind::Vitality, 5.0);
        attrs.set_base(AttributeKind::Agility, 6.0);
        attrs.set_base(AttributeKind::Dexterity, 3.0);
        attrs.set_base(AttributeKind::Intelligence, 2.0);
        attrs.set_base(AttributeKind::Willpower, 3.0);
        attrs.set_base(AttributeKind::Presence, 2.0);
        attrs.set_base(AttributeKind::Luck, 2.0);
        attrs.set_base_attack_range(1);
        attrs.fill_vital_resources();
        Self {
            faction: Faction::Player,
            attrs,
            name: "warrior",
        }
    }

    /// 法师模板：Intelligence=8, Willpower=6
    pub fn mage() -> Self {
        let mut attrs = Attributes::default();
        attrs.set_base(AttributeKind::Might, 2.0);
        attrs.set_base(AttributeKind::Vitality, 3.0);
        attrs.set_base(AttributeKind::Agility, 4.0);
        attrs.set_base(AttributeKind::Dexterity, 3.0);
        attrs.set_base(AttributeKind::Intelligence, 8.0);
        attrs.set_base(AttributeKind::Willpower, 6.0);
        attrs.set_base(AttributeKind::Presence, 4.0);
        attrs.set_base(AttributeKind::Luck, 2.0);
        attrs.set_base_attack_range(3);
        attrs.fill_vital_resources();
        Self {
            faction: Faction::Player,
            attrs,
            name: "mage",
        }
    }

    /// 哥布林模板：低属性敌人
    pub fn goblin() -> Self {
        let mut attrs = Attributes::default();
        attrs.set_base(AttributeKind::Might, 3.0);
        attrs.set_base(AttributeKind::Vitality, 3.0);
        attrs.set_base(AttributeKind::Agility, 5.0);
        attrs.set_base(AttributeKind::Dexterity, 3.0);
        attrs.set_base(AttributeKind::Intelligence, 1.0);
        attrs.set_base(AttributeKind::Willpower, 1.0);
        attrs.set_base(AttributeKind::Presence, 1.0);
        attrs.set_base(AttributeKind::Luck, 3.0);
        attrs.set_base_attack_range(1);
        attrs.fill_vital_resources();
        Self {
            faction: Faction::Enemy,
            attrs,
            name: "goblin",
        }
    }

    /// 设置 HP（使用 set_vital 设置当前值，set_base 对非 Core Stat 无效）
    pub fn with_hp(mut self, hp: f32) -> Self {
        self.attrs.set_vital(AttributeKind::Hp, hp);
        self
    }

    /// 设置为敌方
    pub fn enemy(mut self) -> Self {
        self.faction = Faction::Enemy;
        self
    }

    /// 设置为友方
    pub fn player(mut self) -> Self {
        self.faction = Faction::Player;
        self
    }

    /// 获取属性引用
    pub fn attrs(&self) -> &Attributes {
        &self.attrs
    }

    /// 在 App 中生成角色，返回 Entity
    pub fn spawn(self, app: &mut App) -> Entity {
        app.world_mut()
            .spawn((
                Unit {
                    faction: self.faction,
                    acted: false,
                },
                self.attrs,
                EquipmentSlots::default(),
                Container::backpack(),
                ActiveBuffs::default(),
                TraitCollection::default(),
                GameplayTags::default(),
                PersistentTags::default(),
            ))
            .insert((
                Name::new(self.name),
                UnitId(self.name.to_string()),
                UnitName(self.name.to_string()),
            ))
            .id()
    }
}

/// 旧式辅助函数（向后兼容）
pub fn warrior_attrs() -> Attributes {
    UnitBuilder::warrior().attrs.clone()
}

pub fn mage_attrs() -> Attributes {
    UnitBuilder::mage().attrs.clone()
}

pub fn goblin_attrs() -> Attributes {
    UnitBuilder::goblin().attrs.clone()
}

impl Clone for UnitBuilder {
    fn clone(&self) -> Self {
        Self {
            faction: self.faction,
            attrs: self.attrs.clone(),
            name: self.name,
        }
    }
}
