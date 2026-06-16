//! SharedPlugin — L0 原子层 Plugin
//!
//! 注册 shared 层全局 Resource、通用 System。
//! 当前为骨架，随实现推进逐步填充。

use bevy::prelude::*;

pub struct SharedPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, _app: &mut App) {
        // TODO: 注册 ID 分配器、全局 Resource、通用 System
    }
}
