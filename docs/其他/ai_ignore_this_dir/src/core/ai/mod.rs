/// AI 模块：敌方自动行动
/// 执行流程：决策 → 移动 → 设置 CombatIntent → 切换 ExecuteAction
/// 攻击效果由统一的 Effect Pipeline（generate→modify→execute）处理

/// AIBehavior 数据定义与注册表
mod behavior;
/// enemy_ai_system 主决策系统
mod decision;
/// 移动策略（激进/谨慎/默认）
mod movement;
/// 技能选择策略
mod skill_select;
/// AiStrategyRegistry 策略注册表
mod strategy;
/// 目标选择策略（最近/最弱/最危险）
mod targeting;

use crate::core::turn::AppState;
use bevy::prelude::*;

/// 公共 re-exports
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
