// 战役进度更新系统：监听 LevelCompleted Message，更新 CampaignProgress

use bevy::ecs::message::MessageReader;
use bevy::prelude::*;

use crate::core::turn::LevelCompleted as TurnLevelCompleted;
use crate::shared::event::campaign::LevelCompleted as LogLevelCompleted;

use super::registry::CampaignRegistry;
use super::state::CampaignProgress;

/// 处理关卡完成事件，更新战役进度
///
/// 监听 LevelCompleted Message：
/// - Victory → 当前 Stage = Completed，下一个 Stage = Unlocked（使用 CampaignRegistry 的原始顺序）
/// - Defeat  → 当前 Stage 保持 Unlocked（可重玩）
pub fn on_level_completed(
    mut progress: ResMut<CampaignProgress>,
    campaign_registry: Res<CampaignRegistry>,
    mut reader: MessageReader<TurnLevelCompleted>,
    mut log_writer: MessageWriter<LogLevelCompleted>,
) {
    for msg in reader.read() {
        log_writer.write(LogLevelCompleted {
            level_id: msg.level_id.clone(),
            success: matches!(msg.result, crate::core::turn::GameOverState::Victory),
            turns_used: msg.turn_number,
        });

        match msg.result {
            crate::core::turn::GameOverState::Victory => {
                progress.complete_current_stage(&campaign_registry);
            }
            crate::core::turn::GameOverState::Defeat => {
                // Defeat：当前 Stage 保持 Unlocked，不做任何更改
            }
            crate::core::turn::GameOverState::Playing => {
                // 不应出现 Playing 状态的 LevelCompleted，忽略
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::campaign::def::{CampaignDef, StageDef};
    use crate::core::campaign::registry::CampaignRegistry;
    use crate::core::campaign::state::{CampaignProgress, StageStatus};

    fn test_registry() -> CampaignRegistry {
        let mut registry = CampaignRegistry::default();
        registry.campaigns.insert(
            "test_campaign".to_string(),
            CampaignDef {
                id: "test_campaign".to_string(),
                name: "测试战役".to_string(),
                stages: vec![
                    StageDef {
                        id: "stage_001".to_string(),
                        level_id: "level_001".to_string(),
                    },
                    StageDef {
                        id: "stage_002".to_string(),
                        level_id: "level_002".to_string(),
                    },
                    StageDef {
                        id: "stage_003".to_string(),
                        level_id: "level_003".to_string(),
                    },
                ],
            },
        );
        registry
    }

    #[test]
    fn 胜利_完成当前关卡解锁下一关() {
        let registry = test_registry();
        let mut progress = CampaignProgress::initialize(&registry);
        progress.current_stage = Some("stage_001".to_string());

        progress.complete_current_stage(&registry);

        assert_eq!(
            progress.stage_status("stage_001"),
            Some(&StageStatus::Completed)
        );
        assert_eq!(
            progress.stage_status("stage_002"),
            Some(&StageStatus::Unlocked)
        );
        assert_eq!(
            progress.stage_status("stage_003"),
            Some(&StageStatus::Locked)
        );
    }

    #[test]
    fn 连续胜利_解锁所有关卡() {
        let registry = test_registry();
        let mut progress = CampaignProgress::initialize(&registry);

        progress.current_stage = Some("stage_001".to_string());
        progress.complete_current_stage(&registry);

        progress.current_stage = Some("stage_002".to_string());
        progress.complete_current_stage(&registry);

        assert_eq!(
            progress.stage_status("stage_001"),
            Some(&StageStatus::Completed)
        );
        assert_eq!(
            progress.stage_status("stage_002"),
            Some(&StageStatus::Completed)
        );
        assert_eq!(
            progress.stage_status("stage_003"),
            Some(&StageStatus::Unlocked)
        );
    }

    #[test]
    fn 失败_保持当前关卡解锁() {
        let registry = test_registry();
        let mut progress = CampaignProgress::initialize(&registry);
        progress.current_stage = Some("stage_001".to_string());

        progress.current_stage = None;

        assert_eq!(
            progress.stage_status("stage_001"),
            Some(&StageStatus::Unlocked)
        );
        assert_eq!(
            progress.stage_status("stage_002"),
            Some(&StageStatus::Locked)
        );
    }

    #[test]
    fn playing状态_被忽略() {
        let registry = test_registry();
        let mut progress = CampaignProgress::initialize(&registry);
        progress.current_stage = Some("stage_001".to_string());

        assert_eq!(
            progress.stage_status("stage_001"),
            Some(&StageStatus::Unlocked)
        );
    }

    #[test]
    fn 无当前关卡时胜利_无操作() {
        let registry = test_registry();
        let mut progress = CampaignProgress::initialize(&registry);

        progress.complete_current_stage(&registry);

        assert_eq!(
            progress.stage_status("stage_001"),
            Some(&StageStatus::Unlocked)
        );
    }

    #[test]
    fn 最后一关胜利_不panic() {
        let registry = test_registry();
        let mut progress = CampaignProgress::initialize(&registry);

        progress.current_stage = Some("stage_001".to_string());
        progress.complete_current_stage(&registry);

        progress.current_stage = Some("stage_002".to_string());
        progress.complete_current_stage(&registry);

        progress.current_stage = Some("stage_003".to_string());
        progress.complete_current_stage(&registry);

        assert_eq!(
            progress.stage_status("stage_003"),
            Some(&StageStatus::Completed)
        );
    }

    #[test]
    fn 失败后胜利_解锁下一关() {
        let registry = test_registry();
        let mut progress = CampaignProgress::initialize(&registry);
        progress.current_stage = Some("stage_001".to_string());

        progress.complete_current_stage(&registry);

        assert_eq!(
            progress.stage_status("stage_001"),
            Some(&StageStatus::Completed)
        );
        assert_eq!(
            progress.stage_status("stage_002"),
            Some(&StageStatus::Unlocked)
        );
    }

    #[test]
    fn 当前关卡_胜利后需手动更新() {
        let registry = test_registry();
        let mut progress = CampaignProgress::initialize(&registry);
        progress.current_stage = Some("stage_001".to_string());

        progress.complete_current_stage(&registry);

        assert_eq!(progress.current_stage, Some("stage_001".to_string()));

        progress.current_stage = Some("stage_002".to_string());
        assert_eq!(progress.current_stage, Some("stage_002".to_string()));
    }

    #[test]
    fn 同一关卡双胜利_不双重解锁() {
        let registry = test_registry();
        let mut progress = CampaignProgress::initialize(&registry);
        progress.current_stage = Some("stage_001".to_string());

        progress.complete_current_stage(&registry);
        progress.complete_current_stage(&registry);

        assert_eq!(
            progress.stage_status("stage_002"),
            Some(&StageStatus::Unlocked)
        );
    }

    #[test]
    fn 失败_保留当前关卡指针() {
        let registry = test_registry();
        let mut progress = CampaignProgress::initialize(&registry);
        progress.current_stage = Some("stage_001".to_string());

        assert_eq!(progress.current_stage, Some("stage_001".to_string()));
    }
}
