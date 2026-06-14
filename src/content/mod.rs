/// Content 层：内容桥接（配置 → 规则）
///
/// Layer 5 职责：只做"加载→校验→注册"，不包含游戏规则。
/// 将在 Phase 5 迁移中逐步填充。
pub mod characters;
pub mod skills;
pub mod buffs;
pub mod equipments;
pub mod items;
pub mod terrains;
pub mod stages;
pub mod ai_behaviors;
pub mod classes;
pub mod formulas;
pub mod modifiers;
