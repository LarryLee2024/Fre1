// 核心基础设施：GAS 风格的标签、属性、效果管线、修饰规则
// 这些是跨业务模块的共享基础设施，被 character/battle/buff/skill 等模块依赖

pub mod attribute;
pub mod attribute_def;
pub mod effect;
pub mod modifier_rule;
pub mod registry_loader;
pub mod snapshot;
pub mod tag;
pub mod tag_def;
