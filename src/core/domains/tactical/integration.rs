//! integration — Tactical 域与 Capabilities 的对接入口
//!
//! Tactical 域使用 Capabilities 的 Tag/Attribute/Modifier 系统来实现：
//! - MovementType → TagNamespace::MovementType 映射
//! - MovementPoints → Attribute 读取
//! - 地形消耗 → Modifier 影响
//!
//! 详见 ADR-022 §2.2

use crate::core::capabilities::tag::foundation::TagNamespace;
use crate::core::domains::tactical::components::MovementType;

/// 将 MovementType 映射到 TagNamespace::MovementType 对应的 TagId。
///
/// 当前返回字符串形式的 TagId，后续由 Registry 管理。
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
