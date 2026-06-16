//! GameplayContext 基础类型定义

/// 上下文的原始触发类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContextOrigin {
    /// 直接行为（如主动施放技能）
    Direct,
    /// 链式反应（如反击、连锁闪电）
    ChainReaction,
    /// 触发器触发（如 OnDamaged 触发的被动技能）
    Triggered,
    /// 周期性触发（如 DoT/HoT 的每跳）
    Periodic,
    /// 环境原因（如陷阱、毒池）
    Environmental,
}

/// 元素类型。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ElementType {
    Fire,
    Ice,
    Lightning,
    Acid,
    Poison,
    Holy,
    Dark,
    Physical,
    Custom(String),
}

/// 上下文生命周期状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextStatus {
    Building,
    Validated,
    Active,
    Consumed,
    Archived,
}

/// 上下文构建错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextBuildError {
    /// 缺失必填字段（列出缺失字段名）
    MissingFields(Vec<String>),
    /// 溯源链检测到循环
    CycleDetected,
    /// 溯源链达到长度上限
    ChainTooLong { current: u8, max: u8 },
}

impl std::fmt::Display for ContextBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingFields(fields) => {
                write!(f, "missing required fields: {}", fields.join(", "))
            }
            Self::CycleDetected => write!(f, "context chain cycle detected"),
            Self::ChainTooLong { current, max } => {
                write!(f, "chain length {} exceeds max {}", current, max)
            }
        }
    }
}

impl std::error::Error for ContextBuildError {}
