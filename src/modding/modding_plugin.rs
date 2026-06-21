//! ModdingPlugin — Mod 扩展层
//!
//! Mod 加载沙箱 / Mod API 稳定层 / 版本兼容检查。
//!
//! 详见 `docs/01-architecture/README.md` §3.5

use bevy::prelude::*;

/// Mod 扩展层 Plugin——注册 Mod 加载器、沙箱、稳定 API 层。
pub struct ModdingPlugin;

impl Plugin for ModdingPlugin {
    fn build(&self, _app: &mut App) {
        // TODO[P2][MODDING][2026-06-20]: register mod loader, sandbox, API layer
        // 完成条件: mod 加载管线 + 沙箱隔离 + 稳定 API 层就绪
    }
}
