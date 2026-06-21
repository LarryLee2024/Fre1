use std::collections::HashMap;

use bevy::prelude::*;

use crate::core::capabilities::attribute::foundation::{AttributeId, AttributeValue};

/// 实体上的属性容器。
///
/// 不变量：
/// - attributes 由 AttributeRegistry 在内容加载阶段填充，运行时不可变
/// - derived_cache 由 Aggregator 写入，仅存储 Derived 类属性的缓存值
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
/// 实体属性容器 — 存储所有已注册属性的运行时值。
///
/// 不变量：
/// - attributes 中每个 AttributeValue 的 current_value 由 Aggregator 管线维护
/// - derived_cache 非事实源，可随时删除并重新计算
/// - derived_cache 失效条件：对应属性的 Modifier 增删时标记 Dirty
pub struct AttributeContainer {
    /// 已注册属性的运行时值，key 为 AttributeId
    pub attributes: HashMap<AttributeId, AttributeValue>,
    /// Derived 类属性的缓存聚合值，由 Aggregator 管线维护
    pub derived_cache: HashMap<AttributeId, f32>,
}

impl AttributeContainer {
    /// 创建空的属性容器，后续通过 register 填充。
    pub fn empty() -> Self {
        Self {
            attributes: HashMap::new(),
            derived_cache: HashMap::new(),
        }
    }
}

impl Default for AttributeContainer {
    fn default() -> Self {
        Self::empty()
    }
}
