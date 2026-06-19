//! LoggingPlugin — 日志基础设施 Plugin
//!
//! 注册所有日志 Observer，监听领域事件并生成结构化日志。
//! 领域层不写日志，由本插件通过 Observer 监听 Domain Event 生成。
//!
//! 详见 ADR-052

use bevy::prelude::*;

use super::observers::{battle_logger, spell_logger, turn_logger};

/// 日志基础设施 Plugin。
///
/// 注册战斗、回合、法术等领域的日志 Observer。
pub struct LoggingPlugin;

impl Plugin for LoggingPlugin {
    fn build(&self, app: &mut App) {
        // ── 注册战斗日志 Observer ──
        app.add_observer(battle_logger::on_battle_started)
            .add_observer(battle_logger::on_battle_ended);

        // ── 注册回合日志 Observer ──
        app.add_observer(turn_logger::on_turn_started)
            .add_observer(turn_logger::on_turn_ended);

        // ── 注册法术日志 Observer ──
        app.add_observer(spell_logger::on_spell_cast_result);

        tracing::info!("[LoggingPlugin] initialized (observers registered)");
    }
}
