//! Replay Bridge Integration Tests — 领域内聚四层测试
//!
//! 遵循 test-spec.md §4 四层测试结构：
//! - unit: 纯逻辑单元测试（registry 映射）
//! - integration: 多组件协作测试（recording/playback 集成）
//! - fixtures: Builder 模式测试数据
//! - invariant: 桥接层不变量测试

mod fixtures;
mod integration;
mod invariant;
mod unit;
