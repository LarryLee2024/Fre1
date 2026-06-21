//! Module Name: CharacterCard Widget — 角色卡片复合控件
//!
//! 组合 Panel / Text / ProgressBar / Button 四个原子组件为一个角色卡片。
//! 每个 CharacterCard 显示角色的名称、等级、HP/MP 进度条和动作按钮。
//!
//! Contract:
//!   Props (input):    name, level, hp_current, hp_max, mp_current, mp_max（通过 CharacterCardState）
//!   Events (output):  CharacterAction::Attack/Defend/Skill 标记在按钮实体上供 Observer 路由
//!   Local State:      CharacterCardState（name, level, hp/mp current/max）
//!
//! 详见 `docs/06-ui/02-design-system/widget-composites.md`

pub mod components;
pub mod factory;
pub mod systems;

use bevy::prelude::*;

use self::components::{CharacterAction, CharacterCardLevelLabel, CharacterCardNameLabel, CharacterCardState};
use self::systems::{character_card_update_system, refresh_character_card_from_vm};

/// CharacterCardPlugin — 注册 CharacterCard Widget 所需的 Component/System
pub struct CharacterCardPlugin;

impl Plugin for CharacterCardPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CharacterCardState>()
            .register_type::<CharacterAction>()
            .register_type::<CharacterCardNameLabel>()
            .register_type::<CharacterCardLevelLabel>()
            .add_systems(Update, (character_card_update_system, refresh_character_card_from_vm));
    }
}
