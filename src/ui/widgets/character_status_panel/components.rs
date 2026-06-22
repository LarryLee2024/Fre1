//! CharacterStatusPanel 组件的类型定义
//!
//! 定义 CharacterStatusPanel 标记组件和 CharacterStatusPanelState 本地状态。
//! CharacterStatusPanel 挂载在容器实体上，CharacterStatusPanelState 包含
//! 角色状态面板的全部 Props 数据。
//!
//! 详见 `docs/06-ui/02-design-system/widget-composites.md` §3.2

use bevy::prelude::*;

/// CharacterStatusPanel 标记组件
///
/// 标记角色状态面板的容器实体，供外部系统查询和更新。
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct CharacterStatusPanel;

/// CharacterStatusPanel 本地状态（Widget Contract Local State）
///
/// 包含角色名称、HP/MP/AP 当前/最大值、可选状态文本和活跃标记。
/// Props 字段由 spawn_character_status_panel 的入参决定，runtime 由外部系统更新。
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct CharacterStatusPanelState {
    /// 角色显示名称
    pub name: String,
    /// 当前 HP
    pub hp_current: f32,
    /// 最大 HP
    pub hp_max: f32,
    /// 当前 MP
    pub mp_current: f32,
    /// 最大 MP
    pub mp_max: f32,
    /// 当前 AP
    pub ap_current: f32,
    /// 最大 AP
    pub ap_max: f32,
    /// 可选状态文本（如"待机中""移动中"），None 时不渲染
    pub status_text: Option<String>,
    /// 是否为当前行动角色（影响肖像边框样式）
    pub is_active: bool,
}

/// CharacterStatusPanel 名称文本标记组件
///
/// 标记 CharacterStatusPanel 下用于显示角色名称的 Text 子实体，
/// 供 refresh 系统查询和更新。
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
pub struct CharacterStatusPanelNameLabel;

/// CharacterStatusPanel 状态文本标记组件
///
/// 标记 CharacterStatusPanel 下用于显示状态文本的 Text 子实体，
/// 供 refresh 系统查询和更新。
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
pub struct CharacterStatusPanelStatusLabel;
