//! 关卡内容加载入口
//!
//! 当前关卡由 `MapDataPlugin`（core/map/data.rs）通过
//! `LevelRegistry::load_from_dir_with_terrain("content/stages")` 直接加载。
//! 参见 ADR-004 §4.3 未来可迁移至此模块统一管理。
