// App 构建器：最小/战斗/完整 App

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
