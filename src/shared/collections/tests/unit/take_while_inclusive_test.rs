use crate::shared::collections::{TakeWhileInclusive, TakeWhileInclusiveExt};

#[test]
fn includes_failing_element() {
    let data = vec![1, 2, 3, 4, 5];
    let result: Vec<_> = data.into_iter().take_while_inclusive(|&x| x < 4).collect();
    assert_eq!(result, vec![1, 2, 3, 4]);
}

#[test]
fn empty_iterator_returns_empty() {
    let empty: Vec<i32> = vec![];
    let result: Vec<_> = empty.into_iter().take_while_inclusive(|_| true).collect();
    assert!(result.is_empty());
}

#[test]
fn all_elements_match_predicate() {
    let data = vec![1, 2, 3];
    let result: Vec<_> = data.into_iter().take_while_inclusive(|&x| x < 10).collect();
    assert_eq!(result, vec![1, 2, 3]);
}

#[test]
fn first_element_fails_predicate() {
    let data = vec![10, 1, 2, 3];
    let result: Vec<_> = data.into_iter().take_while_inclusive(|&x| x < 10).collect();
    // 10 不满足谓词，但仍被包含，随后终止
    assert_eq!(result, vec![10]);
}

#[test]
fn single_element_matching() {
    let data = vec![5];
    let result: Vec<_> = data.into_iter().take_while_inclusive(|&x| x > 0).collect();
    assert_eq!(result, vec![5]);
}

#[test]
fn single_element_not_matching() {
    let data = vec![0];
    let result: Vec<_> = data.into_iter().take_while_inclusive(|&x| x > 0).collect();
    assert_eq!(result, vec![0]);
}

#[test]
fn works_with_references() {
    let data = vec!["a", "bb", "ccc", "dddd"];
    let result: Vec<_> = data.iter().take_while_inclusive(|s| s.len() < 4).collect();
    assert_eq!(result, vec![&"a", &"bb", &"ccc", &"dddd"]);
}

#[test]
fn new_creates_adapter_directly() {
    let data = [1i32, 2, 3];
    let iter = TakeWhileInclusive::new(data.iter(), |x: &&i32| **x < 3);
    let result: Vec<_> = iter.collect();
    assert_eq!(result, vec![&1, &2, &3]);
}

#[test]
fn size_hint_after_exhaustion() {
    let data = vec![1, 2, 3, 4, 5];
    let mut iter = data.into_iter().take_while_inclusive(|&x| x <= 3);
    // 消耗到包含不满足元素
    iter.by_ref().for_each(drop);
    assert_eq!(iter.size_hint(), (0, Some(0)));
}

#[test]
fn into_inner_recovers_underlying_iterator() {
    let data = vec![1, 2, 3, 4, 5];
    let adapter = data.into_iter().take_while_inclusive(|&x| x < 3);
    // 恢复底层迭代器（已消耗并熔断）
    let _inner = adapter.into_inner();
}
