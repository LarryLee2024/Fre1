//! 模块名: UnitSummary Widget — 单位摘要复合控件
//!
//! 组合 Panel / Text / ProgressBar 三个原子组件为一个紧凑的右上角信息卡片。
//! 显示当前选中单位的名称、等级和 HP 摘要信息。
//!
//! 契约:
//!   输入属性:    name, level, hp_current, hp_max（通过 spawn_unit_summary 参数）
//!   输出事件:    无（纯显示控件）
//!   本地状态:    UnitSummary（标记组件）
//!
//! 详见 `docs/06-ui/02-design-system/widget-composites.md`
//!
//! # 布局
//!
//! ```text
//! +--------------------+
//! | Aria      Lv.5     |
//! | HP: 80/100         |
//! | [████████░░░░░░░░] |
//! +--------------------+
//! ```

pub mod components;
pub mod factory;

use bevy::prelude::*;

use self::components::UnitSummary;

/// UnitSummaryPlugin — 注册 UnitSummary Widget 所需的 Component
///
/// MVP 阶段无需系统注册，仅注册类型以供 Reflect 使用。
pub struct UnitSummaryPlugin;

impl Plugin for UnitSummaryPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<UnitSummary>();
    }
}
