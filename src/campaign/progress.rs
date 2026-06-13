// CampaignProgress：运行时战役进度状态
// 仅存在于内存中，不持久化（存档系统暂不实现）

use std::collections::HashMap;

use bevy::prelude::*;

use super::registry::CampaignRegistry;

/// 关卡状态
#[derive(Clone, Debug, Reflect, PartialEq, Eq)]
pub enum StageStatus {
    Locked,
    Unlocked,
    Completed,
}

impl StageStatus {
    pub fn label(&self) -> &str {
        match self {
            StageStatus::Locked => "已锁定",
            StageStatus::Unlocked => "已解锁",
            StageStatus::Completed => "已完成",
        }
    }
}

/// 战役进度（Resource，运行时可变）
///
/// 记录当前战役中每个 Stage 的状态（Locked/Unlocked/Completed）。
/// 初始状态：第一个 Stage = Unlocked，其余 = Locked。
/// 不持久化，退出后丢失。
#[derive(Resource, Reflect, Default, Debug)]
#[reflect(Resource)]
pub struct CampaignProgress {
    /// 当前选中的战役 ID
    pub campaign_id: String,
    /// 当前选中的 Stage ID（待进入或进行中的关卡）
    pub current_stage: Option<String>,
    /// 各 Stage 的状态
    pub stages: HashMap<String, StageStatus>,
}

impl CampaignProgress {
    /// 根据 CampaignRegistry 初始化进度
    ///
    /// 第一个 Stage 设为 Unlocked，其余为 Locked。
    /// 如果 registry 为空，保持默认空状态。
    pub fn initialize(registry: &CampaignRegistry) -> Self {
        let Some(campaign) = registry.first() else {
            return Self::default();
        };

        let mut stages = HashMap::new();
        for (i, stage) in campaign.stages.iter().enumerate() {
            let status = if i == 0 {
                StageStatus::Unlocked
            } else {
                StageStatus::Locked
            };
            stages.insert(stage.id.clone(), status);
        }

        Self {
            campaign_id: campaign.id.clone(),
            current_stage: None,
            stages,
        }
    }

    /// 获取指定 stage 的状态
    pub fn stage_status(&self, stage_id: &str) -> Option<&StageStatus> {
        self.stages.get(stage_id)
    }

    /// 标记当前 stage 为已完成，解锁下一个 stage
    ///
    /// 使用 CampaignRegistry 中战役定义的原始 stages 顺序来查找下一个关卡，
    /// 避免依赖 HashMap 的迭代顺序（不保证与定义顺序一致）。
    pub fn complete_current_stage(&mut self, registry: &CampaignRegistry) {
        let Some(ref current) = self.current_stage.clone() else {
            return;
        };

        // 标记当前 stage 为 Completed
        self.stages.insert(current.clone(), StageStatus::Completed);

        // 从战役定义中按原始 stages 顺序查找下一个关卡
        let Some(campaign) = registry.get(&self.campaign_id) else {
            bevy::log::warn!(target: "campaign", campaign_id = %self.campaign_id, "complete_current_stage: 战役未找到");
            return;
        };
        if let Some(pos) = campaign.stages.iter().position(|s| s.id == *current) {
            if pos + 1 < campaign.stages.len() {
                let next_id = &campaign.stages[pos + 1].id;
                if self.stages.get(next_id) == Some(&StageStatus::Locked) {
                    self.stages.insert(next_id.clone(), StageStatus::Unlocked);
                }
            }
        }
    }
}
