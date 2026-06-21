//! Economy Command Handler — handles BuyItem/SellItem GameCommands
//!
//! Listens for CommandExecuted events, matches economy-related commands,
//! and delegates to domain events for downstream processing.

use bevy::prelude::*;
use tracing::info;

use crate::core::capabilities::runtime::command::events::CommandExecuted;
use crate::core::capabilities::runtime::command::foundation::GameCommand;
use crate::core::domains::economy::events::{PurchaseRequested, SaleRequested};

/// Observer: handles economy GameCommands.
///
/// Matches `BuyItem` and `SellItem` variants from the command pipeline
/// and emits domain-specific request events for existing economy systems.
pub fn on_economy_command(trigger: On<CommandExecuted>, mut commands: Commands) {
    match &trigger.event().command {
        GameCommand::BuyItem {
            buyer_id,
            item_def_id,
            quantity,
            shop_id,
        } => {
            info!(target: "economy",
                event = "command.buy_item",
                buyer = %buyer_id,
                item = %item_def_id,
                qty = quantity,
                shop = %shop_id,
                "BuyItem command received"
            );
            commands.trigger(PurchaseRequested {
                buyer_id: buyer_id.clone(),
                item_def_id: item_def_id.clone(),
                quantity: *quantity,
                shop_id: shop_id.clone(),
            });
        }
        GameCommand::SellItem {
            seller_id,
            item_def_id,
            quantity,
            shop_id,
        } => {
            info!(target: "economy",
                event = "command.sell_item",
                seller = %seller_id,
                item = %item_def_id,
                qty = quantity,
                shop = %shop_id,
                "SellItem command received"
            );
            commands.trigger(SaleRequested {
                seller_id: seller_id.clone(),
                item_def_id: item_def_id.clone(),
                quantity: *quantity,
                shop_id: shop_id.clone(),
            });
        }
        _ => {} // Not an economy command
    }
}
