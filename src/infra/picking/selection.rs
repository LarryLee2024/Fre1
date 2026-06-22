use bevy::prelude::*;

/// 当前选择状态（Phase 1 MVP 简化版）
///
/// 存储当前选中的单位 Entity。
/// 后续由 data-architect 升级为 BattleUnitId（ADR-067）。
#[derive(Resource, Default, Debug)]
pub struct Selection {
    /// 当前选中的单位
    pub selected_unit: Option<Entity>,
}
