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
//!   └── 清理 EntityMapper<BattleUnitId>
//! ```
//!
//! 详见 ADR-048 §Communication Design

use bevy::prelude::*;

use super::registry::build_battle_unit_registry;
use crate::core::capabilities::runtime::replay::foundation::{ReplayCommand, ReplayHeader};
use crate::core::capabilities::runtime::replay::mechanism::RecordingSession as CoreRecordingSession;
use crate::core::capabilities::runtime::replay::mechanism::resources::RecordingSession;
use crate::core::domains::combat::components::CombatParticipant;
use crate::core::domains::combat::events::{OnBattleEnd, OnBattleStart, UnitActionComplete};
use crate::shared::ids::BattleUnitId;
use crate::shared::ids::mapping::EntityMapper;
use crate::shared::random::DeterministicRng;

/// Observer: OnBattleStart → 初始化录制。
///
/// 1. 枚举所有 CombatParticipant，分配 BattleUnitId（通过 EntityMapper 建立映射）
/// 2. 创建 CoreRecordingSession 并开始录制
/// 3. 将 RecordingSession 插入 ECS World
pub(crate) fn start_recording_on_battle_begin(
    _trigger: On<'_, '_, OnBattleStart>,
    participants: Query<(Entity, &CombatParticipant)>,
    recording: Option<Res<RecordingSession>>,
    mut commands: Commands,
    rng: Option<Res<DeterministicRng>>,
) {
    // 幂等保护：同一场战斗不应重复初始化录制会话
    if let Some(session) = recording
        && session.0.is_some()
    {
        return;
    }

    // Entity→BattleUnitId 映射供后续录制命令时将 Entity 转换为可序列化 ID
    let mapper = build_battle_unit_registry(&participants);

    let unit_count = mapper.len();
    if unit_count == 0 {
        debug!(target: "combat", "[ReplayBridge] 未找到战斗参与者，跳过录制启动");
        return;
    }

    // ReplayHeader 记录参与者列表，回放时用于校验战斗参与者一致性
    let mut header = ReplayHeader::new(1, "0.1.0", "combat_battle", 0);
    for entity_id in mapper.ids() {
        header.add_participant(entity_id.as_str().to_string());
    }

    // 初始种子写入 ReplayHeader，回放时用于校验 RNG 确定性
    let initial_seed = rng
        .map(|r| r.get_all_seeds())
        .map(|seeds| seeds.combat_seed)
        .unwrap_or(42);
    header.initial_seed = initial_seed;

    // 每 60 帧一个检查点，用于回放时分段校验确定性
    let mut core_session = CoreRecordingSession::new(60);
    core_session.start(header, 0);

    commands.insert_resource(mapper);
    commands.insert_resource(RecordingSession(Some(core_session)));

    debug!(target: "combat",
        "[ReplayBridge] 录制开始：{} 个单位，种子={}",
        unit_count, initial_seed
    );
}

/// Observer: UnitActionComplete → 记录为 ReplayCommand。
///
/// 将战斗单位完成的动作记录为 ReplayCommand::SkipTurn（目前为简略录制）。
/// 后续扩展时可添加 action 类型参数以录制详细命令（UseAbility 等）。
pub(crate) fn record_unit_action(
    trigger: On<'_, '_, UnitActionComplete>,
    mapper: Res<EntityMapper<BattleUnitId>>,
    mut recording: ResMut<RecordingSession>,
) {
    let Some(ref mut session) = recording.0 else {
        return;
    };

    let unit = trigger.event().unit;

    // EntityMapper 映射保证 Entity→BattleUnitId 转换的一致性，Replay 回放时依赖此映射
    let unit_id = match mapper.get_id(&unit) {
        Some(id) => id.as_str().to_string(),
        None => {
            debug!(target: "combat", "[ReplayBridge] 未知实体被记录，跳过");
            return;
        }
    };

    // SkipTurn 为简略录制模式 — 后续扩展 UseAbility 等详细命令时在此转换
    let command = ReplayCommand::SkipTurn { unit: unit_id };
    session.record_command(command);
}

/// Observer: OnBattleEnd → 停止录制。
///
/// 1. 结束录制会话，获取 ReplayLog
/// 2. 清理 EntityMapper<BattleUnitId>
pub(crate) fn stop_recording_on_battle_end(
    _trigger: On<'_, '_, OnBattleEnd>,
    mut recording: ResMut<RecordingSession>,
    mut mapper: ResMut<EntityMapper<BattleUnitId>>,
) {
    let Some(ref mut session) = recording.0 else {
        return;
    };

    // 停止录制生成 ReplayLog，后续用于存档、回放、自动化测试
    match session.stop(0) {
        Ok(log) => {
            debug!(target: "combat",
                "[ReplayBridge] 录制停止：{} 帧，{} 条命令",
                log.header.total_frames,
                log.frames.iter().map(|f| f.commands.len()).sum::<usize>()
            );
        }
        Err(e) => {
            debug!(target: "combat", "[ReplayBridge] 录制停止失败：{:?}", e);
        }
    }

    // 清理
    recording.0 = None;
    mapper.clear();
}
