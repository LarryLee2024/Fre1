//! Targeting 值对象
//!
//! 目标选择定义、选择结果、实体目标条目、选择上下文。
//!
//! 详见 docs/04-data/capabilities/targeting_schema.md §3。

use bevy::asset::Asset;
use bevy::reflect::TypePath;
use serde::{Deserialize, Serialize};

use crate::core::capabilities::targeting::foundation::error::TargetingError;
use crate::core::capabilities::targeting::foundation::types::{
    PriorityRule, TargetShape, TargetType,
};

/// 目标选择定义——技能/效果的目标选择完整配置。
///
/// 通常嵌入在 AbilityDef 中。
#[derive(Debug, Clone, PartialEq, Asset, Serialize, Deserialize, TypePath)]
pub struct TargetingDef {
    /// 目标类别
    pub target_type: TargetType,
    /// 范围形状
    pub shape: TargetShape,
    /// 最大射程（网格单位，None = 无限制）
    pub range: Option<f32>,
    /// 最小射程（None = 无限制）
    pub min_range: Option<f32>,
    /// 最大目标数
    pub max_targets: u32,
    /// 是否允许选择施法者自身
    pub include_self: bool,
    /// 是否需要视野
    pub require_los: bool,
    /// 是否忽略障碍物
    pub ignore_obstacles: bool,
    /// 能否选择已死亡实体
    pub allow_dead_targets: bool,
    /// 优先级排序规则（自动选择时）
    pub priority_rule: Option<PriorityRule>,
}

impl TargetingDef {
    /// 创建新的 TargetingDef，执行基础参数校验。
    ///
    /// 不变量：V1 形状参数合法、V2 max_targets ≥ 1、V4 Single 形状时 max_targets = 1。
    /// # Errors
    /// - 形状参数非法、max_targets < 1、或 Single 但 max_targets ≠ 1
    pub fn new(
        target_type: TargetType,
        shape: TargetShape,
        range: Option<f32>,
        max_targets: u32,
    ) -> Result<Self, TargetingError> {
        // V1: 形状参数合法
        shape.validate()?;

        // V2: 最大目标数 ≥ 1
        if max_targets < 1 {
            return Err(TargetingError::InvalidMaxTargets { max: max_targets });
        }

        // V4: Single 形状时 max_targets 必须为 1
        if shape.is_single() && max_targets != 1 {
            return Err(TargetingError::InvalidShapeParameter {
                shape: shape.name().into(),
                param: "max_targets",
                detail: format!("Single shape requires max_targets = 1, got {}", max_targets),
            });
        }

        Ok(Self {
            target_type,
            shape,
            range,
            min_range: None,
            max_targets,
            include_self: false,
            require_los: true,
            ignore_obstacles: false,
            allow_dead_targets: false,
            priority_rule: None,
        })
    }

    /// 设置最小射程，同时校验 V3: min_range ≤ range。
    ///
    /// # Errors
    /// - min_range > range 时返回 InvalidRange
    pub fn with_min_range(mut self, min: f32) -> Result<Self, TargetingError> {
        if let Some(max) = self.range
            && min > max
        {
            return Err(TargetingError::InvalidRange {
                min: Some(min),
                max: self.range,
                detail: format!("min_range {} > range {}", min, max),
            });
        }
        self.min_range = Some(min);
        Ok(self)
    }

    /// 设置是否允许选择施法者自身。
    pub fn with_include_self(mut self, value: bool) -> Self {
        self.include_self = value;
        self
    }

    /// 设置是否需要视线（LoS）校验。
    pub fn with_require_los(mut self, value: bool) -> Self {
        self.require_los = value;
        self
    }

    /// 设置是否忽略障碍物阻挡。
    pub fn with_ignore_obstacles(mut self, value: bool) -> Self {
        self.ignore_obstacles = value;
        self
    }

    /// 设置是否允许选择已死亡实体。
    pub fn with_allow_dead_targets(mut self, value: bool) -> Self {
        self.allow_dead_targets = value;
        self
    }

    /// 设置自动选择时的优先级排序规则。
    pub fn with_priority_rule(mut self, rule: PriorityRule) -> Self {
        self.priority_rule = Some(rule);
        self
    }
}

/// 单个实体目标条目。
#[derive(Debug, Clone, PartialEq)]
pub struct EntityTarget {
    /// 目标实体标识
    pub entity_id: String,
    /// 目标在范围内的网格位置（如 "5,3"）
    pub position: String,
    /// 目标与施法者之间的距离
    pub distance: f32,
    /// 选择优先级顺序
    pub selection_order: u32,
}

