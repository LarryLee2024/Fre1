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

#[cfg(test)]
mod tests {
    // ================================================
    // AI Self-Check (test_spec.md §13.1)
    // ================================================
    // ✅ 测试行为，不是实现
    // ✅ 符合领域规则 (ui_rules_v1.md 规则 1)
    // ✅ 测试是确定性的
    // ✅ 使用标准测试数据
    // ✅ 没有测试私有实现
    // ✅ 没有生成不在范围内的测试
    // ================================================

    use super::*;

    /// Test ID: UI-CMD-001
    /// Title: UiCommand 各变体可构造
    ///
    /// Given: 无前置条件
    /// When: 构造 UiCommand 的所有变体
    /// Then: 所有变体均可成功构造，无编译错误
    ///
    /// Assertions: 变体构造成功（编译时验证）
    #[test]
    fn ui_command_variants_constructible() {
        // Given - 无

        // When - 构造所有变体
        let _select_unit = UiCommand::SelectUnit {
            entity: Entity::from_bits(1),
        };
        let _move_unit = UiCommand::MoveUnit {
            coord: IVec2::new(3, 4),
        };
        let _attack = UiCommand::Attack;
        let _skill = UiCommand::Skill {
            skill_id: "fireball".into(),
        };
        let _select_target = UiCommand::SelectTarget {
            coord: IVec2::new(1, 1),
        };
        let _wait = UiCommand::Wait;
        let _cancel = UiCommand::Cancel;
        let _end_turn = UiCommand::EndTurn;

        // Then - 编译通过即验证成功
    }

    /// Test ID: UI-CMD-002
    /// Title: UiCommand::SelectUnit 携带正确的 Entity
    ///
    /// Given: 一个 Entity ID (42)
    /// When: 构造 UiCommand::SelectUnit
    /// Then: 命令携带正确的 Entity
    ///
    /// Assertions: 解构后 entity 字段等于 42
    #[test]
    fn ui_command_select_unit_carries_entity() {
        // Given
        let expected_entity = Entity::from_bits(42);

        // When
        let cmd = UiCommand::SelectUnit {
            entity: expected_entity,
        };

        // Then
        if let UiCommand::SelectUnit { entity } = cmd {
            assert_eq!(entity, expected_entity);
        } else {
            panic!("Expected UiCommand::SelectUnit");
        }
    }

    /// Test ID: UI-CMD-003
    /// Title: UiCommand::Skill 携带正确的 skill_id
    ///
    /// Given: 技能 ID "fireball"
    /// When: 构造 UiCommand::Skill
    /// Then: 命令携带正确的 skill_id
    ///
    /// Assertions: 解构后 skill_id 等于 "fireball"
    #[test]
    fn ui_command_skill_carries_skill_id() {
        // Given
        let expected_skill_id = "fireball".to_string();

        // When
        let cmd = UiCommand::Skill {
            skill_id: expected_skill_id.clone(),
        };

        // Then
        if let UiCommand::Skill { skill_id } = cmd {
            assert_eq!(skill_id, expected_skill_id);
        } else {
            panic!("Expected UiCommand::Skill");
        }
    }

    /// Test ID: UI-CMD-004
    /// Title: UiCommand::MoveUnit 携带正确的坐标
    ///
    /// Given: 目标坐标 (3, 4)
    /// When: 构造 UiCommand::MoveUnit
    /// Then: 命令携带正确的坐标
    ///
    /// Assertions: 解构后 coord 等于 (3, 4)
    #[test]
    fn ui_command_move_unit_carries_coord() {
        // Given
        let expected_coord = IVec2::new(3, 4);

        // When
        let cmd = UiCommand::MoveUnit {
            coord: expected_coord,
        };

        // Then
        if let UiCommand::MoveUnit { coord } = cmd {
            assert_eq!(coord, expected_coord);
        } else {
            panic!("Expected UiCommand::MoveUnit");
        }
    }
}
