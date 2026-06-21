use std::collections::HashMap;

use crate::shared::collections::GroupByMap;

#[test]
fn groups_items_by_key_modulo() {
    let data = vec![1, 2, 3, 4, 5, 6];
    let grouped = data.into_iter().group_by_map(|x| (x % 2, x));
    let mut expected = HashMap::new();
    expected.insert(0, vec![2, 4, 6]);
    expected.insert(1, vec![1, 3, 5]);
    assert_eq!(grouped, expected);
}

#[test]
fn groups_by_string_length() {
    let data = vec!["cat", "dog", "bird", "ant", "fish"];
    let grouped = data.into_iter().group_by_map(|s| (s.len(), s));
    assert_eq!(grouped.get(&3).unwrap(), &vec!["cat", "dog", "ant"]);
    assert_eq!(grouped.get(&4).unwrap(), &vec!["bird", "fish"]);
}

#[test]
fn empty_iterator_returns_empty_map() {
    let empty: Vec<i32> = vec![];
    let grouped = empty.into_iter().group_by_map(|x| (x, x));
    assert!(grouped.is_empty());
}

#[test]
fn single_element_returns_single_entry() {
    let data = vec![42];
    let grouped = data.into_iter().group_by_map(|x| (x, x * 2));
    assert_eq!(grouped.len(), 1);
    assert_eq!(grouped.get(&42).unwrap(), &vec![84]);
}

#[test]
fn all_elements_same_key() {
    let data = vec![1, 2, 3, 4, 5];
    let grouped = data.into_iter().group_by_map(|x| (0, x));
    assert_eq!(grouped.len(), 1);
    assert_eq!(grouped.get(&0).unwrap(), &vec![1, 2, 3, 4, 5]);
}

#[test]
fn all_elements_different_keys() {
    let data = vec!["a", "b", "c"];
    let grouped = data.into_iter().group_by_map(|s| (s, s.len()));
    assert_eq!(grouped.len(), 3);
    assert_eq!(grouped.get("a").unwrap(), &vec![1]);
    assert_eq!(grouped.get("b").unwrap(), &vec![1]);
    assert_eq!(grouped.get("c").unwrap(), &vec![1]);
}

#[test]
fn preserves_insertion_order_within_group() {
    let data = vec![
        ("group1", "first"),
        ("group2", "alpha"),
        ("group1", "second"),
        ("group2", "beta"),
        ("group1", "third"),
    ];
    let grouped = data.into_iter().group_by_map(|(k, v)| (k, v));
    assert_eq!(
        grouped.get("group1").unwrap(),
        &vec!["first", "second", "third"]
    );
    assert_eq!(grouped.get("group2").unwrap(), &vec!["alpha", "beta"]);
}
