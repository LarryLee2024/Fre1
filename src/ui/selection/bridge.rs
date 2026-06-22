//! Selection Bridge — PickIntent → Domain Event 桥接
//!
//! 消费 Picking 层产生的 PickIntent 事件，转换为领域事件：
//! - `UnitClicked` — 单位被点击
//! - `TileClicked` — 格子被点击
//! - `SelectionCleared` — 选择被清除
//!
//! 这些事件在此处定义为模块级事件，后续迁移到 core/domains/tactical/events.rs。
//!
//! 详见 ADR-068 §Module Design。

use bevy::picking::pointer::PointerButton;
use bevy::prelude::*;

use crate::core::domains::combat::components::UnitIdComponent;
use crate::ui::picking::pick_target::{InteractionPhase, PickIntent, PickTarget};
use crate::ui::selection::pick_context::PickContext;
use crate::ui::selection::state::SelectionState;

// ─── 模块级领域事件（临时定义，后续迁移到 core/domains） ────────────

/// 单位被点击事件
#[derive(Event, Debug, Clone)]
pub struct UnitClicked {
    /// 被点击的单位 ID
    pub unit_id: String,
    /// 点击交互阶段
    pub phase: InteractionPhase,
    /// 当前选择上下文
    pub context: PickContext,
}

/// 格子被点击事件
#[derive(Event, Debug, Clone)]
pub struct TileClicked {
    /// 被点击的格子位置（由 PickTarget::Tile 携带）
    pub phase: InteractionPhase,
}

/// 选择被清除事件
#[derive(Event, Debug, Clone)]
pub struct SelectionCleared;

// ─── Observer ──────────────────────────────────────────────────────

/// PickIntent 消费者 — 将 PickIntent 转换为领域事件
///
/// 根据 PickIntent.target 和当前 PickContext 决定触发何种领域事件：
/// - `PickTarget::Unit` + `Commit` → `UnitClicked`
/// - `PickTarget::Tile` + `Commit` → `TileClicked`
/// - `PickTarget::Empty` + `Commit` → `SelectionCleared`
/// - `Preview` / `PreviewEnd` → 更新 SelectionState 的 hovered 状态
///
/// 右键点击（Secondary）直接清除选择。
pub fn on_pick_intent(
    ev: On<PickIntent>,
    mut commands: Commands,
    mut selection_state: ResMut<SelectionState>,
    unit_ids: Query<&UnitIdComponent>,
) {
    // 右键点击 → 清除选择（任何阶段）
    if ev.event().button == PointerButton::Secondary && ev.event().phase == InteractionPhase::Commit
    {
        info!(
            target: "ui::selection",
            "[Selection] Right-click — clearing selection",
        );
        selection_state.clear();
        commands.trigger(SelectionCleared);
        return;
    }

    match ev.event().phase {
        InteractionPhase::Commit => {
            handle_commit(ev.event(), &mut commands, &mut selection_state, &unit_ids)
        }
        InteractionPhase::Preview => handle_preview(ev.event(), &mut selection_state),
        InteractionPhase::PreviewEnd => handle_preview_end(ev.event(), &mut selection_state),
    }
}

/// 处理 Commit 阶段（点击确认）
fn handle_commit(
    intent: &PickIntent,
    commands: &mut Commands,
    selection_state: &mut SelectionState,
    _unit_ids: &Query<&UnitIdComponent>,
) {
    match &intent.target {
        PickTarget::Unit(id) => {
            selection_state.selected = Some(intent.target.clone());
            let unit_id = id.clone();
            commands.trigger(UnitClicked {
                unit_id,
                phase: InteractionPhase::Commit,
                context: selection_state.context,
            });
        }
        PickTarget::Tile(_pos) => {
            selection_state.selected = Some(intent.target.clone());
            commands.trigger(TileClicked {
                phase: InteractionPhase::Commit,
            });
        }
        PickTarget::Empty => {
            selection_state.clear();
            commands.trigger(SelectionCleared);
        }
    }
}

/// 处理 Preview 阶段（悬停进入）
fn handle_preview(intent: &PickIntent, selection_state: &mut SelectionState) {
    selection_state.hovered = Some(intent.target.clone());
}

/// 处理 PreviewEnd 阶段（悬停离开）
fn handle_preview_end(intent: &PickIntent, selection_state: &mut SelectionState) {
    // 仅当悬停目标匹配时才清除（避免清除其他原因设置的 hovered）
    if selection_state.hovered == Some(intent.target.clone()) {
        selection_state.hovered = None;
    }
}
