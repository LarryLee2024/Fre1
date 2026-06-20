//! Spec 基础类型定义

use bevy::prelude::Reflect;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_SPEC_ID: AtomicU64 = AtomicU64::new(1);

/// Spec 标识（自增序列，Replay-safe）。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect)]
pub struct SpecId(pub String);

impl SpecId {
    /// 使用 AtomicU64 自增序列，Replay-safe（不受随机数种子影响）。
    pub fn new() -> Self {
        let id = NEXT_SPEC_ID.fetch_add(1, Ordering::Relaxed);
        Self(format!("spec_{:010}", id))
    }

    /// 从字符串创建 SpecId（用于反序列化/测试）。
    pub fn from_str(s: &str) -> Self {
        Self(s.to_string())
    }

    /// 用于与外部系统交互（序列化、存储查找）。
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for SpecId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for SpecId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for SpecId {
    fn from(s: &str) -> Self {
        Self::from_str(s)
    }
}

/// 强化/专长标识。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect)]
pub struct EnhancementId(pub String);

impl EnhancementId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl From<&str> for EnhancementId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Spec 类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum SpecType {
    Ability,
    Effect,
}

