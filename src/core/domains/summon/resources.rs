//! 召唤领域 — 资源配置

use bevy::prelude::*;

/// 召唤系统配置。
#[derive(Debug, Clone, Resource)]
pub struct SummonConfig {
    /// 默认召唤槽位数量
    pub default_max_slots: u32,
    /// 是否允许嵌套召唤
    pub allow_nested_summon: bool,
    /// 召唤者死亡时召唤物的消失延迟（回合数）
    pub caster_death_expire_delay: u32,
}

impl Default for SummonConfig {
    fn default() -> Self {
        Self {
            default_max_slots: 1,
            allow_nested_summon: false,
            caster_death_expire_delay: 0,
        }
    }
}
