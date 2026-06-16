//! ModdingPlugin — Mod 扩展层
//!
//! Mod 加载沙箱 / Mod API 稳定层 / 版本兼容检查。
//!
//! 详见 `docs/01-architecture/README.md` §3.5

use bevy::prelude::*;

pub struct ModdingPlugin;

impl Plugin for ModdingPlugin {
    fn build(&self, _app: &mut App) {
        // TODO: register mod loader, sandbox, API layer
    }
}
