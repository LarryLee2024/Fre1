//! InstanceId<T> 泛型实例 ID 类型测试
//!
//! 验证：new/from_u64/from_runtime_id/value/index/generation/Display/Serialize/Deserialize

use crate::shared::ids::ModifierInstanceId;
use crate::shared::ids::types::runtime_id::{InstanceId, RuntimeId};
use bevy::prelude::Reflect;

// ── from_u64 (测试兼容) ───────────────────────────────────

#[test]
fn from_u64_creates_id_with_zero_generation() {
    let id = ModifierInstanceId::from_u64(42);
    assert_eq!(id.value(), 42);
    assert_eq!(id.index(), 42);
    assert_eq!(id.generation(), 0);
}

// ── new (index + generation) ──────────────────────────────

#[test]
fn new_with_index_and_generation() {
    let id = ModifierInstanceId::new(5, 2);
    assert_eq!(id.index(), 5);
    assert_eq!(id.generation(), 2);
    assert_eq!(id.value(), 5);
}

// ── from_runtime_id ───────────────────────────────────────

#[test]
fn from_runtime_id_wraps_runtime_id() {
    let rt = RuntimeId::new(10, 3);
    let id = ModifierInstanceId::from_runtime_id(rt);
    assert_eq!(id.index(), 10);
    assert_eq!(id.generation(), 3);
}

#[test]
fn runtime_id_roundtrip() {
    let rt = RuntimeId::new(7, 1);
    let id = ModifierInstanceId::from_runtime_id(rt);
    assert_eq!(id.runtime_id(), rt);
}

// ── Display ───────────────────────────────────────────────

#[test]
fn display_shows_index_hash_generation() {
    let id = ModifierInstanceId::new(42, 0);
    assert_eq!(format!("{}", id), "42#0");
}

#[test]
fn display_with_generation() {
    let id = ModifierInstanceId::new(1, 3);
    assert_eq!(format!("{}", id), "1#3");
}

// ── is_stale (generation 保护) ───────────────────────────

#[test]
fn is_stale_detects_generation_mismatch() {
    let old = ModifierInstanceId::new(0, 0); // index=0, gen=0 (released)
    // old 相对于新分配的同一槽位（index=0, gen=1）是 stale
    assert!(old.is_stale(&RuntimeId::new(0, 1)));
    // 同 index 同 generation → not stale
    let current = ModifierInstanceId::new(0, 1);
    assert!(!current.is_stale(&RuntimeId::new(0, 1)));
}

// ── Copy ──────────────────────────────────────────────────

#[test]
fn copy_semantics_no_mutable_state() {
    let a = ModifierInstanceId::from_u64(42);
    let b = a;
    assert_eq!(a.value(), 42);
    assert_eq!(b.value(), 42);
}

// ── PartialEq / Eq ────────────────────────────────────────

#[test]
fn same_values_equal() {
    let a = ModifierInstanceId::new(42, 0);
    let b = ModifierInstanceId::new(42, 0);
    assert_eq!(a, b);
}

#[test]
fn different_generation_not_equal() {
    let a = ModifierInstanceId::new(42, 0);
    let b = ModifierInstanceId::new(42, 1);
    assert_ne!(a, b);
}

// ── Ord ───────────────────────────────────────────────────

#[test]
fn ids_order_by_index_then_generation() {
    let a = ModifierInstanceId::new(1, 0);
    let b = ModifierInstanceId::new(2, 0);
    assert!(a < b);
}

// ── Serialize ─────────────────────────────────────────────

#[test]
fn serialize_uses_runtime_id_format() {
    let id = ModifierInstanceId::new(42, 3);
    let json = serde_json::to_string(&id).unwrap();
    assert_eq!(json, "[42,3]");
}

// ── Deserialize ───────────────────────────────────────────

#[test]
fn deserialize_restores() {
    let json = "[42,3]";
    let id: ModifierInstanceId = serde_json::from_str(json).unwrap();
    assert_eq!(id.index(), 42);
    assert_eq!(id.generation(), 3);
}

#[test]
fn serialize_deserialize_roundtrip() {
    let original = ModifierInstanceId::new(99, 1);
    let json = serde_json::to_string(&original).unwrap();
    let restored: ModifierInstanceId = serde_json::from_str(&json).unwrap();
    assert_eq!(original, restored);
}

// ── 类型隔离 ──────────────────────────────────────────────

/// 测试用的另一个实例类型。
#[derive(Reflect)]
pub struct AnotherMarker;

#[test]
fn different_instance_types_are_different_rust_types() {
    type AnotherId = InstanceId<AnotherMarker>;
    let a = ModifierInstanceId::from_u64(1);
    let b = AnotherId::from_u64(1);
    // 编译期类型不同——不能直接比较
    assert_eq!(a.value(), b.value());
    // assert_eq!(a, b); // 编译错误！
}
