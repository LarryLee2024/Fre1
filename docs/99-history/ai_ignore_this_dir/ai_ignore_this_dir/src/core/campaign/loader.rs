// Campaign 加载系统：从 content/campaigns/ 目录加载 CampaignDef

use std::collections::HashMap;

use bevy::prelude::*;

use crate::core::map::LevelRegistry;

use super::def::CampaignDef;
use super::registry::CampaignRegistry;

/// 加载所有战役配置
///
/// 从 `content/campaigns/` 目录读取所有 RON 文件，反序列化为 CampaignDef，
/// 验证所有 level_id 在 LevelRegistry 中存在，构建 CampaignRegistry。
///
/// 在 Startup 阶段运行（所有 Plugin.build() 完成后，LevelRegistry 已就绪）。
pub fn load_campaigns(
    mut campaign_registry: ResMut<CampaignRegistry>,
    level_registry: Res<LevelRegistry>,
) {
    let dir = "content/campaigns";
    let Ok(entries) = std::fs::read_dir(dir) else {
        bevy::log::warn!(target: "campaign", path = %dir, "战役目录不存在");
        return;
    };

    let mut campaigns = HashMap::new();
    let mut loaded = false;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().map_or(false, |e| e == "ron") {
            match std::fs::read(&path) {
                Ok(bytes) => match ron::de::from_bytes::<CampaignDef>(&bytes) {
                    Ok(def) => {
                        let id = def.id.clone();
                        // 验证所有 level_id 在 LevelRegistry 中存在（3.3）
                        let mut all_valid = true;
                        for stage in &def.stages {
                            if level_registry.get(&stage.level_id).is_none() {
                                bevy::log::error!(
                                    target: "campaign",
                                    campaign_id = %id,
                                    level_id = %stage.level_id,
                                    "关卡 ID 在 LevelRegistry 中不存在，跳过战役"
                                );
                                all_valid = false;
                                break;
                            }
                        }
                        if all_valid {
                            campaigns.insert(id.clone(), def);
                            bevy::log::info!(target: "campaign", event = "campaign_loaded", id = %id, "战役已加载");
                            loaded = true;
                        }
                    }
                    Err(e) => {
                        bevy::log::error!(
                            target: "campaign",
                            path = %path.display(),
                            error = %e,
                            "解析战役配置失败"
                        );
                    }
                },
                Err(e) => {
                    bevy::log::error!(
                        target: "campaign",
                        path = %path.display(),
                        error = %e,
                        "读取战役配置失败"
                    );
                }
            }
        }
    }

    campaign_registry.campaigns = campaigns;

    if !loaded {
        bevy::log::warn!(target: "campaign", "战役目录为空或无有效战役配置");
    }
}
