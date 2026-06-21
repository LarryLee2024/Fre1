use bevy::prelude::*;

use super::foundation::{
    ActiveEffectContainer, DurationCalculation, EffectDuration, EffectInstance, EffectPeriod,
    EffectStage, TickState,
};

/// Effect 能力插件——注册效果系统的 ECS 组件与类型反射。
///
/// # 职责
/// - 注册 `ActiveEffectContainer` 及相关类型到 Bevy 类型注册表（反射支持）
/// - 效果生命周期事件（EffectApplied、EffectRemoved、EffectTicked、EffectImmunityTriggered）
///   通过 `commands.trigger()` 发射，由外部订阅者（日志、UI 投影等）注册 Observer 消费
///
/// # 生命周期集成
/// 效果生命周期的纯函数实现在 `mechanism::lifecycle` 中，由领域插件通过
/// `integration/facade` 直接调用。涉及 ECS 的操作（容器修改、事件发射）已在
/// 纯函数内部通过 `&mut ActiveEffectContainer` + `&mut Commands` 完成，
/// 无需在 Plugin 层额外注册 Observer。
///
/// # 外部观察者
/// - `infra/logging/observers/effect_logger.rs` — 日志记录
/// - `ui/projections/battle.rs` — UI 投影更新
///
/// 详见 docs/02-domain/capabilities/effect_domain.md
/// 详见 docs/04-data/capabilities/effect_schema.md
pub struct EffectPlugin;

impl Plugin for EffectPlugin {
    fn build(&self, app: &mut App) {
        // 注册组件及其嵌套类型到类型注册表，支持反射（UI 投影、Save/Load、调试需要）
        // ActiveEffectContainer 已通过 #[derive(Component)] + #[reflect(Component)] 声明
        app.register_type::<ActiveEffectContainer>()
            .register_type::<EffectInstance>()
            .register_type::<TickState>()
            .register_type::<EffectStage>()
            .register_type::<EffectDuration>()
            .register_type::<DurationCalculation>()
            .register_type::<EffectPeriod>();
    }
}
