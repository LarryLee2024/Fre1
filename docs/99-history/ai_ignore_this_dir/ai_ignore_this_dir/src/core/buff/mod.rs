/// Buff 模块：数据驱动的 Buff/Debuff 定义、实例管理、应用/移除、持续效果结算
/// 支持从 content/buffs/*.ron 外部配置文件加载
///
/// ⚠️ DEPRECATED: 此模块已废弃，请使用 effect 模块中的 ApplyModifier 替代。
/// ADR-026 §二：Buff 统一为带 Duration 的 Effect。
/// 此模块保留用于向后兼容，将在后续版本中移除。

/// Buff 应用/移除逻辑
mod apply;
/// BuffDef 定义与 BuffRegistry 注册表
mod domain;
/// Buff 领域错误（domain/buff_error.rs）
// 错误移至 domain 模块
/// BuffInstance, ActiveBuffs 实例管理
mod instance;
/// 持续效果结算（DoT/HoT/Stun）
pub(crate) mod resolve;
/// ADR-022: Buff 触发系统 — TriggerRegistry、TriggerHandler trait、Trigger 枚举
pub mod trigger;

use crate::core::registry_loader::RegistryLoader;
use crate::core::turn::TurnPhase;
use bevy::prelude::*;

/// 公共 re-exports
#[deprecated(note = "Use effect::ApplyModifier instead")]
pub use apply::{apply_buff, apply_buff_with_stack, remove_all_debuffs, remove_buff};
pub use domain::*;
pub use instance::*;
pub use resolve::resolve_status_effects;
pub use trigger::{Trigger, TriggerHandler, TriggerRegistry};

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
