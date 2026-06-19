//! Effect 基础类型与枚举
//!
//! 定义效果生命周期阶段、持续时间类型、周期参数、效果分类以及领域错误。
//!
//! 详见 docs/02-domain/capabilities/effect_domain.md §1、§3。
//! 详见 docs/04-data/capabilities/effect_schema.md §3。

use bevy::prelude::Reflect;
use serde::{Deserialize, Serialize};

/// 效果生命周期阶段（四阶段状态机）。
///
/// 转换规则见 docs/02-domain/capabilities/effect_domain.md §2。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum EffectStage {
    /// 施加阶段——检查条件，初始化
    Applying,
    /// 持续阶段——周期性 Tick
    Active,
    /// 到期阶段——执行移除前逻辑（Modifier 回退、Tag 清理）
    Expiring,
    /// 已移除
    Removed,
}

impl EffectStage {
    /// 返回阶段名称。
    pub fn name(&self) -> &str {
        match self {
            Self::Applying => "Applying",
            Self::Active => "Active",
            Self::Expiring => "Expiring",
            Self::Removed => "Removed",
        }
    }

    /// 是否处于活跃阶段（Applying 或 Active）。
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Applying | Self::Active)
    }

    /// 是否可 Tick（仅 Active 阶段可 Tick）。
    pub fn can_tick(&self) -> bool {
        matches!(self, Self::Active)
    }
}

/// 效果持续时间类型。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
pub enum EffectDuration {
    /// 瞬时效果（立即执行，无持续阶段）
    Instant,
    /// 有限持续时间（回合数）
    HasDuration {
        /// 持续回合数
        turns: u32,
        /// 持续时间计算方式
        calculation: DurationCalculation,
    },
    /// 无限期（需要显式移除，如光环、永久性 Buff）
    Infinite,
}

impl EffectDuration {
    /// 返回持续时间类型名称。
    pub fn name(&self) -> &str {
        match self {
            Self::Instant => "Instant",
            Self::HasDuration { .. } => "HasDuration",
            Self::Infinite => "Infinite",
        }
    }

    /// 是否为瞬时效果。
    pub fn is_instant(&self) -> bool {
        matches!(self, Self::Instant)
    }

    /// 是否为持续效果（HasDuration 或 Infinite）。
    pub fn is_persistent(&self) -> bool {
        !matches!(self, Self::Instant)
    }

    /// 获取初始剩余回合数。
    ///
    /// Instant → 0, HasDuration → turns, Infinite → i64::MAX。
    pub fn initial_remaining_turns(&self) -> i64 {
        match self {
            Self::Instant => 0,
            Self::HasDuration { turns, .. } => *turns as i64,
            Self::Infinite => i64::MAX,
        }
    }
}

/// 持续时间计算方式。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
pub enum DurationCalculation {
    /// 固定值
    Fixed,
    /// 基于等级（基础 + 每级增量）
    PerLevel {
        /// 基础值（等级 1）
        base: u32,
        /// 每级增量
        per_level: u32,
    },
    /// 基于属性（属性值 × 系数）
    AttributeBased {
        /// 属性标识
        attribute_id: String,
        /// 系数
        multiplier: f32,
    },
}

impl DurationCalculation {
    /// 计算指定等级/属性值下的持续回合数。
    ///
    /// PerLevel: 等级从 1 开始索引。
    /// AttributeBased: 使用传入的 attribute_value。
    pub fn calculate(&self, level: u32, attribute_value: f32) -> u32 {
        match self {
            Self::Fixed => 0, // Fixed 本身没有值，turns 在 EffectDuration 中指定
            Self::PerLevel { base, per_level } => {
                let levels_above_base = level.saturating_sub(1);
                base + per_level * levels_above_base
            }
            Self::AttributeBased {
                attribute_id: _,
                multiplier,
            } => {
                let result = attribute_value * multiplier;
                if result < 0.0 { 0 } else { result as u32 }
            }
        }
    }
}

/// 效果周期 Tick 定义。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
pub struct EffectPeriod {
    /// 间隔回合数（V6: ≥ 1）
    pub interval_turns: u32,
    /// 最大 Tick 次数（None = 不限制）
    pub max_ticks: Option<u32>,
}

impl EffectPeriod {
    /// 创建新的周期定义。
    ///
    /// # Errors
    /// - V6: interval_turns ≥ 1, max_ticks ≥ 1
    pub fn new(interval_turns: u32) -> Result<Self, EffectError> {
        if interval_turns < 1 {
            return Err(EffectError::InvalidPeriod(
                "interval_turns must be ≥ 1".into(),
            ));
        }
        Ok(Self {
            interval_turns,
            max_ticks: None,
        })
    }

    /// 设置最大 Tick 次数。
    pub fn with_max_ticks(mut self, max: u32) -> Result<Self, EffectError> {
        if max < 1 {
            return Err(EffectError::InvalidPeriod("max_ticks must be ≥ 1".into()));
        }
        self.max_ticks = Some(max);
        Ok(self)
    }

    /// 检查是否还有更多 Tick 剩余。
    pub fn has_more_ticks(&self, current_tick_count: u32) -> bool {
        match self.max_ticks {
            Some(max) => current_tick_count < max,
            None => true,
        }
    }
}

/// 效果移除原因。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RemovalReason {
    /// 持续时间耗尽
    Expired,
    /// 被驱散
    Dispelled,
    /// 手动移除
    Manual,
    /// 来源死亡
    SourceDied,
    /// 强制解除
    Forced,
    /// 效果被替代（新效果取代旧效果）
    Replaced,
}

impl RemovalReason {
    /// 返回原因名称。
    pub fn name(&self) -> &str {
        match self {
            Self::Expired => "Expired",
            Self::Dispelled => "Dispelled",
            Self::Manual => "Manual",
            Self::SourceDied => "SourceDied",
            Self::Forced => "Forced",
            Self::Replaced => "Replaced",
        }
    }
}

/// Effect 领域错误。
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum EffectError {
    /// 来源缺失（不变量 3.1）
    #[error("missing source: {0}")]
    MissingSource(String),
    /// 目标缺失
    #[error("missing target: {0}")]
    MissingTarget(String),
    /// 目标已有同源效果（不变量 3.5）
    #[error("duplicate effect '{def_id}': {detail}")]
    DuplicateEffect { def_id: String, detail: String },
    /// 免疫阻止（不变量 3.2）
    #[error("effect '{def_id}' blocked by immunity '{immune_tag}'")]
    ImmunityBlocked { def_id: String, immune_tag: String },
    /// 条件不满足（不变量 3.2）
    #[error("condition not met: {0}")]
    ConditionNotMet(String),
    /// 效果未找到
    #[error("effect '{0}' not found")]
    EffectNotFound(String),
    /// 周期参数非法（V6）
    #[error("invalid period: {0}")]
    InvalidPeriod(String),
    /// 阶段转换非法
    #[error("invalid transition {from:?} → {to:?}: {detail}")]
    InvalidStageTransition {
        from: EffectStage,
        to: EffectStage,
        detail: String,
    },
    /// 效果槽位已满
    #[error("effect slot limit reached ({current} / {max})")]
    SlotLimitReached { current: u32, max: u32 },
    /// 通用运行时错误
    #[error("runtime error: {0}")]
    Runtime(String),
}
