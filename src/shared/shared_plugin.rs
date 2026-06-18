//! SharedPlugin — L0 原子层 Plugin
//!
//! 注册 shared 层全局 Resource、通用 System。

use bevy::prelude::*;

use super::random::GameRng;
use super::time::GameTime;

pub struct SharedPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameRng::default())
            .insert_resource(GameTime::default())
            .add_systems(PreUpdate, advance_game_time);
    }
}

/// 每帧推进游戏时间（PreUpdate 阶段）。
fn advance_game_time(mut time: ResMut<GameTime>) {
    time.advance_frame();
}
