//! SceneRegister — 场景注册辅助 Trait
//!
//! 消除 `App::add_systems(OnEnter/OnExit, ...)` 在每个 GameState 上的重复调用。
//! 自动为每个场景组合 setup_scene_root（OnEnter）和 cleanup_scene（OnExit）。
//!
//! 用法：
//! ```ignore
//! use super::register::SceneRegister;
//!
//! app.register_scene(GameState::Combat, empty, empty);
//! app.register_scene(GameState::PartySetup, spawn_party_setup, despawn_party_setup);
//! ```

use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::system::ScheduleSystem;
use bevy::prelude::*;

use super::components::SceneRoot;
use super::queue::cleanup_scene;
use super::state::GameState;

/// 场景注册辅助 trait，为 App 添加 register_scene 方法。
pub(crate) trait SceneRegister {
    /// 注册一个场景的 OnEnter 和 OnExit 系统。
    ///
    /// 自动注册：
    /// - `OnEnter(state)`: `setup_scene_root` + `on_enter`
    /// - `OnExit(state)`: `cleanup_scene` + `on_exit`
    ///
    /// 无需额外系统时传入 [`empty`]：
    /// ```ignore
    /// app.register_scene(GameState::Combat, empty, empty);
    /// ```
    fn register_scene<M1, M2>(
        &mut self,
        state: GameState,
        on_enter: impl IntoScheduleConfigs<ScheduleSystem, M1>,
        on_exit: impl IntoScheduleConfigs<ScheduleSystem, M2>,
    );
}

impl SceneRegister for App {
    fn register_scene<M1, M2>(
        &mut self,
        state: GameState,
        on_enter: impl IntoScheduleConfigs<ScheduleSystem, M1>,
        on_exit: impl IntoScheduleConfigs<ScheduleSystem, M2>,
    ) {
        self.add_systems(OnEnter(state.clone()), (setup_scene_root, on_enter));
        self.add_systems(OnExit(state), (cleanup_scene, on_exit));
    }
}

/// 空系统——用于 register_scene 的无额外系统参数。
///
/// 替代 `()`，因为 Bevy 0.19 中 `()` 不实现 `IntoScheduleConfigs`。
pub(crate) fn empty() {}

/// 为当前场景创建一个带 SceneRoot 标记的根实体。
/// OnExit 时 cleanup_scene 通过 SceneRoot 标记 despawn 整个场景子树。
fn setup_scene_root(mut commands: Commands) {
    commands.spawn(SceneRoot);
}
