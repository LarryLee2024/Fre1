// 地图 HP 条更新系统
// 从 UI 层移入 map 模块：更新地图上单位实体的 HP 条前景 Sprite 宽度
// [宪法豁免] 此系统操作游戏实体渲染，不属于 UI 面板逻辑

use bevy::prelude::*;

use crate::core::attribute::{AttributeKind, Attributes};
use crate::core::character::{HpBarFg, Unit};

/// 更新地图上单位实体的 HP 条宽度（每个单位独立计算）
pub fn update_hp_bars(
    units: Query<(&Attributes, &Children), With<Unit>>,
    mut hp_fgs: Query<&mut Sprite, With<HpBarFg>>,
    map: Res<crate::core::map::GameMap>,
) {
    let bar_width = map.tile_size * 0.6;
    for (attrs, children) in &units {
        let hp = attrs.get(AttributeKind::Hp);
        let max_hp = attrs.get(AttributeKind::MaxHp);
        let ratio = if max_hp > 0.0 {
            (hp / max_hp).max(0.0)
        } else {
            0.0
        };
        for child in children.iter() {
            if let Ok(mut sprite) = hp_fgs.get_mut(child) {
                sprite.custom_size = Some(Vec2::new(bar_width * ratio, 4.0));
            }
        }
    }
}
