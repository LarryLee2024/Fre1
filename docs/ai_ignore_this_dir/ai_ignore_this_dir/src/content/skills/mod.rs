//! 技能内容加载入口
//!
//! 当前技能内容由 `SkillPlugin`（core/skill/mod.rs）通过
//! `SkillRegistry::load_from_dir("content/skills")` 直接加载。
//! 参见 ADR-004 §4.3 未来可迁移至此模块统一管理。
