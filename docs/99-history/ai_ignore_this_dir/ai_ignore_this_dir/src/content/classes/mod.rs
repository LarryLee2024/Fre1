//! 职业与特质内容加载入口
//!
//! 当前特质由 `TraitPlugin`（core/character/traits/mod.rs）通过
//! `TraitRegistry::load_from_dir("content/classes")` 直接加载。
//! 参见 ADR-004 §4.3 未来可迁移至此模块统一管理。
