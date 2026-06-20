//! Runtime ID — 带 Generation 保护的运行时实例标识。
//!
//! 用于运行时动态分配的实例唯一标识，通过 generation 机制防止 ID 复用导致的引用悬空。
//!
//! # 设计原则
//!
//! - `index`: 数组索引，用于快速查找（O(1)）
//! - `generation`: 代际计数器，每次回收后递增，防止旧引用指向新对象
//!
//! # 示例
//!
//! ```ignore
//! use crate::shared::ids::runtime_id::RuntimeIdAllocator;
//!
//! let mut allocator = RuntimeIdAllocator::new();
//! let id1 = allocator.alloc();  // RuntimeId { index: 0, generation: 0 }
//! allocator.free(id1);
//! let id2 = allocator.alloc();  // RuntimeId { index: 0, generation: 1 }（同 index，不同 generation）
//!
//! // 旧引用 id1 的 generation 不匹配，可以安全检测
//! assert_ne!(id1, id2);
//! ```

use std::collections::VecDeque;

/// 带 Generation 保护的运行时 ID。
///
/// 由 `index`（数组索引）和 `generation`（代际计数器）组成。
/// 每次 ID 被回收后，再次分配时 generation 递增，防止旧引用指向新对象。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RuntimeId {
    /// 数组索引（用于快速查找）
    index: u32,
    /// 代际计数器（每次回收后递增）
    generation: u32,
}

impl RuntimeId {
    /// 创建新的 RuntimeId（仅用于反序列化，不应在业务代码中直接使用）。
    pub fn new(index: u32, generation: u32) -> Self {
        Self { index, generation }
    }

    /// 获取索引。
    pub fn index(&self) -> u32 {
        self.index
    }

    /// 获取代际。
    pub fn generation(&self) -> u32 {
        self.generation
    }

    /// 检查另一个 ID 是否是同一槽位的旧代际。
    pub fn is_stale(&self, other: &RuntimeId) -> bool {
        self.index == other.index && self.generation != other.generation
    }
}

impl std::fmt::Display for RuntimeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}#{}", self.index, self.generation)
    }
}

impl serde::Serialize for RuntimeId {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeTupleStruct;
        let mut ts = serializer.serialize_tuple_struct("RuntimeId", 2)?;
        ts.serialize_field(&self.index)?;
        ts.serialize_field(&self.generation)?;
        ts.end()
    }
}

impl<'de> serde::Deserialize<'de> for RuntimeId {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::{SeqAccess, Visitor};
        struct RuntimeIdVisitor;

        impl<'de> Visitor<'de> for RuntimeIdVisitor {
            type Value = RuntimeId;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "a tuple struct (index: u32, generation: u32)")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let index: u32 = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
                let generation: u32 = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
                Ok(RuntimeId { index, generation })
            }
        }

        deserializer.deserialize_tuple_struct("RuntimeId", 2, RuntimeIdVisitor)
    }
}

/// Runtime ID 分配器。
///
/// 管理 ID 的分配和回收，通过 generation 机制防止 ID 复用。
///
/// # 使用
///
/// ```ignore
/// use crate::shared::ids::runtime_id::RuntimeIdAllocator;
///
/// let mut allocator = RuntimeIdAllocator::new();
/// let id1 = allocator.alloc();
/// let id2 = allocator.alloc();
/// allocator.free(id1);
/// let id3 = allocator.alloc();  // 复用 id1 的 slot，但 generation 递增
/// ```
#[derive(Debug)]
pub struct RuntimeIdAllocator {
    /// 空闲槽位（index + generation）
    free_list: VecDeque<(u32, u32)>,
    /// 下一个新分配的 index
    next_index: u32,
}

impl Default for RuntimeIdAllocator {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeIdAllocator {
    /// 创建新的分配器。
    pub fn new() -> Self {
        Self {
            free_list: VecDeque::new(),
            next_index: 0,
        }
    }

    /// 分配一个 ID。
    ///
    /// 优先复用空闲槽位（generation 递增），否则分配新 index。
    pub fn alloc(&mut self) -> RuntimeId {
        if let Some((index, generation)) = self.free_list.pop_front() {
            RuntimeId { index, generation }
        } else {
            let index = self.next_index;
            self.next_index += 1;
            RuntimeId { index, generation: 0 }
        }
    }

    /// 回收一个 ID。
    ///
    /// 将槽位加入空闲列表，generation 会在下次分配时递增。
    pub fn free(&mut self, id: RuntimeId) {
        self.free_list.push_back((id.index, id.generation + 1));
    }

