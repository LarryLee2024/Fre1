//! ActionMenu 组件的类型定义
//!
//! 定义 ActionType 枚举、ActionMenuItem 数据结构和 ActionMenuState（Widget Contract 的本地状态）。
//! ActionMenuState 挂载在菜单容器实体上，ActionType 挂载在按钮实体上。
//!
//! 详见 `docs/06-ui/02-design-system/widget-composites.md`

use bevy::prelude::*;

/// 战斗行动类型
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub enum ActionType {
    /// 攻击
    Attack,
    /// 取消（退出目标选择模式）
    Cancel,
    /// 防御
    Defend,
    /// 技能
    Skill,
    /// 物品
    Item,
    /// 待机/结束回合
    Wait,
}

/// 行动菜单项数据
#[derive(Debug, Clone, Reflect)]
pub struct ActionMenuItem {
    /// 显示的标签文本
    pub label: String,
    /// 行动类型
    pub action_type: ActionType,
    /// 是否启用
    pub enabled: bool,
}

/// 行动菜单本地状态（Widget Contract Local State）
///
/// 包含所有可用行动的列表。
/// Props 字段由 spawn_action_menu 的入参决定，runtime 由外部系统更新。
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct ActionMenuState {
    /// 行动菜单项列表
    pub actions: Vec<ActionMenuItem>,
}
