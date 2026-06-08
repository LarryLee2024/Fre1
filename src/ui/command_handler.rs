// UI 命令处理器：接收 UiCommand Message，执行游戏逻辑
// 所有 UI→Logic 的交互都通过此模块，UI 层不再直接修改 CombatIntent/TurnPhase

use crate::battle::{CombatIntent, PrevPosition, manhattan_distance};
use crate::character::{
    AttackRange, Faction, GridPosition, MovableRange, MovingUnit, PathArrow, Selected,
    SelectionHighlight, Unit, spawn_path_arrows, despawn_path_arrows,
};
use crate::gameplay::attribute::{AttributeKind, Attributes};
use crate::gameplay::tag::GameplayTags;
use crate::input::{
    clear_markers, clear_selection, show_attack_range, show_move_range, spawn_selection_highlight,
};
use crate::map::{GameMap, TerrainCostRegistry, TerrainMapCache, reconstruct_path, find_reachable_tiles};
use crate::skill::{BASIC_ATTACK_ID, SkillRegistry, SkillSlots, effective_skill_range};
use crate::turn::{ForceEndFaction, TurnPhase};
use crate::ui::action_menu::{ActionMenuEntity, despawn_action_menu, spawn_action_menu};
use crate::ui::events::UiCommand;
use bevy::ecs::message::MessageReader;
use bevy::prelude::*;

