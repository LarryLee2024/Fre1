//! MovementFacade — 移动能力的业务语义 API。
//!
//! 所有 Capabilities 内部类型（AttributeContainer 等）的字段访问都在此文件中完成。
//! Systems 和 Rules 通过 View Types 交互，永远不直接访问 Capabilities 内部。

use crate::core::capabilities::attribute::foundation::AttributeId;
use crate::core::capabilities::attribute::mechanism::AttributeContainer;
use crate::core::capabilities::modifier::foundation::ModifierOp;
use crate::core::capabilities::modifier::mechanism::ModifierContainer;
use crate::core::capabilities::tag::foundation::{TagDefinition, TagId, TagNamespace};
use crate::core::capabilities::tag::mechanism::{TagHierarchy, TagSet};
use crate::core::domains::tactical::components::MovementType;

use super::types::*;

const MOVEMENT_POINTS_ATTR_ID: &str = "attr_movement_points";
const MOVEMENT_COST_MODIFIER_KEY: &str = "movement_cost";

// ─── Tag 查询（封装 TagSet + TagHierarchy） ─────────────────────────

/// 将 MovementType 映射到 TagId 字符串。
pub fn movement_type_to_tag(movement_type: MovementType) -> &'static str {
    match movement_type {
        MovementType::Walk => "tag_000010",
        MovementType::Fly => "tag_000011",
        MovementType::Swim => "tag_000012",
        MovementType::Climb => "tag_000013",
        MovementType::Teleport => "tag_000014",
    }
}

/// 获取 MovementType 对应的 TagNamespace。
pub fn movement_type_namespace(_movement_type: MovementType) -> TagNamespace {
    TagNamespace::MovementType
}

/// 从 TagHierarchy 解析出 MovementType 对应的 TagDefinition。
pub fn resolve_movement_tag_def(
    hierarchy: &TagHierarchy,
    movement_type: MovementType,
) -> Option<&TagDefinition> {
    let tag_id = TagId::new(movement_type_to_tag(movement_type));
    hierarchy.tags.get(&tag_id)
}

/// 检查实体的 TagSet 是否包含指定的 MovementType 标签。
pub fn has_movement_tag(
    tag_set: &TagSet,
    hierarchy: &TagHierarchy,
    movement_type: MovementType,
) -> bool {
    resolve_movement_tag_def(hierarchy, movement_type).is_some_and(|def| tag_set.has_tag(def))
}

/// 检查实体是否能以指定移动类型移动（Tag 管线）。
pub fn can_move_with_type(
    tag_set: &TagSet,
    hierarchy: &TagHierarchy,
    movement_type: MovementType,
) -> bool {
    has_movement_tag(tag_set, hierarchy, movement_type)
}

// ─── Attribute 查询（封装 AttributeContainer） ───────────────────────

/// 从 AttributeContainer 中读取当前移动点数。
fn read_current_mp(attrs: &AttributeContainer) -> MP {
    let attr_id = AttributeId::new(MOVEMENT_POINTS_ATTR_ID);
    MP(attrs
        .attributes
        .get(&attr_id)
        .map(|v| v.current_value)
        .unwrap_or(0.0))
}

/// 从 AttributeContainer 中读取移动点数基础值。
fn read_base_mp(attrs: &AttributeContainer) -> MP {
    let attr_id = AttributeId::new(MOVEMENT_POINTS_ATTR_ID);
    MP(attrs
        .attributes
        .get(&attr_id)
        .map(|v| v.base_value)
        .unwrap_or(0.0))
}

// ─── Modifier 查询（封装 ModifierContainer） ─────────────────────────

/// 计算所有 Modifier 对移动成本的总影响。
fn get_modifier_summary(mods: &ModifierContainer) -> MovementModifierSummary {
    let mut flat_bonus = 0.0f32;
    let mut multiplier = 1.0f32;

    if let Some(modifiers) = mods.modifiers.get(MOVEMENT_COST_MODIFIER_KEY) {
        for m in modifiers {
            match m.op {
                ModifierOp::Add => flat_bonus += m.magnitude,
                ModifierOp::Multiply => multiplier *= m.magnitude,
                _ => {}
            }
        }
    }

    MovementModifierSummary {
        flat_bonus: MP(flat_bonus),
        multiplier,
        total_effect: MP(flat_bonus * multiplier),
    }
}

// ─── 复合操作（组合 Tag + Attribute + Modifier） ─────────────────────

/// 评估实体的完整移动能力（复合查询）。
pub fn build_movement_view(
    tag_set: &TagSet,
    hierarchy: &TagHierarchy,
    attrs: &AttributeContainer,
    mods: &ModifierContainer,
    movement_type: MovementType,
) -> MovementCapabilityView {
    let can_move = has_movement_tag(tag_set, hierarchy, movement_type);
    let effective_points = read_current_mp(attrs);
    let max_points = read_base_mp(attrs);
    let modifier_summary = get_modifier_summary(mods);

    MovementCapabilityView {
        can_move,
        effective_points,
        max_points,
        movement_type,
        modifier_summary,
    }
}

/// 验证移动前提条件（Tag 权限 + Attribute 资源）。
pub fn validate_prerequisites(
    tag_set: &TagSet,
    hierarchy: &TagHierarchy,
    attrs: &AttributeContainer,
    movement_type: MovementType,
) -> Result<(), MovementPrerequisiteError> {
    if !has_movement_tag(tag_set, hierarchy, movement_type) {
        return Err(MovementPrerequisiteError::NoTagForMovementType(
            movement_type,
        ));
    }

    let current = read_current_mp(attrs);
    if current.is_zero() {
        return Err(MovementPrerequisiteError::InsufficientPoints {
            available: current,
            required: MP(1.0),
        });
    }

    Ok(())
}

// ─── 写操作（Modifier 管线） ─────────────────────────────────────────

/// 应用移动消耗。
pub fn apply_movement_cost(
    attrs: &mut AttributeContainer,
    cost: MP,
) -> Result<(), MovementCostError> {
    let attr_id = AttributeId::new(MOVEMENT_POINTS_ATTR_ID);
    if let Some(attr) = attrs.attributes.get_mut(&attr_id) {
        if attr.current_value < cost.0 {
            return Err(MovementCostError::InsufficientPoints {
                available: MP(attr.current_value),
                cost,
            });
        }
        attr.current_value -= cost.0;
        Ok(())
    } else {
        Err(MovementCostError::InsufficientPoints {
            available: MP(0.0),
            cost,
        })
    }
}

/// 重置移动力（回合开始时调用）。
pub fn reset_movement_points(attrs: &mut AttributeContainer) {
    let attr_id = AttributeId::new(MOVEMENT_POINTS_ATTR_ID);
    if let Some(attr) = attrs.attributes.get_mut(&attr_id) {
        attr.current_value = attr.base_value;
    }
}
