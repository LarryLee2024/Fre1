//! Effect 测试公共 fixtures
//!
//! 提取自 facade_test.rs 和 lifecycle_test.rs 的共享工厂函数，
//! 消除跨模块的重复定义。

use crate::core::capabilities::effect::foundation::{
    ActiveEffectContainer, DurationCalculation, EffectDuration, EffectInstance,
};

/// 创建空的测试容器。
pub(crate) fn make_test_container() -> ActiveEffectContainer {
    ActiveEffectContainer::new()
}

/// 创建带固定持续时间的效果实例。
pub(crate) fn make_duration_effect(id: &str, turns: u32) -> EffectInstance {
    EffectInstance::new(
        id,
        "eff_poison",
        "Debuff",
        "caster_001",
        "target_001",
        EffectDuration::HasDuration {
            turns,
            calculation: DurationCalculation::Fixed,
        },
        1,
    )
}

/// 创建永久效果实例。
pub(crate) fn make_infinite_effect(id: &str) -> EffectInstance {
    EffectInstance::new(
        id,
        "eff_aura",
        "Buff",
        "caster_001",
        "target_001",
        EffectDuration::Infinite,
        1,
    )
}
