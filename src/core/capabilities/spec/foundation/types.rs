//! Spec 基础类型定义

use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_SPEC_ID: AtomicU64 = AtomicU64::new(1);

/// Spec 标识（自增序列，Replay-safe）。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SpecId(pub String);

impl SpecId {
    /// 生成一个新的唯一 SpecId。
    pub fn new() -> Self {
        let id = NEXT_SPEC_ID.fetch_add(1, Ordering::Relaxed);
        Self(format!("spec_{:010}", id))
    }

    /// 从字符串创建 SpecId（用于反序列化/测试）。
    pub fn from_str(s: &str) -> Self {
        Self(s.to_string())
    }

    /// 返回内部字符串引用。
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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpecType {
    Ability,
    Effect,
}

/// Spec 相关错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpecError {
    /// Def 未注册
    DefNotRegistered(String),
    /// 等级越界
    LevelOutOfRange { level: u8, min: u8, max: u8 },
    /// 重复 Spec（同一实体已有同 Def 的 Spec）
    DuplicateSpec { def_id: String, spec_id: String },
    /// Spec 不存在（移除不存在的 Spec）
    SpecNotFound(String),
    /// Spec 当前有活跃 Instance，不允许操作
    ActiveInstanceExists(String),
}

impl std::fmt::Display for SpecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DefNotRegistered(did) => write!(f, "def '{}' is not registered", did),
            Self::LevelOutOfRange { level, min, max } => {
                write!(f, "level {} out of range [{}, {}]", level, min, max)
            }
            Self::DuplicateSpec { def_id, spec_id } => {
                write!(
                    f,
                    "duplicate spec for def '{}': existing spec '{}'",
                    def_id, spec_id
                )
            }
            Self::SpecNotFound(sid) => write!(f, "spec '{}' not found", sid),
            Self::ActiveInstanceExists(sid) => {
                write!(f, "spec '{}' has active instances, cannot modify", sid)
            }
        }
    }
}

impl std::error::Error for SpecError {}
