// 步骤 1：生成战斗效果（从技能定义 + 属性计算）

use crate::gameplay::attribute::{AttributeKind, Attributes};
use crate::gameplay::effect::{
    EffectDef, EffectQueue, PendingEffect, PendingEffectData, calculate_damage_from_effect,
};
use crate::gameplay::tag::GameplayTags;
use crate::skill::{BASIC_ATTACK_ID, SkillCooldowns, SkillRegistry};
use crate::map::{GameMap, Tile};
use crate::character::{Faction, GridPosition, Selected, Unit, UnitName};
use bevy::prelude::*;

use super::intent::CombatIntent;

/// 生成战斗效果：从选中单位的技能定义 + 目标属性计算，推入 EffectQueue
pub fn generate_combat_effects(
    mut queue: ResMut<EffectQueue>,
    selected_units: Query<
        (
            Entity,
            &Unit,
            &GridPosition,
            &UnitName,
            &Attributes,
            &GameplayTags,
            &SkillCooldowns,
        ),
        With<Selected>,
    >,
    targets: Query<
        (
            Entity,
            &Unit,
            &GridPosition,
            &UnitName,
            &Attributes,
            &GameplayTags,
            &Transform,
        ),
        Without<Selected>,
    >,
    tiles: Query<&Tile>,
    combat_intent: Res<CombatIntent>,
    skill_registry: Res<SkillRegistry>,
    _map: Res<GameMap>,
) {
    let Ok((
        source_entity,
        source_unit,
        _source_gp,
        _source_name,
        source_attrs,
        source_tags,
        source_cooldowns,
    )) = selected_units.single()
    else {
        return;
    };

    if source_tags.has(crate::gameplay::tag::GameplayTag::STUN) {
        return;
    }

    let Some(target_coord) = combat_intent.target_coord else {
        return;
    };

    let skill_id = combat_intent.skill_id.as_deref().unwrap_or(BASIC_ATTACK_ID);
    let Some(skill_data) = skill_registry.get(skill_id) else {
        return;
    };

    if source_cooldowns.get(skill_id) > 0 {
        return;
    }

    for (
        target_entity,
        target_unit,
        target_gp,
        _target_name,
        target_attrs,
        _target_tags,
        _target_transform,
    ) in &targets
    {
        if target_gp.coord != target_coord || target_unit.faction == source_unit.faction {
            continue;
        }

        let Some(tile) = tiles.iter().find(|t| t.coord == target_gp.coord) else {
            return;
        };
        let terrain = tile.terrain;
        let defense_bonus = tile.defense_bonus;

        for effect_def in &skill_data.effects {
            match effect_def {
                EffectDef::Damage {
                    multiplier,
                    ignore_def_percent,
                } => {
                    let effective_atk = source_attrs.get(AttributeKind::Atk);
                    let effective_def = target_attrs.get(AttributeKind::Def);
                    let base_def = target_attrs
                        .base
                        .get(&AttributeKind::Def)
                        .copied()
                        .unwrap_or(0.0);

                    let amount = calculate_damage_from_effect(
                        effective_atk,
                        effective_def,
                        base_def,
                        *multiplier,
                        *ignore_def_percent,
                        defense_bonus,
                    );

                    queue.push(PendingEffect {
                        source: source_entity,
                        target: target_entity,
                        data: PendingEffectData::Damage {
                            amount,
                            is_skill: skill_id != BASIC_ATTACK_ID,
                        },
                        source_tags: skill_data.tags.clone(),
                        terrain,
                    });
                }
                EffectDef::Heal { amount } => {
                    queue.push(PendingEffect {
                        source: source_entity,
                        target: target_entity,
                        data: PendingEffectData::Heal { amount: *amount },
                        source_tags: skill_data.tags.clone(),
                        terrain,
                    });
                }
                EffectDef::ApplyBuff { buff_id, duration } => {
                    queue.push(PendingEffect {
                        source: source_entity,
                        target: target_entity,
                        data: PendingEffectData::ApplyBuff {
                            buff_id: buff_id.clone(),
                            duration: *duration,
                        },
                        source_tags: skill_data.tags.clone(),
                        terrain,
                    });
                }
                EffectDef::Cleanse => {
                    queue.push(PendingEffect {
                        source: source_entity,
                        target: target_entity,
                        data: PendingEffectData::Cleanse,
                        source_tags: skill_data.tags.clone(),
                        terrain,
                    });
                }
            }
        }
        break;
    }
}
