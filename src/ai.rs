// AI 模块：敌方自动行动，使用 Effect Pipeline

use crate::combat::manhattan_distance;
use crate::core::attribute::{AttributeKind, Attributes};
use crate::core::effect::{
    calculate_damage_from_effect, EffectDef, EffectQueue, PendingEffect, PendingEffectData,
};
use crate::core::tag::GameplayTags;
use crate::data::skill_data::{effective_skill_range, SkillRegistry, SkillSlots};
use crate::map::{GameMap, Terrain, Tile};
use crate::pathfinding::{build_tile_terrain_map, find_reachable_tiles};
use crate::turn::{AiTimer, TurnPhase, TurnState};
use crate::unit::{Faction, GridPosition, Unit, UnitName};
use bevy::prelude::*;

/// 单位快照（避免借用冲突）
struct UnitSnapshot {
    entity: Entity,
    faction: Faction,
    coord: IVec2,
    atk: f32,
    def: f32,
    base_def: f32,
    mov: u32,
    attack_range: u32,
    acted: bool,
    name: String,
    skill_ids: Vec<String>,
}

/// 敌方 AI 系统
pub fn enemy_ai_system(
    time: Res<Time>,
    mut ai_timer: ResMut<AiTimer>,
    turn_state: Res<TurnState>,
    turn_phase: Res<State<TurnPhase>>,
    mut next_phase: ResMut<NextState<TurnPhase>>,
    mut units: Query<(
        Entity,
        &mut Unit,
        &mut GridPosition,
        &mut Transform,
        &UnitName,
        &Attributes,
        &SkillSlots,
    )>,
    _tags_query: Query<&GameplayTags>,
    tiles: Query<&Tile>,
    map: Res<GameMap>,
    mut effect_queue: ResMut<EffectQueue>,
    skill_registry: Res<SkillRegistry>,
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
        .map(|(e, u, gp, _, name, attrs, skills)| UnitSnapshot {
            entity: e,
            faction: u.faction,
            coord: gp.coord,
            atk: attrs.get(AttributeKind::Atk),
            def: attrs.get(AttributeKind::Def),
            base_def: attrs.base.get(&AttributeKind::Def).copied().unwrap_or(0.0),
            mov: attrs.get(AttributeKind::Mov) as u32,
            attack_range: attrs.get(AttributeKind::AttackRange) as u32,
            acted: u.acted,
            name: name.0.clone(),
            skill_ids: skills.skill_ids.clone(),
        })
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
        def: f32,
        base_def: f32,
        attack_range: u32,
        attacker_name: String,
    }

    let mut actions: Vec<AiAction> = Vec::new();

    for snapshot in snapshots
        .iter()
        .filter(|s| s.faction == Faction::Enemy && !s.acted)
    {
        let nearest = *player_positions
            .iter()
            .min_by_key(|pos| manhattan_distance(snapshot.coord, **pos))
            .unwrap();

        let occupation_map: std::collections::HashMap<IVec2, bool> = snapshots
            .iter()
            .filter(|s| s.faction == Faction::Player)
            .map(|s| (s.coord, true))
            .collect();

        let reachable =
            find_reachable_tiles(snapshot.coord, snapshot.mov, &map, &terrain_map, &occupation_map);

        let best_coord = reachable
            .iter()
            .min_by_key(|(coord, _)| manhattan_distance(**coord, nearest))
            .map(|(coord, _)| *coord)
            .unwrap_or(snapshot.coord);

        // 选择使用的技能：优先特殊技能，否则基础攻击
        let skill_id = snapshot
            .skill_ids
            .iter()
            .find(|id| *id != "basic_attack")
            .map(|s| s.as_str())
            .unwrap_or("basic_attack");

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
            def: snapshot.def,
            base_def: snapshot.base_def,
            attack_range: snapshot.attack_range,
            attacker_name: snapshot.name.clone(),
        });
    }

    // 应用行动
    for action in actions {
        // 移动
        let world_pos = map.coord_to_world(action.move_to);
        if let Ok((_, _, mut gp, mut transform, _, _, _)) = units.get_mut(action.entity) {
            gp.coord = action.move_to;
            transform.translation.x = world_pos.x;
            transform.translation.y = world_pos.y;
        }

        // 攻击：通过 Effect Pipeline 生成效果
        if let Some(target_entity) = action.attack_target {
            if let Some(skill_data) = skill_registry.get(&action.skill_id) {
                let _target_attrs = units
                    .get(target_entity)
                    .map(|(_, _, _, _, _, attrs, _)| attrs.clone())
                    .unwrap_or_default();
                let target_gp = units
                    .get(target_entity)
                    .map(|(_, _, gp, _, _, _, _)| gp.coord)
                    .unwrap_or(IVec2::ZERO);

                let terrain = tiles
                    .iter()
                    .find_map(|t| {
                        if t.coord == target_gp {
                            Some(t.terrain)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(Terrain::Plain);

                for effect_def in &skill_data.effects {
                    match effect_def {
                        EffectDef::Damage {
                            multiplier,
                            ignore_def_percent,
                        } => {
                            let amount = calculate_damage_from_effect(
                                action.atk,
                                action.def,
                                action.base_def,
                                *multiplier,
                                *ignore_def_percent,
                                terrain,
                            );
                            effect_queue.push(PendingEffect {
                                source: action.entity,
                                target: target_entity,
                                data: PendingEffectData::Damage {
                                    amount,
                                    is_skill: action.skill_id != "basic_attack",
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
                        EffectDef::Heal { amount } => {
                            effect_queue.push(PendingEffect {
                                source: action.entity,
                                target: target_entity,
                                data: PendingEffectData::Heal { amount: *amount },
                                source_tags: skill_data.tags.clone(),
                                terrain,
                            });
                        }
                        EffectDef::Cleanse => {
                            effect_queue.push(PendingEffect {
                                source: action.entity,
                                target: target_entity,
                                data: PendingEffectData::Cleanse,
                                source_tags: skill_data.tags.clone(),
                                terrain,
                            });
                        }
                    }
                }
            }
        }

        // 标记已行动
        if let Ok((_, mut unit, _, _, _, _, _)) = units.get_mut(action.entity) {
            unit.acted = true;
        }
    }

    // 检查是否所有敌方单位都已行动
    let all_enemy_acted = units
        .iter()
        .filter(|(_, u, _, _, _, _, _)| u.faction == Faction::Enemy)
        .all(|(_, u, _, _, _, _, _)| u.acted);

    if all_enemy_acted {
        next_phase.set(TurnPhase::TurnEnd);
    } else {
        next_phase.set(TurnPhase::SelectUnit);
    }
}

/// AI 插件
pub struct AiPlugin;

impl Plugin for AiPlugin {
    fn build(&self, app: &mut App) {
        use crate::turn::AppState;
        app.add_systems(Update, enemy_ai_system.run_if(in_state(AppState::InGame)));
    }
}
