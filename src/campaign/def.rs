// Campaign 定义：CampaignDef 和 StageDef 是 Definition（不可变配置）
// 从 assets/campaigns/*.ron 加载，运行时只读

use serde::Deserialize;

/// 战役定义（RON 反序列化用，运行时只读）
#[derive(Clone, Debug, Deserialize)]
pub struct CampaignDef {
    pub id: String,
    pub name: String,
    pub stages: Vec<StageDef>,
}

/// 关卡位置标识（战役流程中的单个关卡引用）
///
/// Stage 是战役编排层的轻量引用，只包含 id 和 level_id。
/// 不内嵌任何 Level 数据（地形/单位/胜负条件）。
#[derive(Clone, Debug, Deserialize)]
pub struct StageDef {
    pub id: String,
    pub level_id: String,
}
