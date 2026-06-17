//! Execution 模块类型定义
//!
//! 定义 ExecutionContext、ExecutionResult 等核心数据结构

use bevy::prelude::*;
use std::collections::HashMap;

/// 属性快照 — Execution 计算所需的属性值
#[derive(Clone, Debug, Default)]
pub struct AttributeSnapshot {
    pub attack: f32,
    pub defense: f32,
    pub magic_attack: f32,
    pub magic_defense: f32,
    pub max_hp: f32,
    pub crit_rate: f32,
    pub accuracy: f32,
}

/// 执行上下文 — Execution 计算所需的全部输入参数
#[derive(Clone, Debug)]
pub struct ExecutionContext {
    /// 攻击者 Entity ID
    pub source_entity: Entity,
    /// 目标 Entity ID
    pub target_entity: Entity,
    /// 攻击者属性快照
    pub source_attrs: AttributeSnapshot,
    /// 目标属性快照
    pub target_attrs: AttributeSnapshot,
    /// EffectDef 中声明的基础值（如伤害倍率 multiplier）
    pub base_value: f32,
    /// Modifier 阶段修饰后的附加值
    pub modifier_value: i32,
    /// 当前堆叠层数（来自 Stacking 阶段）
    pub stack_count: u32,
    /// Executor 专用参数（如 ignore_def_percent、crit_multiplier）
    pub execution_params: HashMap<String, f32>,
    /// 地形 ID（用于地形伤害计算）
    pub terrain_id: Option<String>,
    /// 是否为技能攻击
    pub is_skill: bool,
}

/// 计算步骤记录 — 用于 Debug 面板和回放审计
#[derive(Clone, Debug, Reflect)]
pub struct StepRecord {
    /// 步骤名称
    pub name: String,
    /// 输入值
    pub input: f32,
    /// 输出值
    pub output: f32,
}

/// 执行结果 — Execution 计算完成后的输出
#[derive(Clone, Debug, Reflect)]
pub struct ExecutionResult {
    /// 计算结果值（如最终伤害、最终治疗量）
    pub value: i32,
    /// 计算过程记录（用于 Debug 面板和回放审计）
    pub breakdown: Vec<StepRecord>,
    /// 是否暴击
    pub is_critical: bool,
}

impl Default for ExecutionResult {
    fn default() -> Self {
        Self {
            value: 0,
            breakdown: Vec::new(),
            is_critical: false,
        }
    }
}
