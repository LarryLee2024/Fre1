use bevy::prelude::*;

/// 保存请求 — 触发后将当前游戏状态保存到指定路径。
#[derive(Event, Debug, Clone)]
pub struct SaveRequest {
    pub path: Option<String>,
    pub label: Option<String>,
}

impl Default for SaveRequest {
    fn default() -> Self {
        Self {
            path: None,
            label: None,
        }
    }
}

/// 加载请求 — 从指定路径加载存档。
#[derive(Event, Debug, Clone)]
pub struct LoadRequest {
    pub path: String,
}

/// 保存完成事件 — 存档操作成功后触发。
#[derive(Event, Debug, Clone)]
pub struct SaveCompleted {
    pub path: String,
    pub entity_count: u32,
    pub success: bool,
}

/// 加载完成事件 — 读档操作完成后触发。
#[derive(Event, Debug, Clone)]
pub struct LoadCompleted {
    pub path: String,
    pub entity_count: u32,
    pub save_version: u32,
    pub success: bool,
}

/// 存档操作错误。
#[derive(Event, Debug, Clone)]
pub struct SaveError {
    pub message: String,
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
