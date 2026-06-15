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
use tactical_rpg::core::attribute::Attributes;
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
    /// 战士模板：phys_atk=5, max_hp=50, phys_def=3
    pub fn warrior() -> Self {
        let mut attrs = Attributes::default();
        attrs.set_base("phys_atk", 5);
        attrs.set_base("max_hp", 50);
        attrs.set_base("phys_def", 3);
        attrs.set_base("magic_atk", 2);
        attrs.set_base("magic_def", 3);
        attrs.set_base("dodge_rate", 600);
        attrs.set_base("hit_rate", 300);
        attrs.set_base("crit_rate", 200);
        attrs.set_base("move_range", 3);
        attrs.set_base("atk_range", 1);
        attrs.fill_hp();
        Self {
            faction: Faction::Player,
            attrs,
            name: "warrior",
        }
    }

    /// 法师模板：magic_atk=8, max_hp=30, magic_def=6
    pub fn mage() -> Self {
        let mut attrs = Attributes::default();
        attrs.set_base("phys_atk", 2);
        attrs.set_base("max_hp", 30);
        attrs.set_base("phys_def", 1);
        attrs.set_base("magic_atk", 8);
        attrs.set_base("magic_def", 6);
        attrs.set_base("dodge_rate", 400);
        attrs.set_base("hit_rate", 300);
        attrs.set_base("crit_rate", 200);
        attrs.set_base("move_range", 3);
        attrs.set_base("atk_range", 3);
        attrs.fill_hp();
        Self {
            faction: Faction::Player,
            attrs,
            name: "mage",
        }
    }

    /// 哥布林模板：低属性敌人
    pub fn goblin() -> Self {
        let mut attrs = Attributes::default();
        attrs.set_base("phys_atk", 3);
        attrs.set_base("max_hp", 30);
        attrs.set_base("phys_def", 1);
        attrs.set_base("dodge_rate", 500);
        attrs.set_base("hit_rate", 300);
        attrs.set_base("crit_rate", 300);
        attrs.set_base("move_range", 3);
        attrs.set_base("atk_range", 1);
        attrs.fill_hp();
        Self {
            faction: Faction::Enemy,
            attrs,
            name: "goblin",
        }
    }

    /// Test ID: FIX-001
    /// Unit_001 标准战士（§7.1）：phys_atk=15, max_hp=200, phys_def=10
    /// 用于所有战斗相关测试的标准战士单位
    pub fn unit_001() -> Self {
        let mut attrs = Attributes::default();
        attrs.set_base("phys_atk", 15);
        attrs.set_base("max_hp", 200);
        attrs.set_base("phys_def", 10);
        attrs.set_base("magic_atk", 3);
        attrs.set_base("magic_def", 5);
        attrs.set_base("dodge_rate", 1000);
        attrs.set_base("hit_rate", 500);
        attrs.set_base("crit_rate", 300);
        attrs.set_base("move_range", 3);
        attrs.set_base("atk_range", 1);
        attrs.fill_hp();
        Self {
            faction: Faction::Player,
            attrs,
            name: "Unit_001",
        }
    }

    /// Test ID: FIX-002
    /// Unit_002 标准法师（§7.1）：magic_atk=20, max_hp=100, magic_def=10
    /// 用于所有战斗相关测试的标准法师单位
    pub fn unit_002() -> Self {
        let mut attrs = Attributes::default();
        attrs.set_base("phys_atk", 3);
        attrs.set_base("max_hp", 100);
        attrs.set_base("phys_def", 5);
        attrs.set_base("magic_atk", 20);
        attrs.set_base("magic_def", 10);
        attrs.set_base("dodge_rate", 1200);
        attrs.set_base("hit_rate", 500);
        attrs.set_base("crit_rate", 300);
        attrs.set_base("move_range", 3);
        attrs.set_base("atk_range", 3);
        attrs.fill_hp();
        Self {
            faction: Faction::Player,
            attrs,
            name: "Unit_002",
        }
    }

    /// Test ID: FIX-003
    /// Unit_003 标准坦克（§7.1）：phys_atk=10, max_hp=300, phys_def=20
    /// 用于所有战斗相关测试的标准坦克单位
    pub fn unit_003() -> Self {
        let mut attrs = Attributes::default();
        attrs.set_base("phys_atk", 10);
        attrs.set_base("max_hp", 300);
        attrs.set_base("phys_def", 20);
        attrs.set_base("magic_atk", 2);
        attrs.set_base("magic_def", 8);
        attrs.set_base("dodge_rate", 500);
        attrs.set_base("hit_rate", 300);
        attrs.set_base("crit_rate", 200);
        attrs.set_base("move_range", 3);
        attrs.set_base("atk_range", 1);
        attrs.fill_hp();
        Self {
            faction: Faction::Player,
            attrs,
            name: "Unit_003",
        }
    }

    /// 设置当前 HP（直接设置 current_hp 字段）
    pub fn with_hp(mut self, hp: i32) -> Self {
        self.attrs.current_hp = hp;
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
