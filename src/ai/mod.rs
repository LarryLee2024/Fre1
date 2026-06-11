// AI 模块：敌方自动行动
// 执行流程：决策 → 移动 → 设置 CombatIntent → 切换 ExecuteAction
// 攻击效果由统一的 Effect Pipeline（generate→modify→execute）处理

mod behavior;
mod decision;
mod movement;
mod skill_select;
mod strategy;
mod targeting;

use crate::turn::AppState;
use bevy::prelude::*;

// 公共 re-exports
pub use behavior::*;

/// AI 插件
pub struct AiPlugin;

impl Plugin for AiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(strategy::AiStrategyRegistry::default());
        app.add_systems(
            Update,
            decision::enemy_ai_system.run_if(in_state(AppState::InGame)),
        );
    }
}
