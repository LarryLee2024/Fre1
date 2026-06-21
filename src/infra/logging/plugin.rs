//! LoggingPlugin — 日志基础设施 Plugin
//!
//! 注册所有日志 Observer，监听领域事件并生成结构化日志。
//! 领域层不写日志，由本插件通过 Observer 监听 Domain Event 生成。
//!
//! 详见 ADR-052

use bevy::prelude::*;

use super::metrics::{self, MetricsCollector};

use super::observers::{
    ability_logger, battle_logger, camp_rest_logger, content_logger, crafting_logger,
    economy_logger, effect_logger, faction_logger, inventory_logger, narrative_logger,
    party_logger, progression_logger, quest_logger, reaction_logger, spell_logger, summon_logger,
    tactical_logger, terrain_logger, turn_logger,
};

use super::sinks::file_sink::FileSinkConfig;

/// 日志基础设施 Plugin。
///
/// 注册所有日志 Observer + MetricsCollector。
pub struct LoggingPlugin;

impl Plugin for LoggingPlugin {
    fn build(&self, app: &mut App) {
        // ════════════════════════════════════════════
        // MetricsCollector 收集 LogCode 频率统计，供 balance_analyzer 消费
        // ════════════════════════════════════════════
        app.init_resource::<MetricsCollector>();
        app.add_systems(Update, metrics::metrics_flush_system);

        // ════════════════════════════════════════════
        // 日志 Observer 按 Domain 注册，每个 Domain 一个 logger 模块
        // ════════════════════════════════════════════
        // 日志 Observer 数量需与启动日志中的计数保持同步
        // 注意：增删 Observer 后请同步更新下方启动日志中的计数

        // ── BAT: Combat（战斗）──
        app.add_observer(battle_logger::on_battle_started)
            .add_observer(battle_logger::on_battle_ended);

        // ── BAT: Combat（回合）──
        app.add_observer(turn_logger::on_turn_started)
            .add_observer(turn_logger::on_turn_ended);

        // ── TAC: Tactical（战术）──
        app.add_observer(tactical_logger::on_unit_moved)
            .add_observer(tactical_logger::on_position_changed);

        // ── TER: Terrain（地形）──
        app.add_observer(terrain_logger::on_tile_entered)
            .add_observer(terrain_logger::on_surface_changed)
            .add_observer(terrain_logger::on_hazard_triggered)
            .add_observer(terrain_logger::on_terrain_effect_applied);

        // ── SPR: Spell（法术）──
        app.add_observer(spell_logger::on_spell_cast_result);

        // ── RCT: Reaction（反应）──
        app.add_observer(reaction_logger::on_reaction_triggered)
            .add_observer(reaction_logger::on_reaction_executed)
            .add_observer(reaction_logger::on_reaction_declined)
            .add_observer(reaction_logger::on_opportunity_attack)
            .add_observer(reaction_logger::on_counterspell)
            .add_observer(reaction_logger::on_shield_used)
            .add_observer(reaction_logger::on_guardian_used);

        // ── ABL: Ability（技能）──
        app.add_observer(ability_logger::on_ability_activated)
            .add_observer(ability_logger::on_ability_completed)
            .add_observer(ability_logger::on_ability_cancelled)
            .add_observer(ability_logger::on_ability_cooldown_started);

        // ── EFF: Effect（效果）──
        app.add_observer(effect_logger::on_effect_applied)
            .add_observer(effect_logger::on_effect_removed)
            .add_observer(effect_logger::on_effect_ticked)
            .add_observer(effect_logger::on_effect_immunity);

        // ── QST: Quest（任务）──
        app.add_observer(quest_logger::on_quest_accepted)
            .add_observer(quest_logger::on_objective_completed)
            .add_observer(quest_logger::on_quest_turned_in)
            .add_observer(quest_logger::on_quest_failed)
            .add_observer(quest_logger::on_quest_progress_updated);

        // ── PRG: Progression（成长）──
        app.add_observer(progression_logger::on_experience_gained)
            .add_observer(progression_logger::on_level_up)
            .add_observer(progression_logger::on_talent_unlocked)
            .add_observer(progression_logger::on_subclass_chosen)
            .add_observer(progression_logger::on_asi_completed)
            .add_observer(progression_logger::on_class_gained);

        // ── INV: Inventory（背包）──
        app.add_observer(inventory_logger::on_item_acquired)
            .add_observer(inventory_logger::on_item_used)
            .add_observer(inventory_logger::on_equipment_changed)
            .add_observer(inventory_logger::on_item_removed)
            .add_observer(inventory_logger::on_loot_generated);

        // ── ECO: Economy（经济）──
        app.add_observer(economy_logger::on_transaction_completed)
            .add_observer(economy_logger::on_price_changed)
            .add_observer(economy_logger::on_currency_changed);

        // ── CRF: Crafting（制作）──
        app.add_observer(crafting_logger::on_item_crafted)
            .add_observer(crafting_logger::on_enchantment_applied)
            .add_observer(crafting_logger::on_item_upgraded)
            .add_observer(crafting_logger::on_crafting_failed);

        // ── FAC: Faction（阵营）──
        app.add_observer(faction_logger::on_reputation_changed)
            .add_observer(faction_logger::on_faction_relation_changed)
            .add_observer(faction_logger::on_reputation_level_up)
            .add_observer(faction_logger::on_relationship_evaluated);

        // ── PRY: Party（队伍）──
        app.add_observer(party_logger::on_member_joined)
            .add_observer(party_logger::on_member_removed)
            .add_observer(party_logger::on_member_swapped)
            .add_observer(party_logger::on_bond_activated)
            .add_observer(party_logger::on_bond_deactivated);

        // ── CNR: CampRest（营地）──
        app.add_observer(camp_rest_logger::on_short_rest_completed)
            .add_observer(camp_rest_logger::on_long_rest_started)
            .add_observer(camp_rest_logger::on_long_rest_completed)
            .add_observer(camp_rest_logger::on_long_rest_interrupted)
            .add_observer(camp_rest_logger::on_camp_event_triggered);

        // ── NAR: Narrative（叙事）──
        app.add_observer(narrative_logger::on_dialogue_started)
            .add_observer(narrative_logger::on_choice_made)
            .add_observer(narrative_logger::on_story_flag_set)
            .add_observer(narrative_logger::on_cutscene_started)
            .add_observer(narrative_logger::on_cutscene_ended);

        // ── SUM: Summon（召唤）──
        app.add_observer(summon_logger::on_summon_created)
            .add_observer(summon_logger::on_summon_expired)
            .add_observer(summon_logger::on_summon_command)
            .add_observer(summon_logger::on_summon_slot_changed);

        // ── CNT: Content（内容基础设施）──
        app.add_observer(content_logger::on_definition_reloaded);

        tracing::info!(target: "logging", "[LoggingPlugin] 已初始化（Metrics + 73 个 observer）");

        // ── 文件日志输出器（FileSink）──
        // 注意：Bevy 的 DefaultPlugins.LogPlugin 已初始化 tracing-subscriber。
        // FileSinkLayer 的完整集成需要自定义 Bevy LogPlugin。
        // 当前先将 FileSinkConfig 注册为 Resource，后续通过自定义 LogPlugin 追加 Layer。
        let file_sink_config = FileSinkConfig::default();
        if file_sink_config.enabled {
            app.insert_resource(file_sink_config);
        }
    }
}
