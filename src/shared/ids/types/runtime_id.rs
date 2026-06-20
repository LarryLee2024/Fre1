//! RuntimeId — 带 Generation 保护的运行时实例标识。
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
//! use crate::shared::ids::types::runtime_id::RuntimeIdAllocator;
//!
//! let mut allocator = RuntimeIdAllocator::new();
//! let id1 = allocator.alloc();  // RuntimeId { index: 0, generation: 0 }
//! allocator.free(id1);
//! let id2 = allocator.alloc();  // RuntimeId { index: 0, generation: 1 }
//!
//! // 旧引用 id1 的 generation 不匹配，可以安全检测
//! assert_ne!(id1, id2);
//! ```

use std::collections::VecDeque;
use std::marker::PhantomData;

use bevy::prelude::Reflect;

/// 带 Generation 保护的运行时 ID。
///
/// 由 `index`（数组索引）和 `generation`（代际计数器）组成。
/// 每次 ID 被回收后，再次分配时 generation 递增，防止旧引用指向新对象。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
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
/// use crate::shared::ids::types::runtime_id::RuntimeIdAllocator;
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
            RuntimeId {
                index,
                generation: 0,
            }
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

/// 带 Generation 保护的运行时实例 ID 泛型。
///
/// 类型参数 `T` 用于区分不同种类的实例 ID，编译器防止混用。
/// 内部包装 `RuntimeId`（index + generation），所有实例 ID 共享同一套 generation 保护逻辑。
///
/// # 示例
///
/// ```ignore
/// use crate::shared::ids::types::runtime_id::InstanceId;
///
/// pub struct ModifierMarker;
/// pub type ModifierInstanceId = InstanceId<ModifierMarker>;
///
/// let id = ModifierInstanceId::new(0, 0);
/// let other = ModifierInstanceId::from_u64(42);
/// ```
#[derive(Reflect)]
#[reflect(Clone, Hash, PartialEq)]
pub struct InstanceId<T: Reflect + 'static> {
    #[reflect(ignore)]
    inner: RuntimeId,
    #[reflect(ignore)]
    _marker: PhantomData<T>,
}

// 手动实现所有 trait 以避免 PhantomData<T> 的 bound 传播。

impl<T: Reflect + 'static> Clone for InstanceId<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Reflect + 'static> Copy for InstanceId<T> {}

impl<T: Reflect + 'static> std::fmt::Debug for InstanceId<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InstanceId")
            .field("index", &self.inner.index())
            .field("generation", &self.inner.generation())
            .field("type", &std::any::type_name::<T>())
            .finish()
    }
}

impl<T: Reflect + 'static> PartialEq for InstanceId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<T: Reflect + 'static> Eq for InstanceId<T> {}

impl<T: Reflect + 'static> std::hash::Hash for InstanceId<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}

impl<T: Reflect + 'static> PartialOrd for InstanceId<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Reflect + 'static> Ord for InstanceId<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.inner.cmp(&other.inner)
    }
}

impl<T: Reflect + 'static> InstanceId<T> {
    /// 从 index 和 generation 创建（用于反序列化/测试）。
    pub fn new(index: u32, generation: u32) -> Self {
        Self {
            inner: RuntimeId::new(index, generation),
            _marker: PhantomData,
        }
    }

    /// 从 RuntimeId 创建（标准路径，由 Allocator 分配）。
    pub fn from_runtime_id(id: RuntimeId) -> Self {
        Self {
            inner: id,
            _marker: PhantomData,
        }
    }

    /// 从 u64 创建（用于反序列化/测试兼容，generation 默认为 0）。
    pub fn from_u64(id: u64) -> Self {
        Self {
            inner: RuntimeId::new(id as u32, 0),
            _marker: PhantomData,
        }
    }

    /// 返回内部 RuntimeId。
    pub fn runtime_id(&self) -> RuntimeId {
        self.inner
    }

    /// 返回索引值（兼容旧 API）。
    pub fn value(&self) -> u64 {
        self.inner.index() as u64
    }

    /// 返回索引。
    pub fn index(&self) -> u32 {
        self.inner.index()
    }

    /// 返回代际。
    pub fn generation(&self) -> u32 {
        self.inner.generation()
    }

    /// 检查另一个 RuntimeId 是否是同一槽位的旧代际（generation safety）。
    pub fn is_stale(&self, other: &RuntimeId) -> bool {
        self.inner.is_stale(other)
    }
}

impl<T: Reflect + 'static> Default for InstanceId<T> {
    fn default() -> Self {
        Self {
            inner: RuntimeId::new(0, 0),
            _marker: PhantomData,
        }
    }
}

impl<T: Reflect + 'static> std::fmt::Display for InstanceId<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}#{}", self.inner.index(), self.inner.generation())
    }
}

impl<T: Reflect + 'static> serde::Serialize for InstanceId<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.inner.serialize(serializer)
    }
}

impl<'de, T: Reflect + 'static> serde::Deserialize<'de> for InstanceId<T> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        RuntimeId::deserialize(deserializer).map(|inner| Self {
            inner,
            _marker: PhantomData,
        })
    }
}
