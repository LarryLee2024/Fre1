//! 修饰规则内容加载入口
//!
//! 当前修饰规则由 `ModifierRulePlugin`（core/modifier_rule.rs）通过
//! `ModifierRuleRegistry::load_from_dir_vec("content/modifiers")` 直接加载。
//! 参见 ADR-004 §4.3 未来可迁移至此模块统一管理。
