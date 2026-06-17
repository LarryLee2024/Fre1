//! Stacking 基础类型与枚举
//!
//! 定义堆叠策略类型、堆叠标识判定、堆叠决策结果以及领域错误。
//!
//! 详见 docs/02-domain/stacking_domain.md §1、§3。
//! 详见 docs/04-data/capabilities/stacking_schema.md §3。

/// 堆叠类型枚举，定义效果叠加的基本策略。
///
/// 决定当同一效果的第二个实例到达时应如何处理。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StackingType {
    /// 不堆叠——新实例被忽略，保持原状态。
    /// 适用：同一 Buff 第二次施加（如"庇护术"不可叠加）。
    None,
    /// 累加层数——层数叠加，受 max_stacks 限制。
    /// 适用：中毒层数叠加（每层 +1d4 毒伤）。
    Aggregate,
    /// 刷新持续时间——重置剩余时间，层数不变。
    /// 适用：持续伤害刷新时长（重新计算到期时间）。
    RefreshDuration,
    /// 替换——新实例替换旧实例（按优先级或数值判定）。
    /// 适用：不同来源的同类 Buff 替换生效（取更强者）。
    Replace,
}

impl StackingType {
    /// 返回堆叠类型名称。
    pub fn name(&self) -> &str {
        match self {
            Self::None => "None",
            Self::Aggregate => "Aggregate",
            Self::RefreshDuration => "RefreshDuration",
            Self::Replace => "Replace",
        }
    }

    /// 是否需要追踪堆叠层数。
    pub fn tracks_layers(&self) -> bool {
        matches!(self, Self::Aggregate)
    }
}

/// 堆叠配置（Definition 层）。
///
/// 定义堆叠策略的完整参数，嵌入在 EffectDef 中。
#[derive(Debug, Clone, PartialEq)]
pub struct StackingConfig {
    /// 堆叠策略
    pub stacking_type: StackingType,
    /// 最大堆叠层数（≥ 1）
    pub max_stacks: u32,
    /// 是否允许异源堆叠（不同施法者的同效果叠加）
    pub allow_cross_source: bool,
    /// 溢出处理行为
    pub overflow_behavior: OverflowBehavior,
    /// 层数变化时是否重算 Modifier
    pub reapply_modifiers_on_stack: bool,
}

impl StackingConfig {
    /// 创建默认的不可堆叠配置。
    pub fn none() -> Self {
        Self {
            stacking_type: StackingType::None,
            max_stacks: 1,
            allow_cross_source: false,
            overflow_behavior: OverflowBehavior::IgnoreNew,
            reapply_modifiers_on_stack: false,
        }
    }

    /// 创建累加堆叠配置。
    ///
    /// # Errors
    /// - V2: Aggregate 类型 max_stacks 必须 ≥ 2
    pub fn aggregate(max_stacks: u32, allow_cross_source: bool) -> Result<Self, StackingError> {
        if max_stacks < 2 {
            return Err(StackingError::InvalidConfig(
                "Aggregate stacking requires max_stacks ≥ 2".into(),
            ));
        }
        Ok(Self {
            stacking_type: StackingType::Aggregate,
            max_stacks,
            allow_cross_source,
            overflow_behavior: OverflowBehavior::IgnoreNew,
            reapply_modifiers_on_stack: true,
        })
    }

    /// 创建刷新持续时间配置。
    pub fn refresh() -> Self {
        Self {
            stacking_type: StackingType::RefreshDuration,
            max_stacks: 1,
            allow_cross_source: true,
            overflow_behavior: OverflowBehavior::IgnoreNew,
            reapply_modifiers_on_stack: false,
        }
    }

    /// 创建替换配置。
    ///
    /// # Errors
    /// - V3: Replace 类型 max_stacks 必须 = 1
    pub fn replace() -> Result<Self, StackingError> {
        Ok(Self {
            stacking_type: StackingType::Replace,
            max_stacks: 1,
            allow_cross_source: true,
            overflow_behavior: OverflowBehavior::Replace,
            reapply_modifiers_on_stack: true,
        })
    }
}

