/// 移动动画（路径线、箭头、平滑插值）
mod animation;
/// 角色模块：单位组件、生成、模板、特性

/// Unit, UnitName, Faction, GridPosition 等核心组件
mod components;
/// Dead, Selected, MovingUnit 等 Tag 组件
mod marker;
/// 移动执行系统（监听 MovementIntent 消息）
mod movement_execution;
/// 单位生成逻辑
mod spawn;
/// UnitTemplate 数据定义与注册表
pub mod template;
/// Trait 系统（种族/职业/天赋/装备统一抽象）
mod traits;

use crate::core::attribute::{
    AttributeModifierDef, AttributeModifierInstance, BuffInstanceId, ModifierOp, ModifierSource,
};
use crate::core::battle::CharacterDied;
use crate::core::tag::{GameplayTag, GameplayTags};
use crate::core::turn::{AppState, TurnOrder};
use bevy::prelude::*;

pub use crate::core::tag::PersistentTags;
pub use animation::*;
/// 公共 re-exports
pub use components::*;
pub use marker::*;
pub use movement_execution::*;
pub use spawn::TurnOrderLabel;
pub use traits::{
    TraitCollection, TraitData, TraitEffect, TraitEffectHandlerRegistry, TraitPlugin,
    TraitRegistry, TraitSource, TraitTrigger,
};

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
        app.add_plugins((
            template::UnitTemplatePlugin,
            traits::TraitPlugin,
            spawn::UnitPlugin,
        ))
        // 注册 Message
        .add_message::<CharacterDied>()
        // 注册 Dead Observer：响应 Dead Tag 添加，发送 CharacterDied Message
        // 规则3：HP ≤ 0 时只添加 Dead Tag，死亡通知由 Observer 统一发送
        .add_observer(on_dead_added)
        // 注册 Reflect 类型
        .register_type::<Faction>()
        .register_type::<Unit>()
        .register_type::<UnitName>()
        .register_type::<UnitId>()
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
        .register_type::<ModifierOp>()
        .register_type::<AttributeModifierDef>()
        .register_type::<ModifierSource>()
        .register_type::<BuffInstanceId>()
        .register_type::<AttributeModifierInstance>()
        // 核心 tag 类型
        .register_type::<GameplayTag>()
        .register_type::<GameplayTags>()
        // 移动动画系统：只在游戏中运行
        .add_systems(Update, animate_movement.run_if(in_state(AppState::InGame)))
        .add_systems(
            Update,
            update_turn_order_label.run_if(in_state(AppState::InGame)),
        )
        // 统一移动执行系统：监听 MovementIntent 消息
        .add_message::<crate::core::movement::events::MovementIntent>()
        .add_systems(
            PostUpdate,
            movement_execution::movement_execution_system.run_if(in_state(AppState::InGame)),
        );
    }
}
