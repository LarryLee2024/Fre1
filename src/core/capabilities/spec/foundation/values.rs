//! Spec 值对象定义

use std::collections::HashMap;

use crate::core::capabilities::spec::foundation::types::{EnhancementId, SpecId};

/// 消耗覆盖定义。
#[derive(Debug, Clone)]
pub struct CostOverride {
    /// 替代的消耗属性 ID
    pub resource_attribute: String,
    /// 替代的消耗量
    pub amount: f32,
}

/// 角色身上的技能配置实例。
#[derive(Debug, Clone)]
pub struct AbilitySpec {
    /// Spec 唯一标识
    pub spec_id: SpecId,
    /// 引用的 AbilityDef ID
    pub def_id: String,
    /// 技能等级（1..MaxLevel）
    pub level: u8,
    /// 最大等级（由 AbilityDef 定义）
    pub max_level: u8,
    /// 冷却缩减（回合数）
    pub cooldown_reduction: i32,
    /// 冷却覆盖
    pub cooldown_override: Option<u32>,
    /// 消耗覆盖
    pub cost_override: Option<CostOverride>,
    /// 强化/专长列表
    pub enhancements: Vec<EnhancementId>,
    /// 是否隐藏（被动技能）
    pub hidden: bool,
    /// 上次使用帧号
    pub last_used_frame: u64,
    /// 强制解除冷却
    pub forced_cooldown_reset: bool,
}

impl AbilitySpec {
    /// 使用默认参数创建新 AbilitySpec。
    pub fn new(def_id: impl Into<String>, level: u8, max_level: u8) -> Self {
        Self {
            spec_id: SpecId::new(),
            def_id: def_id.into(),
            level,
            max_level,
            cooldown_reduction: 0,
            cooldown_override: None,
            cost_override: None,
            enhancements: Vec::new(),
            hidden: false,
            last_used_frame: 0,
            forced_cooldown_reset: false,
        }
    }
}

/// 效果的来源信息。
#[derive(Debug, Clone)]
pub struct EffectSource {
    /// 来源实体（字符串 ID，因为跨领域传递）
    pub source_entity: String,
    /// 来源能力 ID
    pub source_ability: Option<String>,
    /// 来源物品 ID
    pub source_item: Option<String>,
}

/// 效果属性快照。
#[derive(Debug, Clone)]
pub struct EffectSnapshot {
    /// 快照时的施法者属性值
    pub caster_attributes: HashMap<String, f32>,
    /// 快照时的目标属性值
    pub target_attributes: HashMap<String, f32>,
    /// 快照帧号
    pub snapshot_frame: u64,
}

impl EffectSnapshot {
    /// 创建一个新的空快照。
    pub fn empty(frame: u64) -> Self {
        Self {
            caster_attributes: HashMap::new(),
            target_attributes: HashMap::new(),
            snapshot_frame: frame,
        }
    }
}

/// 角色身上的效果配置实例。
#[derive(Debug, Clone)]
pub struct EffectSpec {
    /// Spec 唯一标识
    pub spec_id: SpecId,
    /// 引用的 EffectDef ID
    pub def_id: String,
    /// 来源上下文
    pub source: EffectSource,
    /// 持续时间修正（帧数）
    pub duration_modifier: i64,
    /// 堆叠层数
    pub stack_count: u32,
    /// 属性快照
    pub snapshot: EffectSnapshot,
    /// 是否为周期性效果
    pub is_periodic: bool,
    /// 周期间隔（帧数）
    pub period_interval: Option<u64>,
    /// 是否已通过条件检查
    pub condition_passed: bool,
}

impl EffectSpec {
    /// 使用默认参数创建新 EffectSpec。
    pub fn new(def_id: impl Into<String>, source: EffectSource, frame: u64) -> Self {
        Self {
            spec_id: SpecId::new(),
            def_id: def_id.into(),
            source,
            duration_modifier: 0,
            stack_count: 1,
            snapshot: EffectSnapshot::empty(frame),
            is_periodic: false,
            period_interval: None,
            condition_passed: false,
        }
    }
}

/// Spec 注册中心配置。
#[derive(Debug, Clone)]
pub struct SpecRegistryConfig {
    /// AbilitySpec 默认等级
    pub initial_level: u8,
    /// AbilitySpec 最大等级
    pub max_level: u8,
    /// 是否允许等级覆盖
    pub allow_level_override: bool,
    /// EffectSpec 默认最大堆叠
    pub max_stack: u32,
    /// 是否启用快照
    pub enable_snapshot: bool,
}

impl Default for SpecRegistryConfig {
    fn default() -> Self {
        Self {
            initial_level: 1,
            max_level: 5,
            allow_level_override: true,
            max_stack: 1,
            enable_snapshot: true,
        }
    }
}
