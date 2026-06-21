//! CharacterPanelVm — 角色面板视图模型
//!
//! 详见 `docs/06-ui/04-data-flow/projection-viewmodel.md` §3

use bevy::prelude::*;

/// 角色面板视图模型
#[derive(Clone, Reflect, Default)]
pub struct CharacterPanelVm {
    /// 角色 ID
    pub character_id: u32,
    /// 角色名称（本地化 Key）
    pub name_key: &'static str,
    /// 等级
    pub level: u32,
    /// 当前 HP
    pub hp: f32,
    /// 最大 HP
    pub max_hp: f32,
    /// 当前 MP
    pub mp: f32,
    /// 最大 MP
    pub max_mp: f32,
}
