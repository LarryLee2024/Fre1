//! RegistryPlugin — 注册中心插件
//!
//! 初始化 DefinitionRegistry Resource，注册热重载事件。

use bevy::prelude::*;

use super::registry::DefinitionRegistry;

/// 注册中心插件。
///
/// 初始化流程（详见 docs/04-data/infrastructure/registry_schema.md §4）：
/// 1. 初始化空的 DefinitionRegistry Resource
/// 2. 注册 OnDefinitionReloaded 事件
/// 3. 注册热重载监听系统（Def 类型 Asset 定型后启用）
///
/// 注：实际配置文件加载由 ContentPlugin 驱动，RegistryPlugin 仅提供注册能力。
pub struct RegistryPlugin;

impl Plugin for RegistryPlugin {
    fn build(&self, app: &mut App) {
        // 事件自动注册（Bevy 0.19+ observer-based，无需 add_event）
        app
            // 初始化空的 DefinitionRegistry
            .init_resource::<DefinitionRegistry>();
    }
}
