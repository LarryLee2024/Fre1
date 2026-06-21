//! Combat Action System — handles combat action requests
//!
//! Processes AttackRequested/SpellCastRequested events through the
//! DamagePolicy pipeline (damage calculation → HitPoints → death check).

use bevy::prelude::*;
use tracing::{info, warn};

use crate::core::domains::combat::components::{Dead, HitPoints, UnitIdComponent};
use crate::core::domains::combat::events::{
    AttackRequested, DamageDealt, SpellCastRequested, UnitDied,
};
use crate::core::domains::combat::rules::damage_policy::{DamageContext, DamagePolicy, DamageType};

/// Observer: handles AttackRequested — calculates and applies damage
pub fn on_attack_requested(
    trigger: On<AttackRequested>,
    unit_query: Query<(Entity, &HitPoints, &UnitIdComponent)>,
    mut commands: Commands,
) {
    let event = trigger.event();
    info!(target: "combat", "AttackRequested: attacker={}, target={}",
        event.attacker_id, event.target_id);

    // Resolve target entity via UnitIdComponent
    let target_entity = unit_query
        .iter()
        .find(|(_, _, uid)| uid.id == event.target_id)
        .map(|(entity, _, _)| entity);

    let Some(target) = target_entity else {
        warn!(target: "combat", "Attack target not found: {}", event.target_id);
        return;
    };

    // TODO[P1][Combat][2026-06-21]: Resolve attacker entity ID bits for better
    //   logging/tracing in DamageContext. The u64 IDs are for display only
    //   (not computational) per DamageContext design.
    let ctx = DamageContext::simple(
        target.to_bits(),
        target.to_bits(),
        10, // base damage (TODO: from weapon/ability definition)
        DamageType::Physical,
    );
    let decision = DamagePolicy::evaluate(&ctx);

    info!(target: "combat",
        "Damage calculated: base={}, final={}, crit={}",
        decision.base_damage, decision.final_damage, decision.is_critical);

    commands.trigger(DamageDealt {
        target_id: event.target_id.clone(),
        attacker_id: event.attacker_id.clone(),
        damage: decision.final_damage,
        damage_type: "physical".to_string(),
        is_critical: decision.is_critical,
    });
}

/// Observer: handles SpellCastRequested — resolves spell effects and applies damage
pub fn on_spell_cast_requested(
    trigger: On<SpellCastRequested>,
    unit_query: Query<(Entity, &HitPoints, &UnitIdComponent)>,
    mut commands: Commands,
) {
    let event = trigger.event();
    info!(target: "combat", "SpellCastRequested: caster={}, spell={}, target={}",
        event.caster_id, event.spell_def_id, event.target_id);

    // Resolve target entity via UnitIdComponent
    let target_entity = unit_query
        .iter()
        .find(|(_, _, uid)| uid.id == event.target_id)
        .map(|(entity, _, _)| entity);

    let Some(target) = target_entity else {
        warn!(target: "combat", "Spell target not found: {}", event.target_id);
        return;
    };

    // TODO[P2][Combat][2026-06-21]: Look up spell def for damage value and damage type
    //   For now, use a higher base damage for spells vs attacks
    let ctx = DamageContext::simple(
        target.to_bits(),
        target.to_bits(),
        25, // placeholder spell base damage
        DamageType::Fire,
    );
    let decision = DamagePolicy::evaluate(&ctx);

    commands.trigger(DamageDealt {
        target_id: event.target_id.clone(),
        attacker_id: event.caster_id.clone(),
        damage: decision.final_damage,
        damage_type: "fire".to_string(),
        is_critical: decision.is_critical,
    });
}

/// Observer: applies DamageDealt to HitPoints
pub fn on_damage_dealt(
    trigger: On<DamageDealt>,
    mut unit_query: Query<(Entity, &mut HitPoints, &UnitIdComponent)>,
    mut commands: Commands,
) {
    let event = trigger.event();

    let mut found = false;
    for (_entity, mut hp, uid) in unit_query.iter_mut() {
        if uid.id == event.target_id {
            let actual = hp.take_damage(event.damage);
            info!(target: "combat",
                "Damage applied: {} to {}, hp now {}/{}",
                actual, event.target_id, hp.current, hp.maximum);

            if !hp.is_alive() {
                info!(target: "combat", "Unit died: {}", event.target_id);
                commands.trigger(UnitDied {
                    entity_id: event.target_id.clone(),
                    killer_id: event.attacker_id.clone(),
                });
            }
            found = true;
            break;
        }
    }

    if !found {
        warn!(target: "combat",
            "Damage target not found: {} (no HitPoints+UnitIdComponent)",
            event.target_id);
    }
}
