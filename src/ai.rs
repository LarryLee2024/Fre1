// AI 模块：敌方自动行动

use crate::combat::{calculate_damage, manhattan_distance, skill_name, skill_range};
use crate::map::{GameMap, Terrain, Tile};
use crate::pathfinding::{build_tile_terrain_map, find_reachable_tiles};
use crate::turn::{AiTimer, TurnPhase, TurnState};
use crate::ui::{CombatLog, CnFont, LogSegment, log_color};
use crate::unit::{Faction, GridPosition, Skill, Unit, UnitName};
use crate::vfx;
use bevy::prelude::*;

/// 单位快照（避免借用冲突）
struct UnitSnapshot {
    entity: Entity,
    faction: Faction,
    coord: IVec2,
    atk: i32,
    def: i32,
    mov: u32,
    attack_range: u32,
    acted: bool,
    name: String,
    skill: Skill,
}

/// 敌方 AI 系统
pub fn enemy_ai_system(
    time: Res<Time>,
    mut ai_timer: ResMut<AiTimer>,
    mut turn_state: ResMut<TurnState>,
    turn_phase: Res<State<TurnPhase>>,
    mut units: Query<(
        Entity,
        &mut Unit,
        &mut GridPosition,
        &mut Transform,
        &UnitName,
    )>,
    tiles: Query<&Tile>,
    map: Res<GameMap>,
    mut commands: Commands,
    mut combat_log: ResMut<CombatLog>,
    cn_font: Res<CnFont>,
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

    // 先收集所有单位快照（只读遍历一次）
    let snapshots: Vec<UnitSnapshot> = units
        .iter()
        .map(|(e, u, gp, _, name)| UnitSnapshot {
            entity: e,
            faction: u.faction,
            coord: gp.coord,
            atk: u.atk,
            def: u.def,
            mov: u.mov,
            attack_range: u.attack_range,
            acted: u.acted,
            name: name.0.clone(),
            skill: u.skill,
        })
        .collect();

    let terrain_map = build_tile_terrain_map(&tiles);

    // 收集玩家位置
    let player_positions: Vec<IVec2> = snapshots
        .iter()
        .filter(|s| s.faction == Faction::Player)
        .map(|s| s.coord)
        .collect();

    // 没有玩家单位则跳过
    if player_positions.is_empty() {
        return;
    }

    // 记录需要执行的 AI 行动
    struct AiAction {
        entity: Entity,
        move_to: IVec2,
        attack_target: Option<Entity>,
        atk: i32,
        def: i32,
        attack_range: u32,
        attacker_name: String,
        skill: Skill,
    }

    let mut actions: Vec<AiAction> = Vec::new();

    // 计算每个敌方单位的行动（纯计算，不修改世界）
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

        let reachable = find_reachable_tiles(
            snapshot.coord,
            snapshot.mov,
            &map,
            &terrain_map,
            &occupation_map,
        );

        let best_coord = reachable
            .iter()
            .min_by_key(|(coord, _)| manhattan_distance(**coord, nearest))
            .map(|(coord, _)| *coord)
            .unwrap_or(snapshot.coord);

        // 使用技能范围判定攻击距离
        let effective_range = skill_range(&snapshot.skill, snapshot.attack_range);

        // 检查攻击范围内是否有玩家单位
        let attack_target = snapshots
            .iter()
            .filter(|s| s.faction == Faction::Player)
            .find(|s| manhattan_distance(best_coord, s.coord) <= effective_range)
            .map(|s| s.entity);

        actions.push(AiAction {
            entity: snapshot.entity,
            move_to: best_coord,
            attack_target,
            atk: snapshot.atk,
            def: snapshot.def,
            attack_range: snapshot.attack_range,
            attacker_name: snapshot.name.clone(),
            skill: snapshot.skill,
        });
    }

    // 应用行动到世界（可变访问）
    for action in actions {
        // 移动
        let world_pos = map.coord_to_world(action.move_to);
        if let Ok((_, _, mut gp, mut transform, _)) = units.get_mut(action.entity) {
            gp.coord = action.move_to;
            transform.translation.x = world_pos.x;
            transform.translation.y = world_pos.y;
        }

        // 攻击
        if let Some(target_entity) = action.attack_target {
            if let Ok((_, mut target_unit, target_gp, target_transform, target_name)) =
                units.get_mut(target_entity)
            {
                let terrain = tiles
                    .iter()
                    .find_map(|t| {
                        if t.coord == target_gp.coord {
                            Some(t.terrain)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(Terrain::Plain);

                let attacker = Unit {
                    faction: Faction::Enemy,
                    mov: 0,
                    hp: 0,
                    max_hp: 0,
                    atk: action.atk,
                    def: action.def,
                    attack_range: action.attack_range,
                    acted: false,
                    skill: action.skill,
                };
                let damage = calculate_damage(&attacker, &target_unit, terrain);
                target_unit.hp -= damage;

                // 伤害数字弹出
                let is_crit = action.skill != Skill::None;
                vfx::spawn_damage_popup(
                    &mut commands,
                    target_transform.translation.truncate(),
                    damage,
                    &cn_font.handle,
                    is_crit,
                );

                // 写入战斗日志（含技能名）
                let skill_label = skill_name(&action.skill);
                let killed = target_unit.hp <= 0;
                combat_log.push(vec![
                    LogSegment { text: format!("[{}]", action.attacker_name), color: log_color::ENEMY },
                    LogSegment { text: format!(" 使用[{}]", skill_label), color: log_color::TURN },
                    LogSegment { text: " 攻击 ".to_string(), color: log_color::NORMAL },
                    LogSegment { text: format!("[{}]", target_name.0), color: log_color::PLAYER },
                    LogSegment { text: " 造成 ".to_string(), color: log_color::NORMAL },
                    LogSegment { text: format!("[{}]", damage), color: log_color::DAMAGE },
                    LogSegment { text: " 伤害".to_string(), color: log_color::NORMAL },
                    LogSegment { text: format!(" ({})", terrain.label()), color: log_color::TERRAIN },
                ]);

                if killed {
                    combat_log.push(vec![
                        LogSegment { text: format!("[{}]", target_name.0), color: log_color::PLAYER },
                        LogSegment { text: " 被击败！".to_string(), color: log_color::KILL },
                    ]);
                    commands.entity(target_entity).despawn();
                }
            }
        }

        // 标记已行动
        if let Ok((_, mut unit, _, _, _)) = units.get_mut(action.entity) {
            unit.acted = true;
        }
    }

    // 切换到玩家回合
    turn_state.current_faction = Faction::Player;
    turn_state.turn_number += 1;

    // 重置玩家单位行动状态
    for (_, mut unit, _, _, _) in units.iter_mut() {
        if unit.faction == Faction::Player {
            unit.acted = false;
        }
    }

    ai_timer.timer.reset();
}
