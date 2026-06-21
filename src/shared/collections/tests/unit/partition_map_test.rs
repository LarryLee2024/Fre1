use crate::shared::collections::PartitionMap;

#[test]
fn partitions_mixed_ok_and_err() {
    let data = vec![Ok(1), Err("a"), Ok(2), Err("b")];
    let (oks, errs): (Vec<i32>, Vec<&str>) = data.into_iter().partition_map(|r| r);
    assert_eq!(oks, vec![1, 2]);
    assert_eq!(errs, vec!["a", "b"]);
}

#[test]
fn empty_iterator_returns_empty_vecs() {
    let empty: Vec<Result<i32, &str>> = vec![];
    let (oks, errs): (Vec<i32>, Vec<&str>) = empty.into_iter().partition_map(|r| r);
    assert!(oks.is_empty());
    assert!(errs.is_empty());
}

#[test]
fn all_ok_returns_empty_err_vec() {
    let data = vec![Ok(1), Ok(2), Ok(3)];
    let (oks, errs): (Vec<i32>, Vec<()>) = data.into_iter().partition_map(|r| r);
    assert_eq!(oks, vec![1, 2, 3]);
    assert!(errs.is_empty());
}

#[test]
fn all_err_returns_empty_ok_vec() {
    let data: Vec<Result<i32, &str>> = vec![Err("x"), Err("y")];
    let (oks, errs): (Vec<i32>, Vec<&str>) = data.into_iter().partition_map(|r| r);
    assert!(oks.is_empty());
    assert_eq!(errs, vec!["x", "y"]);
}

#[test]
fn single_ok_element() {
    let data = vec![Ok(42)];
    let (oks, errs): (Vec<i32>, Vec<()>) = data.into_iter().partition_map(|r| r);
    assert_eq!(oks, vec![42]);
    assert!(errs.is_empty());
}

#[test]
fn single_err_element() {
    let data: Vec<Result<i32, &str>> = vec![Err("error")];
    let (oks, errs): (Vec<i32>, Vec<&str>) = data.into_iter().partition_map(|r| r);
    assert!(oks.is_empty());
    assert_eq!(errs, vec!["error"]);
}

#[test]
fn preserves_order_within_each_partition() {
    let data = vec![
        Ok("first"),
        Err(1),
        Ok("second"),
        Ok("third"),
        Err(2),
        Err(3),
    ];
    let (oks, errs): (Vec<&str>, Vec<i32>) = data.into_iter().partition_map(|r| r);
    assert_eq!(oks, vec!["first", "second", "third"]);
    assert_eq!(errs, vec![1, 2, 3]);
}

#[test]
fn parse_strings_to_numbers() {
    let data = vec!["1", "two", "3", "four", "5"];
    let (numbers, errors): (Vec<i32>, Vec<&str>) = data
        .into_iter()
        .partition_map(|s| s.parse::<i32>().map_err(|_| s));
    assert_eq!(numbers, vec![1, 3, 5]);
    assert_eq!(errors, vec!["two", "four"]);
}

#[test]
fn custom_mapping_function() {
    let data = vec![10, 20, 30, 40, 50];
    let (large, small): (Vec<i32>, Vec<i32>) = data
        .into_iter()
        .partition_map(|x| if x > 25 { Ok(x) } else { Err(x) });
    assert_eq!(large, vec![30, 40, 50]);
    assert_eq!(small, vec![10, 20]);
}
