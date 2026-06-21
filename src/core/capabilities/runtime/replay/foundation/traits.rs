//! Replay 横切能力抽象——事件可回放性 trait
//!
//! 定义 `Replayable` trait 和 `ReplayAction`，配合 Marker Trait 实现自动派生。
//! `Replayable` 标记一个类型可以被回放系统记录和重放。
//!
//! # Blanket Impl
//!
//! ```ignore
//! // 所有 DomainEvent 自动获得 Replayable 能力
//! impl<T: DomainEvent> Replayable for T { ... }
//! ```
//!
//! 禁止手动为每个事件类型重复实现可自动推导的 trait。

use crate::shared::diagnostics::DomainEvent;

/// 回放动作——描述事件在回放系统中应如何处理。
#[derive(Debug, Clone, Default)]
pub struct ReplayAction {
    /// 是否记录此事件到回放日志。
    pub record: bool,
    /// 回放优先级（数值越大越优先）。
    pub priority: u8,
}

/// 可回放——标记一个类型可以被回放系统记录和重放。
///
/// 通过 Blanket Impl，所有实现了 `DomainEvent` 的类型自动获得此能力。
/// 无需手动为每个事件类型实现此 trait。
///
/// # 示例
///
/// ```ignore
/// // LevelUp 实现了 DomainEvent，因此自动获得 Replayable 能力
/// let event = LevelUp { ... };
/// let action = event.replay();
/// assert!(action.record);
/// ```
/// 回放可记录 trait。
///
/// 存在原因：所有 DomainEvent 需要决定是否纳入 Replay 录制，
/// 此 trait 提供 `replay()` 方法返回 ReplayAction（record/skip），由 blanket impl 自动派生。
pub trait Replayable {
    /// 返回该事件的回放动作。
    fn replay(&self) -> ReplayAction;
}

/// Blanket Impl：所有 DomainEvent 自动获得 Replayable 能力。
///
/// 当一个能力可由另一个能力自动推导时，使用 blanket impl 避免重复实现。
/// 这是 Blanket Impl 模式的核心应用：
/// - 你只需实现 `DomainEvent`（添加一个标记）
/// - `Replayable` 能力自动派生
/// - 无需为每个事件类型手动实现 `Replayable`
///
/// # 注意
///
/// `ReplayEvent` 和 `AuditEvent` 不提供 blanket impl 以避免 trait 冲突。
/// 这些标记 trait 仅用于事件分类和自动注册，不属于可回放能力的范畴。
impl<T: DomainEvent + 'static> Replayable for T {
    fn replay(&self) -> ReplayAction {
        // 默认实现：所有 domain event 默认被记录和回放
        ReplayAction {
            record: true,
            priority: 0,
        }
    }
}
