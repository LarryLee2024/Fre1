//! TakeWhileInclusive — Iterator 适配器，包含首个不满足谓词的元素

use std::fmt;

/// TakeWhileInclusive 适配器。
///
/// 行为类似 `std::iter::TakeWhile`，但**包含**首个不满足谓词的元素。
///
/// 当遇到第一个使 `predicate` 返回 `false` 的元素时，该元素被产出，
/// 随后的所有 `next()` 调用返回 `None`。
///
/// 此适配器通过 [`TakeWhileInclusiveExt`] trait 的 `take_while_inclusive` 方法创建。
///
/// # 示例
///
/// ```ignore
/// use crate::shared::collections::TakeWhileInclusive;
///
/// let data = [1, 2, 3, 4, 5];
/// let iter = TakeWhileInclusive::new(data.iter(), |&&x| x < 4);
/// let result: Vec<_> = iter.collect();
/// assert_eq!(result, vec![&1, &2, &3, &4]);
/// ```
pub struct TakeWhileInclusive<I, P> {
    /// 底层迭代器
    iter: I,
    /// 谓词闭包
    predicate: P,
    /// 是否已完成（遇到首个不满足谓词的元素后设为 true）
    done: bool,
}

impl<I: Iterator, P> TakeWhileInclusive<I, P> {
    /// 创建新 `TakeWhileInclusive` 适配器。
    ///
    /// 通常不直接调用，而是通过 [`TakeWhileInclusiveExt::take_while_inclusive`] 创建。
    pub fn new(iter: I, predicate: P) -> Self {
        Self {
            iter,
            predicate,
            done: false,
        }
    }

    /// 获取底层迭代器的引用。
    pub fn get_ref(&self) -> &I {
        &self.iter
    }

    /// 获取底层迭代器的可变引用。
    pub fn get_mut(&mut self) -> &mut I {
        &mut self.iter
    }

    /// 消费适配器，返回底层迭代器。
    pub fn into_inner(self) -> I {
        self.iter
    }
}

impl<I: Iterator, P: FnMut(&I::Item) -> bool> Iterator for TakeWhileInclusive<I, P> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        let item = self.iter.next()?;
        if (self.predicate)(&item) {
            Some(item)
        } else {
            self.done = true;
            Some(item)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.done {
            (0, Some(0))
        } else {
            let (_, upper) = self.iter.size_hint();
            (0, upper)
        }
    }
}

impl<I: fmt::Debug, P> fmt::Debug for TakeWhileInclusive<I, P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TakeWhileInclusive")
            .field("iter", &self.iter)
            .field("done", &self.done)
            .finish()
    }
}

/// TakeWhileInclusive 扩展 trait，为 Iterator 提供 `take_while_inclusive` 方法。
///
/// 与 `std::iter::Iterator::take_while` 不同，`take_while_inclusive` 会**包含**
/// 第一个使谓词返回 `false` 的元素。
///
/// # 示例
///
/// ```ignore
/// use crate::shared::collections::TakeWhileInclusiveExt;
///
/// let data = vec![1, 2, 3, 4, 5];
/// let result: Vec<_> = data.into_iter()
///     .take_while_inclusive(|&x| x < 4)
///     .collect();
/// // 4 是第一个不满足 x < 4 的元素，但仍被包含
/// assert_eq!(result, vec![1, 2, 3, 4]);
/// ```
/// 包含边界元素的 TakeWhile 扩展。
///
/// 存在原因：标准 `take_while` 在谓词首次为 false 时丢弃该元素，
/// 但对话树/任务链等场景需要保留"触发结束的元素"（如最后一个满足条件的对话节点）。
pub trait TakeWhileInclusiveExt: Iterator + Sized {
    /// 创建一个包含首个不满足谓词的元素的 TakeWhileInclusive 适配器。
    ///
    /// `predicate` 是一个闭包，接收元素引用并返回 bool。
    /// 当 `predicate` 返回 `false` 时，该元素仍被产出，随后迭代终止。
    ///
    /// # 参数
    /// - `predicate`: 接收 `&Self::Item` 返回 `bool` 的闭包
    ///
    /// # 示例
    ///
    /// ```ignore
    /// use crate::shared::collections::TakeWhileInclusiveExt;
    ///
    /// // 空迭代器
    /// let empty: Vec<i32> = vec![];
    /// let result: Vec<_> = empty.into_iter()
    ///     .take_while_inclusive(|_| true)
    ///     .collect();
    /// assert!(result.is_empty());
    /// ```
    fn take_while_inclusive<P>(self, predicate: P) -> TakeWhileInclusive<Self, P>
    where
        P: FnMut(&Self::Item) -> bool;
}

impl<I: Iterator + Sized> TakeWhileInclusiveExt for I {
    fn take_while_inclusive<P>(self, predicate: P) -> TakeWhileInclusive<Self, P>
    where
        P: FnMut(&Self::Item) -> bool,
    {
        TakeWhileInclusive::new(self, predicate)
    }
}
