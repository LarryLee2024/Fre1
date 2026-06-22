//! BuffIcon 组件的类型定义
//!
//! 定义 BuffType 枚举和 BuffIconState（Widget Contract 的本地状态）。
//! BuffIconState 挂载在 BuffIcon 容器实体上。
//!
//! 详见 `docs/06-ui/02-design-system/widget-composites.md`

use bevy::prelude::*;

/// Buff 类型
///
/// 区分增益效果、减益效果和中性效果，
/// 用于图标边框颜色和动画效果的选择。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum BuffType {
    /// 增益效果（绿色边框）
    Buff,
    /// 减益效果（红色边框，带呼吸动画）
    Debuff,
    /// 中性效果（黄色边框）
    Neutral,
}

/// 从布尔值转换为 BuffType（保留向后兼容）
///
/// `false` → `Buff`, `true` → `Debuff`
impl From<bool> for BuffType {
    fn from(is_debuff: bool) -> Self {
        if is_debuff { Self::Debuff } else { Self::Buff }
    }
}

impl BuffType {
    /// 返回是否符合减益语义（保留向后兼容）
    pub fn is_debuff(&self) -> bool {
        matches!(self, Self::Debuff)
    }
}

/// BuffIcon 本地状态（Widget Contract Local State）
///
/// 包含 Buff 类型、名称、剩余回合数、叠加层数等信息。
/// Props 字段由 spawn_buff_icon 的入参决定，runtime 由外部系统更新。
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct BuffIconState {
    /// Buff 显示名称
    pub name: String,
    /// Buff 类型（增益/减益/中性）
    pub buff_type: BuffType,
    /// 剩余持续回合数
    pub remaining_turns: u32,
    /// 最大持续回合数
    pub max_turns: u32,
    /// 叠加层数（0 = 无叠加，不显示徽章）
    pub stacks: u32,
    /// 悬浮提示的本地化 key（MVP 阶段可填空字符串）
    pub tooltip_key: String,
}
