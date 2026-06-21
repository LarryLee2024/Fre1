//! 当前屏幕的生命周期状态
//!
//! 跟踪当前活动的屏幕及其生命周期阶段。
//! 与 ScreenStack 配合提供完整的导航状态。

use bevy::prelude::*;

use super::screen_type::ScreenType;

/// 表示当前屏幕的生命周期阶段。
///
/// 屏幕经历以下状态：Defined → Loading → Active，
/// 然后在导航离开时：Active → Background，或在移除时：
/// Active → Destroyed。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum ScreenLifecycle {
    /// 屏幕已注册但尚未加载。
    Defined,
    /// 屏幕资源正在加载（资源、ViewModel、Observer）。
    Loading,
    /// 屏幕完全激活且可交互。
    Active,
    /// 屏幕在后台（另一个屏幕在顶部）。
    Background,
    /// 屏幕已销毁，其实体应被 despawn。
    Destroyed,
}

/// 跟踪当前屏幕的身份和生命周期阶段。
///
/// 作为 Bevy Resource 存储。与跟踪导航历史的 ScreenStack 不同，
/// UiScreenState 仅捕获即时屏幕状态，作为需要当前上下文的系统的快速查找。
#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct UiScreenState {
    /// 当前活动（最顶层）的屏幕类型。
    pub current_screen: Option<ScreenType>,
    /// 当前屏幕的生命周期阶段。
    pub lifecycle: ScreenLifecycle,
    /// 当前屏幕被压入之前的上一个屏幕。
    pub previous_screen: Option<ScreenType>,
}

impl UiScreenState {
    /// 创建没有当前屏幕的新的 UiScreenState。
    pub fn new() -> Self {
        Self {
            current_screen: None,
            lifecycle: ScreenLifecycle::Defined,
            previous_screen: None,
        }
    }
}

impl Default for UiScreenState {
    fn default() -> Self {
        Self::new()
    }
}
