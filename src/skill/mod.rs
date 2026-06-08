// 技能模块：数据驱动的技能定义、槽位管理、效果预览
// 支持从 assets/skills/*.ron 外部配置文件加载

mod domain;
mod slots;
mod preview;
mod plugin;

// 公共 re-exports
pub use domain::*;
pub use slots::*;
pub use preview::*;
pub use plugin::SkillPlugin;
