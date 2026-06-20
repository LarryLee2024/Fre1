//! Ability 基础类型与枚举
//!
//! 定义技能生命周期状态、分类、激活类型以及领域错误。
//!
//! 详见 docs/02-domain/capabilities/ability_domain.md §1、§2。
//! 详见 docs/04-data/capabilities/ability_schema.md §3。

use crate::shared::ids::types::runtime_id::RuntimeId;

/// 技能运行时阶段（状态机），定义技能当前所处的生命周期位置。
///
/// 状态转换图（主流程——自循环，非 DAG）：
/// ```text
///        ┌─────────────────────────────────────┐
///        │                                     │
///        ▼                                     │
///     Ready ──→ Casting ──→ Active ──→ Cooldown─┤
///       │          │                            │
///       │          ▼                            │
///       │     (取消/打断)                        │
///       └──── Ready                             │
///                                              │
///     Blocked (任何状态 ↔ Blocked ─ 由 apply/remove_block 管理)
///     Removed (任何状态 → Removed)
/// ```
///
/// 详见 docs/02-domain/capabilities/ability_domain.md §1。
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
    /// 用于日志、调试显示和 UI 状态文本。
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

    /// 只有 Ready 状态才能通过 try_activate 激活。
    /// 不变量 §3.1：Casting/Active/Cooldown 状态下禁止重复激活。
    pub fn can_activate(&self) -> bool {
        matches!(self, Self::Ready)
    }

    /// Casting 可打断回到 Ready；Active 可终止进入 Removed。
    /// Cooldown/Ready 状态不允许取消（已在冷却或无事可取消）。
    pub fn can_cancel(&self) -> bool {
        matches!(self, Self::Casting | Self::Active)
    }

    /// 只有 Active 状态的技能执行完毕后才能进入 Cooldown。
    /// Casting 被打断后回到 Ready，不经过 Cooldown。
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

/// 技能运行时实例唯一标识（基于 RuntimeId，带 generation 保护）。
///
/// 每次 ID 被回收后，再次分配时 generation 递增，防止旧引用指向新对象。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AbilityInstanceId(RuntimeId);

impl AbilityInstanceId {
    /// 由 AbilityInstanceIdGenerator 分配，确保 generation safety。
    /// 不要直接构造——使用 generator.next_id()。
    pub fn new(id: RuntimeId) -> Self {
        Self(id)
    }

    /// 仅用于反序列化或测试——generation 固定为 0。
    /// 正式路径走 new(RuntimeId) 以获得正确的 generation。
    pub fn from_u64(id: u64) -> Self {
        Self(RuntimeId::new(id as u32, 0))
    }

    /// 用于将 ID 传入需要 RuntimeId 的 API（如序列化层）。
    pub fn runtime_id(&self) -> RuntimeId {
        self.0
    }

    /// 兼容旧序列化格式的数值表示。新代码优先使用 runtime_id()。
    pub fn value(&self) -> u64 {
        self.0.index() as u64
    }

    /// 用于 ID 分配器索引计算。
    pub fn index(&self) -> u32 {
        self.0.index()
    }

    /// 用于 generation safety 校验——旧代际的引用应视为过期。
    pub fn generation(&self) -> u32 {
        self.0.generation()
    }

    /// Generation safety：如果 other 是同一槽位的旧代际，说明该引用已过期。
    /// 调用方应丢弃旧引用并从容器重新查询。
    pub fn is_stale(&self, other: &AbilityInstanceId) -> bool {
        self.0.is_stale(&other.0)
    }
}

impl std::fmt::Display for AbilityInstanceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "inst_{:010}", self.0.index())
    }
}

impl From<u64> for AbilityInstanceId {
    fn from(id: u64) -> Self {
        Self::from_u64(id)
    }
}

impl serde::Serialize for AbilityInstanceId {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for AbilityInstanceId {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        RuntimeId::deserialize(deserializer).map(Self)
    }
}
