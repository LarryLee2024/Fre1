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
    /// 序列化/日志友好形式。Custom 变体返回固定字面量 "Custom"，不包含内部字符串。
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum EventPriority {
    /// 最高优先级（最先分发）
    Highest,
    /// 高优先级
    High,
    /// 正常优先级（默认）
    #[default]
    Normal,
    /// 低优先级
    Low,
    /// 最低优先级（最后分发）
    Lowest,
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
    /// 最小化载荷构造，适用于不需要数值/目标/标签的纯通知事件。
    pub fn from_source(source_entity: impl Into<String>) -> Self {
        Self {
            source_entity: source_entity.into(),
            ..Default::default()
        }
    }

    /// values 用于传递数字型事件数据（如伤害量、治疗量），attribute_id 作为 key。
    pub fn with_value(mut self, key: impl Into<String>, value: f32) -> Self {
        self.values.insert(key.into(), value);
        self
    }

    /// 对于无目标事件（AOE、全局效果），target_entity 保持为 None。
    pub fn with_target(mut self, entity: impl Into<String>) -> Self {
        self.target_entity = Some(entity.into());
        self
    }

    /// 用于传递领域特定的非数值信息（如能力名称、地块类型等）。
    pub fn with_data(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.custom_data.insert(key.into(), value.into());
        self
    }

    /// 标签用于订阅匹配：分配器检查订阅者注册的 EventTag 与标签列表的交集。
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
