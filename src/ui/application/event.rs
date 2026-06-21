//! UiEvent — UI 层内部事件广播
//!
//! UiEvent 用于 UI 子系统间通信（ViewModel 更新、导航变更、主题切换等）。
//! UiEvent 绝不跨出 UI 层边界 (APP-VAL-04)。

use bevy::prelude::*;

use crate::ui::navigation::ScreenType;

/// UI 层内部事件广播枚举。
///
/// # 边界规则 (APP-VAL-04)
/// UiEvent 的事件订阅者仅限于 UI 层内部模块，不跨出 UI 层。
/// Domain Event 通过 Observer 通知 UI，不经过 UiEvent。
///
/// # 使用 Bevy Event 机制
/// UiEvent 使用 `EventWriter`/`EventReader` 在 UI 层内部广播。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect, Event)]
pub enum UiEvent {
    // ── ViewModel 事件 ──
    /// ViewModel 已更新（设置 Dirty 标记后发射）
    ViewModelUpdated(&'static str),

    // ── 导航事件 ──
    /// Screen 推入完成
    ScreenPushed(ScreenType),
    /// Screen 弹出完成
    ScreenPopped(ScreenType),
    /// Screen 被替换（旧, 新）
    ScreenReplaced(ScreenType, ScreenType),
    /// 导航错误
    NavigationError(String),

    // ── 主题事件 ──
    /// 主题切换
    ThemeChanged(String),
}
