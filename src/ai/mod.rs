// AI 模块：敌方自动行动
// 执行流程：决策 → 移动 → 设置 CombatIntent → 切换 ExecuteAction
// 攻击效果由统一的 Effect Pipeline（generate→modify→execute）处理

mod behavior;
mod decision;
mod movement;
mod plugin;
mod skill_select;
mod strategy;
mod targeting;

// 公共 re-exports
pub use behavior::*;
pub use plugin::AiPlugin;
