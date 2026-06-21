//! 横切能力抽象（日志 / 审计 / 事务 / 规则失败）

/// Sealed trait — 防止外部实现破坏框架级 trait 的不变量。
/// 仅 crate 内部类型可实现此 trait。
pub(crate) mod sealed {
    pub trait Sealed {}
}

/// 规则失败标记 trait。
///
/// 业务规则不满足是正常结果（非 Err），与程序错误严格区分。
/// 每个 domain 独立定义各自的 `*Failure` 枚举并实现此 trait。
///
/// # 设计原则
///
/// - `code()` 返回机器可读的错误码（如 `"COMBAT_NOT_YOUR_TURN"`）
/// - Display 由各 domain 自行实现（通过 `thiserror::Error` 派生）
/// - Trait 只负责统一结构，Failure 仍归各领域所有
pub trait RuleFailure: sealed::Sealed + std::fmt::Debug + Send + Sync + 'static {
    /// 返回机器可读的规则失败码。
    fn code(&self) -> &'static str;
}
