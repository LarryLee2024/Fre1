// 战斗 VFX 表现层：监听 Message，生成伤害飘字等特效
// 遵循「Logic 发消息，Presentation 响应」原则

use crate::core::battle::{DamageApplied, DotApplied};
use crate::core::map::GameMap;
use crate::infrastructure::assets::CnFont;
use crate::ui::theme::UiTheme;
use crate::ui::vfx;
use bevy::ecs::message::MessageReader;
use bevy::prelude::*;

/// 响应伤害消息：生成伤害飘字
pub fn on_damage_vfx(
    mut damage_reader: MessageReader<DamageApplied>,
    mut commands: Commands,
    map: Res<GameMap>,
    cn_font: Res<CnFont>,
    theme: Res<UiTheme>,
) {
    for msg in damage_reader.read() {
        let world_pos = map.coord_to_world(msg.target_coord);
        vfx::spawn_damage_popup(
            &mut commands,
            world_pos,
            msg.amount,
            cn_font.as_handle(),
            msg.is_skill,
            &theme,
        );
    }
}

/// 响应 DoT 消息：生成伤害飘字
pub fn on_dot_vfx(
    mut dot_reader: MessageReader<DotApplied>,
    mut commands: Commands,
    map: Res<GameMap>,
    cn_font: Res<CnFont>,
    theme: Res<UiTheme>,
) {
    for msg in dot_reader.read() {
        let world_pos = map.coord_to_world(msg.target_coord);
        vfx::spawn_damage_popup(
            &mut commands,
            world_pos,
            msg.amount,
            cn_font.as_handle(),
            false,
            &theme,
        );
    }
}
