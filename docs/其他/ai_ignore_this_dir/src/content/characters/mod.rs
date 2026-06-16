//! 角色内容加载入口
//!
//! 当前角色模板由 `UnitTemplatePlugin`（core/character/template.rs）通过
//! `UnitTemplateRegistry::load_from_dir("content/characters")` 直接加载。
//! 参见 ADR-004 §4.3 未来可迁移至此模块统一管理。
