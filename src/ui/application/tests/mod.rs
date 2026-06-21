//! UI 应用层测试
//!
//! 测试模块声明，按测试金字塔组织：
//! - unit/: 纯函数单元测试（into_game_command 映射）
//! - integration/: ECS 集成测试（process_ui_commands observer）
//! - fixtures/: 测试数据构建器（预留）

pub mod fixtures;
pub mod integration;
pub mod unit;
