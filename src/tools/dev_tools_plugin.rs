//! DevToolsPlugin — 开发工具集
//!
//! Debug 面板、性能分析、热重载控制台。
//! 仅在 `feature = "dev"` 时编译。
//!
//! 详见 `docs/01-architecture/README.md` §3.5

use bevy::prelude::*;

pub struct DevToolsPlugin;

impl Plugin for DevToolsPlugin {
    fn build(&self, _app: &mut App) {
        // TODO: register debug panels, inspector, dev console
    }
}
