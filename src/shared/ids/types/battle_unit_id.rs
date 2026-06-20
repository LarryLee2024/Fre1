//! BattleUnitId — 战场单位标识（回放系统用）。
//!
//! 用于 Replay 录制/回放时，将 Entity 句柄转换为稳定 String ID。
//! 通过 `EntityMapper<BattleUnitId>` 进行双向映射，不再作为 Component 存储。

use crate::define_string_id;
use bevy::prelude::Reflect;

define_string_id! {
    pub BattleUnitId,
    prefix: "bu",
}
