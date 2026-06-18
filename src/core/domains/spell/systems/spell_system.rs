//! 法术管理 Systems
//!
//! 包括施法请求处理、专注时长管理等 System。
//! 详见 docs/02-domain/domains/spell_domain.md §5

use bevy::prelude::*;

use crate::core::domains::spell::components::{
    Concentration, SpellConfig, SpellSlotPool, Spellbook,
};
use crate::core::domains::spell::events::{
    CastOutcome, ConcentrationBreakReason, ConcentrationBroken, SpellCastRequest, SpellCastResult,
    SpellSlotChanged,
};
use crate::core::domains::spell::rules::check_slot_available;

/// 处理施法请求：校验 → 执行 / 拒绝。
///
/// 监听 SpellCastRequest 事件，进行前置校验：
/// 1. 法术是否已准备
/// 2. 法术位是否充足
/// 3. 专注冲突检查
pub fn on_spell_cast_request(
    _trigger: On<SpellCastRequest>,
    mut commands: Commands,
    spellbook_query: Query<&Spellbook>,
    mut slot_pool_query: Query<&mut SpellSlotPool>,
    concentration_query: Query<&Concentration>,
    config: Res<SpellConfig>,
) {
    let request = _trigger.event();

    // 1. 检查法术书
    let spellbook = match spellbook_query.get(request.caster) {
        Ok(sb) => sb,
        Err(_) => {
            commands.trigger(SpellCastResult {
                caster: request.caster,
                spell_id: request.spell_id.clone(),
                effective_level: request
                    .upcast_level
                    .unwrap_or(crate::core::domains::spell::components::SpellLevel::Cantrip),
                result: CastOutcome::Failed {
                    reason: "施法者无法施法".to_string(),
                },
            });
            return;
        }
    };

    // 2. 检查法术是否已准备
    if !spellbook.has_prepared(&request.spell_id) {
        // 戏法不需要准备（假设所有已知的戏法都可施放）
        if !spellbook.knows(&request.spell_id) {
            let cast_level = request
                .upcast_level
                .unwrap_or(crate::core::domains::spell::components::SpellLevel::Cantrip);
            commands.trigger(SpellCastResult {
                caster: request.caster,
                spell_id: request.spell_id.clone(),
                effective_level: cast_level,
                result: CastOutcome::Failed {
                    reason: "法术未准备".to_string(),
                },
            });
            return;
        }
    }

    // 3. 检查法术位
    let cast_level = request
        .upcast_level
        .unwrap_or(crate::core::domains::spell::components::SpellLevel::Cantrip);

    if cast_level.as_u8() > 0 {
        if let Ok(mut slot_pool) = slot_pool_query.get_mut(request.caster) {
            if check_slot_available(&slot_pool, cast_level).is_ok() {
                // 消耗法术位
                slot_pool.consume(cast_level);
                commands.trigger(SpellSlotChanged {
                    entity: request.caster,
                    level: cast_level,
                    remaining: slot_pool.remaining(cast_level),
                    total: slot_pool
                        .slots_by_level
                        .get((cast_level.as_u8().saturating_sub(1)) as usize)
                        .map_or(0, |e| e.total),
                    source: "spell_cast".to_string(),
                });
            }
        }
    }

    // 4. 检查专注冲突 — 施放新专注法术时会自动解除旧专注
    if let Ok(concentration) = concentration_query.get(request.caster) {
        commands.trigger(ConcentrationBroken {
            entity: request.caster,
            spell_id: concentration.spell_id.clone(),
            reason: ConcentrationBreakReason::ReplacedByNewSpell {
                new_spell_id: request.spell_id.clone(),
            },
        });
        commands.entity(request.caster).remove::<Concentration>();
    }

    // 5. 施法成功
    commands.trigger(SpellCastResult {
        caster: request.caster,
        spell_id: request.spell_id.clone(),
        effective_level: cast_level,
        result: CastOutcome::Success,
    });
}

/// 每回合推进专注计时。
///
/// 专注法术的 duration 按回合递减，到期自动解除。
pub fn tick_concentration_duration(
    mut commands: Commands,
    mut concentration_query: Query<(Entity, &mut Concentration)>,
) {
    for (entity, mut concentration) in &mut concentration_query {
        concentration.elapsed_rounds += 1;

        if concentration.elapsed_rounds >= concentration.total_duration {
            commands.trigger(ConcentrationBroken {
                entity,
                spell_id: concentration.spell_id.clone(),
                reason: ConcentrationBreakReason::DurationExpired,
            });
            commands.entity(entity).remove::<Concentration>();
        }
    }
}
