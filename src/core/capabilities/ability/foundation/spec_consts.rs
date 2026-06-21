//! ConstAbilityMetadata — 编译期能力元数据 trait (#12: Const Trait 模式)
//!
//! 定义能力相关类型的编译期常量，由编译器内联，零运行时开销。
//! 适用于 per-type 静态元数据（每个 Def 类型有自己的常量值）。
//!
//! # 使用场景
//!
//! - ID 前缀校验（`validate_id_format` 依赖的前缀）
//! - 默认配置值（最大等级、默认冷却）
//! - 分类标签前缀

use crate::core::capabilities::ability::foundation::def::AbilityDef;

/// 编译期能力元数据 trait。
///
/// 存在原因：ID 前缀、最大等级、冷却回合数等元数据在编译期已知，
/// 通过 const trait 在编译期内联，零运行时开销，替代运行时配置查找。
pub trait ConstAbilityMetadata {
    /// 该类型定义 ID 的前缀（如 `"abl_"` 用于 AbilityDef）。
    const ID_PREFIX: &'static str;

    /// 不指定等级时的默认最大等级。
    const DEFAULT_MAX_LEVEL: u8;

    /// 默认冷却回合数（0 = 无冷却）。
    const DEFAULT_COOLDOWN_TURNS: u32;

    /// 该类型对应的标签命名空间前缀。
    const TAG_NAMESPACE: &'static str;
}

impl ConstAbilityMetadata for AbilityDef {
    const ID_PREFIX: &'static str = "abl_";
    const DEFAULT_MAX_LEVEL: u8 = 5;
    const DEFAULT_COOLDOWN_TURNS: u32 = 0;
    const TAG_NAMESPACE: &'static str = "Ability";
}
