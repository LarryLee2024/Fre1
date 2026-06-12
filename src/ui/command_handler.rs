// UI 命令处理器：接收 UiCommand Message，执行游戏逻辑
// 所有 UI→Logic 的交互都通过此模块，UI 层不再直接修改 CombatIntent/TurnPhase

use crate::battle::{CombatIntent, PrevPosition, manhattan_distance};
use crate::character::{AttackRange, Faction, GridPosition, MovableRange, Selected, SelectionHighlight, Unit};
use crate::core::attribute::{AttributeKind, Attributes};
use crate::core::tag::GameplayTags;
use crate::map::{GameMap, OccupancyGrid, TerrainCostRegistry, TerrainGrid, TerrainRegistry};
use crate::skill::{BASIC_ATTACK_ID, SkillRegistry, SkillSlots, effective_skill_range};
use crate::turn::{ForceEndTurn, TurnPhase};
use crate::ui::action_menu::{ActionMenuEntity, despawn_action_menu};
use crate::ui::events::{IntentSource, MovementIntent, UiCommand};
use crate::ui::highlight::{
    clear_markers, clear_selection, show_attack_range, show_move_range, spawn_selection_highlight,
};
use crate::ui::theme::UiTheme;
use bevy::ecs::message::MessageReader;
use bevy::prelude::*;

