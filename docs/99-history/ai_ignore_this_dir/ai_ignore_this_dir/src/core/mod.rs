/// Buff 模块（DEPRECATED：ADR-026 已统一为 ApplyModifier，保留向后兼容）
#[allow(deprecated)]
pub mod buff;

/// Attribute 模块：属性系统（ADR-026 §五：Primary/Derived 双分层）
pub mod attribute;

/// Effect 模块：统一效果层（ADR-026 §二：吸收原 Buff）
pub mod effect;

/// Modifier 模块：属性修改器（ADR-026 §三）
pub mod modifier;

/// Stacking 模块：效果堆叠规则中心（ADR-026 §六：4-enum 冻结模型）
pub mod stacking;

/// Execution 模块：效果执行算式层（ADR-026 §三：trait-based 公式执行）
pub mod execution;

/// Cue 模块：表现层信号总线（ADR-026 §四：逻辑与表现彻底分离）
pub mod cue;

/// Tag 模块：游戏标签体系（GAS 核心）
pub mod tag;

/// Trigger 模块：触发器系统
pub mod trigger;

/// Ability 模块：战斗技能领域
pub mod ability;

/// Targeting 模块：目标选取领域
pub mod targeting;

/// Map 模块：地图系统（网格、地形、寻路）
pub mod map;

/// Equipment 模块：装备系统
pub mod equipment;

/// Inventory 模块：背包系统
pub mod inventory;

/// Movement 模块：移动系统
pub mod movement;

/// Character 模块：角色系统
pub mod character;

/// Battle 模块：战斗系统
pub mod battle;

/// AI 模块：人工智能系统
pub mod ai;

/// Turn 模块：回合系统
pub mod turn;

/// Campaign 模块：战役系统
pub mod campaign;

/// RON 文件加载器
pub mod registry_loader;

/// EntitySnapshot 实体快照
pub mod snapshot;
