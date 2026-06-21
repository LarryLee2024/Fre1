//! FactionPlugin — 阵营关系领域 Plugin
//!
//! 注册阵营组件、事件和系统。
//! 处理阵营归属、声望管理、阵营间关系变化。
//!
//! 详见 ADR-022

use bevy::prelude::*;

use super::components::{FactionMembership, FactionRelationTable, KeyCharacter, Reputation};
use super::systems::relationship_system::on_relationship_eval_request;
use super::systems::reputation_system::on_reputation_change_request;
use crate::register_domain_types;

/// 阵营领域 Plugin——注册阵营关系、声望组件和关系计算系统。
pub struct FactionPlugin;

impl Plugin for FactionPlugin {
    fn build(&self, app: &mut App) {
        // ── 注册 Component 类型 ──
        register_domain_types!(app, [FactionMembership, Reputation, KeyCharacter,]);

        // ── 初始化 Resource ──
        app.init_resource::<FactionRelationTable>();

        // ── 注册 Observer System ──
        app.add_observer(on_reputation_change_request);
        app.add_observer(on_relationship_eval_request);
    }
}
