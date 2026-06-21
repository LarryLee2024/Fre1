//! DevToolsPlugin — 开发工具集
//!
//! Debug 面板、性能分析、热重载控制台。
//! 仅在 `feature = "dev"` 时编译。
//!
//! 详见 `docs/01-architecture/README.md` §3.5

use bevy::dev_tools::diagnostics_overlay::{DiagnosticsOverlay, DiagnosticsOverlayPlugin};
use bevy::prelude::*;

/// 开发工具集 Plugin——注册 Debug 面板、性能分析、热重载控制台。
pub struct DevToolsPlugin;

impl Plugin for DevToolsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DiagnosticsOverlayPlugin::default());

        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(DiagnosticsOverlay::fps());
        });
    }
}
