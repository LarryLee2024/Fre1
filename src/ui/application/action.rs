//! UiAction — Widget 级别的交互输出声明
//!
//! Widget 在用户交互时发射 UiAction。Screen/UiActionHandler 将 UiAction
//! 转换为 UiCommand，再通过转换器进入 Domain 层。

use bevy::prelude::*;

/// Widget 级别的交互输出声明。
///
/// Widget 通过交互发射 UiAction。UiAction 不包含业务逻辑，
/// 仅声明用户"做了什么"。Screen 负责将 UiAction 转换为 UiCommand。
///
/// # 边界规则 (APP-VAL-02)
/// UiAction 是纯数据枚举，匹配分支中不包含业务逻辑。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect, Event)]
pub enum UiAction {
    // ── 通用 ──
    /// 点击（Button Widget 主要输出）
    Click,
    /// 确认
    Confirm,
    /// 取消
    Cancel,
    /// 关闭/消除
    Dismiss,

    // ── 选择 ──
    /// 选择技能（SkillId 占位符）
    SelectSkill(u32),
    /// 选择物品（ItemId 占位符）
    SelectItem(u32),
    /// 选择角色（CharacterId 占位符）
    SelectCharacter(u32),

    // ── 切换/筛选 ──
    /// 切换选中状态
    Toggle(bool),
    /// 切换标签页
    ChangeTab(usize),

    // ── 输入 ──
    /// 文本变更
    TextChanged(String),
    /// 文本确认
    TextConfirmed(String),

    // ── 自定义 ──
    /// 自定义动作（携带字符串标识）
    Custom(String),
}
