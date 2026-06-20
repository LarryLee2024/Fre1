//! RuntimeId & RuntimeIdAllocator 单元测试
//!
//! 验证 Runtime ID 的创建、显示、世代比较、相等、排序、序列化往返，
//! 以及 Allocator 的顺序分配、释放复用、批量释放、重置和过时检测。

use crate::shared::ids::types::runtime_id::{RuntimeId, RuntimeIdAllocator};

#[test]
fn runtime_id_basic() {
    let id = RuntimeId::new(0, 0);
    assert_eq!(id.index(), 0);
    assert_eq!(id.generation(), 0);
    assert_eq!(id.to_string(), "0#0");
}

#[test]
fn runtime_id_display() {
    let id = RuntimeId::new(42, 3);
    assert_eq!(format!("{}", id), "42#3");
}

#[test]
fn runtime_id_is_stale() {
    let id1 = RuntimeId::new(0, 0);
    let id2 = RuntimeId::new(0, 1); // 同 index，不同 generation
    let id3 = RuntimeId::new(1, 0); // 不同 index

    assert!(id1.is_stale(&id2)); // 同 index，不同 generation = stale
    assert!(!id1.is_stale(&id3)); // 不同 index = not stale
    assert!(!id1.is_stale(&RuntimeId::new(0, 0))); // 同 index，同 generation = not stale
}

#[test]
fn runtime_id_equality() {
    let id1 = RuntimeId::new(0, 0);
    let id2 = RuntimeId::new(0, 0);
    let id3 = RuntimeId::new(0, 1);

    assert_eq!(id1, id2);
    assert_ne!(id1, id3);
}

#[test]
fn runtime_id_ordering() {
    let id1 = RuntimeId::new(0, 0);
    let id2 = RuntimeId::new(0, 1);
    let id3 = RuntimeId::new(1, 0);

    assert!(id1 < id2);
    assert!(id2 < id3);
}

#[test]
fn runtime_id_serde_roundtrip() {
    let id = RuntimeId::new(42, 3);
    let json = serde_json::to_string(&id).unwrap();
    let restored: RuntimeId = serde_json::from_str(&json).unwrap();
    assert_eq!(id, restored);
}

#[test]
fn allocator_alloc_sequential() {
    let mut allocator = RuntimeIdAllocator::new();
    let id1 = allocator.alloc();
    let id2 = allocator.alloc();
    let id3 = allocator.alloc();

    assert_eq!(id1, RuntimeId::new(0, 0));
    assert_eq!(id2, RuntimeId::new(1, 0));
    assert_eq!(id3, RuntimeId::new(2, 0));
}

#[test]
fn allocator_free_and_reuse() {
    let mut allocator = RuntimeIdAllocator::new();
    let id1 = allocator.alloc();
    let id2 = allocator.alloc();

    assert_eq!(id1, RuntimeId::new(0, 0));
    assert_eq!(id2, RuntimeId::new(1, 0));

    allocator.free(id1);
    let id3 = allocator.alloc();

    // 复用 id1 的 slot，但 generation 递增
    assert_eq!(id3, RuntimeId::new(0, 1));
    assert_ne!(id1, id3);
}

#[test]
fn allocator_free_all() {
    let mut allocator = RuntimeIdAllocator::new();
    let id1 = allocator.alloc();
    let _id2 = allocator.alloc();
    let id3 = allocator.alloc();

    allocator.free_all([id1, id3]);

    assert_eq!(allocator.free_count(), 2);

    let id4 = allocator.alloc();
    let id5 = allocator.alloc();

    // 复用 id1 和 id3 的 slot
    assert_eq!(id4, RuntimeId::new(0, 1));
    assert_eq!(id5, RuntimeId::new(2, 1));
}

#[test]
fn allocator_total_allocated() {
    let mut allocator = RuntimeIdAllocator::new();
    assert_eq!(allocator.total_allocated(), 0);

    let _ = allocator.alloc();
    let _ = allocator.alloc();
    assert_eq!(allocator.total_allocated(), 2);

    allocator.reset();
    assert_eq!(allocator.total_allocated(), 0);
}

#[test]
fn allocator_stale_detection() {
    let mut allocator = RuntimeIdAllocator::new();
    let id1 = allocator.alloc();
    let _id2 = allocator.alloc();

    allocator.free(id1);
    let id3 = allocator.alloc();

    // id1 和 id3 是同一 slot 的不同 generation
    assert!(id1.is_stale(&id3));
    // id2 仍然有效（虽然我们没用它）
}
