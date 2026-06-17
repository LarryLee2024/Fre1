//! Event 基础类型与枚举

use std::collections::HashMap;

/// 事件标签——事件类型的标识，用于路由和订阅匹配。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EventTag {
    /// 造成伤害
    DamageDealt,
    /// 受到伤害
    DamageTaken,
    /// 受到治疗
    Healed,
    /// 单位死亡
    UnitDied,
    /// 单位生成
    UnitSpawned,
    /// 回合开始
    TurnStarted,
    /// 回合结束
    TurnEnded,
    /// 移动开始
    MoveStarted,
    /// 移动结束
    MoveEnded,
    /// 技能被使用
    AbilityUsed,
    /// Buff 被应用
    BuffApplied,
    /// Buff 被移除
    BuffRemoved,
    /// 条件满足
    ConditionMet,
    /// 自定义 Domain 事件
    Custom(String),
}

impl EventTag {
    /// 返回人类可读的标签名。
    pub fn name(&self) -> &str {
        match self {
            Self::DamageDealt => "DamageDealt",
            Self::DamageTaken => "DamageTaken",
            Self::Healed => "Healed",
            Self::UnitDied => "UnitDied",
            Self::UnitSpawned => "UnitSpawned",
            Self::TurnStarted => "TurnStarted",
            Self::TurnEnded => "TurnEnded",
            Self::MoveStarted => "MoveStarted",
            Self::MoveEnded => "MoveEnded",
            Self::AbilityUsed => "AbilityUsed",
            Self::BuffApplied => "BuffApplied",
            Self::BuffRemoved => "BuffRemoved",
            Self::ConditionMet => "ConditionMet",
            Self::Custom(_) => "Custom",
        }
    }
}

/// 事件分发优先级。
///
/// 高优先级事件先于低优先级事件分发（不变量 §3.3）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventPriority {
    /// 最高优先级（最先分发）
    Highest,
    /// 高优先级
    High,
    /// 正常优先级（默认）
    Normal,
    /// 低优先级
    Low,
    /// 最低优先级（最后分发）
    Lowest,
}

impl Default for EventPriority {
    fn default() -> Self {
        Self::Normal
    }
}

/// 事件循环保护限制。
///
/// 同一事件类型在单次处理链中最多重复触发的次数（不变量 §3.4）。
pub const EVENT_CYCLE_LIMIT: u32 = 5;

/// 事件投递状态。
#[derive(Debug, Clone)]
pub enum DeliveryStatus {
    /// 成功投递
    Delivered,
    /// 投递失败（附带原因）
    Failed(String),
}

/// 事件载荷——事件的上下文数据。
///
/// 使用字符串键值对保持跨领域兼容，避免 tight coupling。
#[derive(Debug, Clone, Default)]
pub struct EventPayload {
    /// 来源实体标识
    pub source_entity: String,
    /// 目标实体标识（可选）
    pub target_entity: Option<String>,
    /// 关联的 GameplayContext ID（可选）
    pub context_id: Option<String>,
    /// 数值载荷（attribute_id → value）
    pub values: HashMap<String, f32>,
    /// 关联标签列表
    pub tags: Vec<String>,
    /// 自定义数据（key → value）
    pub custom_data: HashMap<String, String>,
}

impl EventPayload {
    /// 创建仅含来源实体的载荷。
    pub fn from_source(source_entity: impl Into<String>) -> Self {
        Self {
            source_entity: source_entity.into(),
            ..Default::default()
        }
    }

    /// 添加数值。
    pub fn with_value(mut self, key: impl Into<String>, value: f32) -> Self {
        self.values.insert(key.into(), value);
        self
    }

    /// 添加目标实体。
    pub fn with_target(mut self, entity: impl Into<String>) -> Self {
        self.target_entity = Some(entity.into());
        self
    }

    /// 添加自定义数据。
    pub fn with_data(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.custom_data.insert(key.into(), value.into());
        self
    }

    /// 添加标签。
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }
}

/// 核心事件结构——系统间传递数据的统一载体。
#[derive(Debug, Clone)]
pub struct GameplayEvent {
    /// 事件唯一标识
    pub id: String,
    /// 事件类型标签
    pub tag: EventTag,
    /// 来源 Domain/System 标识
    pub source: String,
    /// 分发优先级
    pub priority: EventPriority,
    /// 载荷数据
    pub payload: EventPayload,
}
