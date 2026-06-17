/// 效果管道：EffectDef → PendingEffect → 修饰 → 执行
/// ADR-026 §二：Buff 已删除，统一为 ApplyModifier
mod handler;
mod types;

pub use handler::{
    CleanseHandler, DamageHandler, EffectHandler, EffectHandlerRegistry, EffectPreview,
    ExecuteContext, ExecuteOutput, GenerateContext, HealHandler, ModifierHandler, PendingMessage,
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
