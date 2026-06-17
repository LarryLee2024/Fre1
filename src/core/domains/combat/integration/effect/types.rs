//! Combat 域效果集成视图类型。
//!
//! 这些类型是 Combat 域对 Effect Capability 内部结构的「翻译层」。
//! 当 `ActiveEffectContainer` / `TickResult` 内部变化时，只需修改 `facade.rs`，
//! systems 和 rules 完全无感。
//!
//! # 设计原则
//!
//! - 仅定义 Combat 域需要看到的视图，不重复 Capabilities 内部类型
//! - 使用 newtype 和结构化摘要，避免暴露裸 Capabilities 枚举/集合

/// 效果 Tick 执行结果 — Combat 域的统一返回类型。
///
/// 替代直接暴露 Capabilities 层的 `TickResult`。
/// Systems 通过此类型获取 Tick 结果，完全不知道 `TickResult` 的存在。
#[derive(Debug, Clone)]
pub struct EffectTickOutcome {
    /// 触发了周期 Tick 的 effect instance IDs
    pub ticked: Vec<String>,
    /// 本轮转为 Expiring 的 effect instance IDs
    pub expired: Vec<String>,
    /// 处理过程中出现的错误数
    pub error_count: usize,
}

impl EffectTickOutcome {
    /// 创建空结果。
    pub fn empty() -> Self {
        Self {
            ticked: Vec::new(),
            expired: Vec::new(),
            error_count: 0,
        }
    }

    /// 是否有任何效果被 Tick 或到期。
    pub fn has_activity(&self) -> bool {
        !self.ticked.is_empty() || !self.expired.is_empty()
    }
}
