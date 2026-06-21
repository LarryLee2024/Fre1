//! 基础设施事件
//!
//! 这些事件不属于任何业务领域，是基础设施层面的通知。
//! 不涉及业务逻辑，只描述基础设施行为。

use bevy::prelude::*;

/// 配置已加载
#[derive(Message, Debug, Clone)]
pub struct ConfigLoaded {
    pub config_type: String,
    pub id: String,
}

/// 快照已创建
#[derive(Message, Debug, Clone)]
pub struct SnapshotCreated {
    pub snapshot_id: String,
    pub entity_count: usize,
}
