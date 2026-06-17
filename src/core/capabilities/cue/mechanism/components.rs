//! Cue ECS Components
//!
//! 定义挂载在实体上的 Cue 相关 ECS 组件。
//! 遵循 docs/04-data/capabilities/cue_schema.md §4 的数据分层。

use crate::core::capabilities::cue::foundation::CueContainer;
use bevy::prelude::*;

/// Cue 容器组件——实体上注册的表现信号集合。
///
/// 管理该实体关联的所有 Cue 信号，按触发时机分类。
/// 当 Effect/Ability 生命周期到达对应阶段时，从此组件查询应触发的 Cue。
#[derive(Component, Debug, Clone, PartialEq)]
pub struct CueContainerComponent {
    /// 内部的 Cue 容器
    pub container: CueContainer,
}

impl CueContainerComponent {
    /// 创建空的 Cue 容器组件。
    pub fn new() -> Self {
        Self {
            container: CueContainer::new(),
        }
    }

    /// 从现有容器创建组件。
    pub fn from_container(container: CueContainer) -> Self {
        Self { container }
    }
}

impl Default for CueContainerComponent {
    fn default() -> Self {
        Self::new()
    }
}
