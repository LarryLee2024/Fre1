//! Module Name: BuffIcon Widget — Buff/Debuff 图标复合控件
//!
//! 组合 Panel / Text / ProgressBar 三个原子组件为一个 Buff 图标卡片。
//! 每个 BuffIcon 显示一个 Buff 的名称、剩余回合数和持续时间进度条。
//! 减益效果边框为红色，增益效果边框为绿色。
//!
//! Contract:
//!   Props (input):    name, remaining_turns, max_turns, is_debuff（通过 BuffIconState）
//!   Events (output):  无（纯显示控件）
//!   Local State:      BuffIconState（name, remaining_turns, max_turns, is_debuff）
//!
//! 详见 `docs/06-ui/02-design-system/widget-composites.md`

pub mod components;
pub mod factory;
pub mod systems;

use bevy::prelude::*;

use self::components::BuffIconState;
use self::systems::buff_icon_update_system;

/// BuffIconPlugin — 注册 BuffIcon Widget 所需的 Component/System
pub struct BuffIconPlugin;

impl Plugin for BuffIconPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<BuffIconState>()
            .add_systems(Update, buff_icon_update_system);
    }
}
