//! Recording Systems — Combat 域录制生命周期管理
//!
//! 通过 Observer 监听战斗生命周期事件，自动启动/停止录制会话。
//!
//! # 录制流程
//!
//! ```text
//! OnBattleStart (Trigger)
//!   │
//!   ├── build_battle_unit_registry() → 枚举所有 CombatParticipant，分配 BattleUnitId
//!   ├── 创建 RecordingSession（含 ReplayHeader）
//!   └── 存入 ResMut<RecordingSession>
//!
//! UnitActionComplete (Trigger)
//!   │
//!   ├── 将 UnitActionComplete 转换为 ReplayCommand
//!   └── 记录到 RecordingSession
//!
//! OnBattleEnd (Trigger)
//!   │
//!   ├── session.stop() → 生成 ReplayLog
//!   └── 清理 BattleUnitRegistry
//! ```
//!
//! 详见 ADR-048 §Communication Design

use bevy::prelude::*;

use super::registry::{BattleUnitRegistry, build_battle_unit_registry};
use crate::core::capabilities::runtime::replay::foundation::{ReplayCommand, ReplayHeader};
use crate::core::capabilities::runtime::replay::mechanism::RecordingSession as CoreRecordingSession;
use crate::core::domains::combat::components::CombatParticipant;
use crate::core::domains::combat::events::{OnBattleEnd, OnBattleStart, UnitActionComplete};
use crate::infra::replay::resources::{DeterministicRng, RecordingSession};

/// Observer: OnBattleStart → 初始化录制。
///
/// 1. 枚举所有 CombatParticipant，分配 BattleUnitId（Component + Registry）
/// 2. 创建 CoreRecordingSession 并开始录制
/// 3. 将 RecordingSession 插入 ECS World
pub(crate) fn start_recording_on_battle_begin(
    _trigger: On<'_, '_, OnBattleStart>,
    participants: Query<(Entity, &CombatParticipant)>,
    recording: Option<Res<RecordingSession>>,
    mut commands: Commands,
    rng: Option<Res<DeterministicRng>>,
) {
    // 如果已经有录制会话，跳过
    if let Some(session) = recording {
        if session.0.is_some() {
            return;
        }
    }

    // 构建 BattleUnitRegistry
    let registry = build_battle_unit_registry(&participants, &mut commands);

    let unit_count = registry.len();
    if unit_count == 0 {
        debug!("[ReplayBridge] No combat participants found, skipping recording start");
        return;
    }

    // 收集参与者 ID
    let mut header = ReplayHeader::new(1, "0.1.0", "combat_battle", 0);
    for entity in registry.entities() {
        if let Some(unit_id) = registry.get_id(entity) {
            header.add_participant(unit_id.0.clone());
        }
    }

    // 设置初始种子
    let initial_seed = rng
        .map(|r| r.0.get_all_seeds())
        .map(|seeds| seeds.combat_seed)
        .unwrap_or(42);
    header.initial_seed = initial_seed;

    // 创建录制会话
    let mut core_session = CoreRecordingSession::new(60); // 每 60 帧一个检查点
    core_session.start(header, 0);

    commands.insert_resource(registry);
    commands.insert_resource(RecordingSession(Some(core_session)));

    debug!(
        "[ReplayBridge] Recording started for {} units with seed={}",
        unit_count, initial_seed
    );
}

/// Observer: UnitActionComplete → 记录为 ReplayCommand。
///
/// 将战斗单位完成的动作记录为 ReplayCommand::SkipTurn（目前为简略录制）。
/// 后续扩展时可添加 action 类型参数以录制详细命令（UseAbility 等）。
pub(crate) fn record_unit_action(
    trigger: On<'_, '_, UnitActionComplete>,
    registry: Res<BattleUnitRegistry>,
    mut recording: ResMut<RecordingSession>,
) {
    let Some(ref mut session) = recording.0 else {
        return;
    };

    let unit = trigger.event().unit;

    // 通过 BattleUnitRegistry 将 Entity 转换为 String
    let unit_id = match registry.get_id(&unit) {
        Some(id) => id.0.clone(),
        None => {
            debug!("[ReplayBridge] Unknown entity recorded, skipping");
            return;
        }
    };

    // 记录为 SkipTurn（当前为简略版，后续可扩展）
    let command = ReplayCommand::SkipTurn { unit: unit_id };
    session.record_command(command);
}

/// Observer: OnBattleEnd → 停止录制。
///
/// 1. 结束录制会话，获取 ReplayLog
/// 2. 清理 BattleUnitRegistry
pub(crate) fn stop_recording_on_battle_end(
    _trigger: On<'_, '_, OnBattleEnd>,
    mut recording: ResMut<RecordingSession>,
    mut registry: ResMut<BattleUnitRegistry>,
) {
    let Some(ref mut session) = recording.0 else {
        return;
    };

    // 停止录制并获取日志
    match session.stop(0) {
        Ok(log) => {
            debug!(
                "[ReplayBridge] Recording stopped: {} frames, {} commands",
                log.header.total_frames,
                log.frames.iter().map(|f| f.commands.len()).sum::<usize>()
            );
        }
        Err(e) => {
            debug!("[ReplayBridge] Failed to stop recording: {:?}", e);
        }
    }

    // 清理
    recording.0 = None;
    registry.clear();
}
