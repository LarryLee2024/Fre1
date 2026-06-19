//! Cue 领域事件
//!
//! 定义表现信号的触发事件。
//! 遵循 docs/02-domain/capabilities/cue_domain.md §6 的事件定义。
//!
//! 事件订阅关系：
//! - CueTriggered → Infra 全表现层（VFX/SFX/Animation/UI）
//! - CueSuppressed → 日志 + 性能监控

use bevy::prelude::*;

use crate::core::capabilities::cue::foundation::CueData;

/// Cue 触发时发送的事件。
///
/// Infra 表现层通过订阅此事件来播放 VFX/SFX/动画/UI 反馈。
/// 不变量 3.1: Cue 是单向信号——表现层不得通过此事件反向修改逻辑。
#[derive(Event, Debug, Clone, PartialEq)]
pub struct CueTriggered {
    /// 完整的 Cue 信号数据
    pub data: CueData,
}

/// Cue 因被禁用/性能限制被跳过时发送的事件。
#[derive(Event, Debug, Clone, PartialEq)]
pub struct CueSuppressed {
    /// CueDef ID
    pub cue_def_id: String,
    /// Cue 类型名称
    pub cue_type_name: String,
    /// 被抑制的原因
    pub reason: SuppressReason,
}

/// Cue 被抑制的原因。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SuppressReason {
    /// Cue 被明确禁用
    Disabled,
    /// 性能限制
    Performance,
    /// 超出可视范围
    OutOfRange,
}

impl SuppressReason {
    /// 返回原因名称。
    pub fn name(&self) -> &str {
        match self {
            Self::Disabled => "Disabled",
            Self::Performance => "Performance",
            Self::OutOfRange => "OutOfRange",
        }
    }
}
