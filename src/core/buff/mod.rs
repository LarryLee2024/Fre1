/// Buff 模块：数据驱动的 Buff/Debuff 定义、实例管理、应用/移除、持续效果结算
/// 支持从 content/buffs/*.ron 外部配置文件加载

/// Buff 应用/移除逻辑
mod apply;
/// BuffDef 定义与 BuffRegistry 注册表
mod domain;
/// Buff 领域错误枚举（错误码 BF001-BF004）
mod error;
/// BuffInstance, ActiveBuffs 实例管理
mod instance;
/// 持续效果结算（DoT/HoT/Stun）
pub(crate) mod resolve;

use crate::core::registry_loader::RegistryLoader;
use crate::core::turn::TurnPhase;
use bevy::prelude::*;

/// 公共 re-exports
pub use apply::*;
pub use domain::*;
pub use error::*;
pub use instance::*;
pub use resolve::resolve_status_effects;

/// Buff 插件（注册 BuffRegistry 资源 + 持续效果结算系统）
pub struct BuffPlugin;

impl Plugin for BuffPlugin {
    fn build(&self, app: &mut App) {
        let registry = domain::BuffRegistry::load_from_dir("content/buffs");
        app.insert_resource(registry)
            // 注册 Reflect 类型
            .register_type::<instance::BuffInstance>()
            .register_type::<instance::ActiveBuffs>()
            .add_systems(OnEnter(TurnPhase::SelectUnit), resolve_status_effects);
    }
}
