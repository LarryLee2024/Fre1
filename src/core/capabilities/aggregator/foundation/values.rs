//! Aggregator 值对象定义

use std::collections::HashMap;

use crate::core::capabilities::aggregator::foundation::types::*;

/// 聚合管线使用的精简修改器条目。
///
/// 仅包含管线计算所需的字段，完整 ModifierData 在 lifecycle 层做转换。
#[derive(Debug, Clone)]
pub struct ModifierEntry {
    /// 运算类型
    pub op: ModifierOp,
    /// 数值
    pub magnitude: f32,
    /// 执行优先级（越小越优先）
    pub priority: u8,
    /// 目标属性 ID
    pub target_attribute: String,
}

/// 计算管线配置。
///
/// 绝大多数属性使用默认管线，特殊属性可覆盖。
#[derive(Debug, Clone)]
pub struct CalcPipeline {
    /// 目标属性 ID
    pub attribute_id: String,
    /// 启用哪些阶段（默认全启用）
    pub enabled_stages: Vec<CalcStage>,
    /// true: 优先级数值越小越先执行
    pub priority_ascending: bool,
    /// Clamp 边界覆盖
    pub clamp_override: Option<(f32, f32)>,
    /// 是否启用循环检测
    pub cycle_detection: bool,
}

/// 默认管线常量。
///
/// 适用于所有标准属性：四阶段全开，优先级升序，无 Clamp 覆盖，启用循环检测。
pub const DEFAULT_PIPELINE: CalcPipeline = CalcPipeline {
    attribute_id: String::new(),
    enabled_stages: Vec::new(), // filled at runtime by default_stages()
    priority_ascending: true,
    clamp_override: None,
    cycle_detection: true,
};

/// 返回默认启用阶段列表。
pub fn default_stages() -> Vec<CalcStage> {
    vec![
        CalcStage::Add,
        CalcStage::Multiply,
        CalcStage::Override,
        CalcStage::Clamp,
    ]
}

/// 单次聚合计算的完整结果。
///
/// 运行时中间产物，不持久化。
#[derive(Debug, Clone)]
pub struct AggregationResult {
    /// 计算发生的帧号
    pub frame: u64,
    /// 目标属性 ID
    pub attribute_id: String,
    /// 各阶段的中间值（用于调试和审计）
    pub stage_values: HashMap<CalcStage, f32>,
    /// 参与计算的 Modifier 数量
    pub participating_count: usize,
    /// 是否被 Override 抑制
    pub was_overridden: bool,
    /// 最终值
    pub final_value: f32,
    /// 原始 BaseValue
    pub base_value: f32,
}

impl AggregationResult {
    /// 创建仅含基础信息的聚合结果（用于快速构造）。
    pub fn new(attribute_id: String, base_value: f32, final_value: f32, frame: u64) -> Self {
        Self {
            frame,
            attribute_id,
            stage_values: HashMap::new(),
            participating_count: 0,
            was_overridden: false,
            final_value,
            base_value,
        }
    }
}
