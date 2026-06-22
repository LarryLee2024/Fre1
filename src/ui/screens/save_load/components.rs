//! Module Name: SaveLoadScreen Components
//!
//! Component types for the SaveLoad screen: marker, mode, actions,
//! slot selection state, and ViewModel skeleton.

use bevy::prelude::*;

/// SaveLoad 界面根标记组件。
///
/// 用于 despawn 清理时识别界面层级根节点。
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
pub struct SaveLoadScreen;

/// SaveLoad 界面模式：存档 (Save) 或 读档 (Load)。
///
/// 作为 Component 挂载到根实体上，按需切换。模式决定标题文本
/// 和确认按钮的操作。
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum SaveLoadMode {
    Save,
    Load,
}

impl SaveLoadMode {
    /// 切换存档/读档模式。
    pub fn toggle(&self) -> Self {
        match self {
            SaveLoadMode::Save => SaveLoadMode::Load,
            SaveLoadMode::Load => SaveLoadMode::Save,
        }
    }
}

/// SaveLoad 界面按钮的操作标识。
///
/// 作为 Component 挂载到按钮实体上。Observer 匹配此组件
/// 来确定被点击的按钮对应的操作。
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
pub enum SaveLoadAction {
    /// 切换存档/读档模式
    ToggleMode,
    /// 关闭存档/读档界面
    Close,
    /// 选择一个存档槽位（参数为槽位索引，0-9）
    SelectSlot(usize),
    /// 确认保存/读取
    Confirm,
    /// 删除存档
    Delete,
}

/// 当前选中的存档槽位（屏幕级状态）。
///
/// Resource 而不是 Component，因为同一时刻只有一个 SaveLoad 屏幕处于活动状态。
/// 使用 `Option<usize>` 表示"未选择任何槽位"。
#[derive(Resource, Debug, Clone, PartialEq, Eq, Reflect)]
pub struct SelectedSlot(pub Option<usize>);

impl Default for SelectedSlot {
    fn default() -> Self {
        Self(None)
    }
}

/// 存档槽位的 ViewModel 骨架。
///
/// TODO[P2][SaveLoad][2026-06-22]: 接入真正的存档数据后，此结构体
/// 将包含实际的 SaveMeta 数据并由 Projection 系统填充。
/// 当前为纯占位，所有槽位显示"Empty"。
pub struct SaveSlotVm {
    pub slot_index: usize,
    pub is_occupied: bool,
    pub label: String,
    pub timestamp: Option<String>,
    pub level: Option<u32>,
    pub location: Option<String>,
    pub play_time: Option<String>,
}
