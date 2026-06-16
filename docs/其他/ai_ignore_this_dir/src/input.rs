// 输入处理模块：点击选择、移动、攻击分发
// 通过 UiCommand Message 发出用户意图，不直接修改游戏状态
// 单位点击：bevy_picking Pointer 事件（Pickable 组件驱动）
// 空格子点击：cursor_to_coord 逻辑计算（不创建 Tile Entity）

use crate::core::character::{Faction, GridPosition, Unit};
use crate::core::map::GameMap;
use crate::core::turn::{TurnOrder, TurnPhase, TurnState};
use crate::ui::UiFocusState;
use crate::ui::events::UiCommand;
use crate::ui::view_models::HoveredEntity;
use bevy::ecs::message::MessageWriter;
use bevy::picking::prelude::*;
use bevy::prelude::*;

// ── Pointer 事件 Observer ──

/// 单位点击：Pointer<Click> 触发，发送对应阶段的 UiCommand
/// 仅处理左键点击，右键由 handle_right_cancel 独立处理
pub fn on_unit_pointer_click(
    trigger: On<Pointer<Click>>,
    unit_query: Query<(&Unit, &GridPosition)>,
    turn_state: Res<TurnState>,
    turn_order: Res<TurnOrder>,
    turn_phase: Option<Res<State<TurnPhase>>>,
    mut ui_commands: MessageWriter<UiCommand>,
) {
    // 只处理左键
    if trigger.event.button != PointerButton::Primary {
        return;
    }
    // TurnPhase 可能在 AppState::MainMenu 下未就绪，跳过
    let Some(turn_phase) = turn_phase else {
        return;
    };
    // 只有当前行动单位是玩家阵营时才处理输入
    if turn_state.current_faction != Faction::Player {
        return;
    }

    let entity = trigger.entity;
    let Ok((unit, gp)) = unit_query.get(entity) else {
        return;
    };

    match turn_phase.get() {
        TurnPhase::SelectUnit => {
            // 只有当前轮到的玩家单位可以选中进入移动流程
            if unit.faction == Faction::Player && turn_order.current_unit() == Some(entity) {
                bevy::log::trace!(
                    target: "input",
                    entity = ?entity,
                    "UiCommand::SelectUnit 消息发送(Pointer<Click>)"
                );
                ui_commands.write(UiCommand::SelectUnit { entity });
            }
        }
        TurnPhase::MoveUnit => {
            bevy::log::trace!(
                target: "input",
                coord = ?gp.coord,
                "UiCommand::MoveUnit 消息发送(Pointer<Click>)"
            );
            ui_commands.write(UiCommand::MoveUnit { coord: gp.coord });
        }
        TurnPhase::SelectTarget => {
            bevy::log::trace!(
                target: "input",
                coord = ?gp.coord,
                "UiCommand::SelectTarget 消息发送(Pointer<Click>)"
            );
            ui_commands.write(UiCommand::SelectTarget { coord: gp.coord });
        }
        _ => {}
    }
}

/// 单位悬停进入：Pointer<Over> 触发，更新 HoveredEntity
pub fn on_unit_pointer_over(
    trigger: On<Pointer<Over>>,
    unit_query: Query<&Unit>,
    mut hovered: ResMut<HoveredEntity>,
) {
    let entity = trigger.entity;
    // 仅处理 Unit 实体，忽略 UI 等其他可拾取实体
    if unit_query.get(entity).is_ok() && hovered.entity != Some(entity) {
        hovered.entity = Some(entity);
    }
}

/// 单位悬停离开：Pointer<Out> 触发，清除 HoveredEntity
pub fn on_unit_pointer_out(trigger: On<Pointer<Out>>, mut hovered: ResMut<HoveredEntity>) {
    let entity = trigger.entity;
    if hovered.entity == Some(entity) {
        hovered.entity = None;
    }
}

// ── 空格子点击（保留 cursor_to_coord 逻辑计算） ──

