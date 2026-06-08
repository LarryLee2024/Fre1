use crate::assets::CnFont;
use crate::battle::{
    apply_buff_effect, apply_cleanse_effect, apply_damage_effect, apply_heal_effect,
    CombatLog, log_color,
};
use crate::core::attribute::Attributes;
use crate::core::effect::{EffectQueue, PendingEffectData};
use crate::core::tag::GameplayTags;
use crate::buff::{ActiveBuffs, BuffRegistry};
use crate::map::GameMap;
use crate::skill::{SkillCooldowns, SkillSlots};
use crate::character::{AiBehaviorId, Faction, GridPosition, Unit, UnitName};
use bevy::prelude::*;

/// AI 执行效果队列（使用共享函数，与玩家执行逻辑保持一致）
pub(crate) fn execute_ai_effects(
    commands: &mut Commands,
    queue: &mut EffectQueue,
    units: &mut Query<(
        Entity,
        &mut Unit,
        &mut GridPosition,
        &mut Transform,
        &UnitName,
        &mut Attributes,
        &SkillSlots,
        &mut SkillCooldowns,
        &mut ActiveBuffs,
        &mut GameplayTags,
        &AiBehaviorId,
    )>,
    combat_log: &mut CombatLog,
    buff_registry: &BuffRegistry,
    map: &GameMap,
    cn_font: &CnFont,
) {
    for effect in queue.pending.drain(..) {
        let attacker_color = units
            .get(effect.source)
            .map(|(_, u, _, _, _, _, _, _, _, _, _)| {
                if u.faction == Faction::Player {
                    log_color::PLAYER
                } else {
                    log_color::ENEMY
                }
            })
            .unwrap_or(log_color::NORMAL);
        let defender_color = units
            .get(effect.target)
            .map(|(_, u, _, _, _, _, _, _, _, _, _)| {
                if u.faction == Faction::Player {
                    log_color::PLAYER
                } else {
                    log_color::ENEMY
                }
            })
            .unwrap_or(log_color::NORMAL);
        let attacker_name = units
            .get(effect.source)
            .map(|(_, _, _, _, name, _, _, _, _, _, _)| name.0.clone())
            .unwrap_or("???".to_string());
        let target_name = units
            .get(effect.target)
            .map(|(_, _, _, _, name, _, _, _, _, _, _)| name.0.clone())
            .unwrap_or("???".to_string());

        match effect.data {
            PendingEffectData::Damage { amount, is_skill } => {
                if let Ok((_, _, target_gp, _, _, mut target_attrs, _, _, _, _, _)) =
                    units.get_mut(effect.target)
                {
                    apply_damage_effect(
                        &mut target_attrs,
                        &target_gp,
                        &target_name,
                        effect.target,
                        defender_color,
                        &attacker_name,
                        attacker_color,
                        amount,
                        is_skill,
                        effect.terrain.label(),
                        commands,
                        combat_log,
                        map,
                        cn_font,
                    );
                }
            }
            PendingEffectData::Heal { amount } => {
                if let Ok((_, _, _, _, _, mut target_attrs, _, _, _, _, _)) =
                    units.get_mut(effect.target)
                {
                    apply_heal_effect(&mut target_attrs, &target_name, amount, combat_log);
                }
            }
            PendingEffectData::ApplyBuff { buff_id, duration } => {
                if let Ok((
                    _,
                    _,
                    _,
                    _,
                    _,
                    mut target_attrs,
                    _,
                    _,
                    mut target_buffs,
                    mut target_tags,
                    _,
                )) = units.get_mut(effect.target)
                {
                    apply_buff_effect(
                        &mut target_buffs,
                        &mut target_attrs,
                        &mut target_tags,
                        &buff_id,
                        effect.source,
                        duration,
                        buff_registry,
                    );
                }
            }
            PendingEffectData::Cleanse => {
                if let Ok((
                    _,
                    _,
                    _,
                    _,
                    _,
                    mut target_attrs,
                    _,
                    _,
                    mut target_buffs,
                    mut target_tags,
                    _,
                )) = units.get_mut(effect.target)
                {
                    apply_cleanse_effect(&mut target_buffs, &mut target_attrs, &mut target_tags);
                }
            }
        }
    }
}
