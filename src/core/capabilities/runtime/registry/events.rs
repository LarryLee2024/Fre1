//! Registry 领域事件
//!
//! 定义 Def 注册中心操作过程中的核心事件。

use bevy::prelude::*;

/// Def 注册成功时触发。
///
/// 订阅者：日志、内容验证器。
#[derive(Event, Debug, Clone, PartialEq)]
pub struct DefRegistered {
    /// Def ID
    pub def_id: String,
    /// Def 类型
    pub def_type: String,
}

/// Def 被废弃时触发。
///
/// 订阅者：内容迁移工具、日志。
#[derive(Event, Debug, Clone, PartialEq)]
pub struct DefDeprecated {
    /// Def ID
    pub def_id: String,
    /// 替换者 ID（如果有）
    pub superseded_by: Option<String>,
}

/// 注册校验完成时触发。
///
/// 订阅者：启动流程（根据 passed 决定是否阻止游戏启动）。
#[derive(Event, Debug, Clone, PartialEq)]
pub struct RegistryValidated {
    /// 总 Def 数
    pub total_defs: u32,
    /// 断裂引用数
    pub broken_refs: u32,
    /// 校验是否通过
    pub passed: bool,
}
