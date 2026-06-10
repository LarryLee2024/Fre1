// App 构建器：最小/战斗/装备/完整 App

use bevy::prelude::*;
use tactical_rpg::buff::BuffPlugin;
use tactical_rpg::core::attribute_def::AttributeDefPlugin;
use tactical_rpg::core::effect::EffectPlugin;
use tactical_rpg::core::modifier_rule::ModifierRulePlugin;
use tactical_rpg::core::tag_def::TagDefPlugin;
use tactical_rpg::equipment::EquipmentPlugin;
use tactical_rpg::inventory::InventoryPlugin;

/// 最小 App：仅 MinimalPlugins + StatesPlugin
pub fn minimal_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin));
    app
}

/// 战斗 App：Core + Buff + Equipment + Inventory
pub fn combat_app() -> App {
    let mut app = minimal_app();
    app.add_plugins((
        AttributeDefPlugin,
        EffectPlugin,
        ModifierRulePlugin,
        TagDefPlugin,
        BuffPlugin,
        EquipmentPlugin,
        InventoryPlugin,
    ));
    app
}

/// 装备测试 App：combat_app + TraitPlugin + EquipItem/UnequipItem Message + 穿脱系统
pub fn equipment_app() -> App {
    let mut app = combat_app();
    // 装备系统需要 TraitRegistry 和 TraitEffectHandlerRegistry
    app.add_plugins(tactical_rpg::character::TraitPlugin);
    app.add_message::<tactical_rpg::equipment::EquipItem>()
        .add_message::<tactical_rpg::equipment::UnequipItem>()
        .add_message::<tactical_rpg::equipment::ItemEquipped>()
        .add_message::<tactical_rpg::equipment::ItemUnequipped>()
        .add_message::<tactical_rpg::equipment::EquipFailed>();
    app.add_systems(Update, tactical_rpg::equipment::equip_item_system);
    app.add_systems(Update, tactical_rpg::equipment::unequip_item_system);
    app
}

/// 完整战斗 App：combat_app + Effect Pipeline + BattleRecord + 战斗记录系统
pub fn full_battle_app() -> App {
    let mut app = combat_app();
    app.add_message::<tactical_rpg::battle::DamageApplied>()
        .add_message::<tactical_rpg::battle::HealApplied>()
        .add_message::<tactical_rpg::battle::CharacterDied>()
        .add_message::<tactical_rpg::battle::DotApplied>()
        .add_message::<tactical_rpg::battle::HotApplied>()
        .add_message::<tactical_rpg::battle::StunApplied>();
    app.init_resource::<tactical_rpg::battle::BattleRecord>();
    app.add_systems(Update, tactical_rpg::battle::record_damage);
    app.add_systems(Update, tactical_rpg::battle::record_heal);
    app.add_systems(Update, tactical_rpg::battle::record_character_died);
    app
}
