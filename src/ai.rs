// AI 模块：敌方自动行动

use crate::assets::CnFont;
use crate::combat::{manhattan_distance, skill_range};
use crate::combat_event;
use crate::combat_log::CombatLog;
use crate::map::{GameMap, Tile};
use crate::pathfinding::{build_tile_terrain_map, find_reachable_tiles};
use crate::status::StatusEffects;
use crate::turn::{AiTimer, TurnPhase, TurnState};
use crate::unit::{Faction, GridPosition, Skill, Unit, UnitName};
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

/// 敌方 AI 系统。回合收尾走 `turn::turn_end_on_enter` 统一处理。
pub fn enemy_ai_system(
    time: Res<Time>,
    mut ai_timer: ResMut<AiTimer>,
    turn_state: Res<TurnState>,
    turn_phase: Res<State<TurnPhase>>,
    mut next_phase: ResMut<NextState<TurnPhase>>,
    mut units: Query<(Entity, &mut Unit, &mut GridPosition, &mut Transform, &UnitName)>,
    mut status_effects: Query<&mut StatusEffects>,
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

        let reachable =
            find_reachable_tiles(snapshot.coord, snapshot.mov, &map, &terrain_map, &occupation_map);

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
        let stunned = status_effects
            .get_mut(action.entity)
            .map(|mut s| {
                if s.is_stunned() {
                    s.consume_stun();
                    true
                } else {
                    false
                }
            })
            .unwrap_or(false);

        if stunned {
            if let Ok((_, mut unit, _, _, _)) = units.get_mut(action.entity) {
                unit.acted = true;
            }
            continue;
        }

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
                    .find_map(|t| if t.coord == target_gp.coord { Some(t.terrain) } else { None })
                    .unwrap_or(crate::map::Terrain::Plain);

                let attacker_atk_mod = status_effects
                    .get(action.entity)
                    .map(|s| s.attack_mod())
                    .unwrap_or(0);
                let defender_def_mod = status_effects
                    .get(target_entity)
                    .map(|s| s.defense_mod())
                    .unwrap_or(0);

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

                // 统一攻击处理
                combat_event::execute_attack(
                    &mut commands,
                    &attacker,
                    attacker_atk_mod,
                    &action.attacker_name,
                    target_entity,
                    &mut target_unit,
                    defender_def_mod,
                    target_name,
                    target_transform.translation.truncate(),
                    terrain,
                    &cn_font,
                    &mut combat_log,
                );
            }
        }

        // 标记已行动
        if let Ok((_, mut unit, _, _, _)) = units.get_mut(action.entity) {
            unit.acted = true;
        }
    }

    next_phase.set(TurnPhase::TurnEnd);
}

/// AI 插件
pub struct AiPlugin;

impl Plugin for AiPlugin {
    fn build(&self, app: &mut App) {
        use crate::turn::AppState;
        app.add_systems(Update, enemy_ai_system.run_if(in_state(AppState::InGame)));
    }
}
