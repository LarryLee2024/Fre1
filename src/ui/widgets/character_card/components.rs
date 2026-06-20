//! CharacterCard 组件的类型定义
//!
//! 定义 CharacterCardState（Widget Contract 的本地状态）和 CharacterAction（按钮动作标记）。
//! CharacterCardState 挂载在容器实体上，CharacterAction 挂载在按钮实体上。
//!
//! 详见 `docs/06-ui/02-design-system/widget-composites.md`

use bevy::prelude::*;

/// CharacterCard 本地状态（Widget Contract Local State）
///
/// 包含角色名称、等级、当前/最大 HP、当前/最大 MP。
/// Props 字段由 spawn_character_card 的入参决定，runtime 由外部系统更新。
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct CharacterCardState {
    /// 角色显示名称
    pub name: String,
    /// 角色等级
    pub level: u32,
    /// 当前 HP
    pub hp_current: f32,
    /// 最大 HP
    pub hp_max: f32,
    /// 当前 MP
    pub mp_current: f32,
    /// 最大 MP
    pub mp_max: f32,
}

/// 角色战斗操作按钮动作标记
///
/// 标记 CharacterCard 内的按钮为具体动作，供 Observer 或其他系统
/// 识别角色卡片的交互意图。
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
pub enum CharacterAction {
    /// 攻击
    Attack,
    /// 防御
    Defend,
    /// 技能
    Skill,
}