/// 处理 UI 命令：将用户意图转化为游戏状态变更
///
/// 使用 Local<Option<IVec2>> 替代 Res<PrevPosition>，
/// 避免系统参数超过 Bevy 16 个上限
pub fn handle_ui_commands(
    mut commands: Commands,
    mut events: MessageReader<UiCommand>,
    mut next_phase: ResMut<NextState<TurnPhase>>,
    mut combat_intent: ResMut<CombatIntent>,
    mut menu_entity: ResMut<ActionMenuEntity>,
    map: Res<GameMap>,
    // 合并地形相关 Res 为元组参数，避免系统参数超过 Bevy 16 个上限
    terrain: (Res<TerrainGrid>, Res<TerrainRegistry>),
    occupancy: Res<OccupancyGrid>,
    cost_registry: Res<TerrainCostRegistry>,
    theme: Res<UiTheme>,
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
    // 使用 Local 替代 Res<PrevPosition>，减少系统参数数量
    mut prev_coord: Local<Option<IVec2>>,
    skill_registry: Res<SkillRegistry>,
) {
    for cmd in events.read() {
        match cmd {
            UiCommand::SelectUnit { entity } => {
                clear_selection(&mut commands, &selected_query, &range_entities, &highlights);
                commands.entity(*entity).insert(Selected);
                if let Ok((_, unit, gp, _, _, _, tags)) = units.get(*entity) {
                    let calculator = cost_registry.resolve_from_tags(tags);
                    show_move_range(
                        &mut commands,
                        &map,
                        &terrain.0,
                        &terrain.1,
                        &occupancy,
                        &units,
                        unit,
                        gp.coord,
                        calculator,
                        &theme,
                    );
                    spawn_selection_highlight(&mut commands, &map, gp.coord, &theme);
                }
                next_phase.set(TurnPhase::MoveUnit);
            }

            UiCommand::MoveUnit { coord } => {
                // 检查是否点击了选中单位的当前位置（原地不动）
                if let Ok(selected_entity) = selected_query.single() {
                    if let Ok((_, _, sel_gp, _, _, _, _)) = units.get(selected_entity) {
                        if sel_gp.coord == *coord {
                            *prev_coord = Some(sel_gp.coord);
                            commands.insert_resource(PrevPosition {
                                coord: Some(sel_gp.coord),
                            });
                            for (marker, _) in &range_entities {
                                commands.entity(marker).try_despawn();
                            }
                            for h in &highlights {
                                commands.entity(h).try_despawn();
                            }
                            spawn_selection_highlight(&mut commands, &map, sel_gp.coord, &theme);
                            next_phase.set(TurnPhase::ActionMenu);
                            return;
                        }
                    }
                }

                let is_movable = range_entities
                    .iter()
                    .any(|(_, gp)| gp.map(|g| g.coord == *coord).unwrap_or(false));

                if is_movable {
                    if let Ok(selected_entity) = selected_query.single() {
                        if let Ok((_, _, old_gp, _, _, _, _)) = units.get(selected_entity) {
                            *prev_coord = Some(old_gp.coord);
                            commands.insert_resource(PrevPosition {
                                coord: Some(old_gp.coord),
                            });

                            // 发送移动意图消息，由统一执行系统处理
                            commands.write_message(MovementIntent {
                                entity: selected_entity,
                                target_coord: *coord,
                                source: IntentSource::Player,
                            });
                        }
                        for h in &highlights {
                            commands.entity(h).try_despawn();
                        }
                        spawn_selection_highlight(&mut commands, &map, *coord, &theme);
                    }
                }
                for (marker, _) in &range_entities {
                    commands.entity(marker).try_despawn();
                }
            }

            UiCommand::Attack => {
                combat_intent.skill_id = Some(BASIC_ATTACK_ID.to_string());
                show_range_for_selected(
                    &units,
                    &selected_query,
                    &mut commands,
                    &map,
                    &skill_registry,
                    &theme,
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
                    &theme,
                    skill_id,
                );
                despawn_action_menu(&mut commands, &mut menu_entity);
                next_phase.set(TurnPhase::SelectTarget);
            }

            UiCommand::SelectTarget { coord } => {
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
                // 未选中有效目标，回到行动菜单（on_enter_action_menu 自动弹出）
                clear_markers(&mut commands, &range_entities, &highlights);
                despawn_action_menu(&mut commands, &mut menu_entity);
                next_phase.set(TurnPhase::ActionMenu);
            }

            UiCommand::Wait => {
                despawn_action_menu(&mut commands, &mut menu_entity);
                next_phase.set(TurnPhase::WaitAction);
            }

            UiCommand::Cancel => {
                // 从上下文推断当前阶段：
                // - 有 skill_id → SelectTarget 阶段（取消回到 ActionMenu）
                // - 有菜单实体 → ActionMenu 阶段（取消回到 SelectUnit）
                // - 否则 → MoveUnit 阶段（取消回到 SelectUnit）
                if combat_intent.skill_id.is_some() {
                    // SelectTarget 取消 → 回到 ActionMenu
                    clear_markers(&mut commands, &range_entities, &highlights);
                    combat_intent.target_coord = None;
                    combat_intent.skill_id = None;
                    despawn_action_menu(&mut commands, &mut menu_entity);
                    // on_enter_action_menu 会自动弹出菜单
                    next_phase.set(TurnPhase::ActionMenu);
                } else if menu_entity.entity.is_some() {
                    // ActionMenu 取消 → 回退位置，回到 SelectUnit
                    despawn_action_menu(&mut commands, &mut menu_entity);
                    if let Ok(selected_entity) = selected_query.single() {
                        // 从 Local 读取前一步位置
                        if let Some(prev) = *prev_coord {
                            let world_pos = map.coord_to_world(prev);
                            commands
                                .entity(selected_entity)
                                .insert(Transform::from_xyz(world_pos.x, world_pos.y, 1.0))
                                .insert(GridPosition { coord: prev });
                        }
                    }
                    clear_selection(&mut commands, &selected_query, &range_entities, &highlights);
                    combat_intent.target_coord = None;
                    combat_intent.skill_id = None;
                    next_phase.set(TurnPhase::SelectUnit);
                } else {
                    // MoveUnit 取消 → 回到 SelectUnit
                    clear_selection(&mut commands, &selected_query, &range_entities, &highlights);
                    combat_intent.target_coord = None;
                    next_phase.set(TurnPhase::SelectUnit);
                }
            }

            UiCommand::EndTurn => {
                clear_selection(&mut commands, &selected_query, &range_entities, &highlights);
                // 发送强制结束回合消息，替代 ForceEndFaction Resource
                bevy::log::trace!(
                    target: "input",
                    "ForceEndTurn 消息发送"
                );
                commands.write_message(ForceEndTurn);
                next_phase.set(TurnPhase::TurnEnd);
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
    theme: &UiTheme,
    skill_id: &str,
) {
    if let Ok(selected_entity) = selected_query.single() {
        if let Ok((_, _, gp, _, attrs, _, _)) = units.get(selected_entity) {
            let base_range = attrs.get(AttributeKind::AttackRange) as u32;
            if let Some(skill_data) = skill_registry.get(skill_id) {
                let range = effective_skill_range(skill_data, base_range);
                show_attack_range(commands, map, gp.coord, range, theme);
            }
        }
    }
}
