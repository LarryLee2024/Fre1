//! Buff 内容加载入口
//!
//! 当前 Buff 内容由 `BuffPlugin`（core/buff/mod.rs）通过
//! `BuffRegistry::load_from_dir("content/buffs")` 直接加载。
//! 参见 ADR-004 §4.3 未来可迁移至此模块统一管理。
