//! 焦点导航系统 — 键盘/手柄导航和焦点视觉效果
//!
//! 包含：
//! - keyboard_navigation_system: 方向键/WASD 导航
//! - focus_visual_system: 聚焦元素的视觉效果管理
//!
//! 使用我们自己的 TabIndex 组件进行组内排序，
//! 并与 Bevy 内置 Tab 导航共存（Tab/Shift+Tab 由 Bevy 处理）。
//!
//! 参见 `docs/06-ui/02-design-system/focus-binding.md` §2.4

use bevy::prelude::*;

use super::components::{FocusGroup, FocusNavigation, FocusStyle, Focusable, TabIndex};
use super::manager::FocusManager;

/// 方向枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

/// 键盘方向键导航系统
///
/// 将 Arrow Keys / WASD 映射为焦点移动操作。
/// 仅在当前活跃组内移动焦点，遵循 FocusNavigation 模式定义。
///
/// 注意：Tab / Shift+Tab 由 Bevy 内置的 TabIndex 系统处理，
/// 本系统仅处理方向键。
pub fn keyboard_navigation_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut focus_manager: ResMut<FocusManager>,
    focusables: Query<(Entity, &Focusable, &TabIndex)>,
    focus_groups: Query<&FocusGroup>,
) {
    let direction = if keyboard.just_pressed(KeyCode::ArrowUp)
        || keyboard.just_pressed(KeyCode::KeyW)
    {
        Some(Direction::Up)
    } else if keyboard.just_pressed(KeyCode::ArrowDown) || keyboard.just_pressed(KeyCode::KeyS) {
        Some(Direction::Down)
    } else if keyboard.just_pressed(KeyCode::ArrowLeft) || keyboard.just_pressed(KeyCode::KeyA) {
        Some(Direction::Left)
    } else if keyboard.just_pressed(KeyCode::ArrowRight) || keyboard.just_pressed(KeyCode::KeyD) {
        Some(Direction::Right)
    } else {
        None
    };

    let Some(dir) = direction else {
        return;
    };

    let Some(active_group) = focus_manager.active_group else {
        return;
    };

    // 导航模式决定方向键行为（Grid = 网格导航，Linear = 列表导航）
    let navigation_mode = focus_groups
        .iter()
        .find(|g| g.group_id == active_group)
        .map(|g| g.navigation.clone())
        .unwrap_or(FocusNavigation::Grid { cols: 1 });

    let wrap = focus_groups
        .iter()
        .find(|g| g.group_id == active_group)
        .map(|g| g.wrap)
        .unwrap_or(true);

    // TabIndex 排序决定 Tab 键导航顺序，低值优先
    let mut members: Vec<(Entity, &TabIndex)> = focusables
        .iter()
        .filter(|(_, f, _)| f.group_id == active_group)
        .map(|(e, _, ti)| (e, ti))
        .collect();
    members.sort_by_key(|(_, ti)| ti.0);

    if members.is_empty() {
        return;
    }

    let current_pos = focus_manager
        .focused_entity
        .and_then(|current| members.iter().position(|(e, _)| *e == current));

    let total = members.len();

    let new_pos = match navigation_mode {
        FocusNavigation::Grid { cols } => {
            navigate_grid(current_pos, dir, cols as usize, total, wrap)
        }
        FocusNavigation::Linear => navigate_linear(current_pos, dir, total, wrap),
        FocusNavigation::Custom => {
            // Custom 模式下不处理方向键导航
            return;
        }
    };

    focus_manager.focused_entity = Some(members[new_pos].0);
    focus_manager.active_group = Some(active_group);
    focus_manager.group_indices.insert(active_group, new_pos);
}

/// 网格模式导航计算
fn navigate_grid(
    current_pos: Option<usize>,
    dir: Direction,
    cols: usize,
    total: usize,
    wrap: bool,
) -> usize {
    let Some(pos) = current_pos else {
        return 0;
    };

    let row = pos / cols;
    let col = pos % cols;

    match dir {
        Direction::Up => {
            if row == 0 {
                if wrap {
                    ((total - 1) / cols) * cols + col.min((total - 1) % cols)
                } else {
                    pos
                }
            } else {
                let new_row = row - 1;
                let new_idx = new_row * cols + col;
                new_idx.min(total - 1)
            }
        }
        Direction::Down => {
            let max_row = (total - 1) / cols;
            if row == max_row {
                if wrap { col } else { pos }
            } else {
                let new_row = row + 1;
                let new_idx = new_row * cols + col;
                new_idx.min(total - 1)
            }
        }
        Direction::Left => {
            if col == 0 {
                if wrap {
                    row * cols + (cols - 1).min(total - 1 - row * cols)
                } else {
                    pos
                }
            } else {
                row * cols + (col - 1)
            }
        }
        Direction::Right => {
            let max_col_in_row = (total - 1 - row * cols).min(cols - 1);
            if col >= max_col_in_row {
                if wrap { row * cols } else { pos }
            } else {
                row * cols + col + 1
            }
        }
    }
}

/// 线性模式导航计算
fn navigate_linear(current_pos: Option<usize>, dir: Direction, total: usize, wrap: bool) -> usize {
    let Some(pos) = current_pos else {
        return 0;
    };

    match dir {
        Direction::Up | Direction::Left => {
            if pos > 0 {
                pos - 1
            } else if wrap {
                total - 1
            } else {
                pos
            }
        }
        Direction::Down | Direction::Right => {
            if pos < total - 1 {
                pos + 1
            } else if wrap {
                0
            } else {
                pos
            }
        }
    }
}

/// 焦点视觉效果系统
///
/// 为当前聚焦的元素添加 Outline 高亮（FocusStyle::Outline），
/// 或在聚焦失去后移除高亮。
/// FocusStyle::None 的元素不受本系统影响。
pub fn focus_visual_system(
    mut commands: Commands,
    focus_manager: Res<FocusManager>,
    query: Query<(Entity, &Focusable, Option<&mut Outline>), With<Node>>,
) {
    for (entity, focusable, outline) in query.iter() {
        if focusable.focus_style == FocusStyle::None {
            continue;
        }

        let is_focused = focus_manager.is_focused(entity);

        match (is_focused, outline) {
            (true, None) => {
                // 获得焦点但无 Outline 组件 — 添加
                commands.entity(entity).insert(Outline {
                    color: Color::srgb(1.0, 1.0, 1.0),
                    width: Val::Px(2.0),
                    offset: Val::Px(2.0),
                });
            }
            (false, Some(_outline)) => {
                // 失去焦点且有 Outline 组件 — 移除
                commands.entity(entity).remove::<Outline>();
            }
            _ => {
                // (true, Some(_)) — 已有 Outline，不需操作
                // (false, None) — 无焦点也无 Outline，不需操作
            }
        }
    }
}

/// 焦点变化事件 — 当焦点移动到新元素时触发
///
/// UI 层可监听此事件来驱动 Tooltip 显示、音效等。
#[derive(Event, Debug, Clone, Copy)]
pub struct FocusChanged {
    /// 新聚焦的实体
    pub entity: Entity,
    /// 焦点组 ID
    pub group_id: u32,
    /// 组内索引
    pub index: usize,
}
