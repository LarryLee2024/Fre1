//! Death System — 检测 HP 归零的单位并标记 Dead
//!
//! 作为 safety net 运行 —— 正常 DamageDealt 流程已直接标记死亡。
//! 此系统捕获遗漏的场景（初始化时 HP=0、Effect 管线副作用的 HP 变化等）。

use bevy::prelude::*;
use tracing::info;

use crate::core::domains::combat::components::{Dead, HitPoints};

/// 每帧检测 HP 归零的单位，添加 Dead 标签。
///
/// 仅处理尚未标记 Dead 的单位（Without<Dead>），避免重复插入。
pub fn death_check_system(
    mut commands: Commands,
    hp_query: Query<(Entity, &HitPoints), Without<Dead>>,
) {
    for (entity, hp) in hp_query.iter() {
        if !hp.is_alive() {
            info!(target: "combat",
                action = "death_check",
                entity = ?entity,
                hp = hp.current,
                "Entity has 0 HP — adding Dead tag"
            );
            commands.entity(entity).insert(Dead);
        }
    }
}
