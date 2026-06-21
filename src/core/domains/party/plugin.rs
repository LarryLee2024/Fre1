//! PartyPlugin — 队伍领域 Plugin
//!
//! 注册队伍组件、事件和系统。
//! 处理成员管理、阵型配置、羁绊系统。
//!
//! 详见 ADR-031

use bevy::prelude::*;

use super::components::{BondState, Party, PartyMarker};
use super::systems::{on_member_joined, on_member_removed, on_member_swapped};
use crate::register_domain_types;

/// 队伍领域 Plugin——注册队伍成员、羁绊组件和队伍管理系统。
pub struct PartyPlugin;

impl Plugin for PartyPlugin {
    fn build(&self, app: &mut App) {
        // ── 注册 Component 类型 ──
        register_domain_types!(app, [Party, BondState, PartyMarker,]);

        // ── 初始化 Resource ──
        app.init_resource::<Party>();
        app.init_resource::<BondState>();

        // ── 注册 Observer System ──
        app.add_observer(on_member_joined);
        app.add_observer(on_member_removed);
        app.add_observer(on_member_swapped);
    }
}
