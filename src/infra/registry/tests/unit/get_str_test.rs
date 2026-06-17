use crate::infra::registry::{DefinitionId, RegistryBucket};

/// get_str 与 get 行为一致：相同 ID 返回相同结果。
#[test]
fn get_str_matches_get() {
    let mut bucket: RegistryBucket<i32> = RegistryBucket::new();
    let id = DefinitionId::new("abl_000001");
    bucket.insert(id.clone(), 42);

    let via_get = bucket.get(&id);
    let via_get_str = bucket.get_str("abl_000001");

    assert_eq!(via_get, Some(&42));
    assert_eq!(via_get, via_get_str);
}

/// get_str 对不存在的 ID 返回 None。
#[test]
fn get_str_nonexistent_returns_none() {
    let bucket: RegistryBucket<i32> = RegistryBucket::new();
    let result = bucket.get_str("abl_999999");
    assert!(result.is_none());
}

/// get_str 对空字符串返回 None。
#[test]
fn get_str_empty_string_returns_none() {
    let bucket: RegistryBucket<i32> = RegistryBucket::new();
    let result = bucket.get_str("");
    assert!(result.is_none());
}

/// get_str 在多个条目中能正确检索。
#[test]
fn get_str_retrieves_correct_entry_among_many() {
    let mut bucket: RegistryBucket<i32> = RegistryBucket::new();
    bucket.insert("abl_000001", 10);
    bucket.insert("abl_000002", 20);
    bucket.insert("abl_000003", 30);

    assert_eq!(bucket.get_str("abl_000002"), Some(&20));
    assert_eq!(bucket.get_str("abl_000001"), Some(&10));
    assert_eq!(bucket.get_str("abl_000003"), Some(&30));
}

/// get_str 在插入后立即反映最新值。
#[test]
fn get_str_reflects_latest_insert() {
    let mut bucket: RegistryBucket<String> = RegistryBucket::new();
    bucket.insert("abl_000001", "first".to_string());
    assert_eq!(
        bucket.get_str("abl_000001").map(String::as_str),
        Some("first")
    );

    bucket.insert("abl_000001", "updated".to_string());
    assert_eq!(
        bucket.get_str("abl_000001").map(String::as_str),
        Some("updated")
    );
}

/// get_str 在移除后返回 None。
#[test]
fn get_str_returns_none_after_remove() {
    let mut bucket: RegistryBucket<i32> = RegistryBucket::new();
    let id = DefinitionId::new("abl_000001");
    bucket.insert(id.clone(), 42);

    assert!(bucket.get_str("abl_000001").is_some());
    bucket.remove(&id);
    assert!(bucket.get_str("abl_000001").is_none());
}
