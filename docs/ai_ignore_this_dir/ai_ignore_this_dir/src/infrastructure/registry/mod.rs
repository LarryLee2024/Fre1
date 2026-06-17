//! Registry 模块 — 资产统一注册中心
//!
//! ADR-026 §十一：统一注册中心，管理所有 Definition/Registry 的注册和查找
//! - 技能/效果/算式/标签全局注册
//! - 新增技能/效果只需注册，不修改管线代码

use bevy::prelude::*;

/// Registry 模块插件
pub struct RegistryPlugin;

impl Plugin for RegistryPlugin {
    fn build(&self, _app: &mut App) {
        // Registry 是最底层无依赖模块
        // 各子模块的 Registry 由各自 Plugin 注册
        // 此插件作为统一入口标记
    }
}

/// 统一注册中心 Resource
///
/// 管理所有已注册的资产类型 ID
#[derive(Resource, Default)]
pub struct AssetRegistry {
    /// 已注册的技能 ID
    pub abilities: Vec<String>,
    /// 已注册的效果 ID
    pub effects: Vec<String>,
    /// 已注册的执行器 ID
    pub executions: Vec<String>,
    /// 已注册的标签 ID
    pub tags: Vec<String>,
}

impl AssetRegistry {
    /// 注册技能 ID
    pub fn register_ability(&mut self, id: String) {
        if !self.abilities.contains(&id) {
            self.abilities.push(id);
        }
    }

    /// 注册效果 ID
    pub fn register_effect(&mut self, id: String) {
        if !self.effects.contains(&id) {
            self.effects.push(id);
        }
    }

    /// 注册执行器 ID
    pub fn register_execution(&mut self, id: String) {
        if !self.executions.contains(&id) {
            self.executions.push(id);
        }
    }

    /// 注册标签 ID
    pub fn register_tag(&mut self, id: String) {
        if !self.tags.contains(&id) {
            self.tags.push(id);
        }
    }

    /// 检查技能 ID 是否已注册
    pub fn has_ability(&self, id: &str) -> bool {
        self.abilities.iter().any(|a| a == id)
    }

    /// 检查效果 ID 是否已注册
    pub fn has_effect(&self, id: &str) -> bool {
        self.effects.iter().any(|e| e == id)
    }

    /// 检查执行器 ID 是否已注册
    pub fn has_execution(&self, id: &str) -> bool {
        self.executions.iter().any(|e| e == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asset_registry_register_ability() {
        let mut registry = AssetRegistry::default();
        registry.register_ability("fireball".to_string());
        assert!(registry.has_ability("fireball"));
        assert!(!registry.has_ability("icebolt"));
    }

    #[test]
    fn asset_registry_register_effect() {
        let mut registry = AssetRegistry::default();
        registry.register_effect("damage".to_string());
        assert!(registry.has_effect("damage"));
    }

    #[test]
    fn asset_registry_no_duplicates() {
        let mut registry = AssetRegistry::default();
        registry.register_ability("fireball".to_string());
        registry.register_ability("fireball".to_string());
        assert_eq!(registry.abilities.len(), 1);
    }
}
