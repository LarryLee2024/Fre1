//! Replay 模块 — 确定性战斗回放基建
//!
//! ADR-026 §十三：指令+种子快照持久化
//! - BattleRecord 记录所有战斗指令
//! - ReplayPlayer 确定性回放执行器

use bevy::prelude::*;

/// 战斗记录 Resource
#[derive(Resource, Default, Debug, Clone)]
pub struct BattleRecord {
    /// 随机数种子（确定性回放）
    pub seed: u64,
    /// 回合数
    pub turn_count: u32,
    /// 指令记录
    pub commands: Vec<CommandEntry>,
}

/// 指令条目
#[derive(Debug, Clone, Reflect)]
pub struct CommandEntry {
    /// 回合数
    pub turn: u32,
    /// 指令类型
    pub command_type: CommandType,
    /// 执行者实体
    pub caster: Entity,
    /// 目标实体（可选）
    pub target: Option<Entity>,
    /// 指令数据
    pub data: CommandData,
}

/// 指令类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum CommandType {
    /// 使用技能
    UseSkill,
    /// 移动
    Move,
    /// 等待
    Wait,
    /// 防御
    Defend,
}

/// 指令数据
#[derive(Debug, Clone, Reflect)]
pub enum CommandData {
    /// 技能使用数据
    UseSkill { skill_id: String },
    /// 移动数据
    Move { path: Vec<IVec2> },
    /// 等待（无数据）
    Wait,
    /// 防御（无数据）
    Defend,
}

/// Replay 模块插件
pub struct BattleReplayPlugin;

impl Plugin for BattleReplayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BattleRecord>();
    }
}

/// 回放播放器
pub struct ReplayPlayer {
    /// 战斗记录
    record: BattleRecord,
    /// 当前播放位置
    position: usize,
}

impl ReplayPlayer {
    /// 创建新的回放播放器
    pub fn new(record: BattleRecord) -> Self {
        Self {
            record,
            position: 0,
        }
    }

    /// 获取下一条指令
    pub fn next_command(&mut self) -> Option<&CommandEntry> {
        if self.position < self.record.commands.len() {
            let cmd = &self.record.commands[self.position];
            self.position += 1;
            Some(cmd)
        } else {
            None
        }
    }

    /// 重置播放位置
    pub fn reset(&mut self) {
        self.position = 0;
    }

    /// 是否播放完毕
    pub fn is_finished(&self) -> bool {
        self.position >= self.record.commands.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn battle_record_default() {
        let record = BattleRecord::default();
        assert_eq!(record.turn_count, 0);
        assert!(record.commands.is_empty());
    }

    #[test]
    fn replay_player_next_command() {
        let record = BattleRecord {
            seed: 42,
            turn_count: 1,
            commands: vec![CommandEntry {
                turn: 1,
                command_type: CommandType::UseSkill,
                caster: Entity::from_bits(1),
                target: Some(Entity::from_bits(2)),
                data: CommandData::UseSkill {
                    skill_id: "fireball".to_string(),
                },
            }],
        };

        let mut player = ReplayPlayer::new(record);
        assert!(!player.is_finished());

        let cmd = player.next_command().unwrap();
        assert_eq!(cmd.command_type, CommandType::UseSkill);
        assert!(player.is_finished());
    }

    #[test]
    fn replay_player_reset() {
        let record = BattleRecord {
            seed: 42,
            turn_count: 1,
            commands: vec![CommandEntry {
                turn: 1,
                command_type: CommandType::Wait,
                caster: Entity::from_bits(1),
                target: None,
                data: CommandData::Wait,
            }],
        };

        let mut player = ReplayPlayer::new(record);
        player.next_command();
        assert!(player.is_finished());

        player.reset();
        assert!(!player.is_finished());
    }
}
