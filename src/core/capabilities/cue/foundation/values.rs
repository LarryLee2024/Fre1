//! Cue 值对象
//!
//! 定义 Cue 绑定注册表——实体上管理的表现信号集合。

use bevy::prelude::Reflect;

use super::types::CueDef;

/// Cue 绑定——记录一个 Cue 定义与实体的关联。
#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct CueBinding {
    /// Cue 定义
    pub cue_def: CueDef,
    /// 是否已禁用
    pub disabled: bool,
}

impl CueBinding {
    /// 创建新的 Cue 绑定。disabled 初始为 false。
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
#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct CueContainer {
    /// 已注册的 Cue 绑定列表
    pub bindings: Vec<CueBinding>,
}

impl CueContainer {
    /// bindings 初始为空 Vec，由 CueRegistrationSystem 在效果施加时填充。
    pub fn new() -> Self {
        Self {
            bindings: Vec::new(),
        }
    }

    /// 用于从 EffectDef 的 cue_defs 批量初始化。
    pub fn with_bindings(bindings: Vec<CueBinding>) -> Self {
        Self { bindings }
    }

    /// register 不检查重复，由 CueSystem 保证同一 CueDef 不会重复注册。
    pub fn register(&mut self, binding: CueBinding) {
        self.bindings.push(binding);
    }

    /// 用于 CueDispatchSystem 在效果触发时按 CueTag 批量调度。
    pub fn find_by_tag(&self, tag: &super::types::CueTag) -> Vec<&CueBinding> {
        self.bindings
            .iter()
            .filter(|b| !b.disabled && b.cue_def.cue_tag == *tag)
            .collect()
    }

    /// 查找包括已禁用的 Cue。用于重新启用场景。
    pub fn find_by_id(&self, id: &str) -> Option<&CueBinding> {
        self.bindings.iter().find(|b| b.cue_def.id == id)
    }

    /// 用于 CueSystem 动态修改 Cue 的运行时属性（如禁用/启用）。
    pub fn find_by_id_mut(&mut self, id: &str) -> Option<&mut CueBinding> {
        self.bindings.iter_mut().find(|b| b.cue_def.id == id)
    }

    /// 已禁用的 Cue 不会被 find_by_tag 返回。不变量 3.4 保证。
    pub fn disable(&mut self, id: &str) -> bool {
        if let Some(binding) = self.find_by_id_mut(id) {
            binding.disabled = true;
            true
        } else {
            false
        }
    }

    /// 启用后 find_by_tag 会重新匹配到该 Cue。
    pub fn enable(&mut self, id: &str) -> bool {
        if let Some(binding) = self.find_by_id_mut(id) {
            binding.disabled = false;
            true
        } else {
            false
        }
    }

    /// 完全移除绑定，不影响其他 Cue。返回 true 表示找到并移除。
    pub fn remove(&mut self, id: &str) -> bool {
        let len = self.bindings.len();
        self.bindings.retain(|b| b.cue_def.id != id);
        self.bindings.len() < len
    }

    /// 仅返回 disabled=false 的绑定。用于批量处理需要播放的 Cue。
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

    /// 所有绑定（含已禁用）数量为 0 时返回 true。
    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    /// 含已禁用 Cue。用于 CueContainer 容量检查。
    pub fn count(&self) -> usize {
        self.bindings.len()
    }
}

impl Default for CueContainer {
    fn default() -> Self {
        Self::new()
    }
}
