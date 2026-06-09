// Buff 模块：数据驱动的 Buff/Debuff 定义、实例管理、应用/移除、持续效果结算
// 支持从 assets/buffs/*.ron 外部配置文件加载

mod apply;
mod domain;
mod instance;
mod plugin;
mod resolve;

// 公共 re-exports
pub use apply::*;
pub use domain::*;
pub use instance::*;
pub use plugin::BuffPlugin;
