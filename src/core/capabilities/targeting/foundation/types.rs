//! Targeting 基础类型与枚举
//!
//! 定义目标类别、范围形状、优先级规则。
//!
//! 详见 docs/02-domain/capabilities/targeting_domain.md §1、§2。
//! 详见 docs/04-data/capabilities/targeting_schema.md §3。

use serde::{Deserialize, Serialize};

use super::error::TargetingError;

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
    /// 返回的字符串与 Rust 枚举变体名一致，用于序列化和日志展示。
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
///
/// 与 TargetType 组合使用，共同构成完整的目标选择规则。
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
    /// 返回形状类别名称，忽略变体内部参数。
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

    /// 判断是否为单体形状（Single）。
    ///
    /// 用于校验 Single 形状时 max_targets 必须为 1。
    pub fn is_single(&self) -> bool {
        matches!(self, Self::Single)
    }

    /// 返回该形状的最小最大目标数下限。
    ///
    /// Single 固定为 1，Chain 按 bounces 计算，其余默认为 1。
    pub fn min_max_targets(&self) -> u32 {
        match self {
            Self::Single => 1,
            Self::Chain { bounces, .. } => *bounces,
            _ => 1,
        }
    }

    /// 校验形状内部参数是否合法。
    ///
    /// 不变量 V1: 形状参数必须有效（半径>0、长度>0、角度在(0,360]等）。
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
    /// 返回优先级规则名称，用于 UI 显示和日志。
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
