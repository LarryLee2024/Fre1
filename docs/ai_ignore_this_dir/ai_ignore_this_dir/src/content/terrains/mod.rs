//! 地形内容加载入口
//!
//! 当前地形由 `MapDataPlugin`（core/map/data.rs）通过
//! `TerrainRegistry::load_from_dir("content/terrains")` 直接加载。
//! 参见 ADR-004 §4.3 未来可迁移至此模块统一管理。
