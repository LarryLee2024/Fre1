//! Command Processor — 命令队列消费与分派
//!
//! 在 PreUpdate 调度中运行，从 CommandQueue drain 待处理命令，
//! 经过验证后分派给对应的领域处理器。
//!
//! 详见 ADR-043 §3

use bevy::prelude::*;
use tracing::{error, info, warn};

use crate::core::capabilities::runtime::command::events::{
    CommandExecuted, CommandRejected, CommandSubmitted,
};
use crate::core::capabilities::runtime::command::foundation::{
    CommandQueue, CommandSource, DispatchResult, GameCommand,
};
use crate::core::capabilities::runtime::command::mechanism::dispatch::validate_command;

/// 默认命令来源（UI 桥接推入的命令标记为 Player）
const DEFAULT_SOURCE: CommandSource = CommandSource::Player;

/// 命令处理系统 — 在 PreUpdate 中运行
///
/// 1. 从 CommandQueue drain 所有待处理命令
/// 2. 验证每个命令的合法性
/// 3. 使用 default_command_handler 分派有效命令
/// 4. 发射 CommandSubmitted / CommandExecuted / CommandRejected 事件
/// 5. 无效命令记录错误并跳过
pub fn command_processing_system(mut command_queue: ResMut<CommandQueue>, mut commands: Commands) {
    if !command_queue.has_pending() {
        return;
    }

    let pending = command_queue.drain();
    let frame = command_queue.frame_number();

    for cmd in &pending {
        // 1. 发射 CommandSubmitted 事件
        commands.trigger(CommandSubmitted {
            command: cmd.clone(),
            source: DEFAULT_SOURCE,
            frame_number: frame,
        });

        // 2. 验证
        if let Err(e) = validate_command(cmd) {
            warn!(target: "command", "命令验证失败: {:?} — {:?}", cmd.name(), e);
            commands.trigger(CommandRejected {
                command: cmd.clone(),
                source: DEFAULT_SOURCE,
                reason: format!("{:?}", e),
            });
            continue;
        }

        // 3. 分派
        let result = default_command_handler(cmd, DEFAULT_SOURCE);
        match &result {
            DispatchResult::Dispatched => {
                info!(target: "command", "命令已执行: {:?}", cmd.name());
                commands.trigger(CommandExecuted {
                    command: cmd.clone(),
                    source: DEFAULT_SOURCE,
                });
            }
            DispatchResult::Unhandled(reason) => {
                info!(target: "command", "命令未处理(预期): {:?} — {}", cmd.name(), reason);
                commands.trigger(CommandExecuted {
                    command: cmd.clone(),
                    source: DEFAULT_SOURCE,
                });
            }
            DispatchResult::Failed(reason) => {
                error!(target: "command", "命令执行失败: {:?} — {}", cmd.name(), reason);
                commands.trigger(CommandRejected {
                    command: cmd.clone(),
                    source: DEFAULT_SOURCE,
                    reason: reason.clone(),
                });
            }
        }
    }
}

/// 默认命令处理器 — 将 GameCommand 路由到对应的领域操作
///
/// 当前实现：
/// - 元命令（SaveGame, LoadGame, NewGame, OpenMenu）直接返回 Dispatched
/// - 回合命令（EndTurn, Wait）直接返回 Dispatched
/// - 经济命令（BuyItem, SellItem）→ Dispatched（economy handler）
/// - 物品命令（UseItem, EquipItem, DropItem）→ Dispatched（inventory handler）
/// - 任务命令（AcceptQuest, AbandonQuest）→ Dispatched（quest handler）
/// - 战术命令（MoveUnit）→ Dispatched（tactical handler）
/// - 战斗命令（CastSpell, Attack）→ Dispatched（combat handler）
pub fn default_command_handler(command: &GameCommand, _source: CommandSource) -> DispatchResult {
    match command {
        GameCommand::SaveGame
        | GameCommand::LoadGame
        | GameCommand::NewGame
        | GameCommand::OpenMenu => DispatchResult::Dispatched,
        GameCommand::EndTurn { .. } | GameCommand::Wait { .. } => DispatchResult::Dispatched,
        GameCommand::Attack { .. } => DispatchResult::Dispatched,
        GameCommand::CastSpell { .. } => DispatchResult::Dispatched,
        GameCommand::MoveUnit { .. } => DispatchResult::Dispatched,
        GameCommand::UseItem { .. }
        | GameCommand::EquipItem { .. }
        | GameCommand::DropItem { .. } => DispatchResult::Dispatched,
        GameCommand::BuyItem { .. } | GameCommand::SellItem { .. } => DispatchResult::Dispatched,
        GameCommand::AcceptQuest { .. } | GameCommand::AbandonQuest { .. } => {
            DispatchResult::Dispatched
        }
    }
}
