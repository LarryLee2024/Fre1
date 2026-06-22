//! Passthrough Backend — UI 穿透策略配置
//!
//! 定义 UI Picking 穿透策略，确保 Sprite 点击事件不被 UI 节点拦截。
//! 具体实现（mark_battle_ui_passthrough）保留在 src/ui/screens/battle/visibility.rs。
//!
//! 本模块为未来集中化配置预留扩展点。
//!
//! 详见 ADR-068 §Module Design。

use bevy::prelude::*;

/// UI Pickable 穿透模式
///
/// 控制如何将 Pickable::IGNORE 应用到 UI 节点。
#[derive(Debug, Clone, Copy, PartialEq, Resource)]
pub enum PassthroughMode {
    /// 所有 BattleScreen 后代节点均设置为 IGNORE
    AllBattleUi,
    /// 仅特定 Zone 内的节点穿透
    ZoneWhitelist,
    /// 不启用穿透（默认）
    Disabled,
}

impl Default for PassthroughMode {
    fn default() -> Self {
        Self::Disabled
    }
}
