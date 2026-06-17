//! define_numeric_id! 宏生成的 Numeric ID 类型测试
//!
//! 验证：new/value/Display/From/Deref/Serialize/Deserialize/Copy/Ord

use crate::core::capabilities::modifier::foundation::ModifierInstanceId;

// ── new + value ─────────────────────────────────────────

#[test]
fn new_creates_id_and_saves_value() {
    let id = ModifierInstanceId::new(42);
    assert_eq!(id.value(), 42);
}

// ── Display ─────────────────────────────────────────────

#[test]
fn display_shows_name_value_format() {
    let id = ModifierInstanceId::new(42);
    assert_eq!(format!("{}", id), "ModifierInstanceId(42)");
}

// ── From<u64> ──────────────────────────────────────────

#[test]
fn from_u64_conversion_succeeds() {
    let id = ModifierInstanceId::from(100u64);
    assert_eq!(id.value(), 100);
}

// ── Deref ───────────────────────────────────────────────

#[test]
fn deref_to_u64_usage() {
    let id = ModifierInstanceId::new(42);
    let v: &u64 = &id;
    assert_eq!(*v, 42);
}

// ── Copy ────────────────────────────────────────────────

#[test]
fn copy_semantics_no_mutable_state() {
    let a = ModifierInstanceId::new(42);
    let b = a;
    assert_eq!(a.value(), 42);
    assert_eq!(b.value(), 42);
}

// ── PartialEq / Eq ──────────────────────────────────────

#[test]
fn same_value_ids_equal() {
    let a = ModifierInstanceId::new(42);
    let b = ModifierInstanceId::new(42);
    assert_eq!(a, b);
}

#[test]
fn different_value_ids_not_equal() {
    let a = ModifierInstanceId::new(42);
    let b = ModifierInstanceId::new(43);
    assert_ne!(a, b);
}

// ── Ord ─────────────────────────────────────────────────

#[test]
fn ids_order_by_value() {
    let a = ModifierInstanceId::new(1);
    let b = ModifierInstanceId::new(2);
    assert!(a < b);
}

// ── Serialize ───────────────────────────────────────────

#[test]
fn serialize_returns_pure_u64() {
    let id = ModifierInstanceId::new(42);
    let json = serde_json::to_string(&id).unwrap();
    assert_eq!(json, "42");
}

// ── Deserialize ─────────────────────────────────────────

#[test]
fn deserialize_from_u64_restores() {
    let json = "42";
    let id: ModifierInstanceId = serde_json::from_str(json).unwrap();
    assert_eq!(id.value(), 42);
}

#[test]
fn serialize_deserialize_roundtrip() {
    let original = ModifierInstanceId::new(99);
    let json = serde_json::to_string(&original).unwrap();
    let restored: ModifierInstanceId = serde_json::from_str(&json).unwrap();
    assert_eq!(original, restored);
}
