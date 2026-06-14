// App 构建器：最小/战斗/装备/完整 App

use bevy::prelude::*;
use tactical_rpg::core::attribute_def::AttributeDefPlugin;
use tactical_rpg::core::buff::BuffPlugin;
use tactical_rpg::core::effect::EffectPlugin;
use tactical_rpg::core::equipment::EquipmentPlugin;
use tactical_rpg::core::inventory::InventoryPlugin;
use tactical_rpg::core::modifier_rule::ModifierRulePlugin;
use tactical_rpg::core::tag_def::TagDefPlugin;
use tactical_rpg::infrastructure::logging::events as log_events;

/// 最小 App：仅 MinimalPlugins + StatesPlugin
pub fn minimal_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin));
    app
}

/// 注册所有日志 Message 类型（供测试 App 使用）
pub fn register_logging_messages(app: &mut App) {
    app.add_message::<log_events::ConfigLoaded>()
        .add_message::<log_events::BuffApplied>()
        .add_message::<log_events::BuffRemoved>()
        .add_message::<log_events::BuffExpired>()
        .add_message::<log_events::SkillActivated>()
        .add_message::<log_events::LevelCompletedEvent>()
        .add_message::<log_events::EquipmentEquipped>()
        .add_message::<log_events::EquipmentUnequipped>()
        .add_message::<log_events::ItemUsed>()
        .add_message::<log_events::ItemTransferred>()
        .add_message::<log_events::UnitMoved>()
        .add_message::<log_events::SnapshotCreated>();
}

/// 战斗 App：Core + Buff + Trait + Equipment + Inventory
pub fn combat_app() -> App {
    let mut app = minimal_app();
    app.add_plugins((
        AttributeDefPlugin,
        EffectPlugin,
        ModifierRulePlugin,
        TagDefPlugin,
        BuffPlugin,
        tactical_rpg::core::character::TraitPlugin,
        EquipmentPlugin,
        InventoryPlugin,
    ));
    register_logging_messages(&mut app);
    app
}

/// 装备测试 App：combat_app + EquipItem/UnequipItem Message + 穿脱系统
pub fn equipment_app() -> App {
    let mut app = combat_app();
    // TraitPlugin 已在 combat_app 中添加（TraitRegistry + TraitEffectHandlerRegistry）
    app.add_message::<tactical_rpg::core::equipment::EquipItem>()
        .add_message::<tactical_rpg::core::equipment::UnequipItem>()
        .add_message::<tactical_rpg::core::equipment::ItemEquipped>()
        .add_message::<tactical_rpg::core::equipment::ItemUnequipped>()
        .add_message::<tactical_rpg::core::equipment::EquipFailed>();
    app.add_systems(Update, tactical_rpg::core::equipment::equip_item_system);
    app.add_systems(Update, tactical_rpg::core::equipment::unequip_item_system);
    app
}

/// 完整战斗 App：combat_app + Effect Pipeline + BattleRecord + 战斗记录系统
pub fn full_battle_app() -> App {
    let mut app = combat_app();
    app.add_message::<tactical_rpg::core::battle::DamageApplied>()
        .add_message::<tactical_rpg::core::battle::HealApplied>()
        .add_message::<tactical_rpg::core::battle::CharacterDied>()
        .add_message::<tactical_rpg::core::battle::DotApplied>()
        .add_message::<tactical_rpg::core::battle::HotApplied>()
        .add_message::<tactical_rpg::core::battle::StunApplied>();
    app.init_resource::<tactical_rpg::core::battle::BattleRecord>();
    app.add_systems(Update, tactical_rpg::core::battle::record_damage);
    app.add_systems(Update, tactical_rpg::core::battle::record_heal);
    app.add_systems(Update, tactical_rpg::core::battle::record_character_died);
    app
}
