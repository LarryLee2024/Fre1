//! GroupByMap — Iterator 扩展，按键分组收集到 HashMap

use std::collections::HashMap;
use std::hash::Hash;

/// GroupByMap 扩展 trait，为 Iterator 提供 `group_by_map` 方法。
///
/// `group_by_map` 将迭代器的元素通过映射函数分组为 `HashMap<K, Vec<V>>`。
/// 与 `itertools::group_by` 不同，此方法：
/// - 不要求连续相同键（区别于 `group_by` 的临近分组）
/// - 直接返回 `HashMap` 而非借用迭代器
/// - 不依赖外部 crate
///
/// # 示例
///
/// ```ignore
/// use crate::shared::collections::GroupByMap;
///
/// let data = vec!["a", "bb", "ccc"];
/// let grouped = data.into_iter().group_by_map(|s| (s.len(), s.to_string()));
/// assert_eq!(grouped.get(&1).unwrap(), &vec!["a".to_string()]);
/// assert_eq!(grouped.get(&2).unwrap(), &vec!["bb".to_string()]);
/// assert_eq!(grouped.get(&3).unwrap(), &vec!["ccc".to_string()]);
/// ```
/// 迭代器按键分组收集扩展。
///
/// 存在原因：战斗结算时需要按队伍/阵营分组处理单位，
/// 标准库无内置按键分组收集到 HashMap 的方法。
pub trait GroupByMap: Iterator {
    /// 将迭代器元素按键分组收集到 HashMap。
    ///
    /// `f` 是一个映射函数，接收 `Self::Item` 并返回 `(K, V)` 元组，
    /// 其中 `K` 是分组键，`V` 是收集到向量中的值。
    ///
    /// # 类型参数
    /// - `K`: 分组键类型，必须实现 `Eq + Hash`
    /// - `V`: 收集值类型
    /// - `F`: 映射函数类型，`FnMut(Self::Item) -> (K, V)`
    ///
    /// # 示例
    ///
    /// ```ignore
    /// use crate::shared::collections::GroupByMap;
    ///
    /// let data = vec![1, 2, 3, 4, 5, 6];
    /// let grouped = data.into_iter().group_by_map(|x| (x % 2, x));
    /// // 键 0（偶数）: [2, 4, 6]，键 1（奇数）: [1, 3, 5]
    /// assert_eq!(grouped[&0], vec![2, 4, 6]);
    /// assert_eq!(grouped[&1], vec![1, 3, 5]);
    /// ```
    fn group_by_map<K, V, F>(self, f: F) -> HashMap<K, Vec<V>>
    where
        K: Eq + Hash,
        F: FnMut(Self::Item) -> (K, V);
}

impl<I: Iterator> GroupByMap for I {
    fn group_by_map<K, V, F>(self, f: F) -> HashMap<K, Vec<V>>
    where
        K: Eq + Hash,
        F: FnMut(Self::Item) -> (K, V),
    {
        let mut map: HashMap<K, Vec<V>> = HashMap::new();
        for (k, v) in self.map(f) {
            map.entry(k).or_default().push(v);
        }
        map
    }
}
