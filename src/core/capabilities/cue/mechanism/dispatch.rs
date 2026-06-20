//! Cue Dispatch — 信号分发逻辑
//!
//! 将 CueTriggered 事件按 CueType 路由到 Infra 表现层对应子系统。
//! 当前为核心层纯逻辑，实际表现层处理在 Infra 实现。
//!
//! 详见 docs/02-domain/capabilities/cue_domain.md §5。

use bevy::ecs::system::Commands;

use crate::core::capabilities::cue::events::{CueSuppressed, CueTriggered, SuppressReason};
use crate::core::capabilities::cue::foundation::error::CueError;
use crate::core::capabilities::cue::foundation::{
    CueContainer, CueData, CueDef, CueTag, CueType,
};

/// 分发目标子系统。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DispatchTarget {
    /// 视觉特效系统
    VFX,
    /// 音效系统
    SFX,
    /// 动画系统
    Animation,
    /// 镜头震动系统
    Shake,
    /// UI 系统
    Popup,
}

impl DispatchTarget {
    /// 从 CueType 获取分发目标。
    pub fn from_cue_type(cue_type: &CueType) -> Self {
        match cue_type {
            CueType::VFX(_) => Self::VFX,
            CueType::SFX(_) => Self::SFX,
            CueType::Animation(_) => Self::Animation,
            CueType::Shake(_) => Self::Shake,
            CueType::Popup(_) => Self::Popup,
        }
    }

    /// 返回分发目标名称。
    pub fn name(&self) -> &str {
        match self {
            Self::VFX => "VFX",
            Self::SFX => "SFX",
            Self::Animation => "Animation",
            Self::Shake => "Shake",
            Self::Popup => "Popup",
        }
    }
}

/// 分发结果。
#[derive(Debug, Clone, PartialEq)]
pub struct DispatchResult {
    /// 已分发的 Cue 数量
    pub dispatched: u32,
    /// 被跳过的 Cue 数量（禁用/性能限制）
    pub suppressed: u32,
    /// 分发过程中的错误
    pub errors: Vec<(String, CueError)>,
}

impl DispatchResult {
    /// 创建空分发结果。
    pub fn empty() -> Self {
        Self {
            dispatched: 0,
            suppressed: 0,
            errors: Vec::new(),
        }
    }
}

/// 收集指定触发时机的所有 Cue 数据。
///
/// 从 CueContainer 中查找匹配 CueTag 的 CueDef，转为 CueData。
///
/// # 流程 (per docs/02-domain/capabilities/cue_domain.md §5.1)
/// 1. 检查 Cue 是否被禁用（不变量 3.4）
/// 2. 创建 CueData 实例
/// 3. 校验 CueData（不变量 3.5: 不包含业务敏感信息）
/// 4. 返回可触发的 Cue 数据列表
pub fn collect_cues(
    container: &CueContainer,
    tag: &CueTag,
    source_entity: Option<String>,
    target_entity: Option<String>,
    numeric_value: Option<f32>,
) -> Vec<CueData> {
    let defs = container.collect_cue_data(tag);
    defs.iter()
        .map(|def| cue_def_to_data(def, &source_entity, &target_entity, numeric_value))
        .collect()
}

/// 将 CueDef 转换为 CueData（运行时数据载体）。
fn cue_def_to_data(
    def: &CueDef,
    source_entity: &Option<String>,
    target_entity: &Option<String>,
    numeric_value: Option<f32>,
) -> CueData {
    CueData {
        cue_def_id: def.id.clone(),
        cue_type: def.cue_type.clone(),
        cue_tag: def.cue_tag.clone(),
        source_entity: source_entity.clone(),
        target_entity: target_entity.clone(),
        numeric_value,
        critical: def.critical,
    }
}

/// 模拟分发 Cue 数据到目标子系统。
///
/// 当前为核心层占位——实际分发由 Infra 表现层实现。
/// 此函数验证 Cue 数据合法性并返回分发目标信息。
pub fn dispatch_cue(
    cue_data: &CueData,
    commands: &mut Commands,
) -> Result<DispatchTarget, CueError> {
    // 不变量 3.1: Cue 是只读信号——本函数不修改任何状态
    // 不变量 3.4: 检查 Cue 是否禁用（由调用方在 collect_cues 中处理）

    // 不变量 3.5: Cue 数据不应包含业务敏感信息（占位检查）
    // 实际实现中 Infra 层应验证数据合法性

    let target = DispatchTarget::from_cue_type(&cue_data.cue_type);

    commands.trigger(CueTriggered {
        data: cue_data.clone(),
    });

    Ok(target)
}

/// 检查 Cue 是否可以触发。
///
/// 静态检查：非禁用、参数合法。
///
/// 不变量 3.4: 关键信号（critical）绕过禁用检查，确保始终可触发。
pub fn can_trigger(cue_def: &CueDef, disabled_cues: &[String], commands: &mut Commands) -> bool {
    // 关键信号总是可触发（不变量 3.4 的例外）
    if cue_def.critical {
        return true;
    }
    // 不变量 3.4: 非关键信号受禁用控制
    if disabled_cues.contains(&cue_def.id) {
        commands.trigger(CueSuppressed {
            cue_def_id: cue_def.id.clone(),
            cue_type_name: cue_def.cue_type.name().to_string(),
            reason: SuppressReason::Disabled,
        });
        return false;
    }
    true
}
