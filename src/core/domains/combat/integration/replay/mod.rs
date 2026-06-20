//! replay — Combat 域与 Replay 系统的桥接层
//!
//! 提供战斗过程的录制与确定性回放：
//!
//! - `registry.rs` — EntityMapper<BattleUnitId> 构建辅助（Entity ↔ 稳定 ID 双向映射）
//! - `recording.rs` — 录制生命周期（OnBattleStart → OnBattleEnd）
//! - `playback.rs` — 回放命令分发（从 PlaybackSession 读取并触发 UnitActionComplete）
//!
//! # 设计原则
//!
//! 1. **最小侵入** — 不修改 CombatPipelineDriver、TurnQueue、CombatParticipant 等核心类型
//! 2. **纯挂载层** — 通过 Observer 接入现有事件流，移除后不影响战斗核心逻辑
//! 3. **录制/回放分离** — 两个子系统独立，互不干扰
//! 4. **使用 EntityMapper** — 统一通过 EntityMapper<BattleUnitId> 映射，不再手写 BattleUnitRegistry
//!
//! 详见 ADR-048

pub mod playback;
pub mod recording;
pub mod registry;

#[cfg(test)]
mod tests;

use bevy::prelude::*;

use self::playback::{block_player_input_during_replay, dispatch_combat_replay_commands};
use self::recording::{
    record_unit_action, start_recording_on_battle_begin, stop_recording_on_battle_end,
};
use crate::shared::ids::BattleUnitId;
use crate::shared::ids::mapping::EntityMapper;

/// Combat 域与 Replay 系统的桥接 Plugin。
///
/// 在 `CombatPlugin` 和 `ReplayPlugin` 均已注册后添加此 Plugin 以启用录制/回放功能。
pub struct CombatReplayBridgePlugin;

impl Plugin for CombatReplayBridgePlugin {
    fn build(&self, app: &mut App) {
        // ── Resource ──
        app.insert_resource(EntityMapper::<BattleUnitId>::new());

        // ── Recording Observers ──
        // OnBattleStart → 创建 EntityMapper<BattleUnitId> + 启动录制会话
        app.add_observer(start_recording_on_battle_begin);
        // UnitActionComplete → 记录为 ReplayCommand
        app.add_observer(record_unit_action);
        // OnBattleEnd → 停止录制会话
        app.add_observer(stop_recording_on_battle_end);

        // ── Playback Systems ──
        // Update: 回放模式下从 PlaybackSession 读取命令并恢复管线
        app.add_systems(Update, dispatch_combat_replay_commands);
        // PreUpdate: 回放模式下阻止玩家输入
        app.add_systems(PreUpdate, block_player_input_during_replay);

        debug!("[ReplayBridge] CombatReplayBridgePlugin 已注册");
    }
}
