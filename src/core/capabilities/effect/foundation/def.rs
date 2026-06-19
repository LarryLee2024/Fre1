//! EffectDef — 效果定义（Definition 层）
//!
//! 效果的完整静态定义，从 RON 配置文件反序列化。
//!
//! 详见 docs/04-data/capabilities/effect_schema.md §3.1。

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::core::capabilities::cue::foundation::CueTag;
use crate::core::capabilities::effect::foundation::types::{
    EffectCategory, EffectDuration, EffectPeriod,
};
use crate::core::capabilities::execution::foundation::ExecutionType;
use crate::core::capabilities::modifier::foundation::ModifierOp;
use crate::core::capabilities::stacking::foundation::StackingConfig;
use crate::shared::localization_key::LocalizationKey;

/// 效果定义（Definition 层）。
///
/// 对应 Schema §3.1 EffectDef。
/// 包含效果的全部静态参数：持续时间、周期、修改器、标签、执行计算、表现信号。
#[derive(Debug, Clone, PartialEq, Asset, Serialize, Deserialize, TypePath)]
pub struct EffectDef {
    /// 效果唯一标识（格式：eff_ + 6 位数字，如 eff_000001）
    pub id: String,

    /// 效果名称本地化 Key
    pub name_key: LocalizationKey,

    /// 效果描述本地化 Key
    pub desc_key: LocalizationKey,

    /// 效果图标 Key
    pub icon_key: Option<LocalizationKey>,

    /// 持续时间类型
    pub duration: EffectDuration,

    /// 周期 Tick（仅 Duration 类型有效）
    pub period: Option<EffectPeriod>,

    /// Tick 时触发的执行计算（可选，与 period 配合使用）
    pub tick_execution: Option<ExecutionConfig>,

    /// 效果携带的修改器列表（应用时注册到目标属性）
    pub modifiers: Vec<ModifierConfig>,

    /// 效果授予的标签（应用时添加到目标实体）
    pub granted_tags: Vec<String>,

    /// 效果需要的标签（目标必须拥有效果才能生效）
    pub required_tags: Option<Vec<String>>,

    /// 效果移除时清理的标签
    pub removed_tags: Option<Vec<String>>,

    /// 应用条件 ID（指向 ConditionRegistry）
    pub application_condition: Option<String>,

    /// 效果叠加策略
    #[serde(default)]
    pub stacking: StackingConfig,

    /// 效果类型分类
    pub effect_category: EffectCategory,

    /// 关联的执行计算（可选，Instant 类效果需要）
    pub execution: Option<ExecutionConfig>,

    /// 视觉表现信号
    #[serde(default)]
    pub cues: Vec<EffectCueBinding>,

    /// 可视性
    pub visible: bool,

    /// 是否可被驱散
    pub dispellable: bool,

    /// 优先级（用于显示排序）
    pub display_priority: u8,
}

/// 修改器配置（Definition 层）。
///
/// 嵌入在 EffectDef 中定义对目标属性的数值变更。
/// 运行时由 Modifier 机制消费。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModifierConfig {
    /// 修改器运算类型
    pub op: ModifierOp,

    /// 目标属性标识
    pub target_attribute: String,

    /// 修改器值
    pub value: ModifierValue,

    /// 执行优先级（越小越优先）
    pub priority: u8,
}

/// 修改器值类型（Definition 层）。
///
/// 定义修改器数值的来源与计算方式。
/// 运行时由 Modifier 机制将计算后的值传递给 ModifierData。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ModifierValue {
    /// 固定值
    Fixed(f32),
    /// 按等级缩放（基础值 + 每级增量）
    PerLevel {
        /// 基础值（等级 1）
        base: f32,
        /// 每级增量
        per_level: f32,
    },
    /// 基于属性（属性值 × 系数）
    AttributeBased {
        /// 属性标识
        attribute_id: String,
        /// 系数
        multiplier: f32,
    },
}

/// 执行配置（Definition 层）。
///
/// 包装 ExecutionType 以用于 EffectDef 的 RON 反序列化。
/// 支持伤害、治疗、自定义等执行计算类型。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionConfig {
    /// 执行计算类型及参数
    pub execution_type: ExecutionType,
}

/// 效果-表现信号绑定（Definition 层）。
///
/// 对应 Schema §3.5 CueBinding。
/// 引用已注册的 CueDef，不嵌入完整 Cue 定义。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EffectCueBinding {
    /// 触发时机标签
    pub cue_tag: CueTag,

    /// 要触发的 CueDef ID
    pub cue_def_id: String,

    /// 延迟触发（帧数）
    pub delay_frames: Option<u64>,
}
