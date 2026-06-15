//! Execution 模块 — 效果执行算式层
//!
//! ADR-026 §三：所有伤害/治疗/地形/百分比数值计算，全部抽离为独立 Execution Trait
//! - 新增伤害类型 = 新增 Execution 实现
//! - 数值策划可独立配公式，不侵入业务代码
//! - Execution 无副作用，天然适配单元测试和回放

pub mod damage;
pub mod heal;
pub mod shield;
pub mod types;

pub use damage::*;
pub use heal::*;
pub use shield::*;
pub use types::*;

use bevy::prelude::*;
use std::collections::HashMap;

/// Execution trait — 执行算式接口
///
/// 每种计算类型（伤害、治疗、护盾等）对应一个 Execution 实现，
/// 通过 ExecutionRegistry 注册和分发。
pub trait Execution: Send + Sync + 'static {
    /// 执行器的唯一标识，用于 Registry 查找
    fn type_name(&self) -> &'static str;

    /// 核心计算：接收上下文，返回计算结果
    ///
    /// # 约束
    /// - 纯函数：不修改游戏状态
    /// - 不访问 ECS World
    /// - 不产生随机数
    /// - 相同 ExecutionContext 产生相同 ExecutionResult
    fn calculate(&self, ctx: &ExecutionContext) -> ExecutionResult;
}

/// Execution 注册表 — 全局唯一的执行器注册表 Resource
#[derive(Resource)]
pub struct ExecutionRegistry {
    executors: HashMap<String, Box<dyn Execution>>,
}

impl Default for ExecutionRegistry {
    fn default() -> Self {
        let mut registry = Self {
            executors: HashMap::new(),
        };
        registry.register_defaults();
        registry
    }
}

impl std::fmt::Debug for ExecutionRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExecutionRegistry")
            .field("count", &self.executors.len())
            .finish()
    }
}

impl ExecutionRegistry {
    /// 注册执行器
    pub fn register(&mut self, executor: Box<dyn Execution>) {
        let name = executor.type_name().to_string();
        self.executors.insert(name, executor);
    }

    /// 通过 type_name 查找执行器
    pub fn get(&self, type_name: &str) -> Option<&dyn Execution> {
        self.executors.get(type_name).map(|e| e.as_ref())
    }

    /// 注册所有内置执行器
    fn register_defaults(&mut self) {
        self.register(Box::new(DamageExecution));
        self.register(Box::new(HealExecution));
        self.register(Box::new(ShieldExecution));
    }
}

/// Execution 模块插件
pub struct ExecutionPlugin;

impl Plugin for ExecutionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ExecutionRegistry>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 执行器注册表_默认有3个执行器() {
        let registry = ExecutionRegistry::default();
        assert!(registry.get("Damage").is_some());
        assert!(registry.get("Heal").is_some());
        assert!(registry.get("Shield").is_some());
    }

    #[test]
    fn 执行器注册表_未知返回None() {
        let registry = ExecutionRegistry::default();
        assert!(registry.get("Unknown").is_none());
    }

    #[test]
    fn 执行器插件_注册资源() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(ExecutionPlugin);

        assert!(app.world().get_resource::<ExecutionRegistry>().is_some());
    }
}
