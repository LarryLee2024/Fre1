//! ContentPlugin — 内容桥接层
//!
//! 从 assets/config/ 加载配置 → 校验 → 注册到 Registry。
//!
//! 详见 `docs/01-architecture/README.md` §3.5

use bevy::prelude::*;

pub struct ContentPlugin;

impl Plugin for ContentPlugin {
    fn build(&self, _app: &mut App) {
        // TODO: register asset loaders, config watchers, validation pipeline
    }
}
