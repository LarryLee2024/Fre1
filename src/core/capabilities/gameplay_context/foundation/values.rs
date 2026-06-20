//! GameplayContext 值对象定义

use bevy::prelude::Entity;

use crate::core::capabilities::gameplay_context::foundation::error::ContextBuildError;
use crate::core::capabilities::gameplay_context::foundation::types::*;

/// 行为发起者信息。
#[derive(Debug, Clone)]
pub struct SourceInfo {
    /// 发起者实体
    pub entity: Entity,
    /// 发起者阵营 ID
    pub faction: String,
    /// 发起者位置（可选）
    pub position: Option<(i32, i32)>,
}

/// 行为目标者信息。
#[derive(Debug, Clone)]
pub struct TargetInfo {
    /// 目标实体
    pub entity: Entity,
    /// 目标阵营 ID
    pub faction: String,
    /// 目标位置（可选）
    pub position: Option<(i32, i32)>,
    /// 目标是否有效
    pub is_valid: bool,
}

/// 溯源链的单个节点。
#[derive(Debug, Clone)]
pub struct ChainNode {
    /// 节点触发类型
    pub origin: ContextOrigin,
    /// 该节点的行为发起者
    pub source: SourceInfo,
    /// 该节点的行为目标
    pub target: TargetInfo,
    /// 该节点使用的能力 ID
    pub ability_id: Option<String>,
    /// 节点时间（帧号）
    pub frame: u64,
    /// 节点唯一 ID（用于循环检测）
    pub node_id: u64,
}

/// 上下文元数据。
#[derive(Debug, Clone)]
pub struct ContextMetadata {
    /// 上下文版本
    pub schema_version: u32,
    /// 生命周期状态
    pub status: ContextStatus,
}

/// 溯源链——记录行为链的完整路径。
///
/// 用于防止无限循环（反击/连锁/伤害转移）。
/// Vec 而非链表：更简单的遍历、长度检查、循环检测。
#[derive(Debug, Clone)]
pub struct ContextChain {
    /// 节点序列（从旧到新）
    pub nodes: Vec<ChainNode>,
    /// 链长度上限（默认 10）
    pub max_length: u8,
}

impl ContextChain {
    /// max_length=10，第一个节点由 ContextBuilder 传入，代表行为链的起点。
    pub fn new(first_node: ChainNode) -> Self {
        Self {
            max_length: 10,
            nodes: vec![first_node],
        }
    }

    /// 用于链长度检查（不变量：chain.len() ≤ max_length）。
    pub fn len(&self) -> u8 {
        self.nodes.len() as u8
    }

    /// is_empty 仅检查 nodes 是否为空，不涉及生命周期状态。
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// 用于 trace 日志和循环检测中的节点遍历。
    pub fn last(&self) -> Option<&ChainNode> {
        self.nodes.last()
    }

    /// 检查新节点是否会形成循环。
    ///
    /// 规则：如果链中已存在相同 (source.entity, target.entity, ability_id) 组合 → 循环。
    pub fn would_create_cycle(
        &self,
        new_source: Entity,
        new_target: Entity,
        new_ability: &Option<String>,
    ) -> bool {
        self.nodes.iter().any(|node| {
            node.source.entity == new_source
                && node.target.entity == new_target
                && node.ability_id == *new_ability
        })
    }

    /// 检查链是否已达到长度上限。
    pub fn is_at_max_length(&self) -> bool {
        self.len() >= self.max_length
    }

    /// 尝试添加新节点。
    ///
    /// 如果导致循环或超过上限，返回错误。
    pub fn try_push(&mut self, node: ChainNode) -> Result<(), ContextBuildError> {
        if self.would_create_cycle(node.source.entity, node.target.entity, &node.ability_id) {
            return Err(ContextBuildError::CycleDetected);
        }
        if self.is_at_max_length() {
            return Err(ContextBuildError::ChainTooLong {
                current: self.len(),
                max: self.max_length,
            });
        }
        self.nodes.push(node);
        Ok(())
    }

    /// 用于日志和调试遍历。使用 nodes.iter() 而非 IntoIterator 以明确只读语义。
    pub fn iter(&self) -> impl Iterator<Item = &ChainNode> {
        self.nodes.iter()
    }
}

/// 跨系统传递的统一数据载体。
///
/// 通过 ContextBuilder 构建，构建完成后不可变。
/// 是 Ability → Targeting → Execution → Effect → Cue 全链路的统一数据总线。
#[derive(Debug, Clone)]
pub struct GameplayContextData {
    /// 上下文唯一标识
    pub context_id: String,
    /// 触发类型
    pub origin: ContextOrigin,
    /// 行为发起者信息
    pub source: SourceInfo,
    /// 行为目标者信息
    pub target: TargetInfo,
    /// 使用的能力 ID（可选）
    pub ability_id: Option<String>,
    /// 使用的武器/装备 ID（可选）
    pub equipment_id: Option<String>,
    /// 元素类型（可选）
    pub element_type: Option<ElementType>,
    /// 是否为暴击
    pub is_critical: bool,
    /// 溯源链
    pub chain: ContextChain,
    /// 构建时间（帧号）
    pub created_at_frame: u64,
    /// 元数据
    pub metadata: ContextMetadata,
}

impl GameplayContextData {
    /// 生成唯一上下文 ID（确定性自增序列）。
    pub(crate) fn generate_id(next_id: &mut u64) -> String {
        let id = *next_id;
        *next_id += 1;
        format!("ctx_{:010}", id)
    }
}
