use crate::assets::CnFont;
use crate::battle::manhattan_distance;
use crate::battle::CombatLog;
use crate::gameplay::attribute::{AttributeKind, Attributes};
use crate::gameplay::effect::{
    EffectDef, EffectQueue, PendingEffect, PendingEffectData, calculate_damage_from_effect,
};
use crate::gameplay::modifier_rule::ModifierRuleRegistry;
use crate::gameplay::tag::GameplayTags;
use crate::buff::{ActiveBuffs, BuffRegistry};
use crate::skill::{
    BASIC_ATTACK_ID, SkillCooldowns, SkillRegistry, SkillSlots, effective_skill_range,
};
use crate::map::{GameMap, Tile, build_tile_terrain_map, find_reachable_tiles};
use crate::turn::{AiTimer, TurnPhase, TurnState};
use crate::character::{AiBehaviorId, Faction, GridPosition, Unit, UnitName};
use bevy::prelude::*;

use super::behavior::AiBehaviorRegistry;
use super::effect_exec::execute_ai_effects;
use super::movement::select_move_coord;
use super::skill_select::select_skill;
use super::targeting::{UnitSnapshot, select_target_coord};

/// 敌方 AI 系统：决策 → 移动 → 推入 EffectQueue → 修饰 → 执行
pub fn enemy_ai_system(
    mut commands: Commands,
    time: Res<Time>,
    mut ai_timer: ResMut<AiTimer>,
    turn_state: Res<TurnState>,
    turn_phase: Res<State<TurnPhase>>,
    mut next_phase: ResMut<NextState<TurnPhase>>,
    cn_font: Res<CnFont>,
    map: Res<GameMap>,
    skill_registry: Res<SkillRegistry>,
    buff_registry: Res<BuffRegistry>,
    modifier_rules: Res<ModifierRuleRegistry>,
    ai_behavior_registry: Res<AiBehaviorRegistry>,
    mut effect_queue: ResMut<EffectQueue>,
    mut combat_log: ResMut<CombatLog>,
    mut units: Query<(
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
    tiles: Query<&Tile>,
) {
    if turn_state.current_faction != Faction::Enemy {
        return;
    }
    if *turn_phase.get() != TurnPhase::SelectUnit {
        return;
    }

    ai_timer.timer.tick(time.delta());
    if !ai_timer.timer.just_finished() {
        return;
    }

    // 收集所有单位快照
    let snapshots: Vec<UnitSnapshot> = units
        .iter()
        .map(
            |(e, u, gp, _, _name, attrs, skills, cooldowns, _, _, ai_id)| UnitSnapshot {
                entity: e,
                faction: u.faction,
                coord: gp.coord,
                atk: attrs.get(AttributeKind::Atk),
                hp: attrs.get(AttributeKind::Hp),
                max_hp: attrs.get(AttributeKind::MaxHp),
                mov: attrs.get(AttributeKind::Mov) as u32,
                attack_range: attrs.get(AttributeKind::AttackRange) as u32,
                acted: u.acted,
                skill_ids: skills.skill_ids.clone(),
                cooldowns: cooldowns.clone(),
                ai_behavior_id: ai_id.0.clone(),
            },
        )
        .collect();

    let terrain_map = build_tile_terrain_map(&tiles);

    let player_positions: Vec<IVec2> = snapshots
        .iter()
        .filter(|s| s.faction == Faction::Player)
        .map(|s| s.coord)
        .collect();

    if player_positions.is_empty() {
        return;
    }

    // 记录 AI 行动
    struct AiAction {
        entity: Entity,
        move_to: IVec2,
        attack_target: Option<Entity>,
        skill_id: String,
        atk: f32,
    }

    let mut actions: Vec<AiAction> = Vec::new();

    for snapshot in snapshots
        .iter()
        .filter(|s| s.faction == Faction::Enemy && !s.acted)
    {
        // 获取 AI 行为配置
        let behavior = ai_behavior_registry
            .get(&snapshot.ai_behavior_id)
            .unwrap_or_else(|| ai_behavior_registry.default_behavior());

        // 根据目标策略选择目标
        let target_coord =
            select_target_coord(&snapshots, snapshot.coord, behavior.target_strategy);

        let occupation_map: std::collections::HashMap<IVec2, bool> = snapshots
            .iter()
            .filter(|s| s.faction == Faction::Player)
            .map(|s| (s.coord, true))
            .collect();

        let reachable = find_reachable_tiles(
            snapshot.coord,
            snapshot.mov,
            &map,
            &terrain_map,
            &occupation_map,
        );

        // 根据移动策略选择移动位置
        let best_coord = select_move_coord(
            &reachable,
            snapshot.coord,
            target_coord,
            snapshot.attack_range,
            behavior.move_strategy,
        );

        // 根据技能策略选择技能
        let skill_id = select_skill(
            &snapshot.skill_ids,
            &snapshot.cooldowns,
            behavior.skill_strategy,
            &behavior.skill_priority,
        );

        let effective_range = skill_registry
            .get(skill_id)
            .map(|sd| effective_skill_range(sd, snapshot.attack_range))
            .unwrap_or(snapshot.attack_range);

        let attack_target = snapshots
            .iter()
            .filter(|s| s.faction == Faction::Player)
            .find(|s| manhattan_distance(best_coord, s.coord) <= effective_range)
            .map(|s| s.entity);

        actions.push(AiAction {
            entity: snapshot.entity,
            move_to: best_coord,
            attack_target,
            skill_id: skill_id.to_string(),
            atk: snapshot.atk,
        });
    }

    // 应用行动
    for action in actions {
        // 移动
        let world_pos = map.coord_to_world(action.move_to);
        if let Ok((_, _, mut gp, mut transform, _, _, _, _, _, _, _)) = units.get_mut(action.entity)
        {
            gp.coord = action.move_to;
            transform.translation.x = world_pos.x;
            transform.translation.y = world_pos.y;
        }

        // 通过 EffectQueue 执行攻击效果
        if let Some(target_entity) = action.attack_target {
            if let Some(skill_data) = skill_registry.get(&action.skill_id) {
                // 设置冷却
                if skill_data.cooldown > 0 {
                    if let Ok((_, _, _, _, _, _, _, mut cooldowns, _, _, _)) =
                        units.get_mut(action.entity)
                    {
                        cooldowns.set(&action.skill_id, skill_data.cooldown);
                    }
                }

                // 获取目标信息并推入 EffectQueue
                let target_info = units
                    .get(target_entity)
                    .ok()
                    .and_then(|(_, _, gp, _, _, attrs, _, _, _, _, _)| {
                        tiles
                            .iter()
                            .find(|t| t.coord == gp.coord)
                            .map(|tile| (attrs.clone(), gp.coord, tile.terrain, tile.defense_bonus))
                    });

                if let Some((target_attrs, _target_gp, terrain, defense_bonus)) = target_info {
                    for effect_def in &skill_data.effects {
                        match effect_def {
                            EffectDef::Damage {
                                multiplier,
                                ignore_def_percent,
                            } => {
                                let effective_def = target_attrs.get(AttributeKind::Def);
                                let base_def = target_attrs
                                    .base
                                    .get(&AttributeKind::Def)
                                    .copied()
                                    .unwrap_or(0.0);

                                let amount = calculate_damage_from_effect(
                                    action.atk,
                                    effective_def,
                                    base_def,
                                    *multiplier,
                                    *ignore_def_percent,
                                    defense_bonus,
                                );

                                effect_queue.push(PendingEffect {
                                    source: action.entity,
                                    target: target_entity,
                                    data: PendingEffectData::Damage {
                                        amount,
                                        is_skill: action.skill_id != BASIC_ATTACK_ID,
                                    },
                                    source_tags: skill_data.tags.clone(),
                                    terrain,
                                });
                            }
                            EffectDef::ApplyBuff { buff_id, duration } => {
                                effect_queue.push(PendingEffect {
                                    source: action.entity,
                                    target: target_entity,
                                    data: PendingEffectData::ApplyBuff {
                                        buff_id: buff_id.clone(),
                                        duration: *duration,
                                    },
                                    source_tags: skill_data.tags.clone(),
                                    terrain,
                                });
                            }
                            EffectDef::Heal { .. } | EffectDef::Cleanse => {
                                // AI 不会治疗/净化玩家
                            }
                        }
                    }
                }
            }
        }

        // 标记已行动
        if let Ok((_, mut unit, _, _, _, _, _, _, _, _, _)) = units.get_mut(action.entity) {
            unit.acted = true;
        }
    }

    // 修饰 + 执行 EffectQueue 中的所有效果
    // AI 不经过 TurnPhase::ExecuteAction，所以在这里直接执行
    if !effect_queue.pending.is_empty() {
        // 步骤 2：修饰效果（使用 ModifierRuleRegistry）
        for effect in &mut effect_queue.pending {
            if let PendingEffectData::Damage { ref mut amount, .. } = effect.data {
                if let Ok((_, _, _, _, _, _, _, _, _, tags, _)) = units.get(effect.target) {
                    *amount =
                        modifier_rules.apply_damage_modifiers(*amount, &effect.source_tags, tags);
                }
            }
        }

        // 步骤 3：执行效果（扣血/加 Buff/特效/日志/击杀）
        execute_ai_effects(
            &mut commands,
            &mut effect_queue,
            &mut units,
            &mut combat_log,
            &buff_registry,
            &map,
            &cn_font,
        );
    }

    // 检查是否所有敌方单位都已行动
    let all_enemy_acted = units
        .iter()
        .filter(|(_, u, _, _, _, _, _, _, _, _, _)| u.faction == Faction::Enemy)
        .all(|(_, u, _, _, _, _, _, _, _, _, _)| u.acted);

    if all_enemy_acted {
        next_phase.set(TurnPhase::TurnEnd);
    } else {
        next_phase.set(TurnPhase::SelectUnit);
    }
}
