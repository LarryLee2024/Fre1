//! 装备内容加载入口
//!
//! 当前装备内容由 `EquipmentPlugin`（core/equipment/mod.rs）通过
//! `EquipmentRegistry::load_from_dir("content/equipments")` 直接加载。
//! 参见 ADR-004 §4.3 未来可迁移至此模块统一管理。
