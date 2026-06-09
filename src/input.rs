// 输入处理模块：点击选择、移动、攻击分发
// 通过 UiCommand Message 发出用户意图，不直接修改游戏状态

use crate::character::{Faction, GridPosition, Unit};
use crate::map::GameMap;
use crate::turn::{TurnOrder, TurnPhase, TurnState};
use crate::ui::events::UiCommand;
use crate::ui::view_models::HoveredEntity;
use bevy::ecs::message::MessageWriter;
use bevy::prelude::*;

/// 处理玩家点击：发送 UiCommand Message
/// 新逻辑：基于 TurnOrder 队列，只有当前单位是玩家阵营时才允许操作
pub fn handle_click(
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    map: Res<GameMap>,
    units: Query<(Entity, &Unit, &GridPosition)>,
    turn_state: Res<TurnState>,
    turn_order: Res<TurnOrder>,
    turn_phase: Res<State<TurnPhase>>,
    mut ui_commands: MessageWriter<UiCommand>,
    mut hovered: ResMut<HoveredEntity>,
) {
    // 只有当前行动单位是玩家阵营时才处理输入
    if turn_state.current_faction != Faction::Player {
        return;
    }

    let left_clicked = mouse_button.just_pressed(MouseButton::Left);
    let right_clicked = mouse_button.just_pressed(MouseButton::Right);

    if !left_clicked && !right_clicked {
        return;
    }

    // 右键在 MoveUnit 阶段发送取消
    if right_clicked {
        if *turn_phase.get() == TurnPhase::MoveUnit {
            ui_commands.write(UiCommand::Cancel);
            return;
        }
        if *turn_phase.get() == TurnPhase::ActionMenu
            || *turn_phase.get() == TurnPhase::SelectTarget
        {
            return;
        }
    }

    let (camera, cam_transform) = match camera_query.single() {
        Ok(c) => c,
        Err(_) => return,
    };
    let coord = match cursor_to_coord(&windows, camera, cam_transform, &map) {
        Some(c) => c,
        None => return,
    };

    match turn_phase.get() {
        TurnPhase::SelectUnit => {
            // 点击任何单位都更新信息面板
            for (entity, unit, gp) in &units {
                if gp.coord == coord {
                    hovered.entity = Some(entity);
                    // 只有当前轮到的玩家单位可以选中进入移动流程
                    if unit.faction == Faction::Player && turn_order.current_unit() == Some(entity)
                    {
                        ui_commands.write(UiCommand::SelectUnit { entity });
                    }
                    return;
                }
            }
            // 点击空格子清除信息面板
            hovered.entity = None;
        }
        TurnPhase::MoveUnit => {
            ui_commands.write(UiCommand::MoveUnit { coord });
        }
        TurnPhase::SelectTarget => {
            ui_commands.write(UiCommand::SelectTarget { coord });
        }
        _ => {}
    }
}

/// 处理右键取消（ActionMenu/SelectTarget 阶段）
pub fn handle_right_cancel(
    mouse_button: Res<ButtonInput<MouseButton>>,
    turn_state: Res<TurnState>,
    turn_phase: Res<State<TurnPhase>>,
    mut ui_commands: MessageWriter<UiCommand>,
) {
    if turn_state.current_faction != Faction::Player {
        return;
    }
    if !mouse_button.just_pressed(MouseButton::Right) {
        return;
    }

    match turn_phase.get() {
        TurnPhase::ActionMenu | TurnPhase::SelectTarget => {
            ui_commands.write(UiCommand::Cancel);
        }
        _ => {}
    }
}

/// 按 E 键结束回合（跳过当前玩家单位及后续所有玩家单位的行动）
pub fn handle_end_turn(
    keyboard: Res<ButtonInput<KeyCode>>,
    turn_state: Res<TurnState>,
    turn_phase: Res<State<TurnPhase>>,
    mut ui_commands: MessageWriter<UiCommand>,
) {
    if turn_state.current_faction != Faction::Player {
        return;
    }
    if !keyboard.just_pressed(KeyCode::KeyE) {
        return;
    }
    if *turn_phase.get() != TurnPhase::SelectUnit {
        return;
    }

    ui_commands.write(UiCommand::EndTurn);
}

/// 屏幕光标 → 格子坐标
pub fn cursor_to_coord(
    windows: &Query<&Window>,
    camera: &Camera,
    cam_transform: &GlobalTransform,
    map: &GameMap,
) -> Option<IVec2> {
    let Ok(window) = windows.single() else {
        return None;
    };
    let cursor_pos = window.cursor_position()?;
    let world_pos = camera
        .viewport_to_world_2d(cam_transform, cursor_pos)
        .ok()?;
    let coord = map.world_to_coord(world_pos);
    map.is_in_bounds(coord).then_some(coord)
}

/// 输入处理插件
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        use crate::turn::AppState;
        app.add_systems(Update, handle_click.run_if(in_state(AppState::InGame)))
            .add_systems(
                Update,
                handle_right_cancel.run_if(in_state(AppState::InGame)),
            )
            .add_systems(Update, handle_end_turn.run_if(in_state(AppState::InGame)));
    }
}
