use bevy::prelude::Reflect;

use crate::core::capabilities::modifier::foundation::types::*;

/// 修改器的来源信息。
///
/// 用于追溯 Modifier 由谁施加，支持 UI 提示和平衡分析。
#[derive(Debug, Clone, Reflect)]
pub struct ModifierSource {
    /// 来源大类（Buff、Equipment 等）
    pub source_type: ModifierSourceType,
    /// 来源唯一标识（BuffDef ID、装备 ID 等），不允许为空
    pub source_id: String,
}

/// 修改器运行时实例。
///
/// 由 create_modifier() 创建，经 validate_modifier_data() 校验后方可使用。
/// 不变量：
/// - priority ∈ [0, 100]
/// - source.source_id 非空
/// - target_attribute 非空
#[derive(Debug, Clone, Reflect)]
pub struct ModifierData {
    /// 实例唯一 ID，由 ModifierIdGenerator 分配
    #[reflect(ignore)]
    pub id: ModifierInstanceId,
    /// 运算类型（Add / Multiply / Override）
    pub op: ModifierOp,
    /// 作用的目标属性字符串标识
    pub target_attribute: String,
    /// 运算数值（符号由 op 决定含义）
    pub magnitude: f32,
    /// 执行优先级（越小越优先，Override 取最高优先级）
    pub priority: ModifierPriority,
    /// 来源信息（用于追溯和 UI 提示）
    pub source: ModifierSource,
    /// 持续帧数（None 表示永久生效）
    pub duration_frames: Option<u64>,
    /// 已经过的帧数（用于 Duration 到期自动移除）
    pub elapsed_frames: u64,
}