/// 处理 UI 命令：将用户意图转化为游戏状态变更
pub fn handle_ui_commands(
    mut commands: Commands,
    mut events: MessageReader<UiCommand>,
    mut next_phase: ResMut<NextState<TurnPhase>>,
    mut combat_intent: ResMut<CombatIntent>,
    mut menu_entity: ResMut<ActionMenuEntity>,
    map: Res<GameMap>,
    terrain_cache: Res<TerrainMapCache>,
    cost_registry: Res<TerrainCostRegistry>,
    units: Query<(
        Entity,
        &Unit,
        &GridPosition,
        &Transform,
        &Attributes,
        &SkillSlots,
        &GameplayTags,
    )>,
    selected_query: Query<Entity, With<Selected>>,
    range_entities: Query<
        (Entity, Option<&GridPosition>),
        Or<(With<MovableRange>, With<AttackRange>)>,
    >,
    highlights: Query<Entity, With<SelectionHighlight>>,
    prev_position: Res<PrevPosition>,
    skill_registry: Res<SkillRegistry>,
    turn_phase: Res<State<TurnPhase>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
) {
    // 使用缓存的地形图
    let terrain_map = &terrain_cache.map;

    for cmd in events.read() {
        match cmd {
            UiCommand::SelectUnit { entity } => {
                clear_selection(&mut commands, &selected_query, &range_entities, &highlights);
                commands.entity(*entity).insert(Selected);
                if let Ok((_, unit, gp, _, _, _, tags)) = units.get(*entity) {
                    // 根据单位标签解析地形成本计算器
                    let calculator = cost_registry.resolve_from_tags(tags);
                    show_move_range(
                        &mut commands,
                        &map,
                        terrain_map,
                        &units,
                        unit,
                        gp.coord,
                        calculator,
                    );
                    spawn_selection_highlight(&mut commands, &map, gp.coord);
                }
                next_phase.set(TurnPhase::MoveUnit);
            }

            UiCommand::MoveUnit { coord } => {
                let is_movable = range_entities
                    .iter()
                    .any(|(_, gp)| gp.map(|g| g.coord == *coord).unwrap_or(false));

                if is_movable {
                    if let Ok(selected_entity) = selected_query.single() {
                        if let Ok((_, _, old_gp, _, _, _, tags)) = units.get(selected_entity) {
                            commands.insert_resource(PrevPosition {
                                coord: Some(old_gp.coord),
                            });

                            // 计算路径
                            let calculator = cost_registry.resolve_from_tags(tags);
                            let mov = units
                                .get(selected_entity)
                                .map(|(_, _, _, _, attrs, _, _)| attrs.get(AttributeKind::Mov) as u32)
                                .unwrap_or(3);
                            let occupation_map = std::collections::HashMap::new(); // 移动时不需要阻挡
                            let reachable = find_reachable_tiles(
                                old_gp.coord,
                                mov,
                                &map,
                                terrain_map,
                                &occupation_map,
                                calculator,
                            );
                            let path = reconstruct_path(
                                old_gp.coord,
                                *coord,
                                &reachable,
                                mov,
                                &map,
                                terrain_map,
                                calculator,
                            );

                            // 生成导航箭头
                            spawn_path_arrows(&mut commands, &map, &path);

                            // 挂载移动动画组件
                            commands.entity(selected_entity).insert(MovingUnit {
                                path,
                                current_index: 0,
                                speed: 0.15,
                                elapsed: 0.0,
                                next_phase: TurnPhase::ActionMenu,
                            });
                        }
                        for h in &highlights {
                            commands.entity(h).try_despawn();
                        }
                        spawn_selection_highlight(&mut commands, &map, *coord);
                    }
                }
                for (marker, _) in &range_entities {
                    commands.entity(marker).try_despawn();
                }
                // 不再立即弹出行动菜单，等移动动画完成后由 animate_movement 切换
            }

            UiCommand::Attack => {
                combat_intent.skill_id = Some(BASIC_ATTACK_ID.to_string());
                show_range_for_selected(
                    &units,
                    &selected_query,
                    &mut commands,
                    &map,
                    &skill_registry,
                    BASIC_ATTACK_ID,
                );
                despawn_action_menu(&mut commands, &mut menu_entity);
                next_phase.set(TurnPhase::SelectTarget);
            }

            UiCommand::Skill { skill_id } => {
                combat_intent.skill_id = Some(skill_id.clone());
                show_range_for_selected(
                    &units,
                    &selected_query,
                    &mut commands,
                    &map,
                    &skill_registry,
                    skill_id,
                );
                despawn_action_menu(&mut commands, &mut menu_entity);
                next_phase.set(TurnPhase::SelectTarget);
            }

            UiCommand::SelectTarget { coord } => {
                // 检查点击坐标是否有敌方单位
                let clicked_enemy = units
                    .iter()
                    .find(|(_, u, gp, _, _, _, _)| {
                        u.faction == Faction::Enemy && gp.coord == *coord
                    })
                    .map(|(_, _, gp, _, _, _, _)| gp.coord);

                if let Some(enemy_coord) = clicked_enemy {
                    if let Ok(selected_entity) = selected_query.single() {
                        if let Ok((_, _, sel_gp, _, attrs, _, _)) = units.get(selected_entity) {
                            let skill_id =
                                combat_intent.skill_id.as_deref().unwrap_or(BASIC_ATTACK_ID);
                            if let Some(skill_data) = skill_registry.get(skill_id) {
                                let base_range = attrs.get(AttributeKind::AttackRange) as u32;
                                let effective_range = effective_skill_range(skill_data, base_range);
                                if manhattan_distance(sel_gp.coord, enemy_coord) <= effective_range
                                {
                                    combat_intent.target_coord = Some(enemy_coord);
                                    next_phase.set(TurnPhase::ExecuteAction);
                                    return;
                                }
                            }
                        }
                    }
                }
                // 未选中有效目标，回到行动菜单
                clear_markers(&mut commands, &range_entities, &highlights);
                spawn_menu_at_selected(
                    &mut commands,
                    &units,
                    &selected_query,
                    &map,
                    &camera_query,
                    &mut menu_entity,
                    &skill_registry,
                );
                next_phase.set(TurnPhase::ActionMenu);
            }

            UiCommand::Wait => {
                despawn_action_menu(&mut commands, &mut menu_entity);
                next_phase.set(TurnPhase::WaitAction);
            }

            UiCommand::Cancel => {
                match turn_phase.get() {
                    TurnPhase::ActionMenu => {
                        despawn_action_menu(&mut commands, &mut menu_entity);
                        // 回退位置
                        if let Ok(selected_entity) = selected_query.single() {
                            if let Some(prev_coord) = prev_position.coord {
                                let world_pos = map.coord_to_world(prev_coord);
                                commands
                                    .entity(selected_entity)
                                    .insert(Transform::from_xyz(world_pos.x, world_pos.y, 1.0))
                                    .insert(GridPosition { coord: prev_coord });
                            }
                        }
                        clear_selection(
                            &mut commands,
                            &selected_query,
                            &range_entities,
                            &highlights,
                        );
                        combat_intent.target_coord = None;
                        combat_intent.skill_id = None;
                        next_phase.set(TurnPhase::SelectUnit);
                    }
                    TurnPhase::SelectTarget => {
                        clear_markers(&mut commands, &range_entities, &highlights);
                        combat_intent.target_coord = None;
                        combat_intent.skill_id = None;
                        spawn_menu_at_selected(
                            &mut commands,
                            &units,
                            &selected_query,
                            &map,
                            &camera_query,
                            &mut menu_entity,
                            &skill_registry,
                        );
                        next_phase.set(TurnPhase::ActionMenu);
                    }
                    TurnPhase::MoveUnit => {
                        clear_selection(
                            &mut commands,
                            &selected_query,
                            &range_entities,
                            &highlights,
                        );
                        combat_intent.target_coord = None;
                        next_phase.set(TurnPhase::SelectUnit);
                    }
                    _ => {}
                }
            }

            UiCommand::EndTurn => {
                clear_selection(&mut commands, &selected_query, &range_entities, &highlights);
                // 插入 ForceEndFaction 标记，由 turn_end_on_enter 消费
                commands.insert_resource(ForceEndFaction(true));
                next_phase.set(TurnPhase::TurnEnd);
            }
        }
    }
}

