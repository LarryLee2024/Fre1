//! Save/Load 领域事件
//!
//! 定义存档读档生命周期中的事件。
//! 详见 ADR-042

use bevy::prelude::*;

use crate::shared::error::ErrorContext;

/// 保存请求 — 触发后将当前游戏状态保存到指定路径。
/// 订阅者：save_world_system（核心序列化）、UI（显示保存进度）。
#[derive(Event, Debug, Clone, Reflect, Default)]
pub struct SaveRequest {
    pub path: Option<String>,
    pub label: Option<String>,
}

/// 加载请求 — 从指定路径加载存档。
/// 订阅者：load_system（核心反序列化）、UI（显示加载进度）。
#[derive(Event, Debug, Clone, Reflect)]
pub struct LoadRequest {
    pub path: String,
}

/// 保存完成事件 — 存档操作成功后触发。
/// 订阅者：UI（存档列表刷新）、日志。
#[derive(Event, Debug, Clone, Reflect)]
pub struct SaveCompleted {
    pub path: String,
    pub entity_count: u32,
    pub success: bool,
}

/// 加载完成事件 — 读档操作完成后触发。
/// 订阅者：全场系统重建、UI（切换到游戏画面）。
#[derive(Event, Debug, Clone, Reflect)]
pub struct LoadCompleted {
    pub path: String,
    pub entity_count: u32,
    pub save_version: u32,
    pub success: bool,
}

/// 存档操作错误。
#[derive(Event, Debug, Clone)]
pub struct SaveError {
    pub error_context: ErrorContext<String>,
    pub operation: SaveOperation,
}

/// 存档操作类型。
#[derive(Debug, Clone, PartialEq)]
pub enum SaveOperation {
    Save,
    Load,
    AutoSave,
    Migrate,
}
