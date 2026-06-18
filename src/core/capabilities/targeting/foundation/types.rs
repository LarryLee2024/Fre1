//! Targeting 基础类型与枚举
//!
//! 定义目标类别、范围形状、优先级规则以及领域错误。
//!
//! 详见 docs/02-domain/targeting_domain.md §1、§2。
//! 详见 docs/04-data/capabilities/targeting_schema.md §3。

use serde::{Deserialize, Serialize};

/// 目标类别枚举，定义技能可以选择何种目标。
///
/// 与 TargetShape 组合使用构成完整的目标选择规则。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TargetType {
    /// 自身（施法者）
    Self_,
    /// 友方（同阵营非自身）
    Ally,
    /// 敌方（对立阵营）
    Enemy,
    /// 已死亡的实体
    Dead,
    /// 中立实体
    Neutral,
    /// 所有实体（无差别）
    Any,
    /// 召唤物
    Summon,
    /// 小队全体
    Party,
}

impl TargetType {
    /// 返回人类可读的名称。
    pub fn name(&self) -> &str {
        match self {
            Self::Self_ => "Self",
            Self::Ally => "Ally",
            Self::Enemy => "Enemy",
            Self::Dead => "Dead",
            Self::Neutral => "Neutral",
            Self::Any => "Any",
            Self::Summon => "Summon",
            Self::Party => "Party",
        }
    }
}

/// 目标范围形状枚举，定义技能的影响区域形状。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TargetShape {
    /// 单体（单一目标）
    Single,
    /// 圆形区域（半径）
    Area {
        /// 半径（网格单位）
        radius: f32,
    },
    /// 直线（长度、宽度）
    Line {
        /// 长度
        length: f32,
        /// 宽度
        width: f32,
    },
    /// 锥形（角度、长度）
    Cone {
        /// 锥形长度
        length: f32,
        /// 张开角度（度）
        angle: f32,
    },
    /// 链式弹射（跳数、每跳最大距离）
    Chain {
        /// 总弹射次数（含首次目标）
        bounces: u32,
        /// 每跳最大距离
        bounce_range: f32,
        /// 是否可重复弹射同一目标
        allow_retarget: bool,
    },
    /// 爆炸/迸发（以目标格为中心的二次范围）
    Burst {
        /// 中心半径
        center_radius: f32,
        /// 扩散半径
        burst_radius: f32,
    },
    /// 墙体/连线（起点到终点的所有格子）
    Wall {
        /// 墙体长度
        length: f32,
        /// 墙体宽度
        width: f32,
    },
}

impl TargetShape {
    /// 返回形状名称。
    pub fn name(&self) -> &str {
        match self {
            Self::Single => "Single",
            Self::Area { .. } => "Area",
            Self::Line { .. } => "Line",
            Self::Cone { .. } => "Cone",
            Self::Chain { .. } => "Chain",
            Self::Burst { .. } => "Burst",
            Self::Wall { .. } => "Wall",
        }
    }

    /// 是否为单体形状（Single）。
    pub fn is_single(&self) -> bool {
        matches!(self, Self::Single)
    }

    /// 获取形状的默认最大目标数下限（Single=1，其余按定义）。
    pub fn min_max_targets(&self) -> u32 {
        match self {
            Self::Single => 1,
            Self::Chain { bounces, .. } => *bounces,
            _ => 1,
        }
    }

