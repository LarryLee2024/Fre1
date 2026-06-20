//! BattleUnitId 注册与构建 — 基于 EntityMapper<BattleUnitId>。
//!
//! 提供 `build_battle_unit_registry` 辅助函数，在战斗开始时为所有 CombatParticipant
//! 分配 BattleUnitId 并建立 Entity ↔ BattleUnitId 双向映射。
//!
//! # 设计
//!
//! - 使用 `EntityMapper<BattleUnitId>` 替代旧的 `BattleUnitRegistry`
//! - `BattleUnitId` 不再作为 Component 挂在实体上（遵循宪法 §724 "Entity 只是 ID"）
//! - ID 格式: `"bu:{team_index}:{unit_index}"` 如 `"bu:0:0"`
//!
//! # 生命周期
//!
//! 战斗开始时由 `start_recording_on_battle_begin` 创建并插入 Resource，
//! 战斗结束时由 `stop_recording_on_battle_end` 清理。
//!
//! 详见 ADR-048 §Module Design

use std::collections::HashMap;

use bevy::prelude::*;

use crate::core::domains::combat::components::CombatParticipant;
use crate::shared::ids::BattleUnitId;
use crate::shared::ids::mapping::EntityMapper;

/// 构建战场单位注册表（EntityMapper<BattleUnitId>）。
///
/// 为每个 CombatParticipant 实体分配 BattleUnitId，建立双向映射。
/// 不再向实体插入 BattleUnitId Component（遵循宪法规则）。
pub(crate) fn build_battle_unit_registry(
    participants: &Query<(Entity, &CombatParticipant)>,
) -> EntityMapper<BattleUnitId> {
    let mut mapper = EntityMapper::<BattleUnitId>::new();

    // 按 team 分组，为每个 entity 分配格式为 "bu:{team_index}:{unit_index}" 的 ID
    let mut team_units: HashMap<String, Vec<Entity>> = HashMap::new();
    for (entity, participant) in participants.iter() {
        let team_id = participant.team_id.to_string();
        team_units.entry(team_id).or_default().push(entity);
    }

    for (team_id, entities) in &team_units {
        for (index, entity) in entities.iter().enumerate() {
            let id_str = format!("bu:{}:{}", team_id.replace(' ', "_"), index);
            let unit_id = BattleUnitId::new(&id_str);
            mapper.register(unit_id, *entity);
        }
    }

    mapper
}
