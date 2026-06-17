//! PipelineRegistry — 管线注册中心（ECS Resource）
//!
//! 作为 Bevy Resource 存储已注册的 PipelineDefinition 和全局 Hook。
//! 业务管线通过 `register()` 注册，执行时通过 `get()` 获取。
//!
//! 🟥 禁止运行时动态调整 Stage 顺序（破坏 Replay 确定性）。
//!
//! 详见 ADR-044 §5

use std::collections::HashMap;

use bevy::prelude::Resource;

use crate::core::capabilities::runtime::pipeline::foundation::PipelineDefinition;

use super::hooks::PipelineHook;

/// 管线注册中心 — ECS Resource
///
/// 存储所有已注册的管线定义和全局 Hook。
#[derive(Resource)]
pub struct PipelineRegistry {
    /// 管线定义映射（ID → PipelineDefinition）
    pipelines: HashMap<String, PipelineDefinition>,
    /// 全局 Hook 列表（按注册顺序执行）
    hooks: Vec<Box<dyn PipelineHook>>,
}

impl PipelineRegistry {
    /// 创建空的注册中心。
    pub fn new() -> Self {
        Self {
            pipelines: HashMap::new(),
            hooks: Vec::new(),
        }
    }

    /// 注册一条管线。
    ///
    /// # Panics
    /// 如果管线 ID 已存在，panic（防止静默覆盖导致 Replay 不一致）。
    pub fn register(&mut self, definition: PipelineDefinition) {
        let id = definition.id.clone();
        assert!(
            !self.pipelines.contains_key(&id),
            "[PIPELINE] duplicate pipeline registration: '{}'",
            id
        );
        self.pipelines.insert(id, definition);
    }

    /// 按 ID 获取管线定义。
    pub fn get(&self, id: &str) -> Option<&PipelineDefinition> {
        self.pipelines.get(id)
    }

    /// 添加全局 Hook。
    pub fn add_hook(&mut self, hook: Box<dyn PipelineHook>) {
        self.hooks.push(hook);
    }

    /// 获取所有已注册的 Hook。
    pub fn hooks(&self) -> &[Box<dyn PipelineHook>] {
        &self.hooks
    }

    /// 返回已注册的管线数量。
    pub fn count(&self) -> usize {
        self.pipelines.len()
    }

    /// 遍历所有注册的管线。
    pub fn iter(&self) -> impl Iterator<Item = &PipelineDefinition> {
        self.pipelines.values()
    }
}

impl Default for PipelineRegistry {
    fn default() -> Self {
        Self::new()
    }
}
