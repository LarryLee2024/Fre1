//! Inventory Command Handler — handles UseItem/EquipItem/DropItem GameCommands
//!
//! Listens for CommandExecuted events and delegates to inventory domain events.

use bevy::prelude::*;
use tracing::info;

use crate::core::capabilities::runtime::command::events::CommandExecuted;
use crate::core::capabilities::runtime::command::foundation::GameCommand;
use crate::core::domains::inventory::events::{DropRequested, EquipRequested, ItemUseRequested};

/// Observer: handles inventory GameCommands.
///
/// Matches `UseItem`, `EquipItem`, and `DropItem` variants from the command
/// pipeline and emits domain-specific request events for existing inventory systems.
pub fn on_inventory_command(
    trigger: On<CommandExecuted>,
    mut commands: Commands,
) {
    match &trigger.event().command {
        GameCommand::UseItem {
            user_id,
            item_instance_id,
            target_id,
        } => {
            info!(target: "inventory",
                event = "command.use_item",
                user = %user_id,
                item = %item_instance_id,
                target = ?target_id,
                "UseItem command received"
            );
            commands.trigger(ItemUseRequested {
                user_id: user_id.clone(),
                item_instance_id: item_instance_id.clone(),
                target_id: target_id.clone(),
            });
        }
        GameCommand::EquipItem {
            unit_id,
            item_instance_id,
            slot_index,
        } => {
            info!(target: "inventory",
                event = "command.equip_item",
                unit = %unit_id,
                item = %item_instance_id,
                slot = slot_index,
                "EquipItem command received"
            );
            commands.trigger(EquipRequested {
                unit_id: unit_id.clone(),
                item_instance_id: item_instance_id.clone(),
                slot_index: *slot_index,
            });
        }
        GameCommand::DropItem {
            unit_id,
            item_instance_id,
            quantity,
        } => {
            info!(target: "inventory",
                event = "command.drop_item",
                unit = %unit_id,
                item = %item_instance_id,
                qty = quantity,
                "DropItem command received"
            );
            commands.trigger(DropRequested {
                unit_id: unit_id.clone(),
                item_instance_id: item_instance_id.clone(),
                quantity: *quantity,
            });
        }
        _ => {} // Not an inventory command
    }
}
