use bevy::prelude::Entity;

use crate::core::capabilities::gameplay_context::foundation::{
    ChainNode, ContextBuildError, ContextChain, ContextOrigin, ElementType, SourceInfo, TargetInfo,
};

fn test_entity(index: u32) -> Entity {
    Entity::from_bits((index as u64) << 32 | 0x10000)
}

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
fn initial_chain_is_empty() {
    let node = dummy_node(test_entity(1), 0, 1);
    let chain = ContextChain::new(node);
    assert!(!chain.is_empty());
    assert_eq!(chain.len(), 1);
}

#[test]
fn cycle_self_match_rejected() {
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
fn different_ability_prevents_cycle() {
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
fn chain_exceeds_max_length_rejected() {
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
fn last_returns_newest_node() {
    let e1 = test_entity(1);
    let node1 = dummy_node(e1, 0, 1);
    let mut chain = ContextChain::new(node1);
    let e2 = test_entity(2);
    let node2 = dummy_node(e2, 5, 2);
    chain.try_push(node2).unwrap();
    assert_eq!(chain.last().unwrap().frame, 5);
}
