//! 非加密高速哈希工具
//!
//! 基于 aHash 算法的高性能非加密哈希实现。
//! 用于需要高速哈希计算且不涉及加密安全的场景（如 HashMap/HashSet 的键哈希）。
//!
//! aHash 使用 AES 指令集优化，在支持 AES-NI 的 CPU 上性能极佳；
//! 在不支持 AES-NI 的 CPU 上回退到可靠的替代方案。
//!
//! # 核心类型
//! - [`FastHasher`]: 高速哈希器，包装 `ahash::AHasher`
//! - [`FastBuildHasher`]: 高速哈希构建器，包装 `ahash::RandomState`
//!
//! # 便利函数
//! - [`fast_hash`]: 对任意哈希值计算哈希值
//! - [`new_fast_hashmap`]: 创建使用 FastBuildHasher 的 HashMap
//! - [`new_fast_hashset`]: 创建使用 FastBuildHasher 的 HashSet
//!
//! # 示例
//!
//! ```ignore
//! use fre_shared::hashing::{fast_hash, new_fast_hashmap, new_fast_hashset};
//!
//! let hash = fast_hash(&"hello");
//! let mut map = new_fast_hashmap::<String, i32>();
//! map.insert("key".into(), 42);
//! let mut set = new_fast_hashset::<String>();
//! set.insert("value".into());
//! ```

use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasher, Hasher};
use std::hash::Hash;

/// 高速哈希器。
///
/// 包装 `ahash::AHasher`，提供 `std::hash::Hasher` 接口。
/// 基于 aHash 算法，在非加密场景下性能优于标准 SipHash。
///
/// 通过 [`FastBuildHasher`] 创建实例，或直接使用 `FastHasher::new()` 创建默认实例。
#[derive(Clone, Default)]
pub struct FastHasher(ahash::AHasher);

impl FastHasher {
    /// 创建一个新的高速哈希器。
    ///
    /// 使用与 `ahash::RandomState` 相同的编译期派生密钥初始化。
    pub fn new() -> Self {
        Self(ahash::AHasher::default())
    }
}

impl Hasher for FastHasher {
    /// 返回已写入数据的哈希值。
    #[inline]
    fn finish(&self) -> u64 {
        self.0.finish()
    }

    /// 将字节切片写入哈希器。
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        self.0.write(bytes);
    }

    #[inline]
    fn write_u8(&mut self, i: u8) {
        self.0.write_u8(i);
    }

    #[inline]
    fn write_u16(&mut self, i: u16) {
        self.0.write_u16(i);
    }

    #[inline]
    fn write_u32(&mut self, i: u32) {
        self.0.write_u32(i);
    }

    #[inline]
    fn write_u64(&mut self, i: u64) {
        self.0.write_u64(i);
    }

    #[inline]
    fn write_usize(&mut self, i: usize) {
        self.0.write_usize(i);
    }

    #[inline]
    fn write_i8(&mut self, i: i8) {
        self.0.write_i8(i);
    }

    #[inline]
    fn write_i16(&mut self, i: i16) {
        self.0.write_i16(i);
    }

    #[inline]
    fn write_i32(&mut self, i: i32) {
        self.0.write_i32(i);
    }

    #[inline]
    fn write_i64(&mut self, i: i64) {
        self.0.write_i64(i);
    }

    #[inline]
    fn write_isize(&mut self, i: isize) {
        self.0.write_isize(i);
    }
}

/// 高速哈希构建器。
///
/// 实现 `std::hash::BuildHasher`，创建 [`FastHasher`] 实例。
/// 用于自定义 `HashMap` / `HashSet` 的哈希策略。
///
/// # 示例
///
/// ```ignore
/// use std::collections::HashMap;
/// use fre_shared::hashing::FastBuildHasher;
///
/// let mut map: HashMap<String, i32, FastBuildHasher> =
///     HashMap::with_hasher(FastBuildHasher::default());
/// map.insert("key".into(), 42);
/// ```
#[derive(Clone, Default)]
pub struct FastBuildHasher(ahash::RandomState);

impl FastBuildHasher {
    /// 创建一个新的高速哈希构建器。
    ///
    /// 使用 `ahash::RandomState::default()` 初始化，
    /// 利用进程级随机密钥提供 HashDoS 保护。
    pub fn new() -> Self {
        Self(ahash::RandomState::default())
    }
}

impl BuildHasher for FastBuildHasher {
    type Hasher = FastHasher;

    fn build_hasher(&self) -> FastHasher {
        FastHasher(self.0.build_hasher())
    }
}

/// 计算任意哈希值的快速哈希（u64）。
///
/// 使用 `FastBuildHasher` 创建一个临时哈希器并立即计算。
/// 适用于需要快速获取哈希值但不涉及加密安全的场景。
///
/// # 参数
/// - `val`: 任何实现了 `Hash` 的值
///
/// # 返回值
/// 64 位哈希值。相同的输入总是产生相同的输出（确定性）。
///
/// # 示例
///
/// ```ignore
/// use fre_shared::hashing::fast_hash;
///
/// let hash = fast_hash(&"hello world");
/// assert_eq!(fast_hash(&42), fast_hash(&42));
/// ```
#[inline]
pub fn fast_hash<T: Hash>(val: &T) -> u64 {
    let mut hasher = FastBuildHasher::default().build_hasher();
    val.hash(&mut hasher);
    hasher.finish()
}

/// 创建一个使用 `FastBuildHasher` 的 `HashMap`。
///
/// 比默认的 `HashMap` 在非加密场景下有更高的性能。
/// 适用于频繁插入和查询的 Map 操作。
///
/// # 类型参数
/// - `K`: 键类型
/// - `V`: 值类型
///
/// # 示例
///
/// ```ignore
/// use fre_shared::hashing::new_fast_hashmap;
///
/// let mut map = new_fast_hashmap::<String, i32>();
/// map.insert("answer".into(), 42);
/// assert_eq!(map.get("answer"), Some(&42));
/// ```
#[inline]
pub fn new_fast_hashmap<K, V>() -> HashMap<K, V, FastBuildHasher> {
    HashMap::with_hasher(FastBuildHasher::default())
}

/// 创建一个使用 `FastBuildHasher` 的 `HashSet`。
///
/// 比默认的 `HashSet` 在非加密场景下有更高的性能。
/// 适用于频繁插入和查询的去重集合。
///
/// # 类型参数
/// - `K`: 元素类型
///
/// # 示例
///
/// ```ignore
/// use fre_shared::hashing::new_fast_hashset;
///
/// let mut set = new_fast_hashset::<i32>();
/// set.insert(1);
/// set.insert(2);
/// assert!(set.contains(&1));
/// ```
#[inline]
pub fn new_fast_hashset<K>() -> HashSet<K, FastBuildHasher> {
    HashSet::with_hasher(FastBuildHasher::default())
}

#[cfg(test)]
mod tests;
