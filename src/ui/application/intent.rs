//! UiIntent — 语义化输入意图抽象
//!
//! InputSystem 将原始输入（键盘/鼠标/手柄）映射为业务语义的 UiIntent。
//! UiIntent 是输入管线的第一层：Input -> Intent -> Action -> Command。

use bevy::prelude::*;

use crate::ui::navigation::ScreenType;

/// 语义化输入意图，从硬件细节中抽象而来。
///
/// InputSystem 将原始事件映射为 UiIntent。IntentRouter 将意图分发给
/// 合适的子系统（FocusGroup、Widget、ScreenStack）。
///
/// # 变体分类
/// - Navigation: 焦点组导航（上下左右确认取消）
/// - Selection: 选中技能/目标/位置
/// - Screen: 屏幕导航操作
/// - System: 调试/截图等系统操作
///
/// # 验证规则 (APP-VAL-01)
/// UiIntent 不包含硬件细节 — 没有 KeyCode/GamepadButton 等硬件类型引用。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect, Event)]
pub enum UiIntent {
    // ── 导航意图（FocusGroup 导航） ──
    /// 焦点上移
    NavigateUp,
    /// 焦点下移
    NavigateDown,
    /// 焦点左移
    NavigateLeft,
    /// 焦点右移
    NavigateRight,
    /// 确认当前焦点元素
    Confirm,
    /// 取消/返回
    Cancel,

    // ── 选择意图 ──
    /// 选择技能（SkillId 占位符）
    SelectSkill(u32),
    /// 选择目标（CharacterId 占位符）
    SelectTarget(u32),

    // ── 屏幕操作意图 ──
    /// 打开指定页面
    OpenScreen(ScreenType),
    /// 关闭当前页面
    CloseScreen,
    /// 切换暂停状态
    TogglePause,
    /// 打开设置页面
    OpenSettings,
    /// 打开背包页面
    OpenInventory,

    // ── 系统意图 ──
    /// 切换调试叠加层（仅 dev 构建）
    ToggleDebug,
}
