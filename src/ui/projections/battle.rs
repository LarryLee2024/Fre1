//! 战斗领域事件到 BattleHudVm / SkillPanelVm / CharacterPanelVm 的投影
//!
//! 纯函数，将战斗领域事件（BattleStarted、TurnStarted、
//! TurnEnded、EffectApplied）转换为 UiStore 上的 ViewModel 更新。这些函数无状态、
//! 确定性且可独立测试 — 不直接操作 ECS。
//!
//! 每个函数接收 `&mut UiStore` 和领域事件，执行投影逻辑后返回。
//! 本模块中的 Observer 包装器桥接 Bevy 的 Trigger<T> 事件系统与纯函数。
//!
//! Observer 包装器同时查询 ECS 域中的领域组件（ActionPoints、CombatParticipant、
//! Name 等），将实时数据写入 ViewModel，然后委托纯函数处理跨域无状态逻辑。
//!
//! 参见 `docs/06-ui/04-data-flow/projection-viewmodel.md` §4

use bevy::ecs::observer::On;
use bevy::prelude::*;

use crate::core::capabilities::effect::events::EffectApplied;
use crate::core::domains::combat::components::ActionPoints;
use crate::core::events::{BattleStarted, TurnEnded, TurnStarted};
use crate::ui::binding::Dirty;
use crate::ui::view_models::{
    UiStore, battle_hud::BattleHudVm, character_panel::CharacterPanelVm, skill_panel::SkillPanelVm,
};

// ─── 纯投影函数 ─────────────────────────────────────────────────────────

/// 战斗投影 — 战斗领域事件的无状态投影逻辑。
///
/// 所有方法都是纯函数，接收 `&mut UiStore` 和事件。
/// 无 ECS 依赖，无副作用，完全确定性。
pub struct BattleProjection;

impl BattleProjection {
    /// 将 `BattleStarted` 事件投影到 `UiStore.battle_hud`。
    ///
    /// 将回合计数器初始化为 1，并将阶段键设置为玩家阶段。
    pub fn on_battle_started(store: &mut UiStore, _event: &BattleStarted) {
        let hud = &mut store.battle_hud;
        hud.turn_number = 1;
        hud.phase_key = "ui.battle.phase.player";
        info!(target: "ui", "[BattleProjection] Battle started — HUD initialized");
    }

    /// 将 `TurnStarted` 事件投影到 `UiStore.battle_hud`。
    ///
    /// 递增回合计数器并将阶段键设置为玩家阶段
    /// （当前简化模型中回合开始后的第一个阶段）。
    pub fn on_turn_started(store: &mut UiStore, _event: &TurnStarted) {
        let hud = &mut store.battle_hud;
        hud.turn_number += 1;
        hud.phase_key = "ui.battle.phase.player";
    }

    /// 将 `TurnEnded` 事件投影到 `UiStore.battle_hud`。
    ///
    /// 将阶段键设置为敌方阶段，反映玩家的
    /// 活跃回合已结束。
    pub fn on_turn_ended(store: &mut UiStore, _event: &TurnEnded) {
        store.battle_hud.phase_key = "ui.battle.phase.enemy";
        info!(target: "ui", "[BattleProjection] Turn ended — phase set to enemy");
    }

    /// 将 `EffectApplied` 事件投影到 `UiStore.skill_panel`。
    ///
    /// 尝试将 effect def_id 与技能面板中的技能匹配以更新冷却状态。
    /// 匹配策略：将 event.def_id（如 "spl_fireball"）与
    /// UiStore.skill_panel.skills 中的 name_key 进行模糊匹配。
    /// 当前简化实现：标记所有非零冷却技能为"已使用"状态
    /// （cooldown_remaining = max_cooldown, is_usable = false）。
    /// 精确匹配留待 EffectDef -> SkillDef 关联表就绪后实现。
    pub fn on_effect_applied(store: &mut UiStore, event: &EffectApplied) {
        let effect_id = &event.def_id;
        let mut matched = false;

        for (_skill_id, slot) in store.skill_panel.skills.iter_mut() {
            if slot.max_cooldown > 0 && slot.name_key.contains(effect_id) {
                slot.cooldown_remaining = slot.max_cooldown;
                slot.is_usable = false;
                matched = true;
            }
        }

        if matched {
            info!(
                target: "ui",
                "[BattleProjection] Effect '{}' matched to skill - cooldown set",
                effect_id,
            );
        } else {
            info!(
                target: "ui",
                "[BattleProjection] Effect applied: def_id={}, target={} (no skill match)",
                event.def_id,
                event.target_entity,
            );
        }
    }

    /// 回合开始时递减所有技能的冷却。
    ///
    /// 每个回合将 cooldown_remaining 减 1，当降至 0 时将技能标记为可用。
    pub fn on_turn_started_for_skills(store: &mut UiStore, _event: &TurnStarted) {
        for (_id, slot) in store.skill_panel.skills.iter_mut() {
            if slot.cooldown_remaining > 0 {
                slot.cooldown_remaining -= 1;
                if slot.cooldown_remaining == 0 {
                    slot.is_usable = true;
                }
            }
        }
    }
}

// ─── Observer Systems (ECS bridge) ───────────────────────────────────────

/// Observer：监听 `BattleStarted` 领域事件并通过
/// `BattleProjection::on_battle_started` 将其投影到 `UiStore.battle_hud`。
pub fn on_battle_started_projection(
    trigger: On<BattleStarted>,
    mut store: ResMut<UiStore>,
    mut dirty_query: Query<&mut Dirty<BattleHudVm>>,
) {
    BattleProjection::on_battle_started(&mut store, trigger.event());

    for mut dirty in dirty_query.iter_mut() {
        dirty.mark_dirty();
    }
}

