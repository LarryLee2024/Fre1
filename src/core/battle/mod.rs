/// 战斗模块：效果管线、伤害计算、战斗日志、战斗事件

/// 曼哈顿距离、战斗数值计算
mod combat;
/// 战斗领域错误枚举（错误码 B001-B006）
mod error;
/// 战斗事件定义（DamageApplied, HealApplied, CharacterDied 等）
mod events;
/// CombatLog 资源，结构化战斗日志
mod log;
/// Effect Pipeline：生成→修饰→执行三步管道
mod pipeline;
/// BattleRecord 资源，结构化战斗记录（用于调试面板）
mod record;

use crate::core::effect::{
    EffectDef, EffectQueue, EffectResult, EffectResultData, PendingEffect, PendingEffectData,
};
use bevy::prelude::*;

/// 公共 re-exports
pub use combat::*;
pub use error::*;
pub use events::*;
pub use log::*;
pub use pipeline::{
    CombatIntent, PrevPosition, execute_effects, trigger_on_attack_traits, trigger_on_hit_traits,
    trigger_on_kill_traits,
};
pub use record::*;

/// 战斗插件（组合 Effect Pipeline + CombatLog + BattleRecord 子插件）
pub struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<record::BattleRecord>()
            // 注册 Reflect 类型
            .register_type::<EffectDef>()
            .register_type::<PendingEffectData>()
            .register_type::<PendingEffect>()
            .register_type::<EffectResultData>()
            .register_type::<EffectResult>()
            .register_type::<EffectQueue>()
            .register_type::<record::BattleEntry>()
            .register_type::<record::DamageBreakdown>()
            .register_type::<record::ModifierEntry>()
            .register_type::<record::EntityBattleStats>()
            .register_type::<record::BattleRecord>()
            .add_message::<events::CharacterDied>()
            .add_message::<events::DamageApplied>()
            .add_message::<events::HealApplied>()
            .add_message::<events::StunApplied>()
            .add_message::<events::DotApplied>()
            .add_message::<events::HotApplied>()
            .add_plugins((pipeline::CombatEventPlugin, log::CombatLogPlugin))
            .add_systems(
                Update,
                (
                    record::record_turn_started,
                    record::record_turn_ended,
                    record::record_damage,
                    record::record_heal,
                    record::record_dot,
                    record::record_hot,
                    record::record_stun,
                    record::record_character_died,
                ),
            );
    }
}
