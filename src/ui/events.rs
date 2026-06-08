// UI 命令事件：UI 层发出的用户意图
// 遵循「UI 不操作 ECS，只发出意图」原则

use bevy::prelude::*;

/// UI 发出的用户命令（Command 模式）
#[derive(Message, Debug, Clone)]
pub enum UiCommand {
    /// 选中一个玩家单位
    SelectUnit {
        entity: Entity,
    },
    /// 移动选中单位到目标坐标
    MoveUnit {
        coord: IVec2,
    },
    /// 选择攻击（基础攻击）
    Attack,
    /// 选择技能
    Skill {
        skill_id: String,
    },
    /// 选择攻击目标
    SelectTarget {
        coord: IVec2,
    },
    /// 待机
    Wait,
    /// 取消当前操作
    Cancel,
    /// 结束回合
    EndTurn,
}
