//! Command Dispatcher — 命令分发逻辑
//!
//! 根据命令类型分发给对应的处理器函数。
//! 调用方提供具体的处理函数映射。
//!
//! 详见 docs/01-architecture/40-cross-cutting/ADR-043-command-input.md §4

use crate::core::capabilities::runtime::command::foundation::{
    CommandError, CommandSource, DispatchResult, GameCommand,
};

/// 命令处理器签名。
///
/// - `command`: 待处理的 GameCommand
/// - `source`: 命令来源
pub type CommandHandler = fn(command: &GameCommand, source: CommandSource) -> DispatchResult;

/// 分发一个命令到默认处理器。
///
/// 对每种命令类型调用一个统一的处理函数，
/// 由调用方在 handler 中区分命令类型。
pub fn dispatch_command(
    command: &GameCommand,
    source: CommandSource,
    handler: CommandHandler,
) -> DispatchResult {
    handler(command, source)
}

/// 批量分发一组命令。
///
/// 返回每个命令对应的分发结果。
/// 如果某个命令分发失败，不会影响其他命令。
pub fn dispatch_batch(
    commands: &[GameCommand],
    source: CommandSource,
    handler: CommandHandler,
) -> Vec<DispatchResult> {
    commands
        .iter()
        .map(|cmd| dispatch_command(cmd, source, handler))
        .collect()
}

/// 验证命令是否合法。
///
/// 基本校验：命令携带的参数不为空。
/// # Errors
/// - 命令参数无效 → InvalidCommand
pub fn validate_command(command: &GameCommand) -> Result<(), CommandError> {
    match command {
        GameCommand::MoveUnit { unit_id, path } => {
            if unit_id.is_empty() {
                return Err(CommandError::InvalidCommand { reason:
                    "unit_id must not be empty".into(),
                });
            }
            if path.is_empty() {
                return Err(CommandError::InvalidCommand { reason:
                    "path must not be empty".into(),
                });
            }
        }
        GameCommand::Wait { unit_id } | GameCommand::EndTurn { unit_id } => {
            if unit_id.is_empty() {
                return Err(CommandError::InvalidCommand { reason:
                    "unit_id must not be empty".into(),
                });
            }
        }
        GameCommand::Attack {
            attacker_id,
            target_id,
            ..
        } => {
            if attacker_id.is_empty() || target_id.is_empty() {
                return Err(CommandError::InvalidCommand { reason:
                    "attacker_id and target_id must not be empty".into(),
                });
            }
        }
        GameCommand::CastSpell {
            caster_id,
            spell_def_id,
            target_id,
        } => {
            if caster_id.is_empty() || spell_def_id.is_empty() || target_id.is_empty() {
                return Err(CommandError::InvalidCommand { reason:
                    "caster_id, spell_def_id, and target_id must not be empty".into(),
                });
            }
        }
        GameCommand::UseItem {
            user_id,
            item_instance_id,
            ..
        } => {
            if user_id.is_empty() || item_instance_id.is_empty() {
                return Err(CommandError::InvalidCommand { reason:
                    "user_id and item_instance_id must not be empty".into(),
                });
            }
        }
        GameCommand::OpenMenu | GameCommand::SaveGame | GameCommand::LoadGame => {
            // Meta commands always valid
        }
    }
    Ok(())
}
