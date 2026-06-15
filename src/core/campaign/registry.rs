// Campaign 注册表：运行时只读注册表，存放所有已加载的 CampaignDef

use std::collections::HashMap;

use bevy::prelude::*;

use super::def::CampaignDef;

/// 战役注册表（Resource，运行时只读）
#[derive(Resource, Default)]
pub struct CampaignRegistry {
    pub campaigns: HashMap<String, CampaignDef>,
}

impl CampaignRegistry {
    pub fn get(&self, id: &str) -> Option<&CampaignDef> {
        self.campaigns.get(id)
    }

    /// 获取第一个战役（默认战役）
    pub fn first(&self) -> Option<&CampaignDef> {
        self.campaigns.values().next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::campaign::def::StageDef;

    /// 构造测试用 CampaignRegistry（含 2 个战役）
    fn test_registry() -> CampaignRegistry {
        let mut registry = CampaignRegistry::default();
        registry.campaigns.insert(
            "campaign_a".to_string(),
            CampaignDef {
                id: "campaign_a".to_string(),
                name: "战役 A".to_string(),
                stages: vec![StageDef {
                    id: "stage_a1".to_string(),
                    level_id: "level_a1".to_string(),
                }],
            },
        );
        registry.campaigns.insert(
            "campaign_b".to_string(),
            CampaignDef {
                id: "campaign_b".to_string(),
                name: "战役 B".to_string(),
                stages: vec![StageDef {
                    id: "stage_b1".to_string(),
                    level_id: "level_b1".to_string(),
                }],
            },
        );
        registry
    }

    #[test]
    fn get_existing_campaign() {
        let registry = test_registry();
        let campaign = registry.get("campaign_a");
        assert!(campaign.is_some());
        assert_eq!(campaign.unwrap().id, "campaign_a");
    }

    #[test]
    fn get_nonexistent_campaign_returns_none() {
        let registry = test_registry();
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn get_campaign_有正确关卡() {
        let registry = test_registry();
        let campaign = registry.get("campaign_a").unwrap();
        assert_eq!(campaign.stages.len(), 1);
        assert_eq!(campaign.stages[0].id, "stage_a1");
    }

    #[test]
    fn first_非空时返回some() {
        let registry = test_registry();
        let first = registry.first();
        assert!(first.is_some());
        let campaign = first.unwrap();
        assert!(campaign.id == "campaign_a" || campaign.id == "campaign_b");
    }

    #[test]
    fn first_空时返回none() {
        let registry = CampaignRegistry::default();
        assert!(registry.first().is_none());
    }

    #[test]
    fn first_只返回一个战役() {
        let mut registry = CampaignRegistry::default();
        registry.campaigns.insert(
            "only".to_string(),
            CampaignDef {
                id: "only".to_string(),
                name: "唯一战役".to_string(),
                stages: vec![],
            },
        );
        let first = registry.first().unwrap();
        assert_eq!(first.id, "only");
    }

    #[test]
    fn campaign_def_clone() {
        let def = CampaignDef {
            id: "test".to_string(),
            name: "测试".to_string(),
            stages: vec![StageDef {
                id: "s1".to_string(),
                level_id: "l1".to_string(),
            }],
        };
        let cloned = def.clone();
        assert_eq!(cloned.id, "test");
        assert_eq!(cloned.stages.len(), 1);
    }

    #[test]
    fn stage_def_clone() {
        let def = StageDef {
            id: "stage_1".to_string(),
            level_id: "level_1".to_string(),
        };
        let cloned = def.clone();
        assert_eq!(cloned.id, "stage_1");
        assert_eq!(cloned.level_id, "level_1");
    }
}
