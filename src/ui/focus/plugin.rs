//! FocusPlugin — 焦点导航系统 Plugin
//!
//! 注册 FocusManager Resource、Focusable/FocusGroup/TabIndex Component，
//! 以及键盘导航和焦点视觉效果系统。
//!
//! 在 ThemePlugin 之后、PrimitivesPlugin 之前注册。

use bevy::prelude::*;

use super::components::{FocusGroup, FocusStyle, Focusable, TabIndex};
use super::manager::FocusManager;
use super::navigation::{focus_visual_system, keyboard_navigation_system};

/// FocusPlugin — 注册焦点导航系统
pub struct FocusPlugin;

impl Plugin for FocusPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FocusManager>()
            .register_type::<FocusManager>()
            .register_type::<Focusable>()
            .register_type::<FocusGroup>()
            .register_type::<FocusStyle>()
            .register_type::<TabIndex>()
            .add_systems(Update, (keyboard_navigation_system, focus_visual_system));
    }
}
