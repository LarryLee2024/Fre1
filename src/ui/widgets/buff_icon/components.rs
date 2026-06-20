//! BuffIcon 组件的类型定义
//!
//! 定义 BuffIconState（Widget Contract 的本地状态）。
//! BuffIconState 挂载在 BuffIcon 容器实体上。
//!
//! 详见 `docs/06-ui/02-design-system/widget-composites.md`

use bevy::prelude::*;

/// BuffIcon 本地状态（Widget Contract Local State）
///
/// 包含 Buff 名称、剩余回合数、最大回合数以及是否为减益效果。
/// Props 字段由 spawn_buff_icon 的入参决定，runtime 由外部系统更新。
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct BuffIconState {
    /// Buff 显示名称
    pub name: String,
    /// 剩余持续回合数
    pub remaining_turns: u32,
    /// 最大持续回合数
    pub max_turns: u32,
    /// 是否为减益效果（Debuff）
    pub is_debuff: bool,
}
