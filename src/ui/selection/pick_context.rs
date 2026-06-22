//! PickContext — 选择上下文/交互模式枚举
//!
//! 定义玩家当前处于何种交互模式，影响 PickIntent 到领域事件的转换逻辑。
//! 由 pick_context.rs 模块管理，作为全局 Resource 注册。
//!
//! 详见 ADR-068 §Module Design。

use bevy::prelude::*;

/// 选择上下文 — 当前交互模式
///
/// 影响 PickIntent 到领域事件的转换：
/// - `Normal`：普通选择模式（点击选中/取消选中）
/// - `AttackTargeting`：攻击目标选择模式
/// - `SkillTargeting`：技能目标选择模式（携带 skill_id）
/// - `Inspect`：检视模式（查看单位信息，不产生选择）
#[derive(Debug, Clone, Copy, PartialEq, Resource)]
pub enum PickContext {
    /// 普通选择模式
    Normal,
    /// 攻击目标选择模式
    AttackTargeting,
    /// 技能目标选择模式
    SkillTargeting {
        /// 当前正在选择目标的技能 ID
        skill_id: u32,
    },
    /// 检视模式
    Inspect,
}

impl Default for PickContext {
    fn default() -> Self {
        Self::Normal
    }
}
