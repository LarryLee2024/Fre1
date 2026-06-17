//! define_string_id! 宏生成的 String ID 类型测试
//!
//! 验证：new/as_str/is_empty/len/into_inner/Display/FromStr/Deref/From/Serialize/Deserialize/StrongId

use crate::shared::ids::{AttributeId, StrongId, TagId};

// ── new + as_str ────────────────────────────────────────

#[test]
fn new_returns_internal_value() {
    let id = AttributeId::new("hp_max");
    assert_eq!(id.as_str(), "hp_max");
}

#[test]
fn as_str_returns_original_string() {
    let id = TagId::new("fire");
    assert_eq!(TagId::as_str(&id), "fire");
}

// ── is_empty ────────────────────────────────────────────

#[test]
fn empty_string_id_returns_true() {
    let id = AttributeId::new("");
    assert!(id.is_empty());
}

#[test]
fn non_empty_string_id_returns_false() {
    let id = AttributeId::new("hp");
    assert!(!id.is_empty());
}

// ── len ─────────────────────────────────────────────────

#[test]
fn len_returns_byte_length() {
    let id = AttributeId::new("hp_max");
    assert_eq!(id.len(), 6);
}

#[test]
fn empty_id_len_is_zero() {
    let id = AttributeId::new("");
    assert_eq!(id.len(), 0);
}

// ── into_inner ──────────────────────────────────────────

#[test]
fn into_inner_consumes_id_returns_string() {
    let id = AttributeId::new("hp");
    let inner: String = id.into_inner();
    assert_eq!(inner, "hp");
}

// ── Display ─────────────────────────────────────────────

#[test]
fn display_shows_prefix_colon_value() {
    let id = AttributeId::new("hp_max");
    assert_eq!(format!("{}", id), "attr:hp_max");
}

#[test]
fn tag_id_display_uses_tag_prefix() {
    let id = TagId::new("fire");
    assert_eq!(format!("{}", id), "tag:fire");
}

// ── FromStr ─────────────────────────────────────────────

#[test]
fn from_str_parses_prefix_colon_value() {
    let id: Result<AttributeId, _> = "attr:hp_max".parse();
    assert!(id.is_ok());
    assert_eq!(id.unwrap().as_str(), "hp_max");
}

#[test]
fn from_str_parses_bare_value() {
    let id: Result<AttributeId, _> = "hp_max".parse();
    assert!(id.is_ok());
    assert_eq!(id.unwrap().as_str(), "hp_max");
}

#[test]
fn from_str_rejects_wrong_prefix() {
    let id: Result<AttributeId, _> = "tag:hp_max".parse();
    assert!(id.is_err());
}

#[test]
fn from_str_empty_returns_empty_id() {
    let id: Result<AttributeId, _> = "".parse();
    assert!(id.is_ok());
    assert!(id.unwrap().is_empty());
}

// ── Deref ───────────────────────────────────────────────

#[test]
fn deref_to_str_usage() {
    let id = AttributeId::new("hp_max");
    let s: &str = &id;
    assert_eq!(s, "hp_max");
}

#[test]
fn deref_supports_string_operations() {
    let id = AttributeId::new("hp_max");
    assert!(id.starts_with("hp"));
    assert!(id.ends_with("max"));
}

// ── From<&str> ──────────────────────────────────────────

#[test]
fn from_str_conversion_succeeds() {
    let id = AttributeId::from("hp");
    assert_eq!(id.as_str(), "hp");
}

// ── From<String> ────────────────────────────────────────

#[test]
fn from_string_conversion_succeeds() {
    let id = AttributeId::from("hp".to_string());
    assert_eq!(id.as_str(), "hp");
}

// ── PartialEq / Eq ──────────────────────────────────────

#[test]
fn same_value_ids_equal() {
    let a = AttributeId::new("hp");
    let b = AttributeId::new("hp");
    assert_eq!(a, b);
}

#[test]
fn different_value_ids_not_equal() {
    let a = AttributeId::new("hp");
    let b = AttributeId::new("mp");
    assert_ne!(a, b);
}

// ── Hash ────────────────────────────────────────────────

#[test]
fn same_value_ids_same_hash() {
    use std::collections::HashMap;
    let a = AttributeId::new("hp");
    let b = AttributeId::new("hp");
    let mut map = HashMap::new();
    map.insert(a, 1);
    map.insert(b, 2);
    assert_eq!(map.len(), 1);
}

// ── Ord ─────────────────────────────────────────────────

#[test]
fn ids_order_by_lexicographic() {
    let a = AttributeId::new("aaa");
    let b = AttributeId::new("bbb");
    assert!(a < b);
}

// ── 不同前缀类型不兼容 ─────────────────────────────────

#[test]
fn different_prefix_ids_are_different_types() {
    let attr = AttributeId::new("hp");
    let tag = TagId::new("hp");
    // 编译期类型不同，运行时值相同但类型不同
    assert_eq!(attr.as_str(), tag.as_str());
    // 不能直接比较（类型不同）
    // assert_eq!(attr, tag); // 编译错误
}

// ── StrongId trait ──────────────────────────────────────

#[test]
fn strong_id_prefix_returns_correct_prefix() {
    assert_eq!(AttributeId::prefix(), "attr");
    assert_eq!(TagId::prefix(), "tag");
}

#[test]
fn strong_id_as_str_returns_inner_value() {
    let id = AttributeId::new("hp");
    assert_eq!(StrongId::as_str(&id), "hp");
}

// ── Serialize ───────────────────────────────────────────

#[test]
fn serialize_uses_prefix_colon_format() {
    let id = AttributeId::new("hp_max");
    let json = serde_json::to_string(&id).unwrap();
    assert_eq!(json, "\"attr:hp_max\"");
}

// ── Deserialize ─────────────────────────────────────────

#[test]
fn deserialize_accepts_prefix_colon_format() {
    let json = "\"attr:hp_max\"";
    let id: AttributeId = serde_json::from_str(json).unwrap();
    assert_eq!(id.as_str(), "hp_max");
}

#[test]
fn deserialize_accepts_bare_value() {
    let json = "\"hp_max\"";
    let id: AttributeId = serde_json::from_str(json).unwrap();
    assert_eq!(id.as_str(), "hp_max");
}

#[test]
fn deserialize_rejects_wrong_prefix() {
    let json = "\"tag:hp_max\"";
    let result: Result<AttributeId, _> = serde_json::from_str(json);
    assert!(result.is_err());
}

#[test]
fn serialize_deserialize_roundtrip() {
    let original = AttributeId::new("hp_max");
    let json = serde_json::to_string(&original).unwrap();
    let restored: AttributeId = serde_json::from_str(&json).unwrap();
    assert_eq!(original, restored);
}
