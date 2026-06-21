//! Overlay — 地图覆盖层（光标高亮、移动范围、交互区域）
//!
//! V1 提供基础覆盖层组件定义。实际渲染逻辑后续实现。
//!
//! 详见 docs/06-ui/04-data-flow/map-rendering.md §3 (Overlay)

use bevy::prelude::*;

/// Z 层偏移常量：覆盖层在 Tile 渲染层之上。

// ─── 覆盖层组件 ──────────────────────────────────────────────────

/// 标记覆盖层根 Entity 的组件。
#[derive(Component, Debug, Clone)]
pub struct OverlayRoot;

/// 光标悬停 Tile 高亮。
#[derive(Component, Debug, Clone)]
pub struct CursorHighlight;

/// 移动范围指示器。
#[derive(Component, Debug, Clone)]
pub struct MovementRangeOverlay;

/// 攻击范围/交互范围指示器。
#[derive(Component, Debug, Clone)]
pub struct InteractionOverlay;

// ─── 覆盖层单元格组件 ────────────────────────────────────────────

/// 单个覆盖层单元格——标记该格正处于某种覆盖状态。
#[derive(Component, Debug, Clone)]
pub struct OverlayCell {
    /// 覆盖层类型标识
    pub overlay_type: OverlayType,
    /// 网格坐标 X
    pub grid_x: u32,
    /// 网格坐标 Y
    pub grid_y: u32,
}

/// 覆盖层类型枚举。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OverlayType {
    /// 光标悬停高亮
    Cursor,
    /// 移动范围
    MovementRange,
    /// 攻击范围
    AttackRange,
    /// 技能范围
    AbilityRange,
    /// 交互区域
    Interaction,
}

// ─── 覆盖层生成辅助函数 ──────────────────────────────────────────

/// 生成光标高亮覆盖层。
#[allow(dead_code)]
pub fn spawn_cursor_highlight(commands: &mut Commands, parent: Entity, x: u32, y: u32) {
    let mut entity = commands.spawn((
        Name::new("CursorHighlight"),
        CursorHighlight,
        OverlayCell {
            overlay_type: OverlayType::Cursor,
            grid_x: x,
            grid_y: y,
        },
        Transform::from_xyz(x as f32 * 64.0, y as f32 * 64.0, 1.0),
    ));
    entity.set_parent_in_place(parent);
}

/// 生成移动范围覆盖层单元格。
#[allow(dead_code)]
pub fn spawn_movement_cell(commands: &mut Commands, parent: Entity, x: u32, y: u32) {
    let mut entity = commands.spawn((
        Name::new("MovementRangeCell"),
        MovementRangeOverlay,
        OverlayCell {
            overlay_type: OverlayType::MovementRange,
            grid_x: x,
            grid_y: y,
        },
        Transform::from_xyz(x as f32 * 64.0, y as f32 * 64.0, 1.0),
    ));
    entity.set_parent_in_place(parent);
}
