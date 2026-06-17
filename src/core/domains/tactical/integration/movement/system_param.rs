//! MovementCapabilityParam — Bevy SystemParam，封装所有 Capabilities 查询。
//!
//! Systems 通过此 param 获取移动能力，完全不知道 TagSet / AttributeContainer /
//! ModifierContainer 的存在。
//!
//! # 用法
//! ```rust,ignore
//! fn my_system(mov: MovementCapabilityParam) {
//!     let view = mov.build_view(entity, MovementType::Walk);
//!     if view.can_move { /* ... */ }
//! }
//! ```

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::core::capabilities::attribute::mechanism::AttributeContainer;
use crate::core::capabilities::modifier::mechanism::ModifierContainer;
use crate::core::capabilities::tag::mechanism::{TagHierarchy, TagSet};
use crate::core::domains::tactical::components::MovementType;

use super::facade;
use super::types::*;

/// 移动能力查询参数 — 封装所有 Capabilities 依赖。
///
/// System 签名中使用此类型替代裸 TagSet / AttributeContainer / ModifierContainer。
/// 函数体内所有 Capabilities 交互都通过此 param 的方法完成。
#[derive(SystemParam)]
pub struct MovementCapabilityParam<'w, 's> {
    pub tag_hierarchy: Res<'w, TagHierarchy>,
    pub cap_query: Query<
        'w,
        's,
        (
            &'static TagSet,
            &'static AttributeContainer,
            &'static ModifierContainer,
        ),
    >,
}

impl<'w, 's> MovementCapabilityParam<'w, 's> {
    /// 构建实体的移动能力视图。
    pub fn build_view(
        &self,
        entity: Entity,
        movement_type: MovementType,
    ) -> Result<MovementCapabilityView, MovementPrerequisiteError> {
        let (tag_set, attrs, mods) = self
            .cap_query
            .get(entity)
            .map_err(|_| MovementPrerequisiteError::NoTagForMovementType(movement_type))?;
        Ok(facade::build_movement_view(
            tag_set,
            &self.tag_hierarchy,
            attrs,
            mods,
            movement_type,
        ))
    }

    /// 检查实体是否能以指定移动类型移动。
    pub fn can_move_with_type(&self, entity: Entity, movement_type: MovementType) -> bool {
        self.cap_query
            .get(entity)
            .map(|(tag_set, _, _)| {
                facade::can_move_with_type(tag_set, &self.tag_hierarchy, movement_type)
            })
            .unwrap_or(false)
    }
}
