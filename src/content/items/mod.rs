//! 物品内容加载入口
//!
//! 当前物品内容由 `InventoryPlugin`（core/inventory/mod.rs）通过
//! `ItemRegistry::load_from_dir("content/items")` 直接加载。
//! 参见 ADR-004 §4.3 未来可迁移至此模块统一管理。
