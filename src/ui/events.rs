// UI 命令事件：UI 层发出的用户意图
// 遵循「UI 不操作 ECS，只发出意图」原则

use bevy::prelude::*;

/// UI 发出的用户命令（Command 模式）
#[derive(Message, Debug, Clone)]
pub enum UiCommand {
    /// 选中一个玩家单位
    SelectUnit { entity: Entity },
    /// 移动选中单位到目标坐标
    MoveUnit { coord: IVec2 },
    /// 选择攻击（基础攻击）
    Attack,
    /// 选择技能
    Skill { skill_id: String },
    /// 选择攻击目标
    SelectTarget { coord: IVec2 },
    /// 待机
    Wait,
    /// 取消当前操作
    Cancel,
    /// 结束回合
    EndTurn,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ui_command_各变体可构造() {
        let _ = UiCommand::SelectUnit {
            entity: Entity::from_bits(1),
        };
        let _ = UiCommand::MoveUnit {
            coord: IVec2::new(3, 4),
        };
        let _ = UiCommand::Attack;
        let _ = UiCommand::Skill {
            skill_id: "fireball".into(),
        };
        let _ = UiCommand::SelectTarget {
            coord: IVec2::new(1, 1),
        };
        let _ = UiCommand::Wait;
        let _ = UiCommand::Cancel;
        let _ = UiCommand::EndTurn;
    }

    #[test]
    fn ui_command_select_unit_字段正确() {
        let cmd = UiCommand::SelectUnit {
            entity: Entity::from_bits(42),
        };
        if let UiCommand::SelectUnit { entity } = cmd {
            assert_eq!(entity, Entity::from_bits(42));
        }
    }

    #[test]
    fn ui_command_skill_字段正确() {
        let cmd = UiCommand::Skill {
            skill_id: "fireball".into(),
        };
        if let UiCommand::Skill { skill_id } = cmd {
            assert_eq!(skill_id, "fireball");
        }
    }

    #[test]
    fn ui_command_move_unit_字段正确() {
        let cmd = UiCommand::MoveUnit {
            coord: IVec2::new(3, 4),
        };
        if let UiCommand::MoveUnit { coord } = cmd {
            assert_eq!(coord, IVec2::new(3, 4));
        }
    }
}
