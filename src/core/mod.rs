/// Attributes 组件、修饰符栈、实时计算
pub mod attribute;
/// EffectHandler 效果处理器管道
pub mod effect;
/// ModifierRule 修饰规则注册表
/// 这些是跨业务模块的共享基础设施，被 character/battle/buff/ability 等模块依赖

/// ModifierRule 修饰规则注册表
pub mod modifier;

/// Stacking 模块：效果堆叠规则中心（ADR-026 §六）
pub mod stacking;

/// Execution 模块：效果执行算式层（ADR-026 §三）
pub mod execution;

/// Cue 模块：表现层信号总线（ADR-026 §四）
pub mod cue;

/// RON 文件加载器（RegistryLoader trait）
pub mod registry_loader;

/// EntitySnapshot 实体快照
pub mod snapshot;

/// GameplayTags 组件与标签操作 + TagDef 标签定义注册表
pub mod tag;

/// Trigger 触发器系统：统一注册与分发、嵌套触发栈管理
pub mod trigger;

/// 能力模块：技能定义、冷却、效果预览、执行管线
pub mod ability;

/// 目标选择模块：技能/能力的目标类型定义与解析
pub mod targeting;

/// Buff 模块：Buff 定义、实例、穿戴/移除、持续效果
pub mod buff;

/// 地图模块：网格系统、地形数据、寻路、运行时 Grid
pub mod map;

/// 装备模块：数据驱动的装备定义、实例管理、穿脱逻辑
pub mod equipment;

/// 背包模块：容器系统、物品定义、实例管理、物品转移与使用
pub mod inventory;

/// 移动模块：移动意图事件、寻路、执行
pub mod movement;

/// 角色模块：单位组件、生成、模板、特性
pub mod character;

/// 战斗模块：战斗效果管线、意图、执行
pub mod battle;

/// AI 模块：行为系统、决策、目标选择
pub mod ai;

/// 回合模块：回合状态机、行动顺序、胜负条件
pub mod turn;

/// 战役模块：关卡流程、胜负条件检查
pub mod campaign;
