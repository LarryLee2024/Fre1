use super::components::*;
use super::movement::animate_movement;
use super::spawn::{TurnOrderLabel, UnitPlugin};
use super::template::UnitTemplatePlugin;
use super::traits::TraitPlugin;
use crate::core::attribute::{
    AttributeKind, AttributeModifierDef, AttributeModifierInstance, Attributes, BuffInstanceId,
    ModifierOp, ModifierSource,
};
use crate::core::tag::{GameplayTag, GameplayTags, TagName};
use crate::turn::{AppState, TurnOrder};
use bevy::prelude::*;

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
            // 注册 Reflect 类型
            .register_type::<Faction>()
            .register_type::<Unit>()
            .register_type::<UnitName>()
            .register_type::<UnitRace>()
            .register_type::<UnitClass>()
            .register_type::<GridPosition>()
            .register_type::<Selected>()
            .register_type::<Dead>()
            .register_type::<PersistentTags>()
            .register_type::<HpBarBg>()
            .register_type::<HpBarFg>()
            .register_type::<AiBehaviorId>()
            .register_type::<PathArrow>()
            .register_type::<MovingUnit>()
            // 核心 attribute 类型
            .register_type::<AttributeKind>()
            .register_type::<ModifierOp>()
            .register_type::<AttributeModifierDef>()
            .register_type::<ModifierSource>()
            .register_type::<BuffInstanceId>()
            .register_type::<AttributeModifierInstance>()
            .register_type::<Attributes>()
            // 核心 tag 类型
            .register_type::<GameplayTag>()
            .register_type::<GameplayTags>()
            .register_type::<TagName>()
            // 移动动画系统：只在游戏中运行
            .add_systems(Update, animate_movement.run_if(in_state(AppState::InGame)))
            .add_systems(
                Update,
                update_turn_order_label.run_if(in_state(AppState::InGame)),
            );
    }
}
