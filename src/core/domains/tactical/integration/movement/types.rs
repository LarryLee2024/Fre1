//! Tactical 域移动能力视图类型。
//!
//! 这些类型是 Tactical 对 Capabilities 内部结构的「翻译层」。
//! 当 AttributeContainer / ModifierContainer 内部变化时，只需修改 facade.rs，
//! systems 和 rules 完全无感。

use crate::core::domains::tactical::components::MovementType;

/// 移动力值（newtype，禁止裸 f32 满天飞）。
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct MP(pub f32);

impl Default for MP {
    fn default() -> Self {
        Self::ZERO
    }
}

impl MP {
    pub const ZERO: Self = Self(0.0);

    pub fn is_zero(self) -> bool {
        self.0 <= 0.0
    }

    pub fn can_afford(self, cost: MP) -> bool {
        self.0 >= cost.0
    }
}

impl std::fmt::Display for MP {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.1} MP", self.0)
    }
}

/// 移动修正摘要（替代直接暴露 ModifierData）。
#[derive(Debug, Clone, Default)]
pub struct MovementModifierSummary {
    pub flat_bonus: MP,
    pub multiplier: f32,
    pub total_effect: MP,
}

/// 移动能力评估报告（结构化查询结果）。
///
/// Systems 通过此类型获取所有移动相关数据，
/// 完全不知道 TagSet / AttributeContainer / ModifierContainer 的存在。
#[derive(Debug, Clone)]
pub struct MovementCapabilityView {
    pub can_move: bool,
    pub effective_points: MP,
    pub max_points: MP,
    pub movement_type: MovementType,
    pub modifier_summary: MovementModifierSummary,
}

/// 移动前提条件错误。
#[derive(Debug, Clone, thiserror::Error)]
pub enum MovementPrerequisiteError {
    #[error("no tag registered for movement type {0:?}")]
    NoTagForMovementType(MovementType),
    #[error("insufficient movement points: available {available}, required {required}")]
    InsufficientPoints { available: MP, required: MP },
}

/// 移动成本错误。
#[derive(Debug, Clone, thiserror::Error)]
pub enum MovementCostError {
    #[error("insufficient movement points: available {available}, cost {cost}")]
    InsufficientPoints { available: MP, cost: MP },
}
