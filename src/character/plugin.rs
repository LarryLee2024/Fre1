use super::components::{Faction, MovingUnit, Unit};
use super::movement::animate_movement;
use super::spawn::{TurnOrderLabel, UnitPlugin};
use super::template::UnitTemplatePlugin;
use super::traits::TraitPlugin;
use crate::turn::{AppState, TurnOrder};
use bevy::prelude::*;

/// 已行动单位颜色变灰
fn update_acted_unit_color(mut units: Query<(&Unit, &mut Sprite), Without<MovingUnit>>) {
    for (unit, mut sprite) in &mut units {
        let base_color = unit.faction.unit_color();
        if unit.acted {
            // 变灰：降低饱和度和亮度
            let mut hsla = Hsla::from(base_color);
            hsla.saturation *= 0.2;
            hsla.lightness = hsla.lightness * 0.5 + 0.25;
            sprite.color = Color::from(hsla);
        } else {
            sprite.color = base_color;
        }
    }
}

/// 更新棋子行动顺序数字：行动完后重新编号，剩余单位从1开始
fn update_turn_order_label(
    turn_order: Res<TurnOrder>,
    parent_query: Query<Entity, With<Unit>>,
    children_query: Query<&Children>,
    mut label_query: Query<&mut Text2d, With<TurnOrderLabel>>,
) {
    // 构建实体→队列位置的映射
    let mut index_map: std::collections::HashMap<Entity, usize> = std::collections::HashMap::new();
    for (idx, &entity) in turn_order.queue.iter().enumerate() {
        index_map.insert(entity, idx);
    }

    let current = turn_order.current_index;

    // 更新每个单位的行动顺序标签
    for entity in &parent_query {
        let label_text = index_map
            .get(&entity)
            .and_then(|&idx| {
                if idx >= current {
                    Some(format!("{}", idx - current + 1))
                } else {
                    None // 已行动完毕，不显示编号
                }
            })
            .unwrap_or_default();

        if let Ok(children) = children_query.get(entity) {
            for &child in children {
                if let Ok(mut text) = label_query.get_mut(child) {
                    **text = label_text.clone();
                }
            }
        }
    }
}

/// 角色插件（组合 Unit + Template + Trait 子插件）
pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((UnitTemplatePlugin, TraitPlugin, UnitPlugin))
            // 移动动画系统：只在游戏中运行
            .add_systems(Update, animate_movement.run_if(in_state(AppState::InGame)))
            .add_systems(
                Update,
                update_acted_unit_color.run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                Update,
                update_turn_order_label.run_if(in_state(AppState::InGame)),
            );
    }
}
