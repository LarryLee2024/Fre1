// AI 模块：敌方自动行动，直接执行效果

use crate::assets::CnFont;
use crate::combat::manhattan_distance;
use crate::combat_log::{CombatLog, LogSegment, log_color};
use crate::core::attribute::{AttributeKind, Attributes};
use crate::core::effect::calculate_damage_from_effect;
use crate::core::tag::GameplayTags;
use crate::data::buff_data::{apply_buff, ActiveBuffs, BuffRegistry};
use crate::data::skill_data::{effective_skill_range, SkillCooldowns, SkillRegistry, SkillSlots};
use crate::map::{GameMap, Terrain, Tile};
use crate::pathfinding::{build_tile_terrain_map, find_reachable_tiles};
use crate::turn::{AiTimer, TurnPhase, TurnState};
use crate::unit::{Faction, GridPosition, Unit, UnitName};
use crate::vfx;
use bevy::prelude::*;

/// 敌方 AI 系统
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
    let snapshots: Vec<(Entity, Faction, IVec2, f32, f32, f32, u32, u32, bool, String, Vec<String>, SkillCooldowns)> = units
        .iter()
        .map(|(e, u, gp, _, name, attrs, skills, cooldowns, _, _)| {
            (
                e,
                u.faction,
                gp.coord,
                attrs.get(AttributeKind::Atk),
                attrs.get(AttributeKind::Def),
                attrs.base.get(&AttributeKind::Def).copied().unwrap_or(0.0),
                attrs.get(AttributeKind::Mov) as u32,
                attrs.get(AttributeKind::AttackRange) as u32,
                u.acted,
                name.0.clone(),
                skills.skill_ids.clone(),
                cooldowns.clone(),
            )
        })
        .collect();

    let terrain_map = build_tile_terrain_map(&tiles);

    let player_positions: Vec<IVec2> = snapshots
        .iter()
        .filter(|s| s.1 == Faction::Player)
        .map(|s| s.2)
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
        base_def: f32,
    }

    let mut actions: Vec<AiAction> = Vec::new();

    for snapshot in snapshots
        .iter()
        .filter(|s| s.1 == Faction::Enemy && !s.8) // !acted
    {
        let nearest = *player_positions
            .iter()
            .min_by_key(|pos| manhattan_distance(snapshot.2, **pos))
            .unwrap();

        let occupation_map: std::collections::HashMap<IVec2, bool> = snapshots
            .iter()
            .filter(|s| s.1 == Faction::Player)
            .map(|s| (s.2, true))
            .collect();

        let reachable =
            find_reachable_tiles(snapshot.2, snapshot.6, &map, &terrain_map, &occupation_map);

        let best_coord = reachable
            .iter()
            .min_by_key(|(coord, _)| manhattan_distance(**coord, nearest))
            .map(|(coord, _)| *coord)
            .unwrap_or(snapshot.2);

        // 选择使用的技能：优先特殊技能（跳过冷却中的），否则基础攻击
        let skill_id = snapshot
            .10 // skill_ids
            .iter()
            .find(|id| *id != "basic_attack" && snapshot.11.get(id) == 0) // cooldowns
            .map(|s| s.as_str())
            .unwrap_or("basic_attack");

        let effective_range = skill_registry
            .get(skill_id)
            .map(|sd| effective_skill_range(sd, snapshot.7)) // attack_range
            .unwrap_or(snapshot.7);

        let attack_target = snapshots
            .iter()
            .filter(|s| s.1 == Faction::Player)
            .find(|s| manhattan_distance(best_coord, s.2) <= effective_range)
            .map(|s| s.0); // entity

        actions.push(AiAction {
            entity: snapshot.0,
            move_to: best_coord,
            attack_target,
            skill_id: skill_id.to_string(),
            atk: snapshot.3,
            base_def: snapshot.5,
        });
    }

    // 应用行动
    for action in actions {
        // 移动
        let world_pos = map.coord_to_world(action.move_to);
        if let Ok((_, _, mut gp, mut transform, _, _, _, _, _, _)) = units.get_mut(action.entity) {
            gp.coord = action.move_to;
            transform.translation.x = world_pos.x;
            transform.translation.y = world_pos.y;
        }

        // 直接执行攻击效果（不经过 EffectQueue）
        if let Some(target_entity) = action.attack_target {
            if let Some(skill_data) = skill_registry.get(&action.skill_id) {
                // 获取目标信息
                let (target_name, target_attrs_snapshot, target_gp_coord, target_terrain) = units
                    .get(target_entity)
                    .map(|(_, _, gp, _, name, attrs, _, _, _, _)| {
                        let terrain = tiles
                            .iter()
                            .find(|t| t.coord == gp.coord)
                            .map(|t| t.terrain)
                            .unwrap_or(Terrain::Plain);
                        (name.0.clone(), attrs.clone(), gp.coord, terrain)
                    })
                    .unwrap_or(("???".to_string(), Attributes::default(), IVec2::ZERO, Terrain::Plain));

                // 设置冷却
                if skill_data.cooldown > 0 {
                    if let Ok((_, _, _, _, _, _, _, mut cooldowns, _, _)) = units.get_mut(action.entity) {
                        cooldowns.set(&action.skill_id, skill_data.cooldown);
                    }
                }

                // 计算伤害
                for effect_def in &skill_data.effects {
                    match effect_def {
                        crate::core::effect::EffectDef::Damage { multiplier, ignore_def_percent } => {
                            let effective_atk = action.atk;
                            let effective_def = target_attrs_snapshot.get(AttributeKind::Def);
                            let base_def = action.base_def;

                            let amount = calculate_damage_from_effect(
                                effective_atk,
                                effective_def,
                                base_def,
                                *multiplier,
                                *ignore_def_percent,
                                target_terrain,
                            );

                            // 扣血
                            if let Ok((_, _, _, _, _, mut attrs, _, _, _, _)) = units.get_mut(target_entity) {
                                let hp = attrs.get(AttributeKind::Hp);
                                let new_hp = (hp - amount as f32).max(0.0);
                                attrs.set_base(AttributeKind::Hp, new_hp);
                            }

                            // 伤害数字弹出
                            let target_world_pos = map.coord_to_world(target_gp_coord);
                            vfx::spawn_damage_popup(&mut commands, target_world_pos, amount, &cn_font.handle, false);

                            // 战斗日志
                            let attacker_name = units
                                .get(action.entity)
                                .map(|(_, _, _, _, name, _, _, _, _, _)| name.0.clone())
                                .unwrap_or("???".to_string());

                            combat_log.push(vec![
                                LogSegment { text: format!("[{}]", attacker_name), color: log_color::ENEMY },
                                LogSegment { text: " uses ".to_string(), color: log_color::NORMAL },
                                LogSegment { text: format!("[{}]", skill_data.name), color: log_color::TURN },
                                LogSegment { text: " attacks ".to_string(), color: log_color::NORMAL },
                                LogSegment { text: format!("[{}]", target_name), color: log_color::PLAYER },
                                LogSegment { text: " dealing ".to_string(), color: log_color::NORMAL },
                                LogSegment { text: format!("[{}]", amount), color: log_color::DAMAGE },
                                LogSegment { text: " damage".to_string(), color: log_color::NORMAL },
                            ]);

                            // 击杀检查
                            if let Ok((_, _, _, _, _, attrs, _, _, _, _)) = units.get(target_entity) {
                                if attrs.get(AttributeKind::Hp) <= 0.0 {
                                    combat_log.push(vec![
                                        LogSegment { text: format!("[{}]", target_name), color: log_color::PLAYER },
                                        LogSegment { text: " was defeated!".to_string(), color: log_color::KILL },
                                    ]);
                                    commands.entity(target_entity).try_despawn();
                                }
                            }
                        }
                        crate::core::effect::EffectDef::ApplyBuff { buff_id, duration } => {
                            if let (Ok((_, _, _, _, _, mut attrs, _, _, mut buffs, mut tags)), Some(buff_data)) = (
                                units.get_mut(target_entity),
                                buff_registry.get(buff_id),
                            ) {
                                apply_buff(
                                    &mut buffs,
                                    &mut attrs,
                                    &mut tags,
                                    buff_data,
                                    Some(action.entity),
                                    *duration,
                                );
                            }
                        }
                        crate::core::effect::EffectDef::Heal { .. } => {
                            // AI 不会治疗玩家
                        }
                        crate::core::effect::EffectDef::Cleanse => {
                            // AI 不会净化玩家
                        }
                    }
                }
            }
        }

        // 标记已行动
        if let Ok((_, mut unit, _, _, _, _, _, _, _, _)) = units.get_mut(action.entity) {
            unit.acted = true;
        }
    }

    // 检查是否所有敌方单位都已行动
    let all_enemy_acted = units
        .iter()
        .filter(|(_, u, _, _, _, _, _, _, _, _)| u.faction == Faction::Enemy)
        .all(|(_, u, _, _, _, _, _, _, _, _)| u.acted);

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
