//! CraftingPlugin — 制作/锻造领域 Plugin
//!
//! 注册配方、附魔、升级组件和系统。
//! 详见 docs/02-domain/domains/crafting_domain.md

use bevy::prelude::*;

use super::components::{EnchantmentSlot, UpgradeLevel};
use super::resources::CraftingConfig;
use super::systems::{on_apply_enchantment, on_craft_item, on_upgrade_item};

pub struct CraftingPlugin;

impl Plugin for CraftingPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<EnchantmentSlot>();
        app.register_type::<UpgradeLevel>();

        app.init_resource::<CraftingConfig>();

        app.add_observer(on_craft_item);
        app.add_observer(on_apply_enchantment);
        app.add_observer(on_upgrade_item);
    }
}
