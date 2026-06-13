/// Attributes 组件、修饰符栈、实时计算
pub mod attribute;
/// AttributeDef 属性定义注册表
pub mod attribute_def;
/// EffectHandler 效果处理器管道
pub mod effect;
/// 分领域错误体系：每个业务领域独立错误定义（ADR-004）
pub mod error;
/// 核心基础设施：GAS 风格的标签、属性、效果管线、修饰规则
/// 这些是跨业务模块的共享基础设施，被 character/battle/buff/skill 等模块依赖

/// 强类型 ID 模块：核心领域实体的类型安全标识（ADR-002）
pub mod id;
/// ModifierRule 修饰规则注册表
pub mod modifier_rule;
/// RON 文件加载器（RegistryLoader trait）
pub mod registry_loader;
/// EntitySnapshot 实体快照
pub mod snapshot;
/// GameplayTags 组件与标签操作
pub mod tag;
/// TagDef 标签定义注册表
pub mod tag_def;
