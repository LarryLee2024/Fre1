//! Cue 值对象
//!
//! 定义 Cue 绑定注册表——实体上管理的表现信号集合。

use super::types::CueDef;

/// Cue 绑定——记录一个 Cue 定义与实体的关联。
#[derive(Debug, Clone, PartialEq)]
pub struct CueBinding {
    /// Cue 定义
    pub cue_def: CueDef,
    /// 是否已禁用
    pub disabled: bool,
}

impl CueBinding {
    /// 创建新的 Cue 绑定。
    pub fn new(cue_def: CueDef) -> Self {
        Self {
            cue_def,
            disabled: false,
        }
    }
}

/// Cue 容器——管理实体上关联的所有表现信号。
///
/// 实体可以有多个 Cue（不同触发时机、不同类型的表现）。
/// 不变量 3.4: 所有 Cue 可被独立禁用。
#[derive(Debug, Clone, PartialEq)]
pub struct CueContainer {
    /// 已注册的 Cue 绑定列表
    pub bindings: Vec<CueBinding>,
}

impl CueContainer {
    /// 创建空的 Cue 容器。
    pub fn new() -> Self {
        Self {
            bindings: Vec::new(),
        }
    }

    /// 创建带初始绑定的 Cue 容器。
    pub fn with_bindings(bindings: Vec<CueBinding>) -> Self {
        Self { bindings }
    }

    /// 注册一个 Cue 绑定。
    pub fn register(&mut self, binding: CueBinding) {
        self.bindings.push(binding);
    }

    /// 按触发时机查找所有活跃的 Cue。
    pub fn find_by_tag(&self, tag: &super::types::CueTag) -> Vec<&CueBinding> {
        self.bindings
            .iter()
            .filter(|b| !b.disabled && b.cue_def.cue_tag == *tag)
            .collect()
    }

    /// 按 ID 查找 Cue。
    pub fn find_by_id(&self, id: &str) -> Option<&CueBinding> {
        self.bindings.iter().find(|b| b.cue_def.id == id)
    }

    /// 按 ID 查找 Cue（可变引用）。
    pub fn find_by_id_mut(&mut self, id: &str) -> Option<&mut CueBinding> {
        self.bindings.iter_mut().find(|b| b.cue_def.id == id)
    }

    /// 禁用某个 Cue（不变量 3.4）。
    pub fn disable(&mut self, id: &str) -> bool {
        if let Some(binding) = self.find_by_id_mut(id) {
            binding.disabled = true;
            true
        } else {
            false
        }
    }

    /// 启用某个 Cue。
    pub fn enable(&mut self, id: &str) -> bool {
        if let Some(binding) = self.find_by_id_mut(id) {
            binding.disabled = false;
            true
        } else {
            false
        }
    }

    /// 移除一个 Cue 绑定。
    pub fn remove(&mut self, id: &str) -> bool {
        let len = self.bindings.len();
        self.bindings.retain(|b| b.cue_def.id != id);
        self.bindings.len() < len
    }

    /// 获取所有启用中的 Cue。
    pub fn enabled(&self) -> Vec<&CueBinding> {
        self.bindings.iter().filter(|b| !b.disabled).collect()
    }

    /// 获取所有触发时机下的活跃 Cue 数据。
    pub fn collect_cue_data(&self, tag: &super::types::CueTag) -> Vec<&CueDef> {
        self.bindings
            .iter()
            .filter(|b| !b.disabled && b.cue_def.cue_tag == *tag)
            .map(|b| &b.cue_def)
            .collect()
    }

    /// 容器是否为空。
    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    /// 容器中的 Cue 数量。
    pub fn count(&self) -> usize {
        self.bindings.len()
    }
}

impl Default for CueContainer {
    fn default() -> Self {
        Self::new()
    }
}
