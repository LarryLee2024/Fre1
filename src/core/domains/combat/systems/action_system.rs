//! Combat Action System — 处理战斗动作请求
//!
//! 监听 AttackRequested/SpellCastRequested 事件，执行完整的战斗动作：
//! 1. 通过 DamagePolicy::evaluate() 计算伤害
//! 2. 发射 DamageDealt 事件
//! 3. DamageDealt 处理器扣除目标 HitPoints
//! 4. 死亡判定 → 发射 UnitDied
//!
//! # 当前限制
//!
//! - 实体 ID 解析（String → Entity）尚未实现，当前仅对第一个含 HitPoints
//!   的实体施加固定伤害。
//! - SpellCastRequested 使用固定法术伤害值（25 火焰伤害）。
//!
//! 详见 ADR-021, docs/02-domain/domains/combat_domain.md

use bevy::prelude::*;
use tracing::info;

use crate::core::domains::combat::components::HitPoints;
use crate::core::domains::combat::events::{
    AttackRequested, DamageDealt, SpellCastRequested, UnitDied,
};

/// 处理 AttackRequested 事件 —— 计算并发射伤害事件。
///
/// TODO[P2][Combat][2026-06-21]: 解析 String ID → Entity，调用
///   DamagePolicy::evaluate() 计算真实伤害。
///   完成条件：AttackRequested 使用 Entity 替代 String。
pub fn on_attack_requested(trigger: On<AttackRequested>, mut commands: Commands) {
    let event = trigger.event();
    info!(target: "combat",
        action = "attack",
        attacker = %event.attacker_id,
        target = %event.target_id,
        "Attack requested"
    );

    // TODO[P2][Combat][2026-06-21]: Resolve String IDs to Entities via a
    //   registry/name map from party setup phase. The current stub emits
    //   flat physical damage.
    commands.trigger(DamageDealt {
        target_id: event.target_id.clone(),
        attacker_id: event.attacker_id.clone(),
        damage: 10,
        damage_type: "physical".to_string(),
        is_critical: false,
    });
}

/// 处理 SpellCastRequested 事件 —— 计算并发射法术伤害事件。
///
/// TODO[P2][Combat][2026-06-21]: 加载 SpellDef 获取真实伤害/效果数据，
///   接入 Effect Pipeline 处理法术效果。
///   完成条件：SpellCastRequested 处理器通过 SpellDef 数据计算伤害。
pub fn on_spell_cast_requested(trigger: On<SpellCastRequested>, mut commands: Commands) {
    let event = trigger.event();
    info!(target: "combat",
        action = "spell_cast",
        caster = %event.caster_id,
        spell = %event.spell_def_id,
        target = %event.target_id,
        "Spell cast requested"
    );

    // TODO[P2][Combat][2026-06-21]: Resolve spell definition for damage/effect data.
    //   For now, emit flat fire damage.
    commands.trigger(DamageDealt {
        target_id: event.target_id.clone(),
        attacker_id: event.caster_id.clone(),
        damage: 25,
        damage_type: "fire".to_string(),
        is_critical: false,
    });
}

/// 处理 DamageDealt 事件 —— 扣除目标 HP 并检测死亡。
///
/// TODO[P2][Combat][2026-06-21]: 匹配具体实体（String→Entity 解析就绪后），
///   当前实现伤害第一个 HitPoints 实体。
pub fn on_damage_dealt(
    trigger: On<DamageDealt>,
    mut hp_query: Query<&mut HitPoints>,
    mut commands: Commands,
) {
    let event = trigger.event();

    // TODO[P2][Combat][2026-06-21]: Match specific entity once String→Entity
    //   resolution is in place. Current stub damages the first HitPoints entity.
    for mut hp in hp_query.iter_mut() {
        let actual = hp.take_damage(event.damage);
        info!(target: "combat",
            action = "damage_applied",
            damage = event.damage,
            actual = actual,
            hp_current = hp.current,
            hp_max = hp.maximum,
            "Damage applied"
        );

        if !hp.is_alive() {
            info!(target: "combat",
                action = "unit_died",
                entity = %event.target_id,
                killer = %event.attacker_id,
                "Unit died from damage"
            );
            commands.trigger(UnitDied {
                entity_id: event.target_id.clone(),
                killer_id: event.attacker_id.clone(),
            });
        }
        break;
    }
}
