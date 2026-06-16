//! AI 行为内容加载入口
//!
//! 当前 AI 行为由 `AiBehaviorPlugin`（core/ai/behavior.rs）通过
//! `AiBehaviorRegistry::load_from_dir("content/ai_behaviors")` 直接加载。
//! 参见 ADR-004 §4.3 未来可迁移至此模块统一管理。
