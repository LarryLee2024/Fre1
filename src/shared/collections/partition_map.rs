//! PartitionMap — Iterator 扩展，单次遍历分区为 Ok/Err 两个 Vec

/// PartitionMap 扩展 trait，为 Iterator 提供 `partition_map` 方法。
///
/// 单次遍历将迭代器元素通过 Result 映射函数分区为 `(Vec<A>, Vec<B>)`。
/// `Ok(a)` 值进入第一个 Vec，`Err(b)` 值进入第二个 Vec。
///
/// 与手动 `partition` 不同，此方法允许映射函数返回不同类型，
/// 从而在一次遍历中完成类型转换和分区。
///
/// # 示例
///
/// ```ignore
/// use crate::shared::collections::PartitionMap;
///
/// let data = vec!["1", "two", "3", "four"];
/// let (numbers, errors): (Vec<i32>, Vec<&str>) = data
///     .into_iter()
///     .partition_map(|s| s.parse::<i32>().map_err(|_| s));
/// assert_eq!(numbers, vec![1, 3]);
/// assert_eq!(errors, vec!["two", "four"]);
/// ```
pub trait PartitionMap: Iterator {
    /// 单次遍历将迭代器元素分区为 Ok 值和 Err 值。
    ///
    /// `f` 是一个映射函数，接收 `Self::Item` 并返回 `Result<A, B>`。
    /// 返回 `(Vec<A>, Vec<B>)`，其中 `Vec<A>` 包含所有 Ok 值，`Vec<B>` 包含所有 Err 值。
    ///
    /// # 类型参数
    /// - `A`: 成功值类型
    /// - `B`: 错误值类型
    /// - `F`: 映射函数类型，`FnMut(Self::Item) -> Result<A, B>`
    ///
    /// # 示例
    ///
    /// ```ignore
    /// use crate::shared::collections::PartitionMap;
    ///
    /// // 全 Ok 情况
    /// let data = vec![Ok(1), Ok(2), Ok(3)];
    /// let (oks, errs): (Vec<i32>, Vec<()>) = data
    ///     .into_iter()
    ///     .partition_map(|r| r);
    /// assert_eq!(oks, vec![1, 2, 3]);
    /// assert!(errs.is_empty());
    /// ```
    fn partition_map<A, B, F>(self, f: F) -> (Vec<A>, Vec<B>)
    where
        F: FnMut(Self::Item) -> Result<A, B>;
}

impl<I: Iterator> PartitionMap for I {
    fn partition_map<A, B, F>(self, mut f: F) -> (Vec<A>, Vec<B>)
    where
        F: FnMut(Self::Item) -> Result<A, B>,
    {
        let mut ok = Vec::new();
        let mut err = Vec::new();
        for item in self {
            match f(item) {
                Ok(a) => ok.push(a),
                Err(b) => err.push(b),
            }
        }
        (ok, err)
    }
}
