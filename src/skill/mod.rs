// 技能模块：数据驱动的技能定义、槽位管理、效果预览
// 支持从 assets/skills/*.ron 外部配置文件加载

mod domain;
mod plugin;
mod preview;
mod slots;

// 公共 re-exports
pub use domain::*;
pub use plugin::SkillPlugin;
pub use slots::*;