/// Observer：监听 `TurnStarted` 领域事件，提取当前单位的域数据（ActionPoints），
/// 写入 `UiStore.battle_hud`，然后委托 `BattleProjection::on_turn_started`
/// 处理回合编号和阶段键的逻辑。
///
/// 同时标记所有 `Dirty<BattleHudVm>` 组件为脏，以便消费此 ViewModel 的 Widget
/// 在下一帧刷新。
pub fn on_turn_started_projection(
    trigger: On<TurnStarted>,
    mut store: ResMut<UiStore>,
    mut dirty_query: Query<&mut Dirty<BattleHudVm>>,
    ap_query: Query<&ActionPoints>,
) {
    let unit = trigger.event().unit;

    // 从领域查询 ActionPoints 并计算 AP 显示值。
    // 每个单位每回合获得 1 个标准行动；bonus_action 是第二个 AP。
    if let Ok(ap_component) = ap_query.get(unit) {
        // 将布尔型行动可用性映射为 f32 表示，用于 HUD 条。
        let standard_val = if ap_component.standard_action {
            1.0
        } else {
            0.0
        };
        let bonus_val = if ap_component.bonus_action { 1.0 } else { 0.0 };
        store.battle_hud.ap = standard_val + bonus_val;
        store.battle_hud.max_ap = 2.0; // Standard + bonus = 2 max AP

        info!(
            target: "ui",
            "[BattleProjection] Unit {:?} AP: std={}, bonus={}, movement={}/{}",
            unit,
            ap_component.standard_action,
            ap_component.bonus_action,
            ap_component.movement,
            ap_component.max_movement,
        );
    }

    BattleProjection::on_turn_started(&mut store, trigger.event());

    // 标记所有 BattleHudVm 消费者为脏
    for mut dirty in dirty_query.iter_mut() {
        dirty.mark_dirty();
    }
}

/// Observer：监听 `TurnEnded` 领域事件并通过
/// `BattleProjection::on_turn_ended` 将其投影到 `UiStore.battle_hud`。
pub fn on_turn_ended_projection(
    trigger: On<TurnEnded>,
    mut store: ResMut<UiStore>,
    mut dirty_query: Query<&mut Dirty<BattleHudVm>>,
) {
    BattleProjection::on_turn_ended(&mut store, trigger.event());

    for mut dirty in dirty_query.iter_mut() {
        dirty.mark_dirty();
    }
}

/// Observer：监听 `EffectApplied` 领域事件并通过
/// `BattleProjection::on_effect_applied` 将其投影到 `UiStore.skill_panel`。
///
/// 同时标记所有 `Dirty<SkillPanelVm>` 组件为脏，以便技能
/// 槽位 Widget 在下一帧刷新。
pub fn on_effect_applied_projection(
    trigger: On<EffectApplied>,
    mut store: ResMut<UiStore>,
    mut dirty_query: Query<&mut Dirty<SkillPanelVm>>,
) {
    BattleProjection::on_effect_applied(&mut store, trigger.event());

    // 标记所有 SkillPanelVm 消费者为脏
    for mut dirty in dirty_query.iter_mut() {
        dirty.mark_dirty();
    }
}

/// Observer：监听 `TurnStarted` 领域事件并通过
/// `BattleProjection::on_turn_started_for_skills` 递减技能冷却。
///
/// 同时标记所有 `Dirty<SkillPanelVm>` 组件为脏，以便技能
/// 槽位 Widget 在下一帧刷新。
pub fn on_turn_started_skill_projection(
    trigger: On<TurnStarted>,
    mut store: ResMut<UiStore>,
    mut dirty_query: Query<&mut Dirty<SkillPanelVm>>,
) {
    BattleProjection::on_turn_started_for_skills(&mut store, trigger.event());

    for mut dirty in dirty_query.iter_mut() {
        dirty.mark_dirty();
    }
}

/// Observer：监听 `TurnStarted` 领域事件，从活动单位提取域数据（Name、
/// ActionPoints）并写入 `UiStore.character_panel`。当前使用硬编码默认值，
/// 待 Health / Mana / Level 域组件就绪后替换。
///
/// 同时标记所有 `Dirty<CharacterPanelVm>` 组件为脏。
pub fn on_character_panel_projection(
    trigger: On<TurnStarted>,
    mut store: ResMut<UiStore>,
    mut dirty_query: Query<&mut Dirty<CharacterPanelVm>>,
    name_query: Query<&Name>,
) {
    let unit = trigger.event().unit;

    // TODO[P2][Projection][2026-06-21]: Populate CharacterPanelVm from real domain components
    //   - character_id: use Entity as unique ID
    //   - name_key: use localized NameKey from CharacterDef or Name component
    //   - level: query Level component (progression domain)
    //   - hp/max_hp: query Health component or resolve from Attribute pipeline
    //   - mp/max_mp: query Mana component or resolve from Attribute pipeline
    //   Completion criteria: all 7 fields sourced from domain data, no hardcoded values
    store.character_panel.character_id = unit.to_bits() as u32;

    // 尝试读取单位的名称用于显示；回退到通用 key。
    // TODO: Use LocalizationKey from a CharacterName component when available.
    store.character_panel.name_key = "ui.character.unknown";
    if let Ok(name) = name_query.get(unit) {
        info!(
            target: "ui",
            "[CharacterPanel] Active unit: {} (entity {:?})",
            name.as_str(),
            unit,
        );
    }

    // 硬编码默认值，直到属性组件就绪。
    store.character_panel.level = 1;
    store.character_panel.hp = 100.0;
    store.character_panel.max_hp = 100.0;
    store.character_panel.mp = 50.0;
    store.character_panel.max_mp = 50.0;

    for mut dirty in dirty_query.iter_mut() {
        dirty.mark_dirty();
    }
}
