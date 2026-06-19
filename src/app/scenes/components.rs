//! 场景标记组件 — SceneRoot
//!
//! 每个场景 OnEnter 时 spawn 的根实体标记，
//! OnExit 时 `cleanup_scene` 通过此组件 despawn 整个场景。

use bevy::prelude::*;

/// 场景根实体标记。
///
/// 每个 GameState 的 OnEnter 系统应生成一个带 `SceneRoot` 的根实体，
/// 该场景下的所有 UI/Entity 作为其子级。
/// OnExit 时 `cleanup_scene` 通过此组件递归 despawn 整个场景子树。
#[derive(Component)]
pub struct SceneRoot;
