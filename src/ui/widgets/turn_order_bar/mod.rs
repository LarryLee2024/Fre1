//! 模块名: TurnOrderBar Widget — 行动顺序栏复合控件
//!
//! 组合 Panel / Text 两个原子组件为一个横向排列的行动顺序栏。
//! 显示单位的行动顺序（左侧为当前行动单位，右侧依次排列）。
//!
//! MVP 阶段使用静态占位数据（5 个预定义角色），未来将通过 ViewModel 投影刷新。
//!
//! 契约:
//!   输入属性:    无（MVP 静态数据）
//!   输出事件:    无（MVP 仅显示）
//!   本地状态:    TurnOrderBar（标记组件）, TurnOrderEntry（条目组件）
//!
//! 详见 `docs/06-ui/02-design-system/widget-composites.md`

pub mod components;
pub mod factory;

use bevy::prelude::*;

use self::components::{TurnOrderBar, TurnOrderEntry};

/// TurnOrderBarPlugin — 注册 TurnOrderBar Widget 所需的 Component
///
/// MVP 阶段无系统注册，仅注册类型以供 Reflect 序列化/观察使用。
pub struct TurnOrderBarPlugin;

impl Plugin for TurnOrderBarPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<TurnOrderBar>()
            .register_type::<TurnOrderEntry>();
    }
}
