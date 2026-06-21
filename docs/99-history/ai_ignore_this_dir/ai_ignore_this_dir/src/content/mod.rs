/// Content 层：内容桥接（配置 → 规则）
///
/// Layer 5 职责：只做"加载→校验→注册"，不包含游戏规则。
///
/// 当前各内容类型的加载由对应 core Plugin 直接调用 load_from_dir 完成。
/// ContentPlugin 作为统一入口与合约声明，未来可逐步收拢加载逻辑至此层。
pub mod ai_behaviors;
pub mod buffs;
pub mod characters;
pub mod classes;
pub mod equipments;
pub mod formulas;
pub mod items;
pub mod modifiers;
pub mod plugin;
pub mod skills;
pub mod stages;
pub mod terrains;

pub use plugin::ContentPlugin;