/// 处理空格子点击：当鼠标不在任何 Unit sprite 上时，用 cursor_to_coord 计算格子坐标
/// 单位点击由 Pointer<Click> Observer 处理，此系统仅处理空地点击
pub fn handle_click(
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    map: Res<GameMap>,
    turn_state: Res<TurnState>,
    turn_phase: Res<State<TurnPhase>>,
    mut ui_commands: MessageWriter<UiCommand>,
    hovered: Res<HoveredEntity>,
    focus_state: Res<UiFocusState>,
) {
    // 只有当前行动单位是玩家阵营时才处理输入
    if turn_state.current_faction != Faction::Player {
        return;
    }

    // UI 面板打开时阻止空格子点击
    if focus_state.blocks_input {
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
            bevy::log::trace!(
                target: "input",
                "UiCommand::Cancel 消息发送(右键MoveUnit阶段)"
            );
            ui_commands.write(UiCommand::Cancel);
            return;
        }
        if *turn_phase.get() == TurnPhase::ActionMenu
            || *turn_phase.get() == TurnPhase::SelectTarget
        {
            return;
        }
    }

    // 鼠标悬停在单位上时，点击由 Pointer<Click> Observer 处理，此处跳过
    if hovered.entity.is_some() {
        return;
    }

    // 空格子点击：用 cursor_to_coord 计算格子坐标
    let (camera, cam_transform) = match camera_query.single() {
        Ok(c) => c,
        Err(_) => return,
    };
    let coord = match cursor_to_coord(&windows, camera, cam_transform, &map) {
        Some(c) => c,
        None => return,
    };

    match turn_phase.get() {
        TurnPhase::MoveUnit => {
            bevy::log::trace!(
                target: "input",
                coord = ?coord,
                "UiCommand::MoveUnit 消息发送(空格子)"
            );
            ui_commands.write(UiCommand::MoveUnit { coord });
        }
        TurnPhase::SelectTarget => {
            bevy::log::trace!(
                target: "input",
                coord = ?coord,
                "UiCommand::SelectTarget 消息发送(空格子)"
            );
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
    focus_state: Res<UiFocusState>,
) {
    if turn_state.current_faction != Faction::Player {
        return;
    }

    // UI 面板打开时阻止右键取消（由 ESC 键处理面板关闭）
    if focus_state.blocks_input {
        return;
    }

    if !mouse_button.just_pressed(MouseButton::Right) {
        return;
    }

    match turn_phase.get() {
        TurnPhase::ActionMenu | TurnPhase::SelectTarget => {
            bevy::log::trace!(
                target: "input",
                "UiCommand::Cancel 消息发送(右键取消)"
            );
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
    focus_state: Res<UiFocusState>,
) {
    if turn_state.current_faction != Faction::Player {
        return;
    }

    // UI 面板打开时阻止快捷键
    if focus_state.blocks_input {
        return;
    }

    if !keyboard.just_pressed(KeyCode::KeyE) {
        return;
    }
    if *turn_phase.get() != TurnPhase::SelectUnit {
        return;
    }

    bevy::log::trace!(
        target: "input",
        "UiCommand::EndTurn 消息发送"
    );
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

/// ESC 键处理：面板打开时关闭面板，否则取消当前游戏操作
/// 不受 UiFocusState 阻止，ESC 始终可用
pub fn handle_esc_key(
    keyboard: Res<ButtonInput<KeyCode>>,
    turn_state: Res<TurnState>,
    turn_phase: Res<State<TurnPhase>>,
    mut ui_commands: MessageWriter<UiCommand>,
) {
    if turn_state.current_faction != Faction::Player {
        return;
    }
    if !keyboard.just_pressed(KeyCode::Escape) {
        return;
    }
    // SelectUnit 阶段无操作可取消
    if *turn_phase.get() == TurnPhase::SelectUnit {
        return;
    }

    bevy::log::trace!(
        target: "input",
        "UiCommand::Cancel 消息发送(ESC键)"
    );
    ui_commands.write(UiCommand::Cancel);
}

/// 输入处理插件
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        use crate::core::turn::AppState;
        app.add_observer(on_unit_pointer_click)
            .add_observer(on_unit_pointer_over)
            .add_observer(on_unit_pointer_out)
            .add_systems(Update, handle_click.run_if(in_state(AppState::InGame)))
            .add_systems(
                Update,
                handle_right_cancel.run_if(in_state(AppState::InGame)),
            )
            .add_systems(Update, handle_end_turn.run_if(in_state(AppState::InGame)))
            .add_systems(Update, handle_esc_key.run_if(in_state(AppState::InGame)));
    }
}
