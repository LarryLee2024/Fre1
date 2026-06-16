//! GameplayContext 值对象定义

use std::sync::atomic::{AtomicU64, Ordering};

use bevy::ecs::entity::EntityIndex;
use bevy::prelude::Entity;

use crate::core::capabilities::gameplay_context::foundation::types::*;

// 全局自增上下文 ID 生成器（确定性，Replay-safe）

// 全局自增上下文 ID 生成器（确定性，Replay-safe）
static NEXT_CONTEXT_ID: AtomicU64 = AtomicU64::new(1);

#[cfg(test)]
fn test_entity(index: u32) -> Entity {
    // Use generation=1 to avoid PLACEHOLDER (bits=0x1 = index=1, generation=0)
    Entity::from_bits((index as u64) << 32 | 0x10000)
}

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
    /// 创建一个新的空溯源链。
    pub fn new(first_node: ChainNode) -> Self {
        Self {
            max_length: 10,
            nodes: vec![first_node],
        }
    }

    /// 返回当前链长度。
    pub fn len(&self) -> u8 {
        self.nodes.len() as u8
    }

    /// 链是否为空。
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// 获取最新节点。
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

    /// 遍历所有节点。
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
    pub(crate) fn generate_id() -> String {
        let id = NEXT_CONTEXT_ID.fetch_add(1, Ordering::Relaxed);
        format!("ctx_{:010}", id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_source(entity: Entity) -> SourceInfo {
        SourceInfo {
            entity,
            faction: "fct_000001".to_string(),
            position: Some((0, 0)),
        }
    }

    fn dummy_target(entity: Entity) -> TargetInfo {
        TargetInfo {
            entity,
            faction: "fct_000002".to_string(),
            position: Some((5, 5)),
            is_valid: true,
        }
    }

    fn dummy_node(entity: Entity, frame: u64, id: u64) -> ChainNode {
        ChainNode {
            origin: ContextOrigin::Direct,
            source: dummy_source(entity),
            target: dummy_target(test_entity(2)),
            ability_id: None,
            frame,
            node_id: id,
        }
    }

    #[test]
    fn unit_001_empty_chain_is_empty() {
        let node = dummy_node(test_entity(1), 0, 1);
        let chain = ContextChain::new(node);
        assert!(!chain.is_empty());
        assert_eq!(chain.len(), 1);
    }

    #[test]
    fn unit_002_cycle_detection_matches() {
        let e1 = test_entity(1);
        let e2 = test_entity(2);
        let node1 = ChainNode {
            origin: ContextOrigin::Direct,
            source: dummy_source(e1),
            target: dummy_target(e2),
            ability_id: Some("abl_000001".to_string()),
            frame: 0,
            node_id: 1,
        };
        let mut chain = ContextChain::new(node1);

        // Same source + target + ability → cycle
        let node2 = ChainNode {
            origin: ContextOrigin::ChainReaction,
            source: dummy_source(e1),
            target: dummy_target(e2),
            ability_id: Some("abl_000001".to_string()),
            frame: 1,
            node_id: 2,
        };
        assert!(chain.would_create_cycle(e1, e2, &Some("abl_000001".to_string())));
        assert!(chain.try_push(node2).is_err());
    }

    #[test]
    fn unit_003_different_ability_no_cycle() {
        let e1 = test_entity(1);
        let e2 = test_entity(2);
        let node1 = ChainNode {
            origin: ContextOrigin::Direct,
            source: dummy_source(e1),
            target: dummy_target(e2),
            ability_id: Some("abl_000001".to_string()),
            frame: 0,
            node_id: 1,
        };
        let mut chain = ContextChain::new(node1);

        let node2 = ChainNode {
            origin: ContextOrigin::ChainReaction,
            source: dummy_source(e1),
            target: dummy_target(e2),
            ability_id: Some("abl_000002".to_string()),
            frame: 1,
            node_id: 2,
        };
        assert!(!chain.would_create_cycle(e1, e2, &Some("abl_000002".to_string())));
        assert!(chain.try_push(node2).is_ok());
        assert_eq!(chain.len(), 2);
    }

    #[test]
    fn unit_004_chain_max_length_enforced() {
        let e1 = test_entity(1);
        let e2 = test_entity(2);
        let node1 = dummy_node(e1, 0, 1);
        let mut chain = ContextChain::new(node1);
        chain.max_length = 2;

        let node2 = dummy_node(e2, 1, 2);
        assert!(chain.try_push(node2).is_ok());

        let e3 = test_entity(3);
        let node3 = dummy_node(e3, 2, 3);
        let result = chain.try_push(node3);
        assert!(matches!(
            result,
            Err(ContextBuildError::ChainTooLong { .. })
        ));
    }

    #[test]
    fn unit_005_last_returns_most_recent() {
        let e1 = test_entity(1);
        let node1 = dummy_node(e1, 0, 1);
        let mut chain = ContextChain::new(node1);
        let e2 = test_entity(2);
        let node2 = dummy_node(e2, 5, 2);
        chain.try_push(node2).unwrap();
        assert_eq!(chain.last().unwrap().frame, 5);
    }
}
