// 核心基础设施：GAS 风格的标签、属性、效果管线、修饰规则
// 这些是跨业务模块的共享基础设施，被 character/battle/buff/skill 等模块依赖

pub mod attribute; // Attributes 组件、修饰符栈、实时计算
pub mod attribute_def; // AttributeDef 属性定义注册表
pub mod effect; // EffectHandler 效果处理器管道
pub mod modifier_rule; // ModifierRule 修饰规则注册表
pub mod registry_loader; // RON 文件加载器（RegistryLoader trait）
pub mod snapshot; // EntitySnapshot 实体快照
pub mod tag; // GameplayTags 组件与标签操作
pub mod tag_def; // TagDef 标签定义注册表
