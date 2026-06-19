//! LoggingPlugin — 日志基础设施 Plugin
//!
//! 注册所有日志 Observer，监听领域事件并生成结构化日志。
//! 领域层不写日志，由本插件通过 Observer 监听 Domain Event 生成。
//!
//! 详见 ADR-052

use bevy::prelude::*;

use super::observers::{
    ability_logger, battle_logger, content_logger, effect_logger, quest_logger, spell_logger,
    turn_logger,
};

/// 日志基础设施 Plugin。
///
/// 注册战斗、回合、法术、技能、效果、任务、内容等各领域的日志 Observer。
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

        // ── 注册技能日志 Observer ──
        app.add_observer(ability_logger::on_ability_activated)
            .add_observer(ability_logger::on_ability_completed)
            .add_observer(ability_logger::on_ability_cancelled)
            .add_observer(ability_logger::on_ability_cooldown_started);

        // ── 注册效果日志 Observer ──
        app.add_observer(effect_logger::on_effect_applied)
            .add_observer(effect_logger::on_effect_removed)
            .add_observer(effect_logger::on_effect_ticked)
            .add_observer(effect_logger::on_effect_immunity);

        // ── 注册任务日志 Observer ──
        app.add_observer(quest_logger::on_quest_accepted)
            .add_observer(quest_logger::on_objective_completed)
            .add_observer(quest_logger::on_quest_turned_in)
            .add_observer(quest_logger::on_quest_failed)
            .add_observer(quest_logger::on_quest_progress_updated);

        // ── 注册内容日志 Observer ──
        app.add_observer(content_logger::on_definition_reloaded);

        tracing::info!("[LoggingPlugin] initialized ({} observers registered)", 18);
    }
}
