// 移动意图事件：决策层的输出
// 统一 AI 和玩家的移动请求，实现意图与执行分离

use bevy::prelude::*;

/// 移动意图事件 - 决策层的输出
/// 统一 AI 和玩家的移动请求，实现意图与执行分离
#[derive(Message, Debug, Clone)]
pub struct MovementIntent {
    /// 要移动的单位实体
    pub entity: Entity,
    /// 目标坐标
    pub target_coord: IVec2,
    /// 意图来源
    pub source: IntentSource,
}

/// 移动意图来源
#[derive(Debug, Clone, Copy)]
pub enum IntentSource {
    /// 玩家输入
    Player,
    /// AI 决策
    Ai,
}