    /// 校验形状参数是否合法（V1: 形状参数合法）。
    pub fn validate(&self) -> Result<(), TargetingError> {
        match self {
            Self::Area { radius } => {
                if *radius <= 0.0 {
                    return Err(TargetingError::InvalidShapeParameter {
                        shape: self.name().into(),
                        param: "radius",
                        detail: "radius must be > 0".into(),
                    });
                }
            }
            Self::Line { length, width } => {
                if *length <= 0.0 {
                    return Err(TargetingError::InvalidShapeParameter {
                        shape: self.name().into(),
                        param: "length",
                        detail: "length must be > 0".into(),
                    });
                }
                if *width <= 0.0 {
                    return Err(TargetingError::InvalidShapeParameter {
                        shape: self.name().into(),
                        param: "width",
                        detail: "width must be > 0".into(),
                    });
                }
            }
            Self::Cone { length, angle } => {
                if *length <= 0.0 {
                    return Err(TargetingError::InvalidShapeParameter {
                        shape: self.name().into(),
                        param: "length",
                        detail: "length must be > 0".into(),
                    });
                }
                if *angle <= 0.0 || *angle > 360.0 {
                    return Err(TargetingError::InvalidShapeParameter {
                        shape: self.name().into(),
                        param: "angle",
                        detail: "angle must be in (0, 360]".into(),
                    });
                }
            }
            Self::Chain { bounces, .. } => {
                if *bounces < 1 {
                    return Err(TargetingError::InvalidShapeParameter {
                        shape: self.name().into(),
                        param: "bounces",
                        detail: "bounces must be ≥ 1".into(),
                    });
                }
            }
            Self::Burst {
                center_radius,
                burst_radius,
            } => {
                if *center_radius <= 0.0 {
                    return Err(TargetingError::InvalidShapeParameter {
                        shape: self.name().into(),
                        param: "center_radius",
                        detail: "center_radius must be > 0".into(),
                    });
                }
                if *burst_radius <= 0.0 {
                    return Err(TargetingError::InvalidShapeParameter {
                        shape: self.name().into(),
                        param: "burst_radius",
                        detail: "burst_radius must be > 0".into(),
                    });
                }
            }
            Self::Wall { length, width } => {
                if *length <= 0.0 {
                    return Err(TargetingError::InvalidShapeParameter {
                        shape: self.name().into(),
                        param: "length",
                        detail: "length must be > 0".into(),
                    });
                }
                if *width <= 0.0 {
                    return Err(TargetingError::InvalidShapeParameter {
                        shape: self.name().into(),
                        param: "width",
                        detail: "width must be > 0".into(),
                    });
                }
            }
            Self::Single => {}
        }
        Ok(())
    }
}

/// 优先级排序规则（多个可选目标时的自动选择）。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PriorityRule {
    /// 最近优先
    Nearest,
    /// 最远优先
    Farthest,
    /// 血量最低优先
    LowestHealth,
    /// 血量最高优先
    HighestHealth,
    /// 随机（基于确定性 RNG）
    Random,
}

impl PriorityRule {
    /// 返回规则名称。
    pub fn name(&self) -> &str {
        match self {
            Self::Nearest => "Nearest",
            Self::Farthest => "Farthest",
            Self::LowestHealth => "LowestHealth",
            Self::HighestHealth => "HighestHealth",
            Self::Random => "Random",
        }
    }
}

/// Targeting 领域错误。
#[derive(Debug, Clone, PartialEq)]
pub enum TargetingError {
    /// 形状参数不合法
    InvalidShapeParameter {
        shape: String,
        param: &'static str,
        detail: String,
    },
    /// 最大目标数不合法（V2: max_targets ≥ 1）
    InvalidMaxTargets(u32),
    /// 射程不合法（V3: min_range ≤ range）
    InvalidRange {
        min: Option<f32>,
        max: Option<f32>,
        detail: String,
    },
    /// 没有合法目标
    NoValidTargets { reason: String },
    /// 目标实体不存在
    EntityNotFound(String),
    /// 超出射程
    OutOfRange { distance: f32, max_range: f32 },
    /// 阵营不匹配
    FactionMismatch {
        expected: TargetType,
        actual: String,
    },
    /// 目标数量已达上限
    TargetLimitReached { limit: u32 },
    /// 视野检查失败
    LineOfSightBlocked,
    /// 目标选择被重复调用
    AlreadySelecting,
    /// 通用运行时错误
    Runtime(String),
}

impl std::fmt::Display for TargetingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidShapeParameter {
                shape,
                param,
                detail,
            } => {
                write!(
                    f,
                    "invalid parameter '{}' for shape '{}': {}",
                    param, shape, detail
                )
            }
            Self::InvalidMaxTargets(n) => {
                write!(f, "invalid max_targets: {} (must be ≥ 1)", n)
            }
            Self::InvalidRange { min, max, detail } => {
                write!(
                    f,
                    "invalid range (min={:?}, max={:?}): {}",
                    min, max, detail
                )
            }
            Self::NoValidTargets { reason } => {
                write!(f, "no valid targets: {}", reason)
            }
            Self::EntityNotFound(eid) => write!(f, "entity '{}' not found", eid),
            Self::OutOfRange {
                distance,
                max_range,
            } => {
                write!(f, "distance {} exceeds max range {}", distance, max_range)
            }
            Self::FactionMismatch { expected, actual } => {
                write!(
                    f,
                    "faction mismatch: expected {:?}, got {}",
                    expected, actual
                )
            }
            Self::TargetLimitReached { limit } => {
                write!(f, "target limit of {} reached", limit)
            }
            Self::LineOfSightBlocked => write!(f, "line of sight blocked"),
            Self::AlreadySelecting => write!(f, "target selection already in progress"),
            Self::Runtime(msg) => write!(f, "runtime error: {}", msg),
        }
    }
}

impl std::error::Error for TargetingError {}
