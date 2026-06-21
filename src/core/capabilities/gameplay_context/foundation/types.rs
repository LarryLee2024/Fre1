//! GameplayContext 基础类型定义

/// 上下文的原始触发类型。
///
/// 决定 GameplayContextData 的 origin 字段，影响溯源链的循环检测和日志分类。
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

/// 元素类型——用于伤害属性判定和元素反应链。
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
///
/// 状态流转：Building → Validated → Active → Consumed → Archived
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextStatus {
    Building,
    Validated,
    Active,
    Consumed,
    Archived,
}
