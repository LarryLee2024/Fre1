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
    /// 关卡状态 i18n key
    pub fn i18n_key(&self) -> &'static str {
        match self {
            StageStatus::Locked => "stage.status.locked",
            StageStatus::Unlocked => "stage.status.unlocked",
            StageStatus::Completed => "stage.status.completed",
        }
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::campaign::def::{CampaignDef, StageDef};

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

    // ================================================
    // initialize() 测试
    // ================================================

    /// Test ID: CAMPAIGN-001
    /// 验证 initialize() 设置第一关为 Unlocked，其余为 Locked
    #[test]
    fn initialize_first_stage_unlocked_rest_locked() {
        let registry = test_registry();
        let progress = CampaignProgress::initialize(&registry);

        assert_eq!(progress.campaign_id, "test_campaign");
        assert_eq!(progress.current_stage, None);
        assert_eq!(
            progress.stage_status("stage_001"),
            Some(&StageStatus::Unlocked)
        );
        assert_eq!(
            progress.stage_status("stage_002"),
            Some(&StageStatus::Locked)
        );
        assert_eq!(
            progress.stage_status("stage_003"),
            Some(&StageStatus::Locked)
        );
    }

    /// Test ID: CAMPAIGN-002
    /// 验证空 registry 返回默认状态
    #[test]
    fn initialize_empty_registry_returns_default() {
        let registry = CampaignRegistry::default();
        let progress = CampaignProgress::initialize(&registry);

        assert_eq!(progress.campaign_id, "");
        assert!(progress.stages.is_empty());
    }

    /// Test ID: CAMPAIGN-003
    /// 验证单关卡战役初始化
    #[test]
    fn initialize_single_stage() {
        let mut registry = CampaignRegistry::default();
        registry.campaigns.insert(
            "single".to_string(),
            CampaignDef {
                id: "single".to_string(),
                name: "单关".to_string(),
                stages: vec![StageDef {
                    id: "only_stage".to_string(),
                    level_id: "level_001".to_string(),
                }],
            },
        );

        let progress = CampaignProgress::initialize(&registry);
        assert_eq!(
            progress.stage_status("only_stage"),
            Some(&StageStatus::Unlocked)
        );
    }

    // ================================================
    // stage_status() 测试
    // ================================================

    /// Test ID: CAMPAIGN-004
    /// 验证 stage_status() 返回不存在的关卡为 None
    #[test]
    fn stage_status_nonexistent_returns_none() {
        let registry = test_registry();
        let progress = CampaignProgress::initialize(&registry);

        assert_eq!(progress.stage_status("nonexistent"), None);
    }

    // ================================================
    // complete_current_stage() 测试
    // ================================================

    /// Test ID: CAMPAIGN-005
    /// 验证 complete_current_stage() 标记当前关卡 Completed 并解锁下一关
    #[test]
    fn complete_current_stage_unlocks_next() {
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

    /// Test ID: CAMPAIGN-006
    /// 验证无 current_stage 时 complete_current_stage 不做任何更改
    #[test]
    fn complete_current_stage_no_current_does_nothing() {
        let registry = test_registry();
        let mut progress = CampaignProgress::initialize(&registry);
        // current_stage = None

        progress.complete_current_stage(&registry);

        // 状态不变
        assert_eq!(
            progress.stage_status("stage_001"),
            Some(&StageStatus::Unlocked)
        );
        assert_eq!(
            progress.stage_status("stage_002"),
            Some(&StageStatus::Locked)
        );
    }

    /// Test ID: CAMPAIGN-007
    /// 验证最后一关完成时不崩溃（无下一关可解锁）
    #[test]
    fn complete_last_stage_does_not_panic() {
        let registry = test_registry();
        let mut progress = CampaignProgress::initialize(&registry);

        // 先解锁并完成前两关
        progress.current_stage = Some("stage_001".to_string());
        progress.complete_current_stage(&registry);
        progress.current_stage = Some("stage_002".to_string());
        progress.complete_current_stage(&registry);

        // 完成最后一关
        progress.current_stage = Some("stage_003".to_string());
        progress.complete_current_stage(&registry);

        assert_eq!(
            progress.stage_status("stage_003"),
            Some(&StageStatus::Completed)
        );
    }

    /// Test ID: CAMPAIGN-008
    /// 验证 complete_current_stage() 保持其他关卡状态不变
    #[test]
    fn complete_current_stage_preserves_other_stages() {
        let registry = test_registry();
        let mut progress = CampaignProgress::initialize(&registry);

        // 手动设置 stage_003 为 Unlocked（模拟特殊情况）
        progress
            .stages
            .insert("stage_003".to_string(), StageStatus::Unlocked);

        progress.current_stage = Some("stage_001".to_string());
        progress.complete_current_stage(&registry);

        // stage_003 保持 Unlocked（不被覆盖）
        assert_eq!(
            progress.stage_status("stage_003"),
            Some(&StageStatus::Unlocked)
        );
    }

    /// Test ID: CAMPAIGN-009
    /// 验证连续完成多关
    #[test]
    fn complete_multiple_stages_sequentially() {
        let registry = test_registry();
        let mut progress = CampaignProgress::initialize(&registry);

        // 完成第一关
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

        // 完成第二关
        progress.current_stage = Some("stage_002".to_string());
        progress.complete_current_stage(&registry);
        assert_eq!(
            progress.stage_status("stage_002"),
            Some(&StageStatus::Completed)
        );
        assert_eq!(
            progress.stage_status("stage_003"),
            Some(&StageStatus::Unlocked)
        );
    }

    // ================================================
    // StageStatus::label() 测试
    // ================================================

    /// Test ID: CAMPAIGN-010
    /// 验证 StageStatus::label() 返回正确的中文标签
    #[test]
    fn stage_status_labels() {
        assert_eq!(StageStatus::Locked.label(), "已锁定");
        assert_eq!(StageStatus::Unlocked.label(), "已解锁");
        assert_eq!(StageStatus::Completed.label(), "已完成");
    }

    /// Test ID: CAMPAIGN-011
    /// 验证 StageStatus 的 PartialEq 实现
    #[test]
    fn stage_status_equality() {
        assert_eq!(StageStatus::Locked, StageStatus::Locked);
        assert_ne!(StageStatus::Locked, StageStatus::Unlocked);
        assert_ne!(StageStatus::Unlocked, StageStatus::Completed);
    }

    /// Test ID: CAMPAIGN-012
    /// 验证 StageStatus 的 Clone 实现
    #[test]
    fn stage_status_clone() {
        let status = StageStatus::Unlocked;
        let cloned = status.clone();
        assert_eq!(status, cloned);
    }
}
