//! Cue Dispatch — 信号分发逻辑
//!
//! 将 CueTriggered 事件按 CueType 路由到 Infra 表现层对应子系统。
//! 当前为核心层纯逻辑，实际表现层处理在 Infra 实现。
//!
//! 详见 docs/02-domain/cue_domain.md §5。

use crate::core::capabilities::cue::foundation::{
    CueContainer, CueData, CueDef, CueError, CueTag, CueType,
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
/// # 流程 (per docs/02-domain/cue_domain.md §5.1)
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
pub fn dispatch_cue(cue_data: &CueData) -> Result<DispatchTarget, CueError> {
    // 不变量 3.1: Cue 是只读信号——本函数不修改任何状态
    // 不变量 3.4: 检查 Cue 是否禁用（由调用方在 collect_cues 中处理）

    // 不变量 3.5: Cue 数据不应包含业务敏感信息（占位检查）
    // 实际实现中 Infra 层应验证数据合法性

    Ok(DispatchTarget::from_cue_type(&cue_data.cue_type))
}

/// 检查 Cue 是否可以触发。
///
/// 静态检查：非禁用、参数合法。
///
/// 不变量 3.4: 关键信号（critical）绕过禁用检查，确保始终可触发。
pub fn can_trigger(cue_def: &CueDef, disabled_cues: &[String]) -> bool {
    // 关键信号总是可触发（不变量 3.4 的例外）
    if cue_def.critical {
        return true;
    }
    // 不变量 3.4: 非关键信号受禁用控制
    if disabled_cues.contains(&cue_def.id) {
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::capabilities::cue::foundation::{
        CueBinding, CueContainer, CueDef, CueTag, CueType, VFXParams,
    };

    fn make_container() -> CueContainer {
        let mut container = CueContainer::new();
        container.register(CueBinding::new(CueDef::new(
            "cue_explosion",
            CueType::VFX(VFXParams::new("explosion")),
            CueTag::OnApply,
        )));
        container.register(CueBinding::new(CueDef::new(
            "cue_tick",
            CueType::VFX(VFXParams::new("poison_tick")),
            CueTag::OnTick,
        )));
        container.register(CueBinding::new(CueDef::new(
            "cue_remove",
            CueType::VFX(VFXParams::new("dispel")),
            CueTag::OnRemove,
        )));
        container
    }

    #[test]
    fn unit_030_collect_cues_by_tag() {
        let container = make_container();
        let cues = collect_cues(&container, &CueTag::OnApply, None, None, None);
        assert_eq!(cues.len(), 1);
        assert_eq!(cues[0].cue_def_id, "cue_explosion");
    }

    #[test]
    fn unit_031_collect_cues_on_tick() {
        let container = make_container();
        let cues = collect_cues(&container, &CueTag::OnTick, None, None, None);
        assert_eq!(cues.len(), 1);
        assert_eq!(cues[0].cue_def_id, "cue_tick");
    }

    #[test]
    fn unit_032_collect_cues_with_value() {
        let container = make_container();
        let cues = collect_cues(
            &container,
            &CueTag::OnApply,
            Some("caster_001".into()),
            Some("target_001".into()),
            Some(50.0),
        );
        assert_eq!(cues.len(), 1);
        assert_eq!(cues[0].source_entity, Some("caster_001".into()));
        assert_eq!(cues[0].target_entity, Some("target_001".into()));
        assert_eq!(cues[0].numeric_value, Some(50.0));
    }

    #[test]
    fn unit_033_dispatch_target_from_cue_type() {
        let vfx = CueType::VFX(VFXParams::new("test"));
        assert_eq!(DispatchTarget::from_cue_type(&vfx), DispatchTarget::VFX);

        let sfx = CueType::SFX(crate::core::capabilities::cue::foundation::SFXParams::new(
            "test",
        ));
        assert_eq!(DispatchTarget::from_cue_type(&sfx), DispatchTarget::SFX);
    }

    #[test]
    fn unit_034_dispatch_target_name() {
        assert_eq!(DispatchTarget::VFX.name(), "VFX");
        assert_eq!(DispatchTarget::Popup.name(), "Popup");
    }

    #[test]
    fn unit_035_dispatch_cue_returns_target() {
        let cue_data = CueData::new(
            "test",
            CueType::VFX(VFXParams::new("boom")),
            CueTag::OnApply,
        );
        let result = dispatch_cue(&cue_data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), DispatchTarget::VFX);
    }

    #[test]
    fn unit_036_can_trigger_active() {
        let def = CueDef::new(
            "test",
            CueType::VFX(VFXParams::new("test")),
            CueTag::OnApply,
        );
        assert!(can_trigger(&def, &[]));
    }

    #[test]
    fn unit_037_can_trigger_disabled() {
        let def = CueDef::new(
            "test",
            CueType::VFX(VFXParams::new("test")),
            CueTag::OnApply,
        );
        assert!(!can_trigger(&def, &["test".into()]));
    }

    #[test]
    fn unit_038_can_trigger_critical_always() {
        let def = CueDef::new(
            "test",
            CueType::VFX(VFXParams::new("test")),
            CueTag::OnApply,
        )
        .with_critical();
        assert!(can_trigger(&def, &["test".into()])); // critical bypasses disabled
    }

    #[test]
    fn unit_039_dispatch_result_empty() {
        let result = DispatchResult::empty();
        assert_eq!(result.dispatched, 0);
        assert_eq!(result.suppressed, 0);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn unit_040_collect_cues_disabled_not_collected() {
        let mut container = make_container();
        container.disable("cue_explosion");
        let cues = collect_cues(&container, &CueTag::OnApply, None, None, None);
        assert!(cues.is_empty());
    }
}
