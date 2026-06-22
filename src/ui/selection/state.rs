//! Selection — 选择状态管理
//!
//! 包含：
//! - `Selection`：旧的简易选择资源（向后兼容，Phase 1 迁移目标）
//! - `SelectionState`：五态分离的新选择状态机（Phase 3+ 启用）
//!
//! 详见 ADR-068 §Module Design 和 docs/04-data/domains/tactical_schema.md §6。

use bevy::prelude::*;

use super::pick_context::PickContext;
use crate::ui::picking::pick_target::PickTarget;

// ─── 旧 Selection 资源（向后兼容） ──────────────────────────────────

/// 当前选择状态（Phase 1 MVP 简化版）
///
/// 存储当前选中的单位 Entity。
/// 从 `infra/picking/selection.rs` 迁移至此。
/// 后续由 `SelectionState` 替换（Phase 3）。
#[derive(Resource, Default, Debug)]
pub struct Selection {
    /// 当前选中的单位
    pub selected_unit: Option<Entity>,
}

// ─── 新 SelectionState（五态分离） ──────────────────────────────────

/// 五态分离的 Selection 状态机
///
/// 五种选择状态的信号量设计，每种状态独立追踪选中的目标：
/// - `hovered`：鼠标悬停（无确认选中）
/// - `focused`：键盘/手柄焦点
/// - `selected`：玩家确认选中
/// - `targeted`：技能/攻击目标选择
/// - `activated`：当前行动单位
///
/// 详见 docs/04-data/domains/tactical_schema.md §6。
#[derive(Resource, Debug, Clone, PartialEq)]
pub struct SelectionState {
    /// 鼠标悬停的目标
    pub hovered: Option<PickTarget>,
    /// 键盘/手柄焦点目标
    pub focused: Option<PickTarget>,
    /// 玩家确认选中的目标
    pub selected: Option<PickTarget>,
    /// 技能/攻击目标选择
    pub targeted: Option<PickTarget>,
    /// 当前行动单位
    pub activated: Option<PickTarget>,
    /// 选择上下文（当前交互模式）
    pub context: PickContext,
}

impl Default for SelectionState {
    fn default() -> Self {
        Self {
            hovered: None,
            focused: None,
            selected: None,
            targeted: None,
            activated: None,
            context: PickContext::Normal,
        }
    }
}

impl SelectionState {
    /// 清除所有选择状态
    pub fn clear(&mut self) {
        self.hovered = None;
        self.focused = None;
        self.selected = None;
        self.targeted = None;
        self.activated = None;
        self.context = PickContext::Normal;
    }

    /// 检查当前选择状态是否有效
    ///
    /// 有效条件：至少有一个状态非 None。
    pub fn is_valid(&self) -> bool {
        self.hovered.is_some()
            || self.focused.is_some()
            || self.selected.is_some()
            || self.targeted.is_some()
            || self.activated.is_some()
    }

    /// 获取选中的单位 ID（如果有）
    pub fn selected_unit_id(&self) -> Option<&str> {
        match &self.selected {
            Some(PickTarget::Unit(id)) => Some(id.as_str()),
            _ => None,
        }
    }

    /// 获取悬停的单位 ID（如果有）
    pub fn hovered_unit_id(&self) -> Option<&str> {
        match &self.hovered {
            Some(PickTarget::Unit(id)) => Some(id.as_str()),
            _ => None,
        }
    }
}
