// 装备模块：数据驱动的装备定义、实例管理、穿脱逻辑
// 遵循「装备 = Modifier + Trait + Tag + Rule」四层架构
// 支持从 assets/equipment/*.ron 外部配置文件加载

mod definition;
mod equip;
mod instance;
mod plugin;
mod slots;

// 公共 re-exports
pub use definition::*;
pub use equip::*;
pub use instance::*;
pub use plugin::EquipmentPlugin;
pub use slots::*;
