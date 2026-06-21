//! Combat Command Handler — handles CastSpell/Attack GameCommands
//!
//! Bridges GameCommand to combat domain actions through the existing
//! combat pipeline (step_unit_action, UnitActionComplete observer).
//!
//! Listens for CommandExecuted events, matches combat-related commands,
//! and delegates to domain events for downstream processing.

use bevy::prelude::*;
use tracing::info;

use crate::core::capabilities::runtime::command::events::CommandExecuted;
use crate::core::capabilities::runtime::command::foundation::GameCommand;
use crate::core::domains::combat::events::{AttackRequested, SpellCastRequested};

/// Observer: handles combat GameCommands.
///
/// Matches `CastSpell` and `Attack` variants from the command pipeline
/// and emits domain-specific request events for existing combat systems.
pub fn on_combat_command(trigger: On<CommandExecuted>, mut commands: Commands) {
    match &trigger.event().command {
        GameCommand::CastSpell {
            caster_id,
            spell_def_id,
            target_id,
        } => {
            info!(target: "combat",
                event = "command.cast_spell",
                caster = %caster_id,
                spell = %spell_def_id,
                target = %target_id,
                "CastSpell command received"
            );
            commands.trigger(SpellCastRequested {
                caster_id: caster_id.clone(),
                spell_def_id: spell_def_id.clone(),
                target_id: target_id.clone(),
            });
        }
        GameCommand::Attack {
            attacker_id,
            target_id,
            ability_slot,
        } => {
            info!(target: "combat",
                event = "command.attack",
                attacker = %attacker_id,
                target = %target_id,
                slot = ?ability_slot,
                "Attack command received"
            );
            commands.trigger(AttackRequested {
                attacker_id: attacker_id.clone(),
                target_id: target_id.clone(),
                ability_slot: *ability_slot,
            });
        }
        _ => {} // Not a combat command
    }
}
