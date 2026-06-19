//! Ability 基础类型与枚举
//!
//! 定义技能生命周期状态、分类、激活类型以及领域错误。
//!
//! 详见 docs/02-domain/capabilities/ability_domain.md §1、§2。
//! 详见 docs/04-data/capabilities/ability_schema.md §3。

use core::sync::atomic::{AtomicU64, Ordering};

static NEXT_INSTANCE_ID: AtomicU64 = AtomicU64::new(1);

/// 技能运行时阶段（状态机），定义技能当前所处的生命周期位置。
///
/// 状态转换图见 docs/02-domain/capabilities/ability_domain.md §1。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AbilityState {
    /// 就绪——可激活
    Ready,
    /// 施法/前摇——需要施法时间的技能
    Casting,
    /// 活跃/执行中——技能正作用于目标
    Active,
    /// 冷却中——等待冷却时间结束
    Cooldown,
    /// 被封锁（沉默/眩晕等），独立于主流程的状态
    Blocked,
    /// 已移除
    Removed,
}

impl AbilityState {
    /// 返回人类可读的状态名。
    pub fn name(&self) -> &str {
        match self {
            Self::Ready => "Ready",
            Self::Casting => "Casting",
            Self::Active => "Active",
            Self::Cooldown => "Cooldown",
            Self::Blocked => "Blocked",
            Self::Removed => "Removed",
        }
    }

    /// 该状态下技能是否可以被激活。
    pub fn can_activate(&self) -> bool {
        matches!(self, Self::Ready)
    }

    /// 该状态下是否可以被打断/取消。
    pub fn can_cancel(&self) -> bool {
        matches!(self, Self::Casting | Self::Active)
    }

    /// 该状态下是否可以进入冷却。
    pub fn can_cooldown(&self) -> bool {
        matches!(self, Self::Active)
    }
}

/// 技能分类。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AbilityCategory {
    /// 主动技能——需要玩家或 AI 手动激活
    Active,
    /// 被动技能——常驻效果，不需要激活
    Passive,
    /// 反应技能——回合外自动触发
    Reaction,
    /// 内在能力——种族/职业自带，不可移除
    Innate,
}

impl AbilityCategory {
    /// 是否为被动类技能（不需要激活流程）。
    pub fn is_passive(&self) -> bool {
        matches!(self, Self::Passive | Self::Innate)
    }
}

/// 技能激活类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActivationType {
    /// 瞬发——无施法时间，立即生效
    Instant,
    /// 需要施法时间（帧数）
    CastTime { frames: u64 },
    /// 需要保持专注
    Concentration,
    /// 需要蓄力（可中断）
    Charge { max_charge_frames: u64 },
    /// 反应动作（回合外触发）
    Reaction,
}

impl ActivationType {
    /// 返回该激活类型的施法总帧数。瞬发和 Reaction 返回 0。
    pub fn cast_frames(&self) -> u64 {
        match self {
            Self::Instant | Self::Reaction => 0,
            Self::CastTime { frames } => *frames,
            Self::Concentration => 0, // 专注持续施法，不由固定帧数决定
            Self::Charge { max_charge_frames } => *max_charge_frames,
        }
    }

    /// 是否为瞬发类（无施法等待）。
    pub fn is_instant(&self) -> bool {
        matches!(self, Self::Instant | Self::Reaction)
    }
}

/// 技能运行时实例唯一标识（自增序列，Replay-safe）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AbilityInstanceId(pub u64);

impl AbilityInstanceId {
    /// 生成一个新的唯一 AbilityInstanceId。
    pub fn new() -> Self {
        let id = NEXT_INSTANCE_ID.fetch_add(1, Ordering::Relaxed);
        Self(id)
    }

    /// 从 u64 创建（用于反序列化/测试）。
    pub fn from_u64(id: u64) -> Self {
        Self(id)
    }

    /// 返回内部 u64 值。
    pub fn value(&self) -> u64 {
        self.0
    }
}

impl Default for AbilityInstanceId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for AbilityInstanceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "inst_{:010}", self.0)
    }
}

impl From<u64> for AbilityInstanceId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

/// Ability 领域错误。
#[derive(Debug, Clone, PartialEq)]
pub enum AbilityError {
    /// 技能不在可激活状态（如冷却中/已移除）
    NotReady {
        current_state: AbilityState,
        spec_id: String,
    },
    /// 条件检查不通过
    ConditionFailed { reason: String },
    /// 资源消耗不足
    InsufficientCost {
        resource: String,
        required: f32,
        available: f32,
    },
    /// 技能有正在运行的活跃实例，不允许再次激活
    AlreadyActive {
        spec_id: String,
        instance_id: AbilityInstanceId,
    },
    /// 技能不存在于实体的容器中
    SpecNotFound(String),
    /// 实例不存在
    InstanceNotFound(AbilityInstanceId),
    /// 无效的状态转换
    InvalidTransition {
        from: AbilityState,
        to: AbilityState,
        reason: String,
    },
    /// 冷却中不可激活
    OnCooldown {
        spec_id: String,
        remaining_turns: u32,
    },
    /// Spec 未指定（激活时需要关联 Spec）
    MissingSpec,
    /// 通用运行时错误
    Runtime(String),
}

impl std::fmt::Display for AbilityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotReady {
                current_state,
                spec_id,
            } => {
                write!(
                    f,
                    "ability '{}' not ready to activate (current state: {})",
                    spec_id,
                    current_state.name()
                )
            }
            Self::ConditionFailed { reason } => {
                write!(f, "condition check failed: {}", reason)
            }
            Self::InsufficientCost {
                resource,
                required,
                available,
            } => {
                write!(
                    f,
                    "insufficient '{}': required {}, available {}",
                    resource, required, available
                )
            }
            Self::AlreadyActive {
                spec_id,
                instance_id,
            } => {
                write!(
                    f,
                    "ability '{}' already has active instance {}",
                    spec_id, instance_id
                )
            }
            Self::SpecNotFound(sid) => write!(f, "spec '{}' not found", sid),
            Self::InstanceNotFound(iid) => write!(f, "instance '{}' not found", iid),
            Self::InvalidTransition { from, to, reason } => {
                write!(
                    f,
                    "invalid state transition from {} to {}: {}",
                    from.name(),
                    to.name(),
                    reason
                )
            }
            Self::OnCooldown {
                spec_id,
                remaining_turns,
            } => {
                write!(
                    f,
                    "ability '{}' on cooldown ({} turns remaining)",
                    spec_id, remaining_turns
                )
            }
            Self::MissingSpec => write!(f, "missing spec reference for ability activation"),
            Self::Runtime(msg) => write!(f, "runtime error: {}", msg),
        }
    }
}

impl std::error::Error for AbilityError {}
