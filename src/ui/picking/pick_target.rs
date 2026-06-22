//! PickTarget — Picking 系统的核心产出枚举
//!
//! 定义了玩家通过点击/悬停选中的目标类型（单位/格子/空）。
//! 由 intent/click.rs 和 intent/hover.rs 构造 PickIntent 事件，
//! 由 selection/bridge.rs 消费并转换为领域事件。
//!
//! 详见 ADR-068 §Module Design。

use bevy::prelude::*;

use crate::core::domains::tactical::components::GridPos;

/// Picking 系统的核心产出 — 玩家选中了什么
///
/// 经过 PickTarget 检测后，由 PickIntent 传递给 Selection 层处理。
/// BattleUnitId 当前使用字符串表示，后续迁移到强类型 ID（ADR-068）。
#[derive(Debug, Clone, PartialEq)]
pub enum PickTarget {
    /// 选中了一个战斗单位（使用单位 ID 字符串临时替代强类型）
    Unit(String),
    /// 选中了一个网格格子
    Tile(GridPos),
    /// 点击了空白区域
    Empty,
}

/// InteractionPhase — 交互阶段
///
/// Preview（悬停进入）、PreviewEnd（悬停离开）、Commit（点击确认）。
/// hover.rs 产出 Preview/PreviewEnd，click.rs 产出 Commit。
#[derive(Debug, Clone, PartialEq)]
pub enum InteractionPhase {
    /// 鼠标移入目标（Pointer<Over>）
    Preview,
    /// 鼠标移出目标（Pointer<Out>）
    PreviewEnd,
    /// 点击确认目标（Pointer<Click>）
    Commit,
}

/// PickIntent — Picking 层的最终产出事件
///
/// 由 intent/click.rs 和 intent/hover.rs 触发，
/// 由 bridge.rs 消费并转换为领域事件（UnitClicked / TileClicked 等）。
#[derive(Event, Debug, Clone, PartialEq)]
pub struct PickIntent {
    /// 选中的目标
    pub target: PickTarget,
    /// 交互阶段
    pub phase: InteractionPhase,
    /// 按下的鼠标按钮
    pub button: PointerButton,
}

impl PickIntent {
    /// 创建一个点击提交意图
    pub fn commit(target: PickTarget, button: PointerButton) -> Self {
        Self {
            target,
            phase: InteractionPhase::Commit,
            button,
        }
    }

    /// 创建一个悬停预览意图
    pub fn preview(target: PickTarget) -> Self {
        Self {
            target,
            phase: InteractionPhase::Preview,
            button: PointerButton::Primary,
        }
    }

    /// 创建一个悬停结束意图
    pub fn preview_end(target: PickTarget) -> Self {
        Self {
            target,
            phase: InteractionPhase::PreviewEnd,
            button: PointerButton::Primary,
        }
    }
}