// ── 辅助函数 ──

/// 在选中单位位置弹出行动菜单
fn spawn_menu_at_selected(
    commands: &mut Commands,
    units: &Query<(
        Entity,
        &Unit,
        &GridPosition,
        &Transform,
        &Attributes,
        &SkillSlots,
        &GameplayTags,
    )>,
    selected_query: &Query<Entity, With<Selected>>,
    map: &GameMap,
    camera_query: &Query<(&Camera, &GlobalTransform)>,
    menu_entity: &mut ActionMenuEntity,
    skill_registry: &SkillRegistry,
) {
    if let (Ok(selected_entity), Ok((camera, cam_transform))) =
        (selected_query.single(), camera_query.single())
    {
        if let Ok((_, unit, gp, _, _, skill_slots, _)) = units.get(selected_entity) {
            let unit_world = map.coord_to_world(gp.coord);
            if let Ok(screen_pos) = camera.world_to_viewport(cam_transform, unit_world.extend(1.0))
            {
                spawn_action_menu(
                    commands,
                    screen_pos.x,
                    screen_pos.y,
                    unit,
                    skill_slots,
                    menu_entity,
                    skill_registry,
                );
            }
        }
    }
}

/// 显示选中单位的攻击/技能范围
fn show_range_for_selected(
    units: &Query<(
        Entity,
        &Unit,
        &GridPosition,
        &Transform,
        &Attributes,
        &SkillSlots,
        &GameplayTags,
    )>,
    selected_query: &Query<Entity, With<Selected>>,
    commands: &mut Commands,
    map: &GameMap,
    skill_registry: &SkillRegistry,
    skill_id: &str,
) {
    if let Ok(selected_entity) = selected_query.single() {
        if let Ok((_, _, gp, _, attrs, _, _)) = units.get(selected_entity) {
            let base_range = attrs.get(AttributeKind::AttackRange) as u32;
            if let Some(skill_data) = skill_registry.get(skill_id) {
                let range = effective_skill_range(skill_data, base_range);
                show_attack_range(commands, map, gp.coord, range);
            }
        }
    }
}
