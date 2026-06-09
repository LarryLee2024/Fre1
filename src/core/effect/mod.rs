// 效果管道：EffectDef → PendingEffect → 修饰 → 执行
// 替代 combat_event.rs 中的 execute_attack 大函数

mod handler;
mod types;

// 重新导出所有公共类型，保持外部导入路径兼容
pub use handler::{
    BuffHandler, CleanseHandler, DamageHandler, EffectHandler, EffectHandlerRegistry,
    EffectPreview, GenerateContext, HealHandler, PreviewContext,
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