impl EntityTarget {
    /// 创建新的目标条目，entity_id 必填，其余字段为默认值。
    pub fn new(entity_id: impl Into<String>) -> Self {
        Self {
            entity_id: entity_id.into(),
            position: String::new(),
            distance: 0.0,
            selection_order: 0,
        }
    }

    /// 设置网格坐标位置，由 TargetResolver 在计算后填充。格式为 "x,y"。
    pub fn with_position(mut self, pos: impl Into<String>) -> Self {
        self.position = pos.into();
        self
    }

    /// 设置与施法者的距离，由 TargetResolver 在排序环节计算。
    pub fn with_distance(mut self, distance: f32) -> Self {
        self.distance = distance;
        self
    }
}

/// 目标选择上下文——选择时的环境数据。
///
/// 包含施法者身份信息和帧号，用于阵营判定和 Replay 确定性。
#[derive(Debug, Clone, PartialEq)]
pub struct TargetContext {
    /// 施法者实体标识
    pub caster_entity: String,
    /// 施法者位置
    pub caster_position: String,
    /// 施法者阵营标识
    pub caster_faction: String,
    /// 施法时帧号
    pub frame: u64,
}

impl TargetContext {
    /// 创建选择上下文，caster_entity 和 caster_faction 必填。
    pub fn new(
        caster_entity: impl Into<String>,
        caster_faction: impl Into<String>,
        frame: u64,
    ) -> Self {
        Self {
            caster_entity: caster_entity.into(),
            caster_position: String::new(),
            caster_faction: caster_faction.into(),
            frame,
        }
    }

    /// 设置施法者网格坐标，用于射程和范围计算。
    pub fn with_caster_position(mut self, pos: impl Into<String>) -> Self {
        self.caster_position = pos.into();
        self
    }
}

/// 目标选择结果数据。
///
/// 包含选中的实体列表、位置列表和上下文信息。
#[derive(Debug, Clone, PartialEq)]
pub struct TargetData {
    /// 选中的实体列表
    pub entities: Vec<EntityTarget>,
    /// 选中的位置列表（用于区域技能的位置标记）
    pub positions: Vec<String>,
    /// 选择时的上下文
    pub context: TargetContext,
    /// 是否有合法目标
    pub has_valid_targets: bool,
}

impl TargetData {
    /// 创建空的目标数据，has_valid_targets 为 false。
    ///
    /// 用于技能无合法目标时的占位返回。
    pub fn empty(context: TargetContext) -> Self {
        Self {
            entities: Vec::new(),
            positions: Vec::new(),
            context,
            has_valid_targets: false,
        }
    }

    /// 创建包含目标的结果数据。
    ///
    /// entities 非空时 has_valid_targets 自动设为 true。
    /// entities 为空时仍可包含 positions（用于区域技能无实体目标但有位置标记的场景）。
    pub fn with_targets(
        entities: Vec<EntityTarget>,
        positions: Vec<String>,
        context: TargetContext,
    ) -> Self {
        let has_valid = !entities.is_empty();
        Self {
            entities,
            positions,
            context,
            has_valid_targets: has_valid,
        }
    }

    /// 返回首个目标的实体 ID。
    ///
    /// entities 为空时返回 None。用于 UI 预览和高亮显示。
    pub fn first_target(&self) -> Option<&str> {
        self.entities.first().map(|e| e.entity_id.as_str())
    }

    /// 返回选中的目标数量。
    ///
    /// 用于校验 max_targets 约束。注意 entities 可能包含已被过滤的实体。
    pub fn target_count(&self) -> usize {
        self.entities.len()
    }

    /// 检查目标列表中是否包含指定实体。
    ///
    /// 用于链式弹射中排除已选目标，防止重复选择。
    pub fn contains_entity(&self, entity_id: &str) -> bool {
        self.entities.iter().any(|e| e.entity_id == entity_id)
    }
}

/// 目标校验结果。
///
/// 校验器通过此类型返回 Pass/Fail，失败时必须附带人类可读的原因。
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationResult {
    /// 校验通过
    Pass,
    /// 校验不通过，附带失败原因
    Fail(String),
}

impl ValidationResult {
    /// 与 matches!(self, ValidationResult::Pass) 等价，提供调用方更可读的语法。
    pub fn is_pass(&self) -> bool {
        matches!(self, Self::Pass)
    }

    /// 无参快捷构造，用于目标校验成功时返回。
    pub fn pass() -> Self {
        Self::Pass
    }

    /// 失败时必须附带人类可读的原因，由校验器在发现不匹配时提供（如 "entity not in range"）。
    pub fn fail(reason: impl Into<String>) -> Self {
        Self::Fail(reason.into())
    }
}
