/// 触发器系统：统一注册与分发、嵌套触发栈管理
/// 参考：docs/01-architecture/skill-buff-abstraction.md §4.8
/// 参考：docs/02-domain/trigger/trigger-rules.md
mod registry;
mod stack;
mod types;

pub use registry::{TriggerHandler, TriggerRegistry};
pub use stack::{ExecutionStack, MAX_STACK_DEPTH, StackEntry, StackOverflowError};
pub use types::{Trigger, TriggerContext};

use bevy::prelude::*;

/// 触发器插件：注册 ExecutionStack 和 TriggerRegistry 资源
pub struct TriggerPlugin;

impl Plugin for TriggerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ExecutionStack>()
            .init_resource::<TriggerRegistry>();
    }
}
