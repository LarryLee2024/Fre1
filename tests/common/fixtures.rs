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

// 角色模板：消除 warrior_attrs() 等重复代码
// 标准测试单位（§7.1）：Unit_001/Unit_002/Unit_003

use bevy::prelude::*;
use tactical_rpg::core::attribute::{AttributeKind, Attributes};
use tactical_rpg::core::buff::ActiveBuffs;
use tactical_rpg::core::character::TraitCollection;
use tactical_rpg::core::character::{Faction, PersistentTags, Unit, UnitId, UnitName};
use tactical_rpg::core::equipment::EquipmentSlots;
use tactical_rpg::core::inventory::container::Container;
use tactical_rpg::core::tag::GameplayTags;

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

    /// Test ID: FIX-001
    /// Unit_001 标准战士（§7.1）：HP=100, ATK=30, DEF=10, SPD=10
    /// 用于所有战斗相关测试的标准战士单位
    pub fn unit_001() -> Self {
        let mut attrs = Attributes::default();
        attrs.set_base(AttributeKind::Might, 15.0);
        attrs.set_base(AttributeKind::Vitality, 20.0);
        attrs.set_base(AttributeKind::Agility, 10.0);
        attrs.set_base(AttributeKind::Dexterity, 5.0);
        attrs.set_base(AttributeKind::Intelligence, 3.0);
        attrs.set_base(AttributeKind::Willpower, 5.0);
        attrs.set_base(AttributeKind::Presence, 3.0);
        attrs.set_base(AttributeKind::Luck, 3.0);
        attrs.set_base_attack_range(1);
        attrs.fill_vital_resources();
        Self {
            faction: Faction::Player,
            attrs,
            name: "Unit_001",
        }
    }

    /// Test ID: FIX-002
    /// Unit_002 标准法师（§7.1）：HP=80, ATK=40, DEF=5, SPD=12
    /// 用于所有战斗相关测试的标准法师单位
    pub fn unit_002() -> Self {
        let mut attrs = Attributes::default();
        attrs.set_base(AttributeKind::Might, 3.0);
        attrs.set_base(AttributeKind::Vitality, 10.0);
        attrs.set_base(AttributeKind::Agility, 12.0);
        attrs.set_base(AttributeKind::Dexterity, 5.0);
        attrs.set_base(AttributeKind::Intelligence, 20.0);
        attrs.set_base(AttributeKind::Willpower, 10.0);
        attrs.set_base(AttributeKind::Presence, 5.0);
        attrs.set_base(AttributeKind::Luck, 3.0);
        attrs.set_base_attack_range(3);
        attrs.fill_vital_resources();
        Self {
            faction: Faction::Player,
            attrs,
            name: "Unit_002",
        }
    }

    /// Test ID: FIX-003
    /// Unit_003 标准坦克（§7.1）：HP=150, ATK=20, DEF=20, SPD=5
    /// 用于所有战斗相关测试的标准坦克单位
    pub fn unit_003() -> Self {
        let mut attrs = Attributes::default();
        attrs.set_base(AttributeKind::Might, 10.0);
        attrs.set_base(AttributeKind::Vitality, 30.0);
        attrs.set_base(AttributeKind::Agility, 5.0);
        attrs.set_base(AttributeKind::Dexterity, 3.0);
        attrs.set_base(AttributeKind::Intelligence, 2.0);
        attrs.set_base(AttributeKind::Willpower, 8.0);
        attrs.set_base(AttributeKind::Presence, 5.0);
        attrs.set_base(AttributeKind::Luck, 2.0);
        attrs.set_base_attack_range(1);
        attrs.fill_vital_resources();
        Self {
            faction: Faction::Player,
            attrs,
            name: "Unit_003",
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
