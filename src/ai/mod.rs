// AI 模块：敌方自动行动，通过 EffectQueue 执行效果
// 执行流程：决策 → 移动 → 推入 EffectQueue → 修饰 → 执行

mod behavior;
mod decision;
mod targeting;
mod movement;
mod skill_select;
mod effect_exec;
mod plugin;

// 公共 re-exports
pub use behavior::*;
pub use plugin::AiPlugin;
