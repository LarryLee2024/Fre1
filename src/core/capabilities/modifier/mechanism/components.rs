use std::collections::HashMap;

use bevy::prelude::*;

use crate::core::capabilities::modifier::foundation::{ModifierData, ModifierInstanceId};

/// 实体上的活跃修改器容器。
///
/// 按 target_attribute 分组存储 ModifierData，支持 Override 覆盖索引。
/// 不变量：
/// - modifiers 中每个 Vec 按 priority 排序（升序）
/// - override_index 记录每个属性的 Override 实例，保证同一属性最多一个 Override
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct ModifierContainer {
    /// target_attribute → 该属性上的所有 ModifierData 列表
    pub modifiers: HashMap<String, Vec<ModifierData>>,
    /// target_attribute → Override 类型 Modifier 的实例 ID（同一属性最多一个 Override）
    #[reflect(ignore)]
    pub override_index: HashMap<String, ModifierInstanceId>,
    /// 单个实体上允许的最大 Modifier 数量
    pub max_modifiers: u32,
}

impl ModifierContainer {
    /// 创建空的修改器容器，默认上限为 100 个 Modifier。
    pub fn empty() -> Self {
        Self {
            modifiers: HashMap::new(),
            override_index: HashMap::new(),
            max_modifiers: 100,
        }
    }
}

impl Default for ModifierContainer {
    fn default() -> Self {
        Self::empty()
    }
}
