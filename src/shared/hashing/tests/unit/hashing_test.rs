use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use crate::hashing::{fast_hash, new_fast_hashmap, new_fast_hashset, FastBuildHasher, FastHasher};

#[test]
fn fast_hash_is_deterministic() {
    let a = fast_hash(&"hello");
    let b = fast_hash(&"hello");
    assert_eq!(a, b);
}

#[test]
fn fast_hash_differs_for_different_inputs() {
    let a = fast_hash(&"hello");
    let b = fast_hash(&"world");
    assert_ne!(a, b);
}

#[test]
fn fast_hash_works_for_integers() {
    let a = fast_hash(&42u64);
    let b = fast_hash(&42u64);
    assert_eq!(a, b);

    let c = fast_hash(&43u64);
    assert_ne!(a, c);
}

#[test]
fn fast_hash_works_for_floats() {
    let a = fast_hash(&3.14f64);
    let b = fast_hash(&3.14f64);
    assert_eq!(a, b);
}

#[test]
fn fast_hash_works_for_tuples() {
    let a = fast_hash(&(1, "two", 3.0));
    let b = fast_hash(&(1, "two", 3.0));
    assert_eq!(a, b);

    let c = fast_hash(&(1, "two", 4.0));
    assert_ne!(a, c);
}

#[test]
fn fast_hasher_implements_hasher_trait() {
    let mut hasher = FastHasher::new();
    "test".hash(&mut hasher);
    let hash = hasher.finish();

    let mut std_hasher = FastHasher::new();
    "test".hash(&mut std_hasher);
    assert_eq!(hash, std_hasher.finish());
}

#[test]
fn fast_hasher_writes_bytes_correctly() {
    let mut hasher = FastHasher::new();
    hasher.write(b"hello");
    let hash_a = hasher.finish();

    let mut hasher = FastHasher::new();
    hasher.write(b"hello");
    let hash_b = hasher.finish();

    assert_eq!(hash_a, hash_b);
}

#[test]
fn fast_hasher_writes_u64_correctly() {
    let mut hasher = FastHasher::new();
    hasher.write_u64(42);
    let hash_a = hasher.finish();

    let mut hasher = FastHasher::new();
    hasher.write_u64(42);
    let hash_b = hasher.finish();

    assert_eq!(hash_a, hash_b);
}

#[test]
fn fast_build_hasher_creates_deterministic_hashers() {
    let build_hasher = FastBuildHasher::new();

    let mut h1 = build_hasher.build_hasher();
    let mut h2 = build_hasher.build_hasher();

    h1.write_u64(100);
    h2.write_u64(100);

    assert_eq!(h1.finish(), h2.finish());
}

#[test]
fn fast_build_hasher_default_works() {
    let build_hasher = FastBuildHasher::default();

    let mut h1 = build_hasher.build_hasher();
    let mut h2 = FastBuildHasher::default().build_hasher();

    h1.write_u64(7);
    h2.write_u64(7);

    // 默认构造的 FastBuildHasher 使用进程级随机密钥
    // 每个实例的密钥应相同（编译期派生常量密钥）
    assert_eq!(h1.finish(), h2.finish());
}

#[test]
fn fast_build_hasher_satisfies_build_hasher_trait() {
    fn accepts_build_hasher(_: &impl std::hash::BuildHasher) {}
    accepts_build_hasher(&FastBuildHasher::new());
    accepts_build_hasher(&FastBuildHasher::default());
}

#[test]
fn fast_hashmap_insert_and_get() {
    let mut map = new_fast_hashmap::<String, i32>();
    map.insert("apple".into(), 5);
    map.insert("banana".into(), 10);

    assert_eq!(map.get("apple"), Some(&5));
    assert_eq!(map.get("banana"), Some(&10));
    assert_eq!(map.get("cherry"), None);
}

#[test]
fn fast_hashmap_overwrites_value() {
    let mut map = new_fast_hashmap::<String, i32>();
    map.insert("key".into(), 1);
    map.insert("key".into(), 2);

    assert_eq!(map.get("key"), Some(&2));
}

#[test]
fn fast_hashmap_works_with_integer_keys() {
    let mut map = new_fast_hashmap::<u64, String>();
    map.insert(1, "one".into());
    map.insert(2, "two".into());

    assert_eq!(map.get(&1), Some(&"one"));
    assert_eq!(map.get(&2), Some(&"two"));
}

#[test]
fn fast_hashset_insert_and_contains() {
    let mut set = new_fast_hashset::<i32>();
    set.insert(1);
    set.insert(2);
    set.insert(3);

    assert!(set.contains(&1));
    assert!(set.contains(&2));
    assert!(set.contains(&3));
    assert!(!set.contains(&4));
}

#[test]
fn fast_hashset_deduplicates() {
    let mut set = new_fast_hashset::<i32>();
    set.insert(1);
    set.insert(1);
    set.insert(2);

    assert_eq!(set.len(), 2);
}

#[test]
fn fast_hashset_remove_works() {
    let mut set = new_fast_hashset::<i32>();
    set.insert(1);
    set.insert(2);

    assert!(set.remove(&1));
    assert!(!set.contains(&1));
    assert_eq!(set.len(), 1);
}

#[test]
fn fast_hashmap_works_with_std_hashmap_api() {
    // 验证 FastHashMap 类型兼容 std HashMap API
    let mut map: HashMap<String, i32, FastBuildHasher> = HashMap::with_hasher(
        FastBuildHasher::default(),
    );
    map.insert("a".into(), 1);
    map.insert("b".into(), 2);

    assert_eq!(map.len(), 2);
    assert_eq!(map.get("a"), Some(&1));

    map.remove("a");
    assert_eq!(map.len(), 1);
    assert!(!map.contains_key("a"));
}

#[test]
fn fast_hasher_default_is_equivalent() {
    let mut h1 = FastHasher::default();
    let mut h2 = FastHasher::new();

    h1.write_u32(0xDEADBEEF);
    h2.write_u32(0xDEADBEEF);

    assert_eq!(h1.finish(), h2.finish());
}
