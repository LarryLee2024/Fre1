// Buff 模块：数据驱动的 Buff/Debuff 定义、实例管理、应用/移除
// 支持从 assets/buffs/*.ron 外部配置文件加载

mod domain;
mod instance;
mod apply;
mod plugin;

// 公共 re-exports
pub use domain::*;
pub use instance::*;
pub use apply::*;
pub use plugin::BuffPlugin;
