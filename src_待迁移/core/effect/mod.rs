/// 效果管道：EffectDef → PendingEffect → 修饰 → 执行
/// 替代 combat_event.rs 中的 execute_attack 大函数
mod handler; // EffectHandler trait 与各类型处理器（Damage/Heal/Buff/Cleanse）
mod types; // EffectDef, PendingEffect, EffectResult 等类型定义

// 重新导出所有公共类型，保持外部导入路径兼容
pub use handler::{
    BuffHandler, CleanseHandler, DamageHandler, EffectHandler, EffectHandlerRegistry,
    EffectPreview, ExecuteContext, ExecuteOutput, GenerateContext, HealHandler, PendingMessage,
    PreviewContext,
};
pub use types::*;

use bevy::prelude::*;

/// 效果管道插件
pub struct EffectPlugin;

impl Plugin for EffectPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EffectQueue>()
            .init_resource::<EffectHandlerRegistry>();
    }
}
