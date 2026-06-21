//! SharedPlugin — L0 原子层 Plugin
//!
//! 注册 shared 层全局 Resource、通用 System。

use bevy::prelude::*;

use super::random::DeterministicRng;
use super::time::GameTime;

/// Shared 层 L0 原子层 Plugin。
///
/// 注册 shared 层全局 Resource（`GameTime`、`DeterministicRng`）和通用 System。
pub struct SharedPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameTime::default())
            .init_resource::<DeterministicRng>()
            .add_systems(PreUpdate, advance_game_time);
    }
}

/// 每帧推进游戏时间（PreUpdate 阶段）。
fn advance_game_time(mut time: ResMut<GameTime>) {
    time.advance_frame();
}
