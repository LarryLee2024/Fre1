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
