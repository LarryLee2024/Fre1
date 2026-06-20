//! Module Name: ProgressBar Widget — 进度条原子组件
//!
//! 提供 Hp/Mp/Xp/Generic 四种变体的进度条。
//! 使用 Factory 模式创建，唯一入口为 spawn_progress_bar()。
//! 填充宽度通过 progress_bar_update_system 每帧更新。
//!
//! Contract:
//!   Props (input):    variant, current, maximum, show_label, height（通过 ProgressBarState）
//!   Events (output):  无（纯展示组件）
//!   Local State:      无
//!
//! 详见 `docs/06-ui/02-design-system/widget-atoms.md` §3

pub mod components;
pub mod factory;
pub mod systems;

use bevy::prelude::*;

use self::components::{ProgressBarFill, ProgressBarLabel, ProgressBarState, ProgressBarVariant};
use self::systems::progress_bar_update_system;

/// ProgressBarPlugin — 注册 ProgressBar Widget 所需的 Component/System
pub struct ProgressBarPlugin;

impl Plugin for ProgressBarPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ProgressBarState>()
            .register_type::<ProgressBarVariant>()
            .register_type::<ProgressBarFill>()
            .register_type::<ProgressBarLabel>()
            .add_systems(Update, progress_bar_update_system);
    }
}
