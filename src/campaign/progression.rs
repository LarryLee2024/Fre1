// 战役进度更新系统：监听 LevelCompleted Message，更新 CampaignProgress

use bevy::ecs::message::MessageReader;
use bevy::prelude::*;

use crate::turn::LevelCompleted;

use super::progress::CampaignProgress;
use super::registry::CampaignRegistry;

/// 处理关卡完成事件，更新战役进度
///
/// 监听 LevelCompleted Message：
/// - Victory → 当前 Stage = Completed，下一个 Stage = Unlocked（使用 CampaignRegistry 的原始顺序）
/// - Defeat  → 当前 Stage 保持 Unlocked（可重玩）
pub fn on_level_completed(
    mut progress: ResMut<CampaignProgress>,
    campaign_registry: Res<CampaignRegistry>,
    mut reader: MessageReader<LevelCompleted>,
) {
    for msg in reader.read() {
        bevy::log::info!(
            target: "campaign",
            event = "level_completed",
            level_id = %msg.level_id,
            result = ?msg.result,
            turn = %msg.turn_number,
            "关卡完成"
        );

        match msg.result {
            crate::turn::GameOverState::Victory => {
                progress.complete_current_stage(&campaign_registry);
            }
            crate::turn::GameOverState::Defeat => {
                // Defeat：当前 Stage 保持 Unlocked，不做任何更改
                bevy::log::info!(target: "campaign", event = "level_defeated", "关卡失败，可重玩");
            }
            crate::turn::GameOverState::Playing => {
                // 不应出现 Playing 状态的 LevelCompleted，忽略
            }
        }
    }
}
