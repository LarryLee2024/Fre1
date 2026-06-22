//! 模块名: BuffIcon Widget — Buff/Debuff 图标复合控件
//!
//! 组合 Panel / Text / ProgressBar 三个原子组件为一个 Buff 图标卡片。
//! 每个 BuffIcon 显示一个 Buff 的名称、剩余回合数、叠加层数和持续时间进度条。
//! 增益效果边框为绿色，减益效果边框为红色（带呼吸动画），中性效果边框为黄色。
//!
//! 契约:
//!   输入属性:    name, buff_type, remaining_turns, max_turns, stacks, tooltip_key
//!                （通过 BuffIconState）
//!   输出事件:  无（纯显示控件）
//!   本地状态:   BuffIconState（name, buff_type, remaining_turns, max_turns, stacks,
//!              tooltip_key）
//!
//! 详见 `docs/06-ui/02-design-system/widget-composites.md`

pub mod components;
pub mod factory;
pub mod systems;

use bevy::prelude::*;

use self::components::{BuffIconState, BuffType};
use self::systems::{buff_icon_breathing_system, buff_icon_update_system};

/// BuffIconPlugin — 注册 BuffIcon Widget 所需的 Component/System
pub struct BuffIconPlugin;

impl Plugin for BuffIconPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<BuffIconState>()
            .register_type::<BuffType>()
            .add_systems(Update, buff_icon_update_system)
            .add_systems(Update, buff_icon_breathing_system);
    }
}
