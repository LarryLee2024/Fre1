//! Fre SRPG — 程序入口
//!
//! 根据 feature flag 启动 game / editor / headless 模式。
//! 所有 Plugin 注册委托给 `app::AppPlugin`（唯一 Composition Root）。

use bevy::prelude::*;
use fre::app::AppPlugin;

fn main() -> AppExit {
    App::new().add_plugins(AppPlugin).run()
}
