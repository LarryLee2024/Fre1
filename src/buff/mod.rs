// Buff 模块：数据驱动的 Buff/Debuff 定义、实例管理、应用/移除、持续效果结算
// 支持从 assets/buffs/*.ron 外部配置文件加载

mod apply; // Buff 应用/移除逻辑
mod domain; // BuffDef 定义与 BuffRegistry 注册表
mod instance; // BuffInstance, ActiveBuffs 实例管理
pub(crate) mod resolve; // 持续效果结算（DoT/HoT/Stun）

use crate::core::registry_loader::RegistryLoader;
use crate::turn::TurnPhase;
use bevy::prelude::*;

// 公共 re-exports
pub use apply::*;
pub use domain::*;
pub use instance::*;
pub use resolve::resolve_status_effects;

/// Buff 插件（注册 BuffRegistry 资源 + 持续效果结算系统）
pub struct BuffPlugin;

impl Plugin for BuffPlugin {
    fn build(&self, app: &mut App) {
        let registry = domain::BuffRegistry::load_from_dir("assets/buffs");
        app.insert_resource(registry)
            // 注册 Reflect 类型
            .register_type::<instance::BuffInstance>()
            .register_type::<instance::ActiveBuffs>()
            .add_systems(OnEnter(TurnPhase::SelectUnit), resolve_status_effects);
    }
}
