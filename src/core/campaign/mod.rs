/// 战役模块：战役编排、关卡序列管理、进度跟踪
///
/// 新增一个关卡 = 在 content/campaigns/ 中新增 Stage 条目 + 在 content/stages/ 中新增 Level 配置。
/// 已有代码无需修改。Campaign 只通过 level_id 引用 Level，不内嵌任何 Level 数据。
pub mod def;
pub mod loader;
pub mod state;
pub mod system;
pub mod registry;

use bevy::prelude::*;

/// 战役插件（注册在 Data Layer，所有 Plugin.build() 完成后运行加载系统）
pub struct CampaignPlugin;

impl Plugin for CampaignPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<registry::CampaignRegistry>()
            .init_resource::<state::CampaignProgress>()
            .register_type::<state::CampaignProgress>()
            .register_type::<state::StageStatus>()
            .add_systems(Startup, loader::load_campaigns)
            .add_systems(Update, system::on_level_completed);
    }
}
