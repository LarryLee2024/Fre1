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
    fn victory_completes_current_stage_unlocks_next() {
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
    fn sequential_victories_unlock_all_stages() {
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
    fn defeat_keeps_current_stage_unlocked() {
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
    fn playing_state_is_ignored() {
        let registry = test_registry();
        let mut progress = CampaignProgress::initialize(&registry);
        progress.current_stage = Some("stage_001".to_string());

        assert_eq!(
            progress.stage_status("stage_001"),
            Some(&StageStatus::Unlocked)
        );
    }

    #[test]
    fn victory_with_no_current_stage_does_nothing() {
        let registry = test_registry();
        let mut progress = CampaignProgress::initialize(&registry);

        progress.complete_current_stage(&registry);

        assert_eq!(
            progress.stage_status("stage_001"),
            Some(&StageStatus::Unlocked)
        );
    }

    #[test]
    fn victory_on_last_stage_does_not_panic() {
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
    fn victory_after_defeat_unlocks_next() {
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
    fn current_stage_needs_manual_update_after_victory() {
        let registry = test_registry();
        let mut progress = CampaignProgress::initialize(&registry);
        progress.current_stage = Some("stage_001".to_string());

        progress.complete_current_stage(&registry);

        assert_eq!(progress.current_stage, Some("stage_001".to_string()));

        progress.current_stage = Some("stage_002".to_string());
        assert_eq!(progress.current_stage, Some("stage_002".to_string()));
    }

    #[test]
    fn double_victory_same_stage_does_not_double_unlock() {
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
    fn defeat_preserves_current_stage_pointer() {
        let registry = test_registry();
        let mut progress = CampaignProgress::initialize(&registry);
        progress.current_stage = Some("stage_001".to_string());

        assert_eq!(progress.current_stage, Some("stage_001".to_string()));
    }
}