    /// 回收多个 ID。
    pub fn free_all(&mut self, ids: impl IntoIterator<Item = RuntimeId>) {
        for id in ids {
            self.free(id);
        }
    }

    /// 当前已分配的 ID 数量（包括已回收的）。
    pub fn total_allocated(&self) -> u32 {
        self.next_index
    }

    /// 当前空闲槽位数量。
    pub fn free_count(&self) -> usize {
        self.free_list.len()
    }

    /// 重置分配器（清空所有状态）。
    pub fn reset(&mut self) {
        self.free_list.clear();
        self.next_index = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_id_basic() {
        let id = RuntimeId::new(0, 0);
        assert_eq!(id.index(), 0);
        assert_eq!(id.generation(), 0);
        assert_eq!(id.to_string(), "0#0");
    }

    #[test]
    fn runtime_id_display() {
        let id = RuntimeId::new(42, 3);
        assert_eq!(format!("{}", id), "42#3");
    }

    #[test]
    fn runtime_id_is_stale() {
        let id1 = RuntimeId::new(0, 0);
        let id2 = RuntimeId::new(0, 1); // 同 index，不同 generation
        let id3 = RuntimeId::new(1, 0); // 不同 index

        assert!(id1.is_stale(&id2)); // 同 index，不同 generation = stale
        assert!(!id1.is_stale(&id3)); // 不同 index = not stale
        assert!(!id1.is_stale(&RuntimeId::new(0, 0))); // 同 index，同 generation = not stale
    }

    #[test]
    fn runtime_id_equality() {
        let id1 = RuntimeId::new(0, 0);
        let id2 = RuntimeId::new(0, 0);
        let id3 = RuntimeId::new(0, 1);

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn runtime_id_ordering() {
        let id1 = RuntimeId::new(0, 0);
        let id2 = RuntimeId::new(0, 1);
        let id3 = RuntimeId::new(1, 0);

        assert!(id1 < id2);
        assert!(id2 < id3);
    }

    #[test]
    fn runtime_id_serde_roundtrip() {
        let id = RuntimeId::new(42, 3);
        let json = serde_json::to_string(&id).unwrap();
        let restored: RuntimeId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, restored);
    }

    #[test]
    fn allocator_alloc_sequential() {
        let mut allocator = RuntimeIdAllocator::new();
        let id1 = allocator.alloc();
        let id2 = allocator.alloc();
        let id3 = allocator.alloc();

        assert_eq!(id1, RuntimeId::new(0, 0));
        assert_eq!(id2, RuntimeId::new(1, 0));
        assert_eq!(id3, RuntimeId::new(2, 0));
    }

    #[test]
    fn allocator_free_and_reuse() {
        let mut allocator = RuntimeIdAllocator::new();
        let id1 = allocator.alloc();
        let id2 = allocator.alloc();

        assert_eq!(id1, RuntimeId::new(0, 0));
        assert_eq!(id2, RuntimeId::new(1, 0));

        allocator.free(id1);
        let id3 = allocator.alloc();

        // 复用 id1 的 slot，但 generation 递增
        assert_eq!(id3, RuntimeId::new(0, 1));
        assert_ne!(id1, id3);
    }

    #[test]
    fn allocator_free_all() {
        let mut allocator = RuntimeIdAllocator::new();
        let id1 = allocator.alloc();
        let _id2 = allocator.alloc();
        let id3 = allocator.alloc();

        allocator.free_all([id1, id3]);

        assert_eq!(allocator.free_count(), 2);

        let id4 = allocator.alloc();
        let id5 = allocator.alloc();

        // 复用 id1 和 id3 的 slot
        assert_eq!(id4, RuntimeId::new(0, 1));
        assert_eq!(id5, RuntimeId::new(2, 1));
    }

    #[test]
    fn allocator_total_allocated() {
        let mut allocator = RuntimeIdAllocator::new();
        assert_eq!(allocator.total_allocated(), 0);

        let _ = allocator.alloc();
        let _ = allocator.alloc();
        assert_eq!(allocator.total_allocated(), 2);

        allocator.reset();
        assert_eq!(allocator.total_allocated(), 0);
    }

    #[test]
    fn allocator_stale_detection() {
        let mut allocator = RuntimeIdAllocator::new();
        let id1 = allocator.alloc();
        let _id2 = allocator.alloc();

        allocator.free(id1);
        let id3 = allocator.alloc();

        // id1 和 id3 是同一 slot 的不同 generation
        assert!(id1.is_stale(&id3));
        // id2 仍然有效（虽然我们没用它）
    }
}