impl Default for StackingConfig {
    fn default() -> Self {
        Self::none()
    }
}

/// 堆叠上限溢出处理行为。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OverflowBehavior {
    /// 丢弃新到达的层数，保持原状态
    IgnoreNew,
    /// 丢弃新层数但刷新持续时间
    Refresh,
    /// 移除旧实例，用新实例替换（重置为 1 层）
    Replace,
    /// 移除最早的层（队列式）
    RemoveOldest,
}

impl OverflowBehavior {
    /// 返回溢出行为名称。
    pub fn name(&self) -> &str {
        match self {
            Self::IgnoreNew => "IgnoreNew",
            Self::Refresh => "Refresh",
            Self::Replace => "Replace",
            Self::RemoveOldest => "RemoveOldest",
        }
    }
}

/// 堆叠标识——判断两个 Effect 是否属于同一堆叠的依据。
#[derive(Debug, Clone, PartialEq)]
pub struct StackIdentity {
    /// 匹配的 EffectDefId
    pub effect_def_id: String,
    /// 来源实体 ID（用于同源判定）
    pub source_entity: String,
    /// 来源能力 ID（可选，进一步细化同源判定）
    pub source_ability: Option<String>,
    /// 分组标签（可选，按 Tag 分组堆叠）
    pub group_tag: Option<String>,
}

impl StackIdentity {
    /// 从效果实例参数创建堆叠标识。
    pub fn new(effect_def_id: impl Into<String>, source_entity: impl Into<String>) -> Self {
        Self {
            effect_def_id: effect_def_id.into(),
            source_entity: source_entity.into(),
            source_ability: None,
            group_tag: None,
        }
    }

    /// 设置来源能力 ID。
    pub fn with_ability(mut self, ability_id: impl Into<String>) -> Self {
        self.source_ability = Some(ability_id.into());
        self
    }

    /// 设置分组标签。
    pub fn with_group_tag(mut self, tag: impl Into<String>) -> Self {
        self.group_tag = Some(tag.into());
        self
    }
}

/// 堆叠匹配判定结果。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StackMatchResult {
    /// 完全匹配（同 EffectDef + 同源）——进行堆叠判定
    FullMatch,
    /// 类型匹配但异源——根据 allow_cross_source 决定
    CrossSource,
    /// 分组匹配——按 group_tag 堆叠
    GroupMatch,
    /// 不匹配——各自独立，不参与堆叠
    NoMatch,
}

/// 堆叠判定结果（运行时瞬时数据）。
#[derive(Debug, Clone, PartialEq)]
pub enum StackingDecision {
    /// 拒绝——新实例不应用（None 类型或 Replace 但旧 ≥ 新）
    Reject,
    /// 累加——层数增加
    Accumulate {
        /// 新的堆叠层数
        new_stack_count: u32,
        /// 新增的层数
        added_layers: u32,
    },
    /// 刷新——重置持续时间
    Refresh {
        /// 被刷新实例的 ID
        refreshed_instance_id: String,
        /// 新的持续时间（剩余回合数）
        new_duration: i64,
    },
    /// 替换——移除旧的，应用新的
    Replace {
        /// 被替换实例的 ID
        replaced_instance_id: String,
    },
}

/// Stacking 领域错误。
#[derive(Debug, Clone, PartialEq)]
pub enum StackingError {
    /// 无效的堆叠配置（如 Aggregate 但 max_stacks < 2）
    InvalidConfig(String),
    /// 堆叠标识不匹配
    IdentityMismatch {
        existing_def_id: String,
        incoming_def_id: String,
        detail: String,
    },
    /// 运行时错误
    Runtime(String),
}

impl std::fmt::Display for StackingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidConfig(msg) => write!(f, "invalid stacking config: {}", msg),
            Self::IdentityMismatch {
                existing_def_id,
                incoming_def_id,
                detail,
            } => {
                write!(
                    f,
                    "identity mismatch: '{}' vs '{}': {}",
                    existing_def_id, incoming_def_id, detail
                )
            }
            Self::Runtime(msg) => write!(f, "stacking runtime error: {}", msg),
        }
    }
}

impl std::error::Error for StackingError {}
